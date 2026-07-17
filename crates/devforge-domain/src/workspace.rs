//! 工作区领域模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 工作区 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkspaceId(pub String);

impl WorkspaceId {
    /// 生成新的随机 ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for WorkspaceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 工作区状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceStatus {
    /// 活跃状态
    Active,
    /// 已归档
    Archived,
}

impl std::fmt::Display for WorkspaceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

impl WorkspaceStatus {
    /// 从字符串解析状态
    pub fn parse_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(Self::Active),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }
}

/// 工作区实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// 工作区 ID
    pub id: WorkspaceId,
    /// 工作区名称
    pub name: String,
    /// 工作区描述
    pub description: Option<String>,
    /// 工作区状态
    pub status: WorkspaceStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 最后打开时间
    pub last_opened_at: Option<DateTime<Utc>>,
}

impl Workspace {
    /// 创建新的工作区
    pub fn new(
        name: String,
        description: Option<String>,
    ) -> Result<Self, crate::error::DomainError> {
        if name.trim().is_empty() {
            return Err(crate::error::DomainError::InvalidInput(
                "工作区名称不能为空".to_owned(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id: WorkspaceId::new(),
            name: name.trim().to_owned(),
            description,
            status: WorkspaceStatus::Active,
            created_at: now,
            updated_at: now,
            last_opened_at: None,
        })
    }

    /// 更新工作区名称
    pub fn update_name(&mut self, name: String) -> Result<(), crate::error::DomainError> {
        if name.trim().is_empty() {
            return Err(crate::error::DomainError::InvalidInput(
                "工作区名称不能为空".to_owned(),
            ));
        }
        self.name = name.trim().to_owned();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 更新工作区描述
    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    /// 归档工作区
    pub fn archive(&mut self) {
        self.status = WorkspaceStatus::Archived;
        self.updated_at = Utc::now();
    }

    /// 恢复工作区
    pub fn restore(&mut self) {
        self.status = WorkspaceStatus::Active;
        self.updated_at = Utc::now();
    }

    /// 记录打开时间
    pub fn mark_opened(&mut self) {
        self.last_opened_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_creation_with_valid_name() {
        let workspace = Workspace::new("测试工作区".to_owned(), None).unwrap();
        assert_eq!(workspace.name, "测试工作区");
        assert_eq!(workspace.status, WorkspaceStatus::Active);
        assert!(workspace.description.is_none());
    }

    #[test]
    fn workspace_creation_with_empty_name_fails() {
        let result = Workspace::new("".to_owned(), None);
        assert!(result.is_err());
    }

    #[test]
    fn workspace_creation_with_whitespace_name_fails() {
        let result = Workspace::new("   ".to_owned(), None);
        assert!(result.is_err());
    }

    #[test]
    fn workspace_update_name() {
        let mut workspace = Workspace::new("旧名称".to_owned(), None).unwrap();
        workspace.update_name("新名称".to_owned()).unwrap();
        assert_eq!(workspace.name, "新名称");
    }

    #[test]
    fn workspace_archive_and_restore() {
        let mut workspace = Workspace::new("测试".to_owned(), None).unwrap();
        workspace.archive();
        assert_eq!(workspace.status, WorkspaceStatus::Archived);
        workspace.restore();
        assert_eq!(workspace.status, WorkspaceStatus::Active);
    }

    #[test]
    fn workspace_mark_opened() {
        let mut workspace = Workspace::new("测试".to_owned(), None).unwrap();
        assert!(workspace.last_opened_at.is_none());
        workspace.mark_opened();
        assert!(workspace.last_opened_at.is_some());
    }
}
