//! 数据源领域模型

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::workspace::WorkspaceId;

/// 数据源 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceId(pub String);

impl SourceId {
    /// 生成新的随机 ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for SourceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 数据源类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceKind {
    /// Git 仓库
    Git,
    /// 普通目录
    Directory,
}

impl std::fmt::Display for SourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Git => write!(f, "git"),
            Self::Directory => write!(f, "directory"),
        }
    }
}

impl SourceKind {
    /// 从字符串解析数据源类型
    pub fn parse_str(s: &str) -> Option<Self> {
        match s {
            "git" => Some(Self::Git),
            "directory" => Some(Self::Directory),
            _ => None,
        }
    }
}

/// 数据源实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// 数据源 ID
    pub id: SourceId,
    /// 所属工作区 ID
    pub workspace_id: WorkspaceId,
    /// 数据源名称
    pub name: String,
    /// 根路径
    pub root_path: PathBuf,
    /// 数据源类型
    pub kind: SourceKind,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl Source {
    /// 创建新的 Git 数据源
    pub fn new_git(workspace_id: WorkspaceId, name: String, root_path: PathBuf) -> Self {
        Self {
            id: SourceId::new(),
            workspace_id,
            name,
            root_path,
            kind: SourceKind::Git,
            created_at: Utc::now(),
        }
    }

    /// 创建新的目录数据源
    pub fn new_directory(workspace_id: WorkspaceId, name: String, root_path: PathBuf) -> Self {
        Self {
            id: SourceId::new(),
            workspace_id,
            name,
            root_path,
            kind: SourceKind::Directory,
            created_at: Utc::now(),
        }
    }

    /// 检测数据源类型
    pub fn detect_kind(root_path: &std::path::Path) -> SourceKind {
        let git_dir = root_path.join(".git");
        if git_dir.exists() {
            SourceKind::Git
        } else {
            SourceKind::Directory
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn source_creation_git() {
        let workspace_id = WorkspaceId::new();
        let source = Source::new_git(
            workspace_id.clone(),
            "test-repo".to_owned(),
            PathBuf::from("/test/path"),
        );
        assert_eq!(source.kind, SourceKind::Git);
        assert_eq!(source.name, "test-repo");
    }

    #[test]
    fn source_creation_directory() {
        let workspace_id = WorkspaceId::new();
        let source = Source::new_directory(
            workspace_id.clone(),
            "docs".to_owned(),
            PathBuf::from("/test/docs"),
        );
        assert_eq!(source.kind, SourceKind::Directory);
    }

    #[test]
    fn detect_kind_git() {
        let temp_dir = tempfile::tempdir().unwrap();
        let git_dir = temp_dir.path().join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();
        let kind = Source::detect_kind(temp_dir.path());
        assert_eq!(kind, SourceKind::Git);
    }

    #[test]
    fn detect_kind_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let kind = Source::detect_kind(temp_dir.path());
        assert_eq!(kind, SourceKind::Directory);
    }
}
