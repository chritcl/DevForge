use devforge_application::app_info::DbStatus;
use devforge_application::ports::DatabaseStatusProvider;
use devforge_storage::migrator;
use devforge_storage::pool::Database;
use devforge_storage::status::SqliteDatabaseStatus;

/// 测试 1：数据库打开和连接配置
#[tokio::test]
async fn database_opens_with_correct_pragmas() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let db = Database::open(&db_path).await?;
    let pool = db.pool();

    // 验证数据库文件被创建
    assert!(db_path.exists(), "数据库文件应被创建");

    // 验证 PRAGMA journal_mode = wal
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(pool)
        .await?;
    assert_eq!(journal_mode, "wal", "journal_mode 应为 wal");

    // 验证 PRAGMA foreign_keys = 1
    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(pool)
        .await?;
    assert_eq!(foreign_keys, 1, "foreign_keys 应为 1");

    // 验证 PRAGMA busy_timeout = 5000
    let busy_timeout: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
        .fetch_one(pool)
        .await?;
    assert_eq!(busy_timeout, 5000, "busy_timeout 应为 5000");

    pool.close().await;
    Ok(())
}

/// 测试 2：空数据库执行 Migration
#[tokio::test]
async fn migration_runs_on_empty_database() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let db = Database::open(&db_path).await?;
    let pool = db.pool();

    // 执行 migration
    migrator::run_migrations(pool).await?;

    // 验证 app_meta 表存在
    let table_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'app_meta'",
    )
    .fetch_one(pool)
    .await?;
    assert_eq!(table_count.0, 1, "app_meta 表应存在");

    // 验证 schema_version == 1
    let version = migrator::schema_version(pool).await?;
    assert_eq!(version, 1, "schema_version 应为 1");

    pool.close().await;
    Ok(())
}

/// 测试 3：Migration 重复执行保持幂等
#[tokio::test]
async fn migration_is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let db = Database::open(&db_path).await?;
    let pool = db.pool();

    // 第一次执行
    migrator::run_migrations(pool).await?;

    // 第二次执行
    migrator::run_migrations(pool).await?;

    // 验证 schema_version 仍为 1
    let version = migrator::schema_version(pool).await?;
    assert_eq!(version, 1, "重复执行后 schema_version 应仍为 1");

    // 验证 app_meta 仍只有一个表定义
    let table_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'app_meta'",
    )
    .fetch_one(pool)
    .await?;
    assert_eq!(table_count.0, 1, "app_meta 表应仍只有一个");

    pool.close().await;
    Ok(())
}

/// 测试 4：健康状态 Ready
#[tokio::test]
async fn health_status_ready_after_migration() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let db = Database::open(&db_path).await?;
    let pool = db.pool();

    // 执行 migration
    migrator::run_migrations(pool).await?;

    // 构造 SqliteDatabaseStatus
    let provider = SqliteDatabaseStatus::new(pool.clone());
    let status = provider.status().await;

    // 验证返回 Ready
    match status {
        DbStatus::Ready { migration_version } => {
            assert_eq!(migration_version, 1, "migration_version 应为 1");
        }
        other => panic!("预期 Ready，实际：{other:?}"),
    }

    pool.close().await;
    Ok(())
}

/// 测试 5：未迁移数据库返回 Error
#[tokio::test]
async fn health_status_error_without_migration() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let db = Database::open(&db_path).await?;
    let pool = db.pool();

    // 不执行 migration，直接构造 SqliteDatabaseStatus
    let provider = SqliteDatabaseStatus::new(pool.clone());
    let status = provider.status().await;

    // 验证返回 Error（不是 NotInitialized，不是 panic）
    match status {
        DbStatus::Error { .. } => {
            // 预期结果
        }
        other => panic!("预期 Error，实际：{other:?}"),
    }

    pool.close().await;
    Ok(())
}
