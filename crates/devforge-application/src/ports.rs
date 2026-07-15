use async_trait::async_trait;

use crate::app_info::{AppMetadata, DbStatus};

/// 应用元数据查询端口
///
/// 由 Platform Adapter 实现，提供版本和数据目录。
/// 不感知数据库状态。
pub trait AppMetadataProvider: Send + Sync {
    fn metadata(&self) -> AppMetadata;
}

/// 数据库状态查询端口（异步）
///
/// 使用 async 因为 SQLx 查询天然是异步的。
#[async_trait]
pub trait DatabaseStatusProvider: Send + Sync {
    async fn status(&self) -> DbStatus;
}

/// 默认数据库状态（Task 4 使用，Task 7 替换为真实实现）
pub struct NotInitializedDbStatus;

#[async_trait]
impl DatabaseStatusProvider for NotInitializedDbStatus {
    async fn status(&self) -> DbStatus {
        DbStatus::NotInitialized
    }
}
