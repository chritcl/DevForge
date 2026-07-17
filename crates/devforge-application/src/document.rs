//! 文档查询用例

use std::path::PathBuf;
use std::sync::Arc;

use devforge_domain::document::{Document, DocumentId, Sensitivity};
use devforge_domain::error::DomainError;
use devforge_domain::path_guard::PathGuard;
use devforge_domain::source::SourceId;

/// 文档查询错误
#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum DocumentError {
    #[error("文档不存在")]
    DocumentNotFound,
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
    pub kind: String,
    pub size: u64,
    pub sensitivity: String,
    pub content_readable: bool,
}

impl From<&Document> for DocumentDto {
    fn from(doc: &Document) -> Self {
        Self {
            id: doc.id.0.clone(),
            source_id: doc.source_id.0.clone(),
            relative_path: doc.relative_path.to_string_lossy().to_string(),
            kind: doc.kind.to_string(),
            size: doc.size,
            sensitivity: doc.sensitivity.to_string(),
            content_readable: doc.content_readable,
        }
    }
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

/// 读取文档内容用例
pub struct ReadDocumentContent {
    document_repo: Arc<dyn crate::discovery::DocumentRepository>,
}

impl ReadDocumentContent {
    pub fn new(document_repo: Arc<dyn crate::discovery::DocumentRepository>) -> Self {
        Self { document_repo }
    }

    pub async fn execute(
        &self,
        document_id: String,
        source_root: PathBuf,
    ) -> Result<String, DocumentError> {
        let document_id = DocumentId(document_id);

        // 获取文档元数据
        let document = self
            .document_repo
            .get(&document_id)
            .await?
            .ok_or(DocumentError::DocumentNotFound)?;

        // 检查是否可读
        if !document.content_readable {
            if document.sensitivity == Sensitivity::Sensitive {
                return Err(DocumentError::SensitiveFile);
            }
            return Err(DocumentError::Io("文件不可读".to_owned()));
        }

        // 检查文件大小（最大 10MB）
        if document.size > 10 * 1024 * 1024 {
            return Err(DocumentError::FileTooLarge);
        }

        // 验证路径安全
        let guard = PathGuard::new(source_root).map_err(|e| DocumentError::Path(e.to_string()))?;

        let full_path = guard
            .resolve(&document.relative_path)
            .canonicalize()
            .map_err(|e| DocumentError::Path(e.to_string()))?;

        guard
            .validate(&full_path)
            .map_err(|e| DocumentError::Path(e.to_string()))?;

        // 读取文件内容
        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| DocumentError::Io(format!("无法读取文件: {e}")))?;

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::DocumentRepository;
    use devforge_domain::document::DocumentKind;
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
    impl crate::discovery::DocumentRepository for InMemoryDocumentRepository {
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
        let source_root = temp.path();

        // 创建测试文件
        let file_path = source_root.join("test.txt");
        std::fs::write(&file_path, "Hello, World!").unwrap();

        let repo = Arc::new(InMemoryDocumentRepository::new());
        let source_id = SourceId("source-1".to_owned());

        let doc = Document::new(
            source_id.clone(),
            PathBuf::from("test.txt"),
            13,
            chrono::Utc::now(),
        );

        repo.create(&doc).await.unwrap();

        let use_case = ReadDocumentContent::new(repo.clone());
        let content = use_case
            .execute(doc.id.0.clone(), source_root.to_path_buf())
            .await
            .unwrap();

        assert_eq!(content, "Hello, World!");
    }

    #[tokio::test]
    async fn read_sensitive_file_fails() {
        let temp = tempfile::tempdir().unwrap();
        let source_root = temp.path();

        // 创建敏感文件
        let file_path = source_root.join(".env");
        std::fs::write(&file_path, "SECRET=value").unwrap();

        let repo = Arc::new(InMemoryDocumentRepository::new());
        let source_id = SourceId("source-1".to_owned());

        let doc = Document::new(
            source_id.clone(),
            PathBuf::from(".env"),
            12,
            chrono::Utc::now(),
        );

        repo.create(&doc).await.unwrap();

        let use_case = ReadDocumentContent::new(repo.clone());
        let result = use_case
            .execute(doc.id.0.clone(), source_root.to_path_buf())
            .await;

        assert!(result.is_err());
    }
}
