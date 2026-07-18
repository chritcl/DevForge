//! 文档查询用例

use std::path::PathBuf;
use std::sync::Arc;

use devforge_domain::document::{Document, DocumentId, DocumentKind, Sensitivity};
use devforge_domain::error::DomainError;
use devforge_domain::path_guard::PathGuard;
use devforge_domain::source::SourceId;

/// 文档查询错误
#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum DocumentError {
    #[error("文档不存在")]
    DocumentNotFound,
    #[error("数据源不存在")]
    SourceNotFound,
    #[error("敏感文件不可读")]
    SensitiveFile,
    #[error("文件过大")]
    FileTooLarge,
    #[error("路径错误: {0}")]
    Path(String),
    #[error("IO 错误: {0}")]
    Io(String),
    #[error("领域错误: {0}")]
    Domain(String),
}

impl From<DomainError> for DocumentError {
    fn from(err: DomainError) -> Self {
        DocumentError::Domain(err.to_string())
    }
}

/// 文档 DTO（用于 IPC 传输）
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct DocumentDto {
    pub id: String,
    pub source_id: String,
    pub relative_path: String,
    pub kind: DocumentKind,
    pub size: u64,
    pub sensitivity: Sensitivity,
    pub content_readable: bool,
}

impl From<&Document> for DocumentDto {
    fn from(doc: &Document) -> Self {
        Self {
            id: doc.id.0.clone(),
            source_id: doc.source_id.0.clone(),
            relative_path: doc.relative_path.to_string_lossy().to_string(),
            kind: doc.kind.clone(),
            size: doc.size,
            sensitivity: doc.sensitivity.clone(),
            content_readable: doc.content_readable,
        }
    }
}

/// 文件树条目类型
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum FileTreeEntryKind {
    Directory,
    File,
}

/// 文件树条目（应用查询模型）
#[derive(Debug, Clone)]
pub struct FileTreeEntry {
    /// 唯一标识
    pub key: String,
    /// 数据源 ID
    pub source_id: SourceId,
    /// 相对路径
    pub relative_path: PathBuf,
    /// 显示名称
    pub name: String,
    /// 条目类型
    pub entry_kind: FileTreeEntryKind,
    /// 关联文档（仅文件有值）
    pub document: Option<Document>,
}

/// 文件树条目 DTO（IPC 传输）
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct FileTreeEntryDto {
    pub key: String,
    pub source_id: String,
    pub relative_path: String,
    pub name: String,
    pub entry_kind: FileTreeEntryKind,
    pub document: Option<DocumentDto>,
}

impl From<&FileTreeEntry> for FileTreeEntryDto {
    fn from(entry: &FileTreeEntry) -> Self {
        Self {
            key: entry.key.clone(),
            source_id: entry.source_id.0.clone(),
            relative_path: entry.relative_path.to_string_lossy().to_string(),
            name: entry.name.clone(),
            entry_kind: entry.entry_kind.clone(),
            document: entry.document.as_ref().map(DocumentDto::from),
        }
    }
}

impl FileTreeEntry {
    /// 生成目录 key
    pub fn directory_key(source_id: &SourceId, relative_path: &std::path::Path) -> String {
        let normalized = relative_path.to_string_lossy().replace('\\', "/");
        format!("dir:{}:{}", source_id.0, normalized)
    }

    /// 生成文件 key
    pub fn file_key(document_id: &DocumentId) -> String {
        format!("file:{}", document_id.0)
    }
}

/// 文档查找状态
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DocumentLookupStatus {
    /// 找到文档且元数据完整
    Found,
    /// 文档元数据不存在（已从数据库删除）
    DocumentMissing,
    /// 文档对应的数据源不存在
    SourceMissing,
    /// 文件在磁盘上已不存在
    FileMissing,
    /// 路径安全验证失败
    PathInvalid,
}

