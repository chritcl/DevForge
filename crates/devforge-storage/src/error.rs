use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("无法打开 SQLite 数据库：{path}")]
    OpenDatabase {
        path: PathBuf,
        #[source]
        source: sqlx::Error,
    },

    #[error("无法执行 SQLite migration")]
    Migration {
        #[source]
        source: sqlx::migrate::MigrateError,
    },

    #[error("无法读取 SQLite schema 版本")]
    SchemaVersion {
        #[source]
        source: sqlx::Error,
    },

    #[error("migration 版本超出应用支持范围：{version}")]
    MigrationVersionOutOfRange { version: i64 },
}
