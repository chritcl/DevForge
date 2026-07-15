use async_trait::async_trait;
use devforge_application::app_info::DbStatus;
use devforge_application::ports::DatabaseStatusProvider;
use sqlx::SqlitePool;

use crate::migrator;

/// 基于 SQLx 的数据库状态提供者。
///
/// 只表示已经成功建立的 SQLite 数据源。
/// 未初始化状态由 Application 层的 `NotInitializedDbStatus` 表示。
pub struct SqliteDatabaseStatus {
    pool: SqlitePool,
}

impl SqliteDatabaseStatus {
    /// 使用已建立的连接池创建状态提供者。
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DatabaseStatusProvider for SqliteDatabaseStatus {
    async fn status(&self) -> DbStatus {
        match migrator::schema_version(&self.pool).await {
            Ok(version) => DbStatus::Ready {
                migration_version: version,
            },
            Err(error) => DbStatus::Error {
                message: error.to_string(),
            },
        }
    }
}
