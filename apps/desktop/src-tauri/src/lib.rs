#![forbid(unsafe_code)]

/// Tauri 应用入口（Phase 0 最小版本）
///
/// Task 4 会添加 commands 和 state。
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
