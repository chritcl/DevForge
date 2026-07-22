//! 标签页用例

use std::sync::Arc;

use devforge_domain::document::DocumentId;
use devforge_domain::error::DomainError;
use devforge_domain::opentab::OpenTab as OpenTabEntity;
use devforge_domain::workspace::WorkspaceId;

/// 标签页 Repository Trait（应用层端口）
#[async_trait::async_trait]
pub trait TabRepository: Send + Sync {
    async fn create(&self, tab: &OpenTabEntity) -> Result<(), DomainError>;
    async fn get(&self, id: &str) -> Result<Option<OpenTabEntity>, DomainError>;
    async fn list_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<OpenTabEntity>, DomainError>;
    async fn set_active(&self, workspace_id: &WorkspaceId, tab_id: &str)
        -> Result<(), DomainError>;
    async fn delete(&self, id: &str) -> Result<(), DomainError>;
    async fn delete_by_workspace(&self, workspace_id: &WorkspaceId) -> Result<(), DomainError>;
}

/// 标签页 DTO
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct TabDto {
    pub id: String,
    pub workspace_id: String,
    pub document_id: String,
    pub position: i32,
    pub is_active: bool,
    pub opened_at: String,
}

impl From<&OpenTabEntity> for TabDto {
    fn from(tab: &OpenTabEntity) -> Self {
        Self {
            id: tab.id.clone(),
            workspace_id: tab.workspace_id.0.clone(),
            document_id: tab.document_id.0.clone(),
            position: tab.position,
            is_active: tab.is_active,
            opened_at: tab.opened_at.to_rfc3339(),
        }
    }
}

/// 标签页错误
#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum TabError {
    #[error("标签页不存在")]
    TabNotFound,
    #[error("领域错误: {0}")]
    Domain(String),
}

impl From<DomainError> for TabError {
    fn from(err: DomainError) -> Self {
        TabError::Domain(err.to_string())
    }
}

/// 打开标签页用例
pub struct OpenTab {
    tab_repo: Arc<dyn TabRepository>,
}

impl OpenTab {
    pub fn new(tab_repo: Arc<dyn TabRepository>) -> Self {
        Self { tab_repo }
    }

    pub async fn execute(
        &self,
        workspace_id: String,
        document_id: String,
    ) -> Result<TabDto, TabError> {
        let workspace_id = WorkspaceId(workspace_id);
        let document_id = DocumentId(document_id);

        // 检查是否已经有相同文档的标签
        let existing_tabs = self.tab_repo.list_by_workspace(&workspace_id).await?;
        for tab in &existing_tabs {
            if tab.document_id == document_id {
                // 已存在，设为活动标签并返回
                self.tab_repo.set_active(&workspace_id, &tab.id).await?;
                return Ok(TabDto::from(tab));
            }
        }

        // 创建新标签
        let position = existing_tabs.len() as i32;
        let mut tab = OpenTabEntity::new(workspace_id.clone(), document_id, position);

        self.tab_repo.create(&tab).await?;

        // 设置新标签为活动标签（会自动将其他标签设为非活动）
        self.tab_repo.set_active(&workspace_id, &tab.id).await?;

        // 更新本地标签状态
        tab.set_active(true);

        Ok(TabDto::from(&tab))
    }
}

/// 关闭标签页用例
pub struct CloseTab {
    tab_repo: Arc<dyn TabRepository>,
}

impl CloseTab {
    pub fn new(tab_repo: Arc<dyn TabRepository>) -> Self {
        Self { tab_repo }
    }

    pub async fn execute(&self, id: String) -> Result<(), TabError> {
        // 查询被删除标签的信息，不存在则报错
        let tab = self.tab_repo.get(&id).await?.ok_or(TabError::TabNotFound)?;
        let is_active = tab.is_active;
        let workspace_id = tab.workspace_id;

        self.tab_repo.delete(&id).await?;

        // 如果删除的是活动标签，自动选择下一个活动标签
        if is_active {
            let remaining = self.tab_repo.list_by_workspace(&workspace_id).await?;
            if let Some(next_tab) = remaining.first() {
                self.tab_repo
                    .set_active(&workspace_id, &next_tab.id)
                    .await?;
            }
        }

        Ok(())
    }
}

/// 列出标签页用例
pub struct ListTabs {
    tab_repo: Arc<dyn TabRepository>,
}

impl ListTabs {
    pub fn new(tab_repo: Arc<dyn TabRepository>) -> Self {
        Self { tab_repo }
    }

