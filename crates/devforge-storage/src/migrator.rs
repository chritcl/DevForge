use sqlx::migrate::Migrator;
use sqlx::SqlitePool;

use crate::error::StorageError;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// 执行所有尚未应用的数据库 Migration。
///
/// # Errors
///
/// 当 Migration 文件无效、SQL 执行失败或数据库被锁定时返回错误。
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), StorageError> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(|source| StorageError::Migration { source })
}

/// 查询最后一个成功执行的 Migration 版本。
///
/// 尚无成功 Migration 时返回版本 0。
///
/// # Errors
///
/// 当 `_sqlx_migrations` 无法查询，或版本超出 `u32` 范围时返回错误。
pub async fn schema_version(pool: &SqlitePool) -> Result<u32, StorageError> {
    let row: (Option<i64>,) =
        sqlx::query_as("SELECT MAX(version) FROM _sqlx_migrations WHERE success = TRUE")
            .fetch_one(pool)
            .await
            .map_err(|source| StorageError::SchemaVersion { source })?;

    let version = row.0.unwrap_or(0);

    u32::try_from(version).map_err(|_| StorageError::MigrationVersionOutOfRange { version })
}
