//! 文件发现用例

use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use devforge_domain::document::{Document, DocumentId, DocumentKind};
use devforge_domain::error::DomainError;
use devforge_domain::path_guard::PathGuard;
use devforge_domain::source::SourceId;

use crate::document::FileTreeEntry;

/// 全文索引端口（应用层端口）
///
/// 应用层通过此 trait 调用索引操作，不依赖具体实现。
pub trait IndexerPort: Send + Sync {
    /// 索引单个文档
    fn index_document(
        &self,
        document_id: &str,
        source_id: &str,
        path: &str,
        file_name: &str,
        content: &str,
    ) -> Result<(), DomainError>;

    /// 批量索引文档
    fn index_documents(
        &self,
        documents: &[IndexDocument<'_>],
    ) -> Result<(), DomainError>;

    /// 删除单个文档的索引
    fn remove_document(&self, document_id: &str) -> Result<(), DomainError>;

    /// 删除指定数据源的所有文档索引
    fn remove_by_source(&self, source_id: &str) -> Result<(), DomainError>;
}

/// 待索引文档（应用层 DTO）
pub struct IndexDocument<'a> {
    pub document_id: &'a str,
    pub source_id: &'a str,
    pub path: &'a str,
    pub file_name: &'a str,
    pub content: &'a str,
}

/// Document Repository Trait（应用层端口）
#[async_trait::async_trait]
pub trait DocumentRepository: Send + Sync {
    async fn create(&self, document: &Document) -> Result<(), DomainError>;
    async fn get(&self, id: &DocumentId) -> Result<Option<Document>, DomainError>;
    async fn list_by_source(&self, source_id: &SourceId) -> Result<Vec<Document>, DomainError>;
    async fn list_by_source_and_parent(
        &self,
        source_id: &SourceId,
        parent_path: Option<&str>,
    ) -> Result<Vec<Document>, DomainError>;
    async fn upsert(&self, document: &Document) -> Result<(), DomainError>;
    async fn delete_by_source(&self, source_id: &SourceId) -> Result<(), DomainError>;

    /// 列出文件树条目
    ///
    /// 返回 parent_path 的直接子项。目录在前，文件在后。
    /// 目录 key 唯一且稳定。文件使用真实 Document ID。
    ///
    /// 注意：当前实现读取整个 Source 后投影直接子项，
    /// 对大型仓库存在性能缺口。后续将通过 SQL 前缀查询优化。
    async fn list_file_tree(
        &self,
        source_id: &SourceId,
        parent_path: Option<&str>,
    ) -> Result<Vec<FileTreeEntry>, DomainError>;
}

/// 扫描结果
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct ScanResult {
    /// 新增文档数
    pub added: u32,
    /// 更新文档数
    pub updated: u32,
    /// 删除文档数
    pub removed: u32,
    /// 跳过文档数
    pub skipped: u32,
}

/// 文件发现错误
#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum DiscoveryError {
    #[error("数据源不存在")]
    SourceNotFound,
    #[error("路径错误: {0}")]
    Path(String),
    #[error("IO 错误: {0}")]
    Io(String),
    #[error("领域错误: {0}")]
    Domain(String),
}

impl From<DomainError> for DiscoveryError {
    fn from(err: DomainError) -> Self {
        DiscoveryError::Domain(err.to_string())
    }
}

/// 忽略规则
pub struct IgnoreRules {
    patterns: Vec<String>,
}

impl IgnoreRules {
    /// 从目录加载忽略规则
    pub fn load(root: &std::path::Path) -> Self {
        let mut patterns = vec![
            ".git".to_owned(),
            "node_modules".to_owned(),
            "target".to_owned(),
            ".DS_Store".to_owned(),
            "Thumbs.db".to_owned(),
        ];

        // 加载 .gitignore
        let gitignore_path = root.join(".gitignore");
        if let Ok(content) = std::fs::read_to_string(&gitignore_path) {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    patterns.push(line.to_owned());
                }
            }
        }

        // 加载 .devforgeignore
        let devforgeignore_path = root.join(".devforgeignore");
        if let Ok(content) = std::fs::read_to_string(&devforgeignore_path) {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    patterns.push(line.to_owned());
                }
            }
        }

        Self { patterns }
    }

    /// 检查路径是否应被忽略
    pub fn is_ignored(&self, path: &std::path::Path) -> bool {
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return false,
        };

        for pattern in &self.patterns {
            // 简单匹配：目录名或文件名
            if file_name == pattern.as_str() {
                return true;
            }
            // 通配符匹配：*.ext
            if let Some(ext) = pattern.strip_prefix("*.") {
                if file_name.to_lowercase().ends_with(&format!(".{ext}")) {
                    return true;
                }
            }
            // 前缀匹配：dir/
            if pattern.ends_with('/') {
                let prefix = &pattern[..pattern.len() - 1];
                if file_name == prefix {
                    return true;
                }
            }
        }

        false
    }
}