/// 文档查找结果 DTO
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct DocumentLookupDto {
    /// 请求的文档 ID
    pub document_id: String,
    /// 查找状态
    pub status: DocumentLookupStatus,
    /// 文档元数据（仅 Found 时有值）
    pub document: Option<DocumentDto>,
}

/// 列出文档用例
pub struct ListDocuments {
    document_repo: Arc<dyn crate::discovery::DocumentRepository>,
}

impl ListDocuments {
    pub fn new(document_repo: Arc<dyn crate::discovery::DocumentRepository>) -> Self {
        Self { document_repo }
    }

    pub async fn execute(
        &self,
        source_id: String,
        parent_path: Option<String>,
    ) -> Result<Vec<DocumentDto>, DocumentError> {
        let source_id = SourceId(source_id);
        let documents = self
            .document_repo
            .list_by_source_and_parent(&source_id, parent_path.as_deref())
            .await?;

        Ok(documents.iter().map(DocumentDto::from).collect())
    }
}

/// 列出文件树条目用例
pub struct ListFileTree {
    document_repo: Arc<dyn crate::discovery::DocumentRepository>,
}

impl ListFileTree {
    pub fn new(document_repo: Arc<dyn crate::discovery::DocumentRepository>) -> Self {
        Self { document_repo }
    }

    pub async fn execute(
        &self,
        source_id: String,
        parent_path: Option<String>,
    ) -> Result<Vec<FileTreeEntryDto>, DocumentError> {
        let source_id = SourceId(source_id);
        let entries = self
            .document_repo
            .list_file_tree(&source_id, parent_path.as_deref())
            .await?;

        Ok(entries.iter().map(FileTreeEntryDto::from).collect())
    }
}

/// 读取文档内容用例
pub struct ReadDocumentContent {
    document_repo: Arc<dyn crate::discovery::DocumentRepository>,
    source_repo: Arc<dyn crate::source::SourceRepository>,
}

impl ReadDocumentContent {
    pub fn new(
        document_repo: Arc<dyn crate::discovery::DocumentRepository>,
        source_repo: Arc<dyn crate::source::SourceRepository>,
    ) -> Self {
        Self {
            document_repo,
            source_repo,
        }
    }

    /// 读取文档内容
    ///
    /// 可信调用链：
    /// document_id → DocumentRepository 查询 Document
    /// → document.source_id → SourceRepository 查询 Source
    /// → Source.root_path → canonicalize 可信根目录
    /// → PathGuard → 验证 relative_path
    /// → sensitivity / kind / size 检查 → 读取文件
    pub async fn execute(&self, document_id: String) -> Result<String, DocumentError> {
        let document_id = DocumentId(document_id);

        // 1. 查询文档元数据
        let document = self
            .document_repo
            .get(&document_id)
            .await?
            .ok_or(DocumentError::DocumentNotFound)?;

        // 2. 查询数据源（从数据库获取可信根目录）
        let source = self
            .source_repo
            .get(&document.source_id)
            .await?
            .ok_or(DocumentError::SourceNotFound)?;

        // 3. 检查是否可读
        if !document.content_readable {
            if document.sensitivity == Sensitivity::Sensitive {
                return Err(DocumentError::SensitiveFile);
            }
            return Err(DocumentError::Io("文件不可读".to_owned()));
        }

        // 4. 检查文件大小（最大 10MB）
        if document.size > 10 * 1024 * 1024 {
            return Err(DocumentError::FileTooLarge);
        }

        // 5. 使用可信根目录创建 PathGuard
        let guard = PathGuard::new(source.root_path.clone())
            .map_err(|e| DocumentError::Path(e.to_string()))?;

        // 6. 安全解析并验证相对文件路径
        let full_path = guard
            .resolve_relative_file(&document.relative_path)
            .map_err(|e| DocumentError::Path(e.to_string()))?;

        // 7. 读取文件内容
        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| DocumentError::Io(format!("无法读取文件: {e}")))?;

        Ok(content)
    }
}

/// 批量获取文档用例
pub struct GetDocumentsByIds {
    document_repo: Arc<dyn crate::discovery::DocumentRepository>,
    source_repo: Arc<dyn crate::source::SourceRepository>,
}

