//! 数据源 Tauri 命令

use std::path::PathBuf;

use tauri::State;

use devforge_application::source::{
    AddLocalSource, ListSources, RemoveSource, SourceDto, SourceError,
};

use crate::state::AppState;

/// 添加本地数据源（后端自动识别类型）
#[tauri::command]
#[specta::specta]
pub async fn add_local_source(
    state: State<'_, AppState>,
    workspace_id: String,
    path: String,
) -> Result<SourceDto, SourceError> {
    let use_case = AddLocalSource::new(state.source_repo());
    use_case.execute(workspace_id, PathBuf::from(path)).await
}

/// 列出数据源
#[tauri::command]
#[specta::specta]
pub async fn list_sources(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<Vec<SourceDto>, SourceError> {
    let use_case = ListSources::new(state.source_repo());
    use_case.execute(workspace_id).await
}

/// 移除数据源
///
/// 注意：移除数据源只删除元数据，不删除源目录。
#[tauri::command]
#[specta::specta]
pub async fn remove_source(state: State<'_, AppState>, id: String) -> Result<(), SourceError> {
    let use_case = RemoveSource::new(state.source_repo());
    use_case.execute(id).await
}
