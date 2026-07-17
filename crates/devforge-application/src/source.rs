//! 数据源用例

use std::path::PathBuf;
use std::sync::Arc;

use devforge_domain::error::DomainError;
use devforge_domain::path_guard::PathError;
use devforge_domain::source::{Source, SourceId};
use devforge_domain::workspace::WorkspaceId;

/// Source Repository Trait（应用层端口）
#[async_trait::async_trait]
pub trait SourceRepository: Send + Sync {
    async fn create(&self, source: &Source) -> Result<(), DomainError>;
    async fn get(&self, id: &SourceId) -> Result<Option<Source>, DomainError>;
    async fn list_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<Source>, DomainError>;
    async fn delete(&self, id: &SourceId) -> Result<(), DomainError>;
}

/// 应用层错误
#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum SourceError {
    #[error("路径错误: {0}")]
    Path(String),
    #[error("数据源不存在")]
    SourceNotFound,
    #[error("路径已存在")]
    PathAlreadyExists,
    #[error("路径不存在: {0}")]
    PathNotExists(String),
    #[error("路径不是目录: {0}")]
    NotDirectory(String),
    #[error("无权限访问: {0}")]
    PermissionDenied(String),
    #[error("不是 Git 仓库: {0}")]
    NotGitRepository(String),
    #[error("领域错误: {0}")]
    Domain(String),
}

impl From<DomainError> for SourceError {
    fn from(err: DomainError) -> Self {
        SourceError::Domain(err.to_string())
    }
}

impl From<PathError> for SourceError {
    fn from(err: PathError) -> Self {
        SourceError::Path(err.to_string())
    }
}

/// 添加 Git 数据源用例
pub struct AddGitSource {
    source_repo: Arc<dyn SourceRepository>,
}

impl AddGitSource {
    pub fn new(source_repo: Arc<dyn SourceRepository>) -> Self {
        Self { source_repo }
    }

    pub async fn execute(
        &self,
        workspace_id: String,
        path: PathBuf,
    ) -> Result<Source, SourceError> {
        let workspace_id = WorkspaceId(workspace_id);

        // 验证路径存在且是目录
        if !path.exists() {
            return Err(SourceError::PathNotExists(path.display().to_string()));
        }
        if !path.is_dir() {
            return Err(SourceError::NotDirectory(path.display().to_string()));
        }

        // 验证是 Git 仓库
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            return Err(SourceError::NotGitRepository(path.display().to_string()));
        }

        // 检查是否已存在
        let existing = self.source_repo.list_by_workspace(&workspace_id).await?;
        for src in &existing {
            if src.root_path == path {
                return Err(SourceError::PathAlreadyExists);
            }
        }

        // 从路径提取名称
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "未命名".to_owned());

        let source = Source::new_git(workspace_id, name, path);
        self.source_repo.create(&source).await?;
        Ok(source)
    }
}

/// 添加目录数据源用例
pub struct AddDirectorySource {
    source_repo: Arc<dyn SourceRepository>,
}

impl AddDirectorySource {
    pub fn new(source_repo: Arc<dyn SourceRepository>) -> Self {
        Self { source_repo }
    }

    pub async fn execute(
        &self,
        workspace_id: String,
        path: PathBuf,
    ) -> Result<Source, SourceError> {
        let workspace_id = WorkspaceId(workspace_id);

        // 验证路径存在且是目录
        if !path.exists() {
            return Err(SourceError::PathNotExists(path.display().to_string()));
        }
        if !path.is_dir() {
            return Err(SourceError::NotDirectory(path.display().to_string()));
        }

        // 检查是否已存在
        let existing = self.source_repo.list_by_workspace(&workspace_id).await?;
        for src in &existing {
            if src.root_path == path {
                return Err(SourceError::PathAlreadyExists);
            }
        }

        // 从路径提取名称
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "未命名".to_owned());

        let source = Source::new_directory(workspace_id, name, path);
        self.source_repo.create(&source).await?;
        Ok(source)
    }
}

/// 列出数据源用例
pub struct ListSources {
    source_repo: Arc<dyn SourceRepository>,
}

impl ListSources {
    pub fn new(source_repo: Arc<dyn SourceRepository>) -> Self {
        Self { source_repo }
    }

