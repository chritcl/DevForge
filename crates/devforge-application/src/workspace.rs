//! 工作区用例

use std::sync::Arc;

use devforge_domain::error::DomainError;
use devforge_domain::workspace::{Workspace, WorkspaceId, WorkspaceStatus};

/// 工作区 DTO（用于 IPC 传输）
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct WorkspaceDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: WorkspaceStatus,
    pub created_at: String,
    pub updated_at: String,
    pub last_opened_at: Option<String>,
}

impl From<&Workspace> for WorkspaceDto {
    fn from(workspace: &Workspace) -> Self {
        Self {
            id: workspace.id.0.clone(),
            name: workspace.name.clone(),
            description: workspace.description.clone(),
            status: workspace.status.clone(),
            created_at: workspace.created_at.to_rfc3339(),
            updated_at: workspace.updated_at.to_rfc3339(),
            last_opened_at: workspace.last_opened_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// 工作区 Repository Trait（应用层端口）
#[async_trait::async_trait]
pub trait WorkspaceRepository: Send + Sync {
    async fn create(&self, workspace: &Workspace) -> Result<(), DomainError>;
    async fn get(&self, id: &WorkspaceId) -> Result<Option<Workspace>, DomainError>;
    async fn list(&self) -> Result<Vec<Workspace>, DomainError>;
    async fn update(&self, workspace: &Workspace) -> Result<(), DomainError>;
    async fn delete(&self, id: &WorkspaceId) -> Result<(), DomainError>;
}

/// 应用层错误
#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum AppError {
    #[error("领域错误: {0}")]
    Domain(String),
    #[error("工作区不存在")]
    WorkspaceNotFound,
    #[error("工作区名称已存在")]
    DuplicateName,
}

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        AppError::Domain(err.to_string())
    }
}

/// 创建工作区用例
pub struct CreateWorkspace {
    repo: Arc<dyn WorkspaceRepository>,
}

