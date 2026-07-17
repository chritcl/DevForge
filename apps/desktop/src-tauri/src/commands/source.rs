//! 数据源 Tauri 命令

use std::path::PathBuf;

use tauri::State;

use devforge_application::source::{
    AddDirectorySource, AddGitSource, ListSources, RemoveSource, SourceError,
};
use devforge_domain::source::Source;

use crate::state::AppState;

/// 添加 Git 数据源
#[tauri::command]
pub async fn add_git_source(
    state: State<'_, AppState>,
    workspace_id: String,
    path: String,
) -> Result<Source, SourceError> {
    let use_case = AddGitSource::new(state.source_repo());
    use_case.execute(workspace_id, PathBuf::from(path)).await
}

/// 添加目录数据源
#[tauri::command]
pub async fn add_directory_source(
    state: State<'_, AppState>,
    workspace_id: String,
    path: String,
) -> Result<Source, SourceError> {
    let use_case = AddDirectorySource::new(state.source_repo());
    use_case.execute(workspace_id, PathBuf::from(path)).await
}

/// 列出数据源
#[tauri::command]
pub async fn list_sources(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<Vec<Source>, SourceError> {
    let use_case = ListSources::new(state.source_repo());
    use_case.execute(workspace_id).await
}

/// 移除数据源
///
/// 注意：移除数据源只删除元数据，不删除源目录。
#[tauri::command]
pub async fn remove_source(state: State<'_, AppState>, id: String) -> Result<(), SourceError> {
    let use_case = RemoveSource::new(state.source_repo());
    use_case.execute(id).await
}
