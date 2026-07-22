//! 文件发现 Tauri 命令

use std::sync::Arc;

use tauri::State;

use devforge_application::discovery::{DiscoveryError, ScanResult, ScanSource};
use devforge_application::source::SourceRepository;

use crate::state::AppState;

/// 扫描数据源
///
/// 后端通过 source_id 从数据库获取可信路径，不接受前端传入的路径。
/// 扫描时自动建立全文索引。
#[tauri::command]
#[specta::specta]
pub async fn scan_source(
    state: State<'_, AppState>,
    source_id: String,
) -> Result<ScanResult, DiscoveryError> {
    // 尝试获取工作区索引（如果索引不可用，扫描仍然继续）
    let indexer = state
        .source_repo()
        .get(&devforge_domain::source::SourceId(source_id.clone()))
        .await
        .ok()
        .flatten()
        .and_then(|source| {
            state
                .workspace_index(&source.workspace_id.0)
                .ok()
                .map(|idx| Arc::new(idx) as Arc<dyn devforge_application::discovery::IndexerPort>)
        });

    let mut use_case = ScanSource::new(state.source_repo(), state.document_repo());
    if let Some(idx) = indexer {
        use_case = use_case.with_indexer(idx);
    }
    use_case.execute(source_id).await
}
