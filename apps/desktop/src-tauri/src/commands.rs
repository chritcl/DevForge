use tauri::{AppHandle, Manager};

use devforge_application::app_info::AppInfo;

use crate::state::AppState;

/// 获取应用信息
///
/// AppHandle 由 Tauri 注入，不出现在生成的 TypeScript 参数中。
/// AppState 已由 Composition Root 注册为 managed state。
#[tauri::command]
#[specta::specta]
pub async fn get_app_info(app: AppHandle) -> AppInfo {
    app.state::<AppState>().app_info().await
}
