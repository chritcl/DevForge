use std::path::Path;
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

use crate::error::StorageError;

/// SQLite 数据库连接池。
///
/// 使用文件数据库、WAL、外键约束以及独立的锁等待和连接池等待超时。
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// 打开 SQLite 数据库并建立连接池。
    ///
    /// 本函数不会创建数据库父目录。父目录由 Task 7 的 Composition Root 创建。
    ///
    /// # Errors
    ///
    /// 当数据库路径无效、文件无法创建、SQLite 配置失败或连接池无法建立时返回错误。
    pub async fn open(db_path: &Path) -> Result<Self, StorageError> {
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5))
            .foreign_keys(true)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(options)
            .await
            .map_err(|source| StorageError::OpenDatabase {
                path: db_path.to_path_buf(),
                source,
            })?;

        Ok(Self { pool })
    }

    /// 返回底层 SQLx 连接池。
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
