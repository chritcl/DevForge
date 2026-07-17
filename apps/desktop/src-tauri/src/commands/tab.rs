//! 标签页 Tauri 命令

use tauri::State;

use devforge_application::tab::{CloseTab, ListTabs, OpenTab, SetActiveTab, TabDto, TabError};

use crate::state::AppState;

/// 打开标签页
#[tauri::command]
pub async fn open_tab(
    state: State<'_, AppState>,
    workspace_id: String,
    document_id: String,
) -> Result<TabDto, TabError> {
    let use_case = OpenTab::new(state.tab_repo());
    use_case.execute(workspace_id, document_id).await
}

/// 关闭标签页
#[tauri::command]
pub async fn close_tab(state: State<'_, AppState>, id: String) -> Result<(), TabError> {
    let use_case = CloseTab::new(state.tab_repo());
    use_case.execute(id).await
}

/// 列出标签页
#[tauri::command]
pub async fn list_tabs(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<Vec<TabDto>, TabError> {
    let use_case = ListTabs::new(state.tab_repo());
    use_case.execute(workspace_id).await
}

/// 设置活动标签页
#[tauri::command]
pub async fn set_active_tab(
    state: State<'_, AppState>,
    workspace_id: String,
    tab_id: String,
) -> Result<(), TabError> {
    let use_case = SetActiveTab::new(state.tab_repo());
    use_case.execute(workspace_id, tab_id).await
}
