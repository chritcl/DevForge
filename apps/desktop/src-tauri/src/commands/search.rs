//! 搜索相关 Tauri 命令

use crate::state::AppState;

/// 搜索结果条目 DTO
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct SearchResultDto {
    /// 文档 ID
    pub document_id: String,
    /// 文件路径（相对于数据源根目录）
    pub path: String,
    /// 文件名
    pub file_name: String,
    /// 匹配分数（越高越相关）
    pub score: f32,
}

/// 搜索工作区
///
/// 在工作区的所有数据源中搜索关键词。
/// 空查询不执行搜索。
#[tauri::command]
#[specta::specta]
pub async fn search_workspace(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
    query: String,
) -> Result<Vec<SearchResultDto>, String> {
    // 空查询不搜索
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let index = state
        .workspace_index(&workspace_id)
        .map_err(|e| format!("打开索引失败: {e}"))?;

    let hits = index
        .search(query, 50)
        .map_err(|e| format!("搜索失败: {e}"))?;

    let results = hits
        .into_iter()
        .map(|hit| SearchResultDto {
            document_id: hit.document_id,
            path: hit.path,
            file_name: hit.file_name,
            score: hit.score,
        })
        .collect();

    Ok(results)
}