/// 检查文件是否为敏感文件
pub fn is_sensitive(path: &std::path::Path) -> bool {
    devforge_domain::document::is_sensitive(path)
}

/// 识别文档类型
pub fn identify_document_kind(path: &std::path::Path) -> DocumentKind {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(DocumentKind::from_extension)
        .unwrap_or(DocumentKind::Unknown)
}

/// 扫描源目录用例
///
/// 通过 source_id 从数据库获取可信路径，不接受前端传入的路径参数。
/// 如果提供了 indexer，扫描时会自动建立全文索引。
pub struct ScanSource {
    source_repo: Arc<dyn crate::source::SourceRepository>,
    document_repo: Arc<dyn DocumentRepository>,
    indexer: Option<Arc<dyn IndexerPort>>,
}

impl ScanSource {
    pub fn new(
        source_repo: Arc<dyn crate::source::SourceRepository>,
        document_repo: Arc<dyn DocumentRepository>,
    ) -> Self {
        Self {
            source_repo,
            document_repo,
            indexer: None,
        }
    }

    /// 设置全文索引器
    pub fn with_indexer(mut self, indexer: Arc<dyn IndexerPort>) -> Self {
        self.indexer = Some(indexer);
        self
    }

    pub async fn execute(&self, source_id: String) -> Result<ScanResult, DiscoveryError> {
        let source_id = SourceId(source_id.clone());

        // 从数据库获取可信路径
        let source = self
            .source_repo
            .get(&source_id)
            .await
            .map_err(|e| DiscoveryError::Domain(e.to_string()))?
            .ok_or(DiscoveryError::SourceNotFound)?;
        let root_path = source.root_path;

        // 验证路径存在
        if !root_path.exists() {
            return Err(DiscoveryError::Path(format!(
                "路径不存在: {}",
                root_path.display()
            )));
        }

        let guard =
            PathGuard::new(root_path.clone()).map_err(|e| DiscoveryError::Path(e.to_string()))?;

        let ignore = IgnoreRules::load(&root_path);

        // 收集当前文件系统中的文档
        let mut current_docs = Vec::new();
        self.walk_directory(&root_path, &root_path, &guard, &ignore, &mut current_docs)?;

        // 获取数据库中已有的文档
        let existing_docs = self.document_repo.list_by_source(&source_id).await?;
        let existing_paths: std::collections::HashMap<String, &Document> = existing_docs
            .iter()
            .map(|d| (d.relative_path.to_string_lossy().to_string(), d))
            .collect();

        let mut added = 0u32;
        let mut updated = 0u32;
        let mut skipped = 0u32;

        // 处理当前文件
        for (relative_path, file_path) in &current_docs {
            let metadata = match std::fs::metadata(file_path) {
                Ok(m) => m,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };

            let size = metadata.len();
            let modified_at = metadata
                .modified()
                .map(|t| {
                    // 将 SystemTime 转换为 DateTime<Utc>
                    let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                    let secs = duration.as_secs() as i64;
                    let nanos = duration.subsec_nanos();
                    chrono::DateTime::from_timestamp(secs, nanos).unwrap_or_else(Utc::now)
                })
                .unwrap_or_else(|_| Utc::now());

            let relative = std::path::PathBuf::from(relative_path);

            if let Some(existing) = existing_paths.get(relative_path) {
                // 检查是否需要更新
                if existing.size != size || existing.modified_at != modified_at {
                    let mut doc = Document::new(source_id.clone(), relative, size, modified_at);
                    doc.id = existing.id.clone();
                    self.document_repo.upsert(&doc).await?;

                    // 更新索引
                    if let Some(indexer) = &self.indexer {
                        if doc.content_readable && doc.sensitivity == devforge_domain::document::Sensitivity::Normal {
                            if let Ok(content) = std::fs::read_to_string(file_path) {
                                let file_name = file_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("");
                                let _ = indexer.index_document(
                                    &doc.id.0,
                                    &source_id.0,
                                    relative_path,
                                    file_name,
                                    &content,
                                );
                            }
                        }
                    }

                    updated += 1;
                } else {
                    skipped += 1;
                }
            } else {
                // 新增文档
                let doc = Document::new(source_id.clone(), relative, size, modified_at);
                self.document_repo.create(&doc).await?;

                // 索引新文档
                if let Some(indexer) = &self.indexer {
                    if doc.content_readable && doc.sensitivity == devforge_domain::document::Sensitivity::Normal {
                        if let Ok(content) = std::fs::read_to_string(file_path) {
                            let file_name = file_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("");
                            let _ = indexer.index_document(
                                &doc.id.0,
                                &source_id.0,
                                relative_path,
                                file_name,
                                &content,
                            );
                        }
                    }
                }

                added += 1;
            }
        }

        // 删除已不存在的文档
        let current_paths: std::collections::HashSet<String> =
            current_docs.iter().map(|(p, _)| p.clone()).collect();

        let mut removed = 0u32;
        for (path, existing_doc) in existing_paths.iter() {
            if !current_paths.contains(path) {
                // 文档已从文件系统删除，标记为不可读
                let mut doc = (*existing_doc).clone();
                doc.content_readable = false;
                self.document_repo.upsert(&doc).await?;

                // 从索引中移除
                if let Some(indexer) = &self.indexer {
                    let _ = indexer.remove_document(&doc.id.0);
                }

                removed += 1;
            }
        }

        Ok(ScanResult {
            added,
            updated,
            removed,
            skipped,
        })
    }