impl GetDocumentsByIds {
    pub fn new(
        document_repo: Arc<dyn crate::discovery::DocumentRepository>,
        source_repo: Arc<dyn crate::source::SourceRepository>,
    ) -> Self {
        Self {
            document_repo,
            source_repo,
        }
    }

    /// 批量获取文档信息
    ///
    /// 对每个 document_id：
    /// 1. 查询 Document 表
    /// 2. 查询对应的 Source 表
    /// 3. 检查文件是否存在于磁盘
    ///
    /// 单个文档失败不影响其他文档。
    pub async fn execute(
        &self,
        document_ids: Vec<String>,
    ) -> Result<Vec<DocumentLookupDto>, DocumentError> {
        let mut results = Vec::with_capacity(document_ids.len());

        for doc_id in document_ids {
            let lookup = self.get_single_document(&doc_id).await;
            results.push(lookup);
        }

        Ok(results)
    }

    async fn get_single_document(&self, document_id: &str) -> DocumentLookupDto {
        // 1. 查询文档元数据
        let doc_id = DocumentId(document_id.to_owned());
        let document = match self.document_repo.get(&doc_id).await {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                return DocumentLookupDto {
                    document_id: document_id.to_owned(),
                    status: DocumentLookupStatus::DocumentMissing,
                    document: None,
                };
            }
            Err(_) => {
                return DocumentLookupDto {
                    document_id: document_id.to_owned(),
                    status: DocumentLookupStatus::DocumentMissing,
                    document: None,
                };
            }
        };

        // 2. 查询数据源
        let source = match self.source_repo.get(&document.source_id).await {
            Ok(Some(src)) => src,
            Ok(None) => {
                return DocumentLookupDto {
                    document_id: document_id.to_owned(),
                    status: DocumentLookupStatus::SourceMissing,
                    document: None,
                };
            }
            Err(_) => {
                return DocumentLookupDto {
                    document_id: document_id.to_owned(),
                    status: DocumentLookupStatus::SourceMissing,
                    document: None,
                };
            }
        };

        // 3. 验证路径并检查文件是否存在
        let guard = match PathGuard::new(source.root_path.clone()) {
            Ok(g) => g,
            Err(_) => {
                return DocumentLookupDto {
                    document_id: document_id.to_owned(),
                    status: DocumentLookupStatus::PathInvalid,
                    document: None,
                };
            }
        };

