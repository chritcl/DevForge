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

    // 在内部 async block 中执行所有可能使用 ? 的数据库操作
    let observed: Result<_, Box<dyn std::error::Error>> = async {
        let file_exists = db_path.exists();

        let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(pool)
            .await?;

        let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(pool)
            .await?;

        let busy_timeout: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
            .fetch_one(pool)
            .await?;

        Ok((file_exists, journal_mode, foreign_keys, busy_timeout))
    }
    .await;

    // 无条件关闭 Pool
    pool.close().await;

    // 传播数据库操作错误
    let (file_exists, journal_mode, foreign_keys, busy_timeout) = observed?;

    // 所有断言在关闭 Pool 之后执行
    assert!(file_exists, "数据库文件应被创建");
    assert_eq!(journal_mode, "wal", "journal_mode 应为 wal");
    assert_eq!(foreign_keys, 1, "foreign_keys 应为 1");
    assert_eq!(busy_timeout, 5000, "busy_timeout 应为 5000");

    Ok(())
}

/// 测试 2：空数据库执行 Migration
#[tokio::test]
async fn migration_runs_on_empty_database() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let db = Database::open(&db_path).await?;
    let pool = db.pool();

    let observed: Result<_, Box<dyn std::error::Error>> = async {
        // 执行 migration
        migrator::run_migrations(pool).await?;

        // 验证 app_meta 表存在
        let table_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'app_meta'",
        )
        .fetch_one(pool)
        .await?;

        // 验证 schema_version == 5（当前共 5 个 Migration）
        let version = migrator::schema_version(pool).await?;

        Ok((table_count.0, version))
    }
    .await;

    // 无条件关闭 Pool
    pool.close().await;

    // 传播数据库操作错误
    let (table_count, version) = observed?;

    // 所有断言在关闭 Pool 之后执行
    assert_eq!(table_count, 1, "app_meta 表应存在");
    assert_eq!(version, 5, "schema_version 应为 5");

    Ok(())
}

/// 测试 3：Migration 重复执行保持幂等
#[tokio::test]
async fn migration_is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let db = Database::open(&db_path).await?;
    let pool = db.pool();

    let observed: Result<_, Box<dyn std::error::Error>> = async {
        // 第一次执行
        migrator::run_migrations(pool).await?;

        // 第二次执行
        migrator::run_migrations(pool).await?;

        // 验证 schema_version 仍为 5（当前共 5 个 Migration）
        let version = migrator::schema_version(pool).await?;

        // 验证 app_meta 仍只有一个表定义
        let table_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'app_meta'",
        )
        .fetch_one(pool)
        .await?;

        Ok((version, table_count.0))
    }
    .await;

    // 无条件关闭 Pool
    pool.close().await;

    // 传播数据库操作错误
    let (version, table_count) = observed?;

    // 所有断言在关闭 Pool 之后执行
    assert_eq!(version, 5, "重复执行后 schema_version 应仍为 5");
    assert_eq!(table_count, 1, "app_meta 表应仍只有一个");

    Ok(())
}

/// 测试 4：健康状态 Ready
#[tokio::test]
async fn health_status_ready_after_migration() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let db = Database::open(&db_path).await?;
    let pool = db.pool();

    let observed: Result<_, Box<dyn std::error::Error>> = async {
        // 执行 migration
        migrator::run_migrations(pool).await?;

        // 构造 SqliteDatabaseStatus
        let provider = SqliteDatabaseStatus::new(pool.clone());
        let status = provider.status().await;

        Ok((status, provider))
    }
    .await;

    // 无条件关闭 Pool
    pool.close().await;

    // 传播数据库操作错误
    let (status, _provider) = observed?;

    // 计算布尔值，然后断言（当前共 5 个 Migration）
    let is_ready = matches!(
        status,
        DbStatus::Ready {
            migration_version: 5
        }
    );

    assert!(is_ready, "数据库状态应为 Ready，migration_version 应为 5");

    Ok(())
}

/// 测试 5：未迁移数据库返回 Error
#[tokio::test]
async fn health_status_error_without_migration() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let db = Database::open(&db_path).await?;
    let pool = db.pool();

    let observed: Result<_, Box<dyn std::error::Error>> = async {
        // 不执行 migration，直接构造 SqliteDatabaseStatus
        let provider = SqliteDatabaseStatus::new(pool.clone());
        let status = provider.status().await;

        Ok((status, provider))
    }
    .await;

    // 无条件关闭 Pool
    pool.close().await;

    // 传播数据库操作错误
    let (status, _provider) = observed?;

    // 计算布尔值，然后断言
    let is_error = matches!(status, DbStatus::Error { .. });

    assert!(is_error, "数据库状态应为 Error");

    Ok(())
}