    pub async fn execute(&self, workspace_id: String) -> Result<Vec<Source>, SourceError> {
        let workspace_id = WorkspaceId(workspace_id);
        Ok(self.source_repo.list_by_workspace(&workspace_id).await?)
    }
}

/// 移除数据源用例
///
/// 注意：移除数据源只删除元数据，不删除源目录。
pub struct RemoveSource {
    source_repo: Arc<dyn SourceRepository>,
}

impl RemoveSource {
    pub fn new(source_repo: Arc<dyn SourceRepository>) -> Self {
        Self { source_repo }
    }

    pub async fn execute(&self, id: String) -> Result<(), SourceError> {
        let source_id = SourceId(id);
        // 验证数据源存在
        self.source_repo
            .get(&source_id)
            .await?
            .ok_or(SourceError::SourceNotFound)?;

        self.source_repo.delete(&source_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

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
    async fn add_git_source() {
        let temp = tempfile::tempdir().unwrap();
        let git_dir = temp.path().join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();

        let repo = Arc::new(InMemorySourceRepository::new());
        let use_case = AddGitSource::new(repo.clone());

        let source = use_case
            .execute("workspace-1".to_owned(), temp.path().to_path_buf())
            .await
            .unwrap();

        assert_eq!(source.kind, devforge_domain::source::SourceKind::Git);
    }

    #[tokio::test]
    async fn add_directory_source() {
        let temp = tempfile::tempdir().unwrap();

        let repo = Arc::new(InMemorySourceRepository::new());
        let use_case = AddDirectorySource::new(repo.clone());

        let source = use_case
            .execute("workspace-1".to_owned(), temp.path().to_path_buf())
            .await
            .unwrap();

        assert_eq!(source.kind, devforge_domain::source::SourceKind::Directory);
    }

    #[tokio::test]
    async fn add_git_source_not_git_repo_fails() {
        let temp = tempfile::tempdir().unwrap();

        let repo = Arc::new(InMemorySourceRepository::new());
        let use_case = AddGitSource::new(repo.clone());

        let result = use_case
            .execute("workspace-1".to_owned(), temp.path().to_path_buf())
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn add_source_not_exists_fails() {
        let repo = Arc::new(InMemorySourceRepository::new());
        let use_case = AddDirectorySource::new(repo.clone());

        let result = use_case
            .execute("workspace-1".to_owned(), PathBuf::from("/nonexistent/path"))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn add_duplicate_source_fails() {
        let temp = tempfile::tempdir().unwrap();

        let repo = Arc::new(InMemorySourceRepository::new());
        let use_case = AddDirectorySource::new(repo.clone());

        // 第一次添加成功
        use_case
            .execute("workspace-1".to_owned(), temp.path().to_path_buf())
            .await
            .unwrap();

        // 第二次添加失败
        let result = use_case
            .execute("workspace-1".to_owned(), temp.path().to_path_buf())
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_sources() {
        let temp1 = tempfile::tempdir().unwrap();
        let temp2 = tempfile::tempdir().unwrap();

        let repo = Arc::new(InMemorySourceRepository::new());
        let add = AddDirectorySource::new(repo.clone());
        let list = ListSources::new(repo.clone());

        add.execute("workspace-1".to_owned(), temp1.path().to_path_buf())
            .await
            .unwrap();
        add.execute("workspace-1".to_owned(), temp2.path().to_path_buf())
            .await
            .unwrap();

        let sources = list.execute("workspace-1".to_owned()).await.unwrap();
        assert_eq!(sources.len(), 2);
    }

    #[tokio::test]
    async fn remove_source() {
        let temp = tempfile::tempdir().unwrap();

        let repo = Arc::new(InMemorySourceRepository::new());
        let add = AddDirectorySource::new(repo.clone());
        let remove = RemoveSource::new(repo.clone());
        let list = ListSources::new(repo.clone());

        let source = add
            .execute("workspace-1".to_owned(), temp.path().to_path_buf())
            .await
            .unwrap();

        remove.execute(source.id.0.clone()).await.unwrap();

        let sources = list.execute("workspace-1".to_owned()).await.unwrap();
        assert_eq!(sources.len(), 0);
    }
}
