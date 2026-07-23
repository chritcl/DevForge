//! 全文索引模块
//!
//! 基于 Tantivy 实现每工作区独立的全文索引。
//! 索引存储在磁盘上，应用重启后自动恢复。

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, Value, STORED, STRING, TEXT};
use tantivy::snippet::SnippetGenerator;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term};

/// 索引错误
#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("索引 IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tantivy 错误: {0}")]
    Tantivy(#[from] tantivy::TantivyError),
    #[error("查询解析错误: {0}")]
    QueryParse(#[from] tantivy::query::QueryParserError),
    #[error("索引字段缺失: {0}")]
    MissingField(String),
}

/// 索引搜索结果
#[derive(Debug, Clone)]
pub struct IndexSearchHit {
    /// 文档 ID（SQLite 中的 Document ID）
    pub document_id: String,
    /// 文件路径
    pub path: String,
    /// 文件名
    pub file_name: String,
    /// 匹配分数
    pub score: f32,
    /// 匹配内容片段
    pub snippet: String,
    /// 首个匹配位置的行号（从 1 开始）
    pub line_number: u32,
}

/// Tantivy 索引管理器
///
/// 每个工作区拥有一个独立的 WorkspaceIndex 实例。
/// 索引数据存储在工作区目录下的 `indexes/lexical/` 子目录中。
pub struct WorkspaceIndex {
    index: Index,
    reader: IndexReader,
    writer: Mutex<IndexWriter>,
    /// 文档 ID 字段
    field_document_id: tantivy::schema::Field,
    /// 数据源 ID 字段
    field_source_id: tantivy::schema::Field,
    /// 文件路径字段
    field_path: tantivy::schema::Field,
    /// 文件名字段
    field_file_name: tantivy::schema::Field,
    /// 文件内容字段
    field_content: tantivy::schema::Field,
}

impl WorkspaceIndex {
    /// 创建或打开工作区索引
    ///
    /// 如果索引目录不存在则创建新索引，否则打开已有索引。
    pub fn open(index_dir: &Path) -> Result<Self, IndexError> {
        std::fs::create_dir_all(index_dir)?;

        let mut schema_builder = Schema::builder();
        // STRING 类型：不分词、精确匹配，适合 ID 字段
        // STRING | STORED：既可精确匹配删除，又可在搜索结果中返回值
        let field_document_id = schema_builder.add_text_field("document_id", STRING | STORED);
        let field_source_id = schema_builder.add_text_field("source_id", STRING | STORED);
        let field_path = schema_builder.add_text_field("path", TEXT | STORED);
        let field_file_name = schema_builder.add_text_field("file_name", TEXT | STORED);
        let field_content = schema_builder.add_text_field("content", TEXT | STORED);
        let schema = schema_builder.build();

        let index = if index_dir.join("meta.json").exists() {
            Index::open_in_dir(index_dir)?
        } else {
            Index::create_in_dir(index_dir, schema.clone())?
        };

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;

        let writer = index.writer(50_000_000)?; // 50MB 堆内存

        Ok(Self {
            index,
            reader,
            writer: Mutex::new(writer),
            field_document_id,
            field_source_id,
            field_path,
            field_file_name,
            field_content,
        })
    }

    /// 提交写入并刷新读取器，使变更立即可搜索
    fn commit_and_reload(&self, writer: &mut IndexWriter) -> Result<(), IndexError> {
        writer.commit()?;
        self.reader.reload()?;
        Ok(())
    }

    /// 构建 Tantivy 文档
    fn make_doc(
        &self,
        document_id: &str,
        source_id: &str,
        path: &str,
        file_name: &str,
        content: &str,
    ) -> TantivyDocument {
        let mut doc = TantivyDocument::new();
        doc.add_text(self.field_document_id, document_id);
        doc.add_text(self.field_source_id, source_id);
        doc.add_text(self.field_path, path);
        doc.add_text(self.field_file_name, file_name);
        doc.add_text(self.field_content, content);
        doc
    }

    /// 索引单个文档
    ///
    /// 如果已存在相同 document_id 的文档，先删除旧记录再插入新记录。
    pub fn index_document(
        &self,
        document_id: &str,
        source_id: &str,
        path: &str,
        file_name: &str,
        content: &str,
    ) -> Result<(), IndexError> {
        let mut writer = self.writer.lock().unwrap();

        // 先删除已有记录
        writer.delete_term(Term::from_field_text(self.field_document_id, document_id));

        // 插入新记录
        let doc = self.make_doc(document_id, source_id, path, file_name, content);
        writer.add_document(doc)?;

        self.commit_and_reload(&mut writer)?;
        Ok(())
    }

    /// 批量索引文档
    ///
    /// 一次性提交多个文档，比逐个索引更高效。
    pub fn index_documents(&self, documents: &[IndexDocument<'_>]) -> Result<(), IndexError> {
        let mut writer = self.writer.lock().unwrap();

        for d in documents {
            // 先删除已有记录
            writer.delete_term(Term::from_field_text(self.field_document_id, d.document_id));

            let doc = self.make_doc(d.document_id, d.source_id, d.path, d.file_name, d.content);
            writer.add_document(doc)?;
        }

        self.commit_and_reload(&mut writer)?;
        Ok(())
    }

    /// 删除单个文档的索引
    pub fn remove_document(&self, document_id: &str) -> Result<(), IndexError> {
        let mut writer = self.writer.lock().unwrap();
        writer.delete_term(Term::from_field_text(self.field_document_id, document_id));
        self.commit_and_reload(&mut writer)?;
        Ok(())
    }

    /// 删除指定数据源的所有文档索引
    pub fn remove_by_source(&self, source_id: &str) -> Result<(), IndexError> {
        let mut writer = self.writer.lock().unwrap();
        writer.delete_term(Term::from_field_text(self.field_source_id, source_id));
        self.commit_and_reload(&mut writer)?;
        Ok(())
    }

    /// 清空整个索引
    pub fn clear(&self) -> Result<(), IndexError> {
        let mut writer = self.writer.lock().unwrap();
        writer.delete_all_documents()?;
        self.commit_and_reload(&mut writer)?;
        Ok(())
    }

    /// 关键词搜索
    ///
    /// 在文件路径、文件名和内容中搜索关键词。
    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<IndexSearchHit>, IndexError> {
        let searcher = self.reader.searcher();

        let query_parser = QueryParser::new(
            self.index.schema(),
            vec![self.field_path, self.field_file_name, self.field_content],
            self.index.tokenizers().clone(),
        );

        let query = query_parser.parse_query(query_str)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit).order_by_score())?;

        // 创建 snippet 生成器
        let snippet_generator = SnippetGenerator::create(&searcher, &*query, self.field_content)?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher.doc(doc_address)?;
            let document_id = doc
                .get_first(self.field_document_id)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned();
            let path = doc
                .get_first(self.field_path)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned();
            let file_name = doc
                .get_first(self.field_file_name)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned();

            // 生成匹配内容片段并计算行号
            let snippet_obj = snippet_generator.snippet_from_doc(&doc);
            let snippet_html: String = snippet_obj.to_html().chars().take(300).collect();

            // 从内容中计算首个匹配位置的行号
            let line_number =
                if let Some(content) = doc.get_first(self.field_content).and_then(|v| v.as_str()) {
                    let fragment = snippet_obj.fragment();
                    if fragment.is_empty() {
                        1
                    } else {
                        content
                            .find(fragment)
                            .map(|pos| content[..pos].lines().count() as u32 + 1)
                            .unwrap_or(1)
                    }
                } else {
                    1
                };

            results.push(IndexSearchHit {
                document_id,
                path,
                file_name,
                score,
                snippet: snippet_html,
                line_number,
            });
        }

        Ok(results)
    }

    /// 获取索引中的文档数量
    pub fn document_count(&self) -> Result<u64, IndexError> {
        let searcher = self.reader.searcher();
        Ok(searcher.num_docs())
    }
}