    pub async fn execute(&self, workspace_id: String) -> Result<Vec<TabDto>, TabError> {
        let workspace_id = WorkspaceId(workspace_id);
        let tabs = self.tab_repo.list_by_workspace(&workspace_id).await?;
        Ok(tabs.iter().map(TabDto::from).collect())
    }
}

/// 设置活动标签页用例
pub struct SetActiveTab {
    tab_repo: Arc<dyn TabRepository>,
}

impl SetActiveTab {
    pub fn new(tab_repo: Arc<dyn TabRepository>) -> Self {
        Self { tab_repo }
    }

    pub async fn execute(&self, workspace_id: String, tab_id: String) -> Result<(), TabError> {
        let workspace_id = WorkspaceId(workspace_id);
        self.tab_repo.set_active(&workspace_id, &tab_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct InMemoryTabRepository {
        tabs: Mutex<HashMap<String, OpenTabEntity>>,
    }

    impl InMemoryTabRepository {
        fn new() -> Self {
            Self {
                tabs: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl TabRepository for InMemoryTabRepository {
        async fn create(&self, tab: &OpenTabEntity) -> Result<(), DomainError> {
            let mut tabs = self.tabs.lock().unwrap();
            tabs.insert(tab.id.clone(), tab.clone());
            Ok(())
        }

        async fn get(&self, id: &str) -> Result<Option<OpenTabEntity>, DomainError> {
            let tabs = self.tabs.lock().unwrap();
            Ok(tabs.get(id).cloned())
        }

        async fn list_by_workspace(
            &self,
            workspace_id: &WorkspaceId,
        ) -> Result<Vec<OpenTabEntity>, DomainError> {
            let tabs = self.tabs.lock().unwrap();
            Ok(tabs
                .values()
                .filter(|t| t.workspace_id == *workspace_id)
                .cloned()
                .collect())
        }

        async fn set_active(
            &self,
            workspace_id: &WorkspaceId,
            tab_id: &str,
        ) -> Result<(), DomainError> {
            let mut tabs = self.tabs.lock().unwrap();
            for tab in tabs.values_mut() {
                if tab.workspace_id == *workspace_id {
                    tab.set_active(tab.id == tab_id);
                }
            }
            Ok(())
        }

        async fn delete(&self, id: &str) -> Result<(), DomainError> {
            let mut tabs = self.tabs.lock().unwrap();
            tabs.remove(id);
            Ok(())
        }

        async fn delete_by_workspace(&self, workspace_id: &WorkspaceId) -> Result<(), DomainError> {
            let mut tabs = self.tabs.lock().unwrap();
            tabs.retain(|_, t| t.workspace_id != *workspace_id);
            Ok(())
        }
    }

    #[tokio::test]
    async fn open_tab() {
        let repo = Arc::new(InMemoryTabRepository::new());
        let use_case = OpenTab::new(repo.clone());

        let tab = use_case
            .execute("workspace-1".to_owned(), "document-1".to_owned())
            .await
            .unwrap();

        assert_eq!(tab.document_id, "document-1");
        assert!(tab.is_active);
    }

    #[tokio::test]
    async fn open_duplicate_tab() {
        let repo = Arc::new(InMemoryTabRepository::new());
        let use_case = OpenTab::new(repo.clone());

        // 第一次打开
        let tab1 = use_case
            .execute("workspace-1".to_owned(), "document-1".to_owned())
            .await
            .unwrap();

        // 第二次打开相同文档
        let tab2 = use_case
            .execute("workspace-1".to_owned(), "document-1".to_owned())
            .await
            .unwrap();

        // 应该返回同一个标签
        assert_eq!(tab1.id, tab2.id);
    }

    #[tokio::test]
    async fn list_tabs() {
        let repo = Arc::new(InMemoryTabRepository::new());
        let open = OpenTab::new(repo.clone());
        let list = ListTabs::new(repo.clone());

        open.execute("workspace-1".to_owned(), "document-1".to_owned())
            .await
            .unwrap();
        open.execute("workspace-1".to_owned(), "document-2".to_owned())
            .await
            .unwrap();

        let tabs = list.execute("workspace-1".to_owned()).await.unwrap();
        assert_eq!(tabs.len(), 2);
    }

    #[tokio::test]
    async fn close_tab() {
        let repo = Arc::new(InMemoryTabRepository::new());
        let open = OpenTab::new(repo.clone());
        let close = CloseTab::new(repo.clone());
        let list = ListTabs::new(repo.clone());

        let tab = open
            .execute("workspace-1".to_owned(), "document-1".to_owned())
            .await
            .unwrap();

        close.execute(tab.id.clone()).await.unwrap();

        let tabs = list.execute("workspace-1".to_owned()).await.unwrap();
        assert_eq!(tabs.len(), 0);
    }

    #[tokio::test]
    async fn set_active_tab() {
        let repo = Arc::new(InMemoryTabRepository::new());
        let open = OpenTab::new(repo.clone());
        let set_active = SetActiveTab::new(repo.clone());
        let list = ListTabs::new(repo.clone());

        let tab1 = open
            .execute("workspace-1".to_owned(), "document-1".to_owned())
            .await
            .unwrap();
        let _tab2 = open
            .execute("workspace-1".to_owned(), "document-2".to_owned())
            .await
            .unwrap();

        set_active
            .execute("workspace-1".to_owned(), tab1.id.clone())
            .await
            .unwrap();

        let tabs = list.execute("workspace-1".to_owned()).await.unwrap();
        let active_tab = tabs.iter().find(|t| t.is_active).unwrap();
        assert_eq!(active_tab.id, tab1.id);
    }

    #[tokio::test]
    async fn close_active_tab_selects_next() {
        let repo = Arc::new(InMemoryTabRepository::new());
        let open = OpenTab::new(repo.clone());
        let close = CloseTab::new(repo.clone());
        let list = ListTabs::new(repo.clone());

        // 打开两个标签，第二个为活动标签
        let _tab1 = open
            .execute("workspace-1".to_owned(), "document-1".to_owned())
            .await
            .unwrap();
        let tab2 = open
            .execute("workspace-1".to_owned(), "document-2".to_owned())
            .await
            .unwrap();
        assert!(tab2.is_active);

        // 关闭活动标签
        close.execute(tab2.id.clone()).await.unwrap();

        // 剩余标签中应有一个活动标签
        let tabs = list.execute("workspace-1".to_owned()).await.unwrap();
        assert_eq!(tabs.len(), 1);
        assert!(
            tabs[0].is_active,
            "关闭活动标签后，剩余标签应自动成为活动标签"
        );
    }

    #[tokio::test]
    async fn close_inactive_tab_preserves_active() {
        let repo = Arc::new(InMemoryTabRepository::new());
        let open = OpenTab::new(repo.clone());
        let close = CloseTab::new(repo.clone());
        let list = ListTabs::new(repo.clone());

        // 打开两个标签
        let _tab1 = open
            .execute("workspace-1".to_owned(), "document-1".to_owned())
            .await
            .unwrap();
        let tab2 = open
            .execute("workspace-1".to_owned(), "document-2".to_owned())
            .await
            .unwrap();
        // tab2 是活动标签

        // 获取 tab1 的实际 ID（重新查询）
        let tabs = list.execute("workspace-1".to_owned()).await.unwrap();
        let inactive_tab = tabs.iter().find(|t| !t.is_active).unwrap();

        // 关闭非活动标签
        close.execute(inactive_tab.id.clone()).await.unwrap();

        // 剩余标签中活动标签应保持不变
        let tabs = list.execute("workspace-1".to_owned()).await.unwrap();
        assert_eq!(tabs.len(), 1);
        assert!(tabs[0].is_active, "关闭非活动标签后，活动标签应保持不变");
        assert_eq!(tabs[0].id, tab2.id);
    }

    #[tokio::test]
    async fn close_nonexistent_tab_returns_error() {
        let repo = Arc::new(InMemoryTabRepository::new());
        let close = CloseTab::new(repo.clone());

        let result = close.execute("nonexistent".to_owned()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TabError::TabNotFound => {} // 预期错误
            other => panic!("期望 TabNotFound，实际: {other:?}"),
        }
    }

    #[tokio::test]
    async fn close_last_tab_no_error() {
        let repo = Arc::new(InMemoryTabRepository::new());
        let open = OpenTab::new(repo.clone());
        let close = CloseTab::new(repo.clone());
        let list = ListTabs::new(repo.clone());

        // 打开一个标签并关闭
        let tab = open
            .execute("workspace-1".to_owned(), "document-1".to_owned())
            .await
            .unwrap();

        // 关闭唯一标签不应报错
        close.execute(tab.id.clone()).await.unwrap();

        let tabs = list.execute("workspace-1".to_owned()).await.unwrap();
        assert_eq!(tabs.len(), 0);
    }
}
