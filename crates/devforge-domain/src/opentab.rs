//! 打开标签领域模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::document::DocumentId;
use crate::workspace::WorkspaceId;

/// 打开标签实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTab {
    /// 标签 ID
    pub id: String,
    /// 所属工作区 ID
    pub workspace_id: WorkspaceId,
    /// 关联文档 ID
    pub document_id: DocumentId,
    /// 标签位置（从 0 开始）
    pub position: i32,
    /// 是否为活动标签
    pub is_active: bool,
    /// 打开时间
    pub opened_at: DateTime<Utc>,
}

impl OpenTab {
    /// 创建新的打开标签
    pub fn new(workspace_id: WorkspaceId, document_id: DocumentId, position: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            workspace_id,
            document_id,
            position,
            is_active: false,
            opened_at: Utc::now(),
        }
    }

    /// 设置为活动标签
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    /// 更新位置
    pub fn set_position(&mut self, position: i32) {
        self.position = position;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_tab_creation() {
        let workspace_id = WorkspaceId::new();
        let document_id = DocumentId::new();
        let tab = OpenTab::new(workspace_id.clone(), document_id.clone(), 0);
        assert_eq!(tab.workspace_id, workspace_id);
        assert_eq!(tab.document_id, document_id);
        assert_eq!(tab.position, 0);
        assert!(!tab.is_active);
    }

    #[test]
    fn open_tab_set_active() {
        let workspace_id = WorkspaceId::new();
        let document_id = DocumentId::new();
        let mut tab = OpenTab::new(workspace_id, document_id, 0);
        assert!(!tab.is_active);
        tab.set_active(true);
        assert!(tab.is_active);
    }
}