/// 待索引文档
pub struct IndexDocument<'a> {
    pub document_id: &'a str,
    pub source_id: &'a str,
    pub path: &'a str,
    pub file_name: &'a str,
    pub content: &'a str,
}

/// 获取工作区索引目录路径
pub fn workspace_index_dir(workspace_id: &str, base_dir: &Path) -> PathBuf {
    base_dir
        .join("workspaces")
        .join(workspace_id)
        .join("indexes")
        .join("lexical")
}

/// 为 WorkspaceIndex 实现应用层 IndexerPort trait
impl devforge_application::discovery::IndexerPort for WorkspaceIndex {
    fn index_document(
        &self,
        document_id: &str,
        source_id: &str,
        path: &str,
        file_name: &str,
        content: &str,
    ) -> Result<(), devforge_domain::error::DomainError> {
        WorkspaceIndex::index_document(self, document_id, source_id, path, file_name, content)
            .map_err(|e| {
                devforge_domain::error::DomainError::Io(std::io::Error::other(e.to_string()))
            })
    }

    fn index_documents(
        &self,
        documents: &[devforge_application::discovery::IndexDocument<'_>],
    ) -> Result<(), devforge_domain::error::DomainError> {
        let converted: Vec<IndexDocument<'_>> = documents
            .iter()
            .map(|d| IndexDocument {
                document_id: d.document_id,
                source_id: d.source_id,
                path: d.path,
                file_name: d.file_name,
                content: d.content,
            })
            .collect();
        WorkspaceIndex::index_documents(self, &converted).map_err(|e| {
            devforge_domain::error::DomainError::Io(std::io::Error::other(e.to_string()))
        })
    }

    fn remove_document(
        &self,
        document_id: &str,
    ) -> Result<(), devforge_domain::error::DomainError> {
        WorkspaceIndex::remove_document(self, document_id).map_err(|e| {
            devforge_domain::error::DomainError::Io(std::io::Error::other(e.to_string()))
        })
    }

    fn remove_by_source(&self, source_id: &str) -> Result<(), devforge_domain::error::DomainError> {
        WorkspaceIndex::remove_by_source(self, source_id).map_err(|e| {
            devforge_domain::error::DomainError::Io(std::io::Error::other(e.to_string()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_index() -> (tempfile::TempDir, WorkspaceIndex) {
        let dir = tempfile::tempdir().unwrap();
        let index_dir = dir.path().join("lexical");
        let index = WorkspaceIndex::open(&index_dir).unwrap();
        (dir, index)
    }

    #[test]
    fn index_and_search_single_document() {
        let (_dir, index) = temp_index();

        index
            .index_document(
                "doc-1",
                "source-1",
                "src/main.rs",
                "main.rs",
                "fn main() { println!(\"hello\"); }",
            )
            .unwrap();

        let results = index.search("main", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, "doc-1");
        assert_eq!(results[0].path, "src/main.rs");
    }

    #[test]
    fn index_batch_documents() {
        let (_dir, index) = temp_index();

        let docs = vec![
            IndexDocument {
                document_id: "doc-1",
                source_id: "source-1",
                path: "src/main.rs",
                file_name: "main.rs",
                content: "fn main() {}",
            },
            IndexDocument {
                document_id: "doc-2",
                source_id: "source-1",
                path: "src/lib.rs",
                file_name: "lib.rs",
                content: "pub fn hello() {}",
            },
        ];

        index.index_documents(&docs).unwrap();

        assert_eq!(index.document_count().unwrap(), 2);
    }

    #[test]
    fn remove_document_from_index() {
        let (_dir, index) = temp_index();

        index
            .index_document("doc-1", "source-1", "a.rs", "a.rs", "content")
            .unwrap();
        assert_eq!(index.document_count().unwrap(), 1);

        index.remove_document("doc-1").unwrap();

        // 搜索应该返回 0 结果
        let results = index.search("content", 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn remove_by_source() {
        let (_dir, index) = temp_index();

        index
            .index_document("doc-1", "source-1", "a.rs", "a.rs", "alpha")
            .unwrap();
        index
            .index_document("doc-2", "source-2", "b.rs", "b.rs", "beta")
            .unwrap();

        index.remove_by_source("source-1").unwrap();

        // source-1 的文档应被删除
        let results = index.search("alpha", 10).unwrap();
        assert_eq!(results.len(), 0);

        // source-2 的文档应保留
        let results = index.search("beta", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, "doc-2");
    }

    #[test]
    fn clear_index() {
        let (_dir, index) = temp_index();

        index
            .index_document("doc-1", "source-1", "a.rs", "a.rs", "content")
            .unwrap();
        index.clear().unwrap();

        assert_eq!(index.document_count().unwrap(), 0);
    }

    #[test]
    fn reopen_existing_index() {
        let dir = tempfile::tempdir().unwrap();
        let index_dir = dir.path().join("lexical");

        // 第一次打开并索引
        {
            let index = WorkspaceIndex::open(&index_dir).unwrap();
            index
                .index_document("doc-1", "source-1", "a.rs", "a.rs", "persistent content")
                .unwrap();
        }

        // 第二次打开，数据应保留
        {
            let index = WorkspaceIndex::open(&index_dir).unwrap();
            let results = index.search("persistent", 10).unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].document_id, "doc-1");
        }
    }

    #[test]
    fn update_existing_document() {
        let (_dir, index) = temp_index();

        // 第一次索引
        index
            .index_document("doc-1", "source-1", "a.rs", "a.rs", "old content")
            .unwrap();

        // 更新同一文档
        index
            .index_document("doc-1", "source-1", "a.rs", "a.rs", "new content")
            .unwrap();

        // 搜索旧内容不应命中
        let results = index.search("old", 10).unwrap();
        assert_eq!(results.len(), 0);

        // 搜索新内容应命中
        let results = index.search("new", 10).unwrap();
        assert_eq!(results.len(), 1);
    }
}
