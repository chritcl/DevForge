//! 数据源 Tauri 命令

use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;

use devforge_application::source::{
    AddLocalSource, ListSources, RemoveSource, SourceDto, SourceError, SourceRepository,
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
/// 同时清理该数据源的全文索引。
#[tauri::command]
#[specta::specta]
pub async fn remove_source(state: State<'_, AppState>, id: String) -> Result<(), SourceError> {
    // 尝试获取索引器用于清理
    let indexer = state
        .source_repo()
        .get(&devforge_domain::source::SourceId(id.clone()))
        .await
        .ok()
        .flatten()
        .and_then(|source| {
            state
                .workspace_index(&source.workspace_id.0)
                .ok()
                .map(|idx| Arc::new(idx) as Arc<dyn devforge_application::discovery::IndexerPort>)
        });

    let mut use_case = RemoveSource::new(state.source_repo());
    if let Some(idx) = indexer {
        use_case = use_case.with_indexer(idx);
    }
    use_case.execute(id).await
}
