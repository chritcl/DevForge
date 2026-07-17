//! 工作区 Tauri 命令

use tauri::State;

use devforge_application::workspace::{
    AppError, ArchiveWorkspace, CreateWorkspace, DeleteWorkspace, GetWorkspace, ListWorkspaces,
    RestoreWorkspace, UpdateWorkspace,
};
use devforge_domain::workspace::Workspace;

use crate::state::AppState;

/// 创建工作区
#[tauri::command]
pub async fn create_workspace(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
) -> Result<Workspace, AppError> {
    let use_case = CreateWorkspace::new(state.workspace_repo());
    use_case.execute(name, description).await
}

/// 获取工作区
#[tauri::command]
pub async fn get_workspace(state: State<'_, AppState>, id: String) -> Result<Workspace, AppError> {
    let use_case = GetWorkspace::new(state.workspace_repo());
    use_case.execute(id).await
}

/// 列出工作区
#[tauri::command]
pub async fn list_workspaces(state: State<'_, AppState>) -> Result<Vec<Workspace>, AppError> {
    let use_case = ListWorkspaces::new(state.workspace_repo());
    use_case.execute().await
}

/// 更新工作区
#[tauri::command]
pub async fn update_workspace(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    description: Option<Option<String>>,
) -> Result<Workspace, AppError> {
    let use_case = UpdateWorkspace::new(state.workspace_repo());
    use_case.execute(id, name, description).await
}

/// 归档工作区
#[tauri::command]
pub async fn archive_workspace(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let use_case = ArchiveWorkspace::new(state.workspace_repo());
    use_case.execute(id).await
}

/// 恢复工作区
#[tauri::command]
pub async fn restore_workspace(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let use_case = RestoreWorkspace::new(state.workspace_repo());
    use_case.execute(id).await
}

/// 删除工作区
#[tauri::command]
pub async fn delete_workspace(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let use_case = DeleteWorkspace::new(state.workspace_repo());
    use_case.execute(id).await
}
