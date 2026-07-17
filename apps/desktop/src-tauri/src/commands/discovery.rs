//! 文件发现 Tauri 命令

use std::path::PathBuf;

use tauri::State;

use devforge_application::discovery::{DiscoveryError, ScanResult, ScanSource};

use crate::state::AppState;

/// 扫描数据源
#[tauri::command]
pub async fn scan_source(
    state: State<'_, AppState>,
    source_id: String,
    root_path: String,
) -> Result<ScanResult, DiscoveryError> {
    let use_case = ScanSource::new(state.document_repo());
    use_case.execute(source_id, PathBuf::from(root_path)).await
}