        match guard.resolve_relative_file(&document.relative_path) {
            Ok(_) => {
                // 文件存在且路径有效
                DocumentLookupDto {
                    document_id: document_id.to_owned(),
                    status: DocumentLookupStatus::Found,
                    document: Some(DocumentDto::from(&document)),
                }
            }
            Err(devforge_domain::path_guard::PathError::NotFile(_)) => {
                // 路径有效但文件不存在
                DocumentLookupDto {
                    document_id: document_id.to_owned(),
                    status: DocumentLookupStatus::FileMissing,
                    document: Some(DocumentDto::from(&document)),
                }
            }
            Err(_) => {
                // 路径安全验证失败
                DocumentLookupDto {
                    document_id: document_id.to_owned(),
                    status: DocumentLookupStatus::PathInvalid,
                    document: None,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::DocumentRepository;
    use crate::source::SourceRepository;
    use devforge_domain::document::DocumentKind;
    use devforge_domain::source::Source;
    use devforge_domain::workspace::WorkspaceId;
    use std::collections::HashMap;
    use std::sync::Mutex;

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

        async fn list_by_source(&self, source_id: &SourceId) -> Result<Vec<Document>, DomainError> {
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
                        if let Some(parent_of) = path.parent() {
                            if parent_of.to_string_lossy() == parent {
                                result.push(doc.clone());
                            }
                        }
                    }
                    None => {
                        let components: Vec<_> = path.components().collect();
                        if components.len() == 1 {
                            result.push(doc.clone());
                        } else if components.len() > 1 {
                            let dir_name = components[0].as_os_str().to_string_lossy().to_string();
                            if !seen_dirs.contains(&dir_name) {
                                seen_dirs.insert(dir_name.clone());
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

        async fn list_file_tree(
            &self,
            source_id: &SourceId,
            parent_path: Option<&str>,
        ) -> Result<Vec<FileTreeEntry>, DomainError> {
            let docs = self.docs.lock().unwrap();
            let mut entries = Vec::new();
            let mut seen_dirs = std::collections::BTreeSet::new();

            for doc in docs.values() {
                if doc.source_id != *source_id {
                    continue;
                }

                let rel = &doc.relative_path;

                match parent_path {
                    Some(parent) => {
                        let parent_path = std::path::Path::new(parent);
                        if let Ok(rest) = rel.strip_prefix(parent_path) {
                            let components: Vec<_> = rest.components().collect();
                            if components.len() == 1 {
                                if let std::path::Component::Normal(name) = components[0] {
                                    let name = name.to_string_lossy().to_string();
                                    entries.push(FileTreeEntry {
                                        key: FileTreeEntry::file_key(&doc.id),
                                        source_id: doc.source_id.clone(),
                                        relative_path: rel.clone(),
                                        name,
                                        entry_kind: FileTreeEntryKind::File,
                                        document: Some(doc.clone()),
                                    });
                                }
                            } else if components.len() > 1 {
                                if let std::path::Component::Normal(dir_name) = components[0] {
                                    let dir_name = dir_name.to_string_lossy().to_string();
                                    let dir_rel = std::path::PathBuf::from(parent).join(&dir_name);
                                    let dir_key = FileTreeEntry::directory_key(source_id, &dir_rel);
                                    if seen_dirs.insert(dir_key.clone()) {
                                        entries.push(FileTreeEntry {
                                            key: dir_key,
                                            source_id: source_id.clone(),
                                            relative_path: dir_rel,
                                            name: dir_name,
                                            entry_kind: FileTreeEntryKind::Directory,
                                            document: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        let components: Vec<_> = rel.components().collect();
                        if components.len() == 1 {
                            let name = rel.to_string_lossy().to_string();
                            entries.push(FileTreeEntry {
                                key: FileTreeEntry::file_key(&doc.id),
                                source_id: doc.source_id.clone(),
                                relative_path: rel.clone(),
                                name,
                                entry_kind: FileTreeEntryKind::File,
                                document: Some(doc.clone()),
                            });
                        } else if components.len() > 1 {
                            let dir_name = components[0].as_os_str().to_string_lossy().to_string();
                            let dir_rel = std::path::PathBuf::from(&dir_name);
                            let dir_key = FileTreeEntry::directory_key(source_id, &dir_rel);
                            if seen_dirs.insert(dir_key.clone()) {
                                entries.push(FileTreeEntry {
                                    key: dir_key,
                                    source_id: source_id.clone(),
                                    relative_path: dir_rel,
                                    name: dir_name,
                                    entry_kind: FileTreeEntryKind::Directory,
                                    document: None,
                                });
                            }
                        }
                    }
                }
            }

            entries.sort_by(|a, b| match (&a.entry_kind, &b.entry_kind) {
                (FileTreeEntryKind::Directory, FileTreeEntryKind::File) => std::cmp::Ordering::Less,
                (FileTreeEntryKind::File, FileTreeEntryKind::Directory) => {
                    std::cmp::Ordering::Greater
                }
                _ => a
                    .name
                    .to_lowercase()
                    .cmp(&b.name.to_lowercase())
                    .then_with(|| a.name.cmp(&b.name)),
            });

            Ok(entries)
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
    }

    struct InMemorySourceRepository {
        sources: Mutex<HashMap<String, Source>>,
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
        async fn create(&self, source: &Source) -> Result<(), DomainError> {
            let mut sources = self.sources.lock().unwrap();
            sources.insert(source.id.0.clone(), source.clone());
            Ok(())
        }

        async fn get(&self, id: &SourceId) -> Result<Option<Source>, DomainError> {
            let sources = self.sources.lock().unwrap();
            Ok(sources.get(&id.0).cloned())
        }

        async fn list_by_workspace(
            &self,
            workspace_id: &WorkspaceId,
        ) -> Result<Vec<Source>, DomainError> {
            let sources = self.sources.lock().unwrap();
            Ok(sources
                .values()
                .filter(|s| s.workspace_id == *workspace_id)
                .cloned()
                .collect())
        }

        async fn delete(&self, id: &SourceId) -> Result<(), DomainError> {
            let mut sources = self.sources.lock().unwrap();
            sources.remove(&id.0);
            Ok(())
        }
    }

    #[tokio::test]
    async fn list_documents() {
        let repo = Arc::new(InMemoryDocumentRepository::new());
        let source_id = SourceId("source-1".to_owned());

        // 创建测试文档
        let doc1 = Document::new(
            source_id.clone(),
            PathBuf::from("src/main.rs"),
            1024,
            chrono::Utc::now(),
        );
        let doc2 = Document::new(
            source_id.clone(),
            PathBuf::from("README.md"),
            512,
            chrono::Utc::now(),
        );

        repo.create(&doc1).await.unwrap();
        repo.create(&doc2).await.unwrap();

        let use_case = ListDocuments::new(repo.clone());
        let result = use_case.execute("source-1".to_owned(), None).await.unwrap();

        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn read_document_content() {
        let temp = tempfile::tempdir().unwrap();
        let source_root = temp.path().to_path_buf();

        // 创建测试文件
        let file_path = source_root.join("test.txt");
        std::fs::write(&file_path, "Hello, World!").unwrap();

        let doc_repo = Arc::new(InMemoryDocumentRepository::new());
        let source_repo = Arc::new(InMemorySourceRepository::new());
        let source_id = SourceId("source-1".to_owned());

        // 创建数据源
        let source =
            Source::new_directory(WorkspaceId::new(), "test".to_owned(), source_root.clone());
        source_repo.create(&source).await.unwrap();

        let doc = Document::new(
            source_id.clone(),
            PathBuf::from("test.txt"),
            13,
            chrono::Utc::now(),
        );

        doc_repo.create(&doc).await.unwrap();

        // 更新文档的 source_id 为实际的 source id
        let mut doc_with_source = doc.clone();
        doc_with_source.source_id = source.id.clone();
        doc_repo.upsert(&doc_with_source).await.unwrap();

        let use_case = ReadDocumentContent::new(doc_repo.clone(), source_repo.clone());
        let content = use_case
            .execute(doc_with_source.id.0.clone())
            .await
            .unwrap();

        assert_eq!(content, "Hello, World!");
    }

    #[tokio::test]
    async fn read_sensitive_file_fails() {
        let temp = tempfile::tempdir().unwrap();
        let source_root = temp.path().to_path_buf();

        // 创建敏感文件
        let file_path = source_root.join(".env");
        std::fs::write(&file_path, "SECRET=value").unwrap();

        let doc_repo = Arc::new(InMemoryDocumentRepository::new());
        let source_repo = Arc::new(InMemorySourceRepository::new());

        // 创建数据源
        let source =
            Source::new_directory(WorkspaceId::new(), "test".to_owned(), source_root.clone());
        source_repo.create(&source).await.unwrap();

        let doc = Document::new(
            source.id.clone(),
            PathBuf::from(".env"),
            12,
            chrono::Utc::now(),
        );

        doc_repo.create(&doc).await.unwrap();

        let use_case = ReadDocumentContent::new(doc_repo.clone(), source_repo.clone());
        let result = use_case.execute(doc.id.0.clone()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn read_content_source_not_found() {
        let doc_repo = Arc::new(InMemoryDocumentRepository::new());
        let source_repo = Arc::new(InMemorySourceRepository::new());

        let doc = Document::new(
            SourceId("nonexistent".to_owned()),
            PathBuf::from("test.txt"),
            100,
            chrono::Utc::now(),
        );

        doc_repo.create(&doc).await.unwrap();

        let use_case = ReadDocumentContent::new(doc_repo.clone(), source_repo.clone());
        let result = use_case.execute(doc.id.0.clone()).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DocumentError::SourceNotFound));
    }

    #[tokio::test]
    async fn get_documents_by_ids_found() {
        let temp = tempfile::tempdir().unwrap();
        let source_root = temp.path().to_path_buf();

        // 创建测试文件
        std::fs::write(source_root.join("test.txt"), "content").unwrap();

        let doc_repo = Arc::new(InMemoryDocumentRepository::new());
        let source_repo = Arc::new(InMemorySourceRepository::new());

        let source = Source::new_directory(WorkspaceId::new(), "test".to_owned(), source_root);
        source_repo.create(&source).await.unwrap();

        let doc = Document::new(
            source.id.clone(),
            PathBuf::from("test.txt"),
            100,
            chrono::Utc::now(),
        );
        doc_repo.create(&doc).await.unwrap();

        let use_case = GetDocumentsByIds::new(doc_repo.clone(), source_repo.clone());
        let results = use_case.execute(vec![doc.id.0.clone()]).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, DocumentLookupStatus::Found);
        assert!(results[0].document.is_some());
    }

    #[tokio::test]
    async fn get_documents_by_ids_document_missing() {
        let doc_repo = Arc::new(InMemoryDocumentRepository::new());
        let source_repo = Arc::new(InMemorySourceRepository::new());

        let use_case = GetDocumentsByIds::new(doc_repo.clone(), source_repo.clone());
        let results = use_case
            .execute(vec!["nonexistent".to_owned()])
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, DocumentLookupStatus::DocumentMissing);
    }

    #[tokio::test]
    async fn get_documents_by_ids_source_missing() {
        let doc_repo = Arc::new(InMemoryDocumentRepository::new());
        let source_repo = Arc::new(InMemorySourceRepository::new());

        let doc = Document::new(
            SourceId("nonexistent".to_owned()),
            PathBuf::from("test.txt"),
            100,
            chrono::Utc::now(),
        );
        doc_repo.create(&doc).await.unwrap();

        let use_case = GetDocumentsByIds::new(doc_repo.clone(), source_repo.clone());
        let results = use_case.execute(vec![doc.id.0.clone()]).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, DocumentLookupStatus::SourceMissing);
    }

    #[tokio::test]
    async fn get_documents_by_ids_mixed() {
        let temp = tempfile::tempdir().unwrap();
        let source_root = temp.path().to_path_buf();

        std::fs::write(source_root.join("exists.txt"), "content").unwrap();

        let doc_repo = Arc::new(InMemoryDocumentRepository::new());
        let source_repo = Arc::new(InMemorySourceRepository::new());

        let source = Source::new_directory(WorkspaceId::new(), "test".to_owned(), source_root);
        source_repo.create(&source).await.unwrap();

        let doc_exists = Document::new(
            source.id.clone(),
            PathBuf::from("exists.txt"),
            100,
            chrono::Utc::now(),
        );
        let doc_missing = Document::new(
            source.id.clone(),
            PathBuf::from("missing.txt"),
            100,
            chrono::Utc::now(),
        );
        doc_repo.create(&doc_exists).await.unwrap();
        doc_repo.create(&doc_missing).await.unwrap();

        let use_case = GetDocumentsByIds::new(doc_repo.clone(), source_repo.clone());
        let results = use_case
            .execute(vec![
                doc_exists.id.0.clone(),
                doc_missing.id.0.clone(),
                "nonexistent".to_owned(),
            ])
            .await
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].status, DocumentLookupStatus::Found);
        assert_eq!(results[1].status, DocumentLookupStatus::FileMissing);
        assert_eq!(results[2].status, DocumentLookupStatus::DocumentMissing);
    }
}
