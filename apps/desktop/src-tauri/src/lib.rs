#![forbid(unsafe_code)]

mod commands;
mod state;

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

/// 启动 Tauri 应用
///
/// Composition Root 统一解析 data_dir 和版本号，注入 PlatformMetadata。
///
/// # Errors
///
/// 当无法解析本地数据目录或无法启动 Tauri 应用时返回错误。
pub fn run() -> anyhow::Result<()> {
    let builder = create_builder();

    let data_dir = dirs::data_local_dir()
        .context("无法解析本地数据目录")?
        .join("DevForge");
    let app_version = env!("CARGO_PKG_VERSION").to_owned();
    let app_state = AppState::new(app_version, data_dir);

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .context("无法启动 Tauri 应用")?;

    Ok(())
}
