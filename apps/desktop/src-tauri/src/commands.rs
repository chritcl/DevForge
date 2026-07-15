use tauri::State;

use devforge_application::app_info::AppInfo;

use crate::state::AppState;

/// 获取应用信息
///
/// specta 从此函数签名自动推导 TypeScript 类型。
/// State 参数由 Tauri 注入，specta 自动排除，不出现在 TS 签名中。
/// 通过窄接口调用，不直接访问 AppState 的内部字段。
///
/// Tauri 要求包含引用参数的异步 Command 返回 Result，
/// 此处使用 Infallible 作为错误类型，运行时不会实际产生错误。
#[tauri::command]
#[specta::specta]
pub async fn get_app_info(state: State<'_, AppState>) -> Result<AppInfo, String> {
    Ok(state.app_info().await)
}