impl CreateWorkspace {
    pub fn new(repo: Arc<dyn WorkspaceRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<WorkspaceDto, AppError> {
        let workspace = Workspace::new(name, description)?;
        self.repo.create(&workspace).await?;
        Ok(WorkspaceDto::from(&workspace))
    }
}

/// 获取工作区用例
pub struct GetWorkspace {
    repo: Arc<dyn WorkspaceRepository>,
}

impl GetWorkspace {
    pub fn new(repo: Arc<dyn WorkspaceRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: String) -> Result<WorkspaceDto, AppError> {
        let workspace_id = WorkspaceId(id);
        let workspace = self
            .repo
            .get(&workspace_id)
            .await?
            .ok_or(AppError::WorkspaceNotFound)?;
        Ok(WorkspaceDto::from(&workspace))
    }
}

/// 列出工作区用例
pub struct ListWorkspaces {
    repo: Arc<dyn WorkspaceRepository>,
}

impl ListWorkspaces {
    pub fn new(repo: Arc<dyn WorkspaceRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<WorkspaceDto>, AppError> {
        let workspaces = self.repo.list().await?;
        Ok(workspaces.iter().map(WorkspaceDto::from).collect())
    }
}

/// 更新工作区用例
pub struct UpdateWorkspace {
    repo: Arc<dyn WorkspaceRepository>,
}

impl UpdateWorkspace {
    pub fn new(repo: Arc<dyn WorkspaceRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        id: String,
        name: Option<String>,
        description: Option<Option<String>>,
    ) -> Result<WorkspaceDto, AppError> {
        let workspace_id = WorkspaceId(id);
        let mut workspace = self
            .repo
            .get(&workspace_id)
            .await?
            .ok_or(AppError::WorkspaceNotFound)?;

        if let Some(new_name) = name {
            workspace.update_name(new_name)?;
        }

        if let Some(new_desc) = description {
            workspace.update_description(new_desc);
        }

        self.repo.update(&workspace).await?;
        Ok(WorkspaceDto::from(&workspace))
    }
}

/// 归档工作区用例
pub struct ArchiveWorkspace {
    repo: Arc<dyn WorkspaceRepository>,
}

impl ArchiveWorkspace {
    pub fn new(repo: Arc<dyn WorkspaceRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: String) -> Result<(), AppError> {
        let workspace_id = WorkspaceId(id);
        let mut workspace = self
            .repo
            .get(&workspace_id)
            .await?
            .ok_or(AppError::WorkspaceNotFound)?;

        workspace.archive();
        self.repo.update(&workspace).await?;
        Ok(())
    }
}

/// 恢复工作区用例
pub struct RestoreWorkspace {
    repo: Arc<dyn WorkspaceRepository>,
}

impl RestoreWorkspace {
    pub fn new(repo: Arc<dyn WorkspaceRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: String) -> Result<(), AppError> {
        let workspace_id = WorkspaceId(id);
        let mut workspace = self
            .repo
            .get(&workspace_id)
            .await?
            .ok_or(AppError::WorkspaceNotFound)?;

        workspace.restore();
        self.repo.update(&workspace).await?;
        Ok(())
    }
}

/// 删除工作区用例
pub struct DeleteWorkspace {
    repo: Arc<dyn WorkspaceRepository>,
}

impl DeleteWorkspace {
    pub fn new(repo: Arc<dyn WorkspaceRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: String) -> Result<(), AppError> {
        let workspace_id = WorkspaceId(id);
        // 验证工作区存在
        self.repo
            .get(&workspace_id)
            .await?
            .ok_or(AppError::WorkspaceNotFound)?;

        self.repo.delete(&workspace_id).await?;
        Ok(())
    }
}

/// 标记工作区已打开用例
pub struct MarkWorkspaceOpened {
    repo: Arc<dyn WorkspaceRepository>,
}

impl MarkWorkspaceOpened {
    pub fn new(repo: Arc<dyn WorkspaceRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: String) -> Result<(), AppError> {
        let workspace_id = WorkspaceId(id);
        let mut workspace = self
            .repo
            .get(&workspace_id)
            .await?
            .ok_or(AppError::WorkspaceNotFound)?;

        workspace.mark_opened();
        self.repo.update(&workspace).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// 内存中的工作区 Repository（用于测试）
    struct InMemoryWorkspaceRepository {
        workspaces: std::sync::Mutex<HashMap<String, Workspace>>,
    }

    impl InMemoryWorkspaceRepository {
        fn new() -> Self {
            Self {
                workspaces: std::sync::Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl WorkspaceRepository for InMemoryWorkspaceRepository {
        async fn create(&self, workspace: &Workspace) -> Result<(), DomainError> {
            let mut workspaces = self.workspaces.lock().unwrap();
            workspaces.insert(workspace.id.0.clone(), workspace.clone());
            Ok(())
        }

        async fn get(&self, id: &WorkspaceId) -> Result<Option<Workspace>, DomainError> {
            let workspaces = self.workspaces.lock().unwrap();
            Ok(workspaces.get(&id.0).cloned())
        }

        async fn list(&self) -> Result<Vec<Workspace>, DomainError> {
            let workspaces = self.workspaces.lock().unwrap();
            let mut list: Vec<Workspace> = workspaces.values().cloned().collect();
            list.sort_by_key(|b| std::cmp::Reverse(b.updated_at));
            Ok(list)
        }

        async fn update(&self, workspace: &Workspace) -> Result<(), DomainError> {
            let mut workspaces = self.workspaces.lock().unwrap();
            workspaces.insert(workspace.id.0.clone(), workspace.clone());
            Ok(())
        }

        async fn delete(&self, id: &WorkspaceId) -> Result<(), DomainError> {
            let mut workspaces = self.workspaces.lock().unwrap();
            workspaces.remove(&id.0);
            Ok(())
        }
    }

    #[tokio::test]
    async fn create_workspace() {
        let repo = Arc::new(InMemoryWorkspaceRepository::new());
        let use_case = CreateWorkspace::new(repo.clone());

        let workspace = use_case
            .execute("测试工作区".to_owned(), Some("描述".to_owned()))
            .await
            .unwrap();

        assert_eq!(workspace.name, "测试工作区");
        assert_eq!(workspace.description, Some("描述".to_owned()));
    }

    #[tokio::test]
    async fn get_workspace() {
        let repo = Arc::new(InMemoryWorkspaceRepository::new());
        let create = CreateWorkspace::new(repo.clone());
        let get = GetWorkspace::new(repo.clone());

        let workspace = create.execute("测试".to_owned(), None).await.unwrap();

        let fetched = get.execute(workspace.id.clone()).await.unwrap();
        assert_eq!(fetched.name, "测试");
    }

    #[tokio::test]
    async fn list_workspaces() {
        let repo = Arc::new(InMemoryWorkspaceRepository::new());
        let create = CreateWorkspace::new(repo.clone());
        let list = ListWorkspaces::new(repo.clone());

        create.execute("工作区1".to_owned(), None).await.unwrap();
        create.execute("工作区2".to_owned(), None).await.unwrap();

        let workspaces = list.execute().await.unwrap();
        assert_eq!(workspaces.len(), 2);
    }

    #[tokio::test]
    async fn update_workspace() {
        let repo = Arc::new(InMemoryWorkspaceRepository::new());
        let create = CreateWorkspace::new(repo.clone());
        let update = UpdateWorkspace::new(repo.clone());

        let workspace = create.execute("旧名称".to_owned(), None).await.unwrap();

        let updated = update
            .execute(
                workspace.id.clone(),
                Some("新名称".to_owned()),
                Some(Some("新描述".to_owned())),
            )
            .await
            .unwrap();

        assert_eq!(updated.name, "新名称");
        assert_eq!(updated.description, Some("新描述".to_owned()));
    }

    #[tokio::test]
    async fn archive_and_restore_workspace() {
        let repo = Arc::new(InMemoryWorkspaceRepository::new());
        let create = CreateWorkspace::new(repo.clone());
        let archive = ArchiveWorkspace::new(repo.clone());
        let restore = RestoreWorkspace::new(repo.clone());
        let get = GetWorkspace::new(repo.clone());

        let workspace = create.execute("测试".to_owned(), None).await.unwrap();

        archive.execute(workspace.id.clone()).await.unwrap();
        let archived = get.execute(workspace.id.clone()).await.unwrap();
        assert_eq!(
            archived.status,
            devforge_domain::workspace::WorkspaceStatus::Archived
        );

        restore.execute(workspace.id.clone()).await.unwrap();
        let restored = get.execute(workspace.id.clone()).await.unwrap();
        assert_eq!(
            restored.status,
            devforge_domain::workspace::WorkspaceStatus::Active
        );
    }

    #[tokio::test]
    async fn delete_workspace() {
        let repo = Arc::new(InMemoryWorkspaceRepository::new());
        let create = CreateWorkspace::new(repo.clone());
        let delete = DeleteWorkspace::new(repo.clone());
        let get = GetWorkspace::new(repo.clone());

        let workspace = create.execute("测试".to_owned(), None).await.unwrap();

        delete.execute(workspace.id.clone()).await.unwrap();

        let result = get.execute(workspace.id.clone()).await;
        assert!(result.is_err());
    }
}
