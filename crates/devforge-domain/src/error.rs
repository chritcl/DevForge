//! 领域错误类型

use std::path::PathBuf;

/// 领域层错误
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    /// 工作区不存在
    #[error("工作区不存在: {0}")]
    WorkspaceNotFound(String),

    /// 数据源不存在
    #[error("数据源不存在: {0}")]
    SourceNotFound(String),

    /// 文档不存在
    #[error("文档不存在: {0}")]
    DocumentNotFound(String),

    /// 路径安全违规
    #[error("路径安全违规: {path} - {reason}")]
    PathViolation { path: PathBuf, reason: String },

    /// 敏感文件不可读
    #[error("敏感文件不可读: {0}")]
    SensitiveFile(PathBuf),

    /// 文件过大
    #[error("文件过大: {0} bytes")]
    FileTooLarge(u64),

    /// 无效输入
    #[error("无效输入: {0}")]
    InvalidInput(String),

    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}
