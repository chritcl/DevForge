//! 全文索引相关 Tauri 命令

use std::sync::Arc;

use devforge_application::source::SourceRepository;

use crate::state::AppState;

/// 索引状态 DTO
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct IndexStatusDto {
    /// 工作区 ID
    pub workspace_id: String,
    /// 索引中的文档数量
    pub document_count: u32,
    /// 索引目录是否存在
    pub exists: bool,
}

/// 获取工作区的全文索引状态
#[tauri::command]
#[specta::specta]
pub async fn get_index_status(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
) -> Result<IndexStatusDto, String> {
    let index = state
        .workspace_index(&workspace_id)
        .map_err(|e| format!("打开索引失败: {e}"))?;

    let document_count = index
        .document_count()
        .map_err(|e| format!("获取索引状态失败: {e}"))?;

    #[allow(clippy::cast_possible_truncation)]
    let document_count = document_count as u32;

    Ok(IndexStatusDto {
        workspace_id,
        document_count,
        exists: true,
    })
}

/// 重建工作区的全文索引
///
/// 清空现有索引，然后对工作区的所有数据源重新扫描和索引。
#[tauri::command]
#[specta::specta]
pub async fn rebuild_workspace_index(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
) -> Result<IndexStatusDto, String> {
    let index = state
        .workspace_index(&workspace_id)
        .map_err(|e| format!("打开索引失败: {e}"))?;

    // 清空现有索引
    index.clear().map_err(|e| format!("清空索引失败: {e}"))?;

    // 获取工作区的所有数据源
    let sources = state
        .source_repo()
        .list_by_workspace(&devforge_domain::workspace::WorkspaceId(
            workspace_id.clone(),
        ))
        .await
        .map_err(|e| format!("获取数据源列表失败: {e}"))?;

    let index_arc = Arc::new(index);

    // 对每个数据源执行扫描和索引
    for source in sources {
        let scan = devforge_application::discovery::ScanSource::new(
            state.source_repo(),
            state.document_repo(),
        )
        .with_indexer(index_arc.clone());

        let _ = scan.execute(source.id.0.clone()).await;
    }

    let document_count = index_arc
        .document_count()
        .map_err(|e| format!("获取索引状态失败: {e}"))?;

    #[allow(clippy::cast_possible_truncation)]
    let document_count = document_count as u32;

    Ok(IndexStatusDto {
        workspace_id,
        document_count,
        exists: true,
    })
}
