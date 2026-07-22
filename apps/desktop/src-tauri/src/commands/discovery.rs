//! 文件发现 Tauri 命令

use tauri::State;

use devforge_application::discovery::{DiscoveryError, ScanResult, ScanSource};

use crate::state::AppState;

/// 扫描数据源
///
/// 后端通过 source_id 从数据库获取可信路径，不接受前端传入的路径。
#[tauri::command]
#[specta::specta]
pub async fn scan_source(
    state: State<'_, AppState>,
    source_id: String,
) -> Result<ScanResult, DiscoveryError> {
    let use_case = ScanSource::new(state.source_repo(), state.document_repo());
    use_case.execute(source_id).await
}
