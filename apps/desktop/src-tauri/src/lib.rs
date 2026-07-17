#![forbid(unsafe_code)]

mod commands;
mod state;

use std::sync::Arc;

use anyhow::Context;
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};

use state::AppState;

/// 创建 specta Builder（绑定生成和 run 共用同一个 Builder）
fn create_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![commands::get_app_info,])
}

/// 导出 specta 生成的 TypeScript 绑定
///
/// 输出路径基于 CARGO_MANIFEST_DIR 构造，不依赖进程当前工作目录。
/// 输出文件：apps/desktop/src/bindings.ts
/// 该文件提交到 git，前端直接 import 使用。
///
/// # Errors
///
/// 当无法导出 Specta TypeScript bindings 时返回错误。
pub fn export_bindings() -> anyhow::Result<()> {
    let builder = create_builder();
    let out_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("src")
        .join("bindings.ts");
    builder
        .export(Typescript::default(), &out_path)
        .context("无法导出 TypeScript bindings")?;
    Ok(())
}

/// 初始化应用状态
///
/// 打开 SQLite 数据库、执行 Migration，并构造完整的 AppState。
/// Database 包装器在函数结束时释放，但 SqlitePool clone 由 SqliteDatabaseStatus 持有。
async fn initialize_app_state(
    version: String,
    data_dir: std::path::PathBuf,
) -> anyhow::Result<AppState> {
    let db_path = data_dir.join("devforge.db");

    let database = devforge_storage::pool::Database::open(&db_path)
        .await
        .with_context(|| format!("无法打开 SQLite 数据库：{}", db_path.display()))?;

    devforge_storage::migrator::run_migrations(database.pool())
        .await
        .context("无法执行 SQLite migration")?;

    let platform_metadata = devforge_platform::app_info::PlatformMetadata::new(version, data_dir);

    let database_status =
        devforge_storage::status::SqliteDatabaseStatus::new(database.pool().clone());

    let workspace_repo = Arc::new(
        devforge_storage::repository::SqliteWorkspaceRepository::new(database.pool().clone()),
    );

    Ok(AppState::new(
        platform_metadata,
        database_status,
        workspace_repo,
    ))
}

/// 启动 Tauri 应用。
///
/// Composition Root 负责解析数据目录、创建目录、初始化 SQLite、
/// 执行 Migration，并构造完整的 AppState。
///
/// # Errors
///
/// 以下情况返回错误：
///
/// - 无法解析本地数据目录；
/// - 无法创建应用数据目录；
/// - SQLite 数据库无法打开；
/// - Migration 执行失败；
/// - Tauri 应用无法启动。
pub fn run() -> anyhow::Result<()> {
    let data_dir = dirs::data_local_dir()
        .context("无法解析本地数据目录")?
        .join("DevForge");

    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("无法创建本地数据目录：{}", data_dir.display()))?;

    let app_version = env!("CARGO_PKG_VERSION").to_owned();

    let app_state = tauri::async_runtime::block_on(initialize_app_state(app_version, data_dir))
        .context("无法初始化桌面应用状态")?;

    let builder = create_builder();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(builder.invoke_handler())
        .invoke_handler(tauri::generate_handler![
            commands::create_workspace,
            commands::get_workspace,
            commands::list_workspaces,
            commands::update_workspace,
            commands::archive_workspace,
            commands::restore_workspace,
            commands::delete_workspace,
        ])
        .run(tauri::generate_context!())
        .context("无法启动 Tauri 应用")?;

    Ok(())
}