    /// 递归遍历目录
    fn walk_directory(
        &self,
        root: &std::path::Path,
        dir: &std::path::Path,
        guard: &PathGuard,
        ignore: &IgnoreRules,
        files: &mut Vec<(String, PathBuf)>,
    ) -> Result<(), DiscoveryError> {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                return Err(DiscoveryError::Io(format!(
                    "无法读取目录 {}: {}",
                    dir.display(),
                    e
                )));
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();

            // 检查是否应忽略
            if ignore.is_ignored(&path) {
                continue;
            }

            // 验证路径安全
            if guard.validate(&path).is_err() {
                continue;
            }

            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if file_type.is_dir() {
                // 递归遍历子目录
                self.walk_directory(root, &path, guard, ignore, files)?;
            } else if file_type.is_file() {
                // 计算相对路径
                let relative = match path.strip_prefix(root) {
                    Ok(r) => r.to_string_lossy().to_string(),
                    Err(_) => continue,
                };

                // 跳过二进制文件
                if is_binary(&path) {
                    continue;
                }

                // 跳过超大文件（>100MB）
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if metadata.len() > 100 * 1024 * 1024 {
                        continue;
                    }
                }

                files.push((relative, path));
            }
        }

        Ok(())
    }
}

/// 检查文件是否为二进制文件
fn is_binary(path: &std::path::Path) -> bool {
    // 检查已知二进制扩展名
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        return matches!(
            ext.to_lowercase().as_str(),
            "exe"
                | "dll"
                | "so"
                | "dylib"
                | "bin"
                | "obj"
                | "o"
                | "a"
                | "lib"
                | "pdb"
                | "png"
                | "jpg"
                | "jpeg"
                | "gif"
                | "bmp"
                | "svg"
                | "webp"
                | "ico"
                | "mp3"
                | "mp4"
                | "avi"
                | "mov"
                | "wmv"
                | "flv"
                | "wav"
                | "ogg"
                | "zip"
                | "tar"
                | "gz"
                | "rar"
                | "7z"
                | "bz2"
                | "pdf"
                | "doc"
                | "docx"
                | "xls"
                | "xlsx"
                | "ppt"
                | "pptx"
        );
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignore_rules_default() {
        let temp = tempfile::tempdir().unwrap();
        let ignore = IgnoreRules::load(temp.path());

        assert!(ignore.is_ignored(&temp.path().join(".git")));
        assert!(ignore.is_ignored(&temp.path().join("node_modules")));
        assert!(ignore.is_ignored(&temp.path().join("target")));
        assert!(!ignore.is_ignored(&temp.path().join("src")));
    }

    #[test]
    fn ignore_rules_gitignore() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join(".gitignore"), "*.log\nbuild/\n").unwrap();

        let ignore = IgnoreRules::load(temp.path());

        assert!(ignore.is_ignored(&temp.path().join("test.log")));
        assert!(ignore.is_ignored(&temp.path().join("build")));
        assert!(!ignore.is_ignored(&temp.path().join("src")));
    }

    #[test]
    fn ignore_rules_devforgeignore() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join(".devforgeignore"), "secret/\n").unwrap();

        let ignore = IgnoreRules::load(temp.path());

        assert!(ignore.is_ignored(&temp.path().join("secret")));
        assert!(!ignore.is_ignored(&temp.path().join("src")));
    }

    #[test]
    fn identify_kind() {
        assert_eq!(
            identify_document_kind(std::path::Path::new("test.rs")),
            DocumentKind::Text
        );
        assert_eq!(
            identify_document_kind(std::path::Path::new("README.md")),
            DocumentKind::Markdown
        );
        assert_eq!(
            identify_document_kind(std::path::Path::new("image.png")),
            DocumentKind::Image
        );
        assert_eq!(
            identify_document_kind(std::path::Path::new("app.exe")),
            DocumentKind::Binary
        );
    }

    #[test]
    fn is_binary_check() {
        assert!(is_binary(std::path::Path::new("app.exe")));
        assert!(is_binary(std::path::Path::new("image.png")));
        assert!(!is_binary(std::path::Path::new("test.rs")));
        assert!(!is_binary(std::path::Path::new("README.md")));
    }

    #[tokio::test]
    async fn scan_source() {
        use std::collections::HashMap;
        use std::sync::Mutex;

        use crate::source::SourceRepository;

        struct InMemorySourceRepository {
            sources: Mutex<HashMap<String, devforge_domain::source::Source>>,
        }

        impl InMemorySourceRepository {
            fn new() -> Self {
                Self {
                    sources: Mutex::new(HashMap::new()),
                }
            }
        }

        #[async_trait::async_trait]
        impl SourceRepository for InMemorySourceRepository {
            async fn create(
                &self,
                source: &devforge_domain::source::Source,
            ) -> Result<(), DomainError> {
                let mut sources = self.sources.lock().unwrap();
                sources.insert(source.id.0.clone(), source.clone());
                Ok(())
            }

            async fn get(
                &self,
                id: &devforge_domain::source::SourceId,
            ) -> Result<Option<devforge_domain::source::Source>, DomainError> {
                let sources = self.sources.lock().unwrap();
                Ok(sources.get(&id.0).cloned())
            }

            async fn list_by_workspace(
                &self,
                _workspace_id: &devforge_domain::workspace::WorkspaceId,
            ) -> Result<Vec<devforge_domain::source::Source>, DomainError> {
                let sources = self.sources.lock().unwrap();
                Ok(sources.values().cloned().collect())
            }

            async fn delete(
                &self,
                id: &devforge_domain::source::SourceId,
            ) -> Result<(), DomainError> {
                let mut sources = self.sources.lock().unwrap();
                sources.remove(&id.0);
                Ok(())
            }
        }

        struct InMemoryDocumentRepository {
            docs: Mutex<HashMap<String, Document>>,
        }

        impl InMemoryDocumentRepository {
            fn new() -> Self {
                Self {
                    docs: Mutex::new(HashMap::new()),
                }
            }
        }

        #[async_trait::async_trait]
        impl DocumentRepository for InMemoryDocumentRepository {
            async fn create(&self, document: &Document) -> Result<(), DomainError> {
                let mut docs = self.docs.lock().unwrap();
                docs.insert(document.id.0.clone(), document.clone());
                Ok(())
            }

            async fn get(&self, id: &DocumentId) -> Result<Option<Document>, DomainError> {
                let docs = self.docs.lock().unwrap();
                Ok(docs.get(&id.0).cloned())
            }

            async fn list_by_source(
                &self,
                source_id: &SourceId,
            ) -> Result<Vec<Document>, DomainError> {
                let docs = self.docs.lock().unwrap();
                Ok(docs
                    .values()
                    .filter(|d| d.source_id == *source_id)
                    .cloned()
                    .collect())
            }

            async fn list_by_source_and_parent(
                &self,
                source_id: &SourceId,
                parent_path: Option<&str>,
            ) -> Result<Vec<Document>, DomainError> {
                let docs = self.docs.lock().unwrap();
                let mut result = Vec::new();
                let mut seen_dirs = std::collections::HashSet::new();

                for doc in docs.values() {
                    if doc.source_id != *source_id {
                        continue;
                    }

                    let path_str = doc.relative_path.to_string_lossy().to_string();
                    let path = std::path::PathBuf::from(&path_str);

                    match parent_path {
                        Some(parent) => {
                            // 只返回直接子文件和子目录
                            if let Some(parent_of) = path.parent() {
                                if parent_of.to_string_lossy() == parent {
                                    result.push(doc.clone());
                                }
                            }
                        }
                        None => {
                            // 根目录：返回根目录下的文件和第一层目录
                            let components: Vec<_> = path.components().collect();
                            if components.len() == 1 {
                                // 根目录下的文件
                                result.push(doc.clone());
                            } else if components.len() > 1 {
                                // 子目录：只返回目录条目（使用第一个组件作为目录名）
                                let dir_name =
                                    components[0].as_os_str().to_string_lossy().to_string();
                                if !seen_dirs.contains(&dir_name) {
                                    seen_dirs.insert(dir_name.clone());
                                    // 创建一个虚拟的目录文档
                                    let mut dir_doc = doc.clone();
                                    dir_doc.relative_path = std::path::PathBuf::from(&dir_name);
                                    dir_doc.kind = DocumentKind::Unknown;
                                    dir_doc.size = 0;
                                    dir_doc.content_readable = false;
                                    result.push(dir_doc);
                                }
                            }
                        }
                    }
                }

                result.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
                Ok(result)
            }

            async fn upsert(&self, document: &Document) -> Result<(), DomainError> {
                let mut docs = self.docs.lock().unwrap();
                docs.insert(document.id.0.clone(), document.clone());
                Ok(())
            }

            async fn delete_by_source(&self, source_id: &SourceId) -> Result<(), DomainError> {
                let mut docs = self.docs.lock().unwrap();
                docs.retain(|_, d| d.source_id != *source_id);
                Ok(())
            }

            async fn list_file_tree(
                &self,
                _source_id: &SourceId,
                _parent_path: Option<&str>,
            ) -> Result<Vec<crate::document::FileTreeEntry>, DomainError> {
                // 简化实现，仅用于测试
                Ok(Vec::new())
            }
        }

        let temp = tempfile::tempdir().unwrap();

        // 创建测试文件结构
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();
        std::fs::write(temp.path().join("README.md"), "# Test").unwrap();
        std::fs::create_dir_all(temp.path().join(".git")).unwrap();
        std::fs::write(temp.path().join(".git/config"), "").unwrap();

        // 先创建 Source 记录到数据库
        let source_repo = Arc::new(InMemorySourceRepository::new());
        let workspace_id = devforge_domain::workspace::WorkspaceId::new();
        let source = devforge_domain::source::Source::new_git(
            workspace_id,
            "test-repo".to_owned(),
            temp.path().to_path_buf(),
        );
        let source_id = source.id.0.clone();
        source_repo.create(&source).await.unwrap();

        let doc_repo = Arc::new(InMemoryDocumentRepository::new());
        let use_case = ScanSource::new(source_repo.clone(), doc_repo.clone());

        let result = use_case.execute(source_id).await.unwrap();

        assert_eq!(result.added, 2); // main.rs 和 README.md
        assert_eq!(result.skipped, 0);
    }
}
