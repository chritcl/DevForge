//! 工作区 Tauri 命令

use tauri::State;

use devforge_application::workspace::{
    AppError, ArchiveWorkspace, CreateWorkspace, DeleteWorkspace, GetWorkspace,
    ListArchivedWorkspaces, ListWorkspaces, MarkWorkspaceOpened, RestoreWorkspace, UpdateWorkspace,
    WorkspaceDto,
};

use crate::state::AppState;

/// 创建工作区
#[tauri::command]
#[specta::specta]
pub async fn create_workspace(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
) -> Result<WorkspaceDto, AppError> {
    let use_case = CreateWorkspace::new(state.workspace_repo());
    use_case.execute(name, description).await
}

/// 获取工作区
#[tauri::command]
#[specta::specta]
pub async fn get_workspace(
    state: State<'_, AppState>,
    id: String,
) -> Result<WorkspaceDto, AppError> {
    let use_case = GetWorkspace::new(state.workspace_repo());
    use_case.execute(id).await
}

/// 列出活跃工作区
#[tauri::command]
#[specta::specta]
pub async fn list_workspaces(state: State<'_, AppState>) -> Result<Vec<WorkspaceDto>, AppError> {
    let use_case = ListWorkspaces::new(state.workspace_repo());
    use_case.execute().await
}

/// 列出已归档工作区
#[tauri::command]
#[specta::specta]
pub async fn list_archived_workspaces(
    state: State<'_, AppState>,
) -> Result<Vec<WorkspaceDto>, AppError> {
    let use_case = ListArchivedWorkspaces::new(state.workspace_repo());
    use_case.execute().await
}

/// 更新工作区
///
/// 完整表单提交：名称必填，描述可选。
#[tauri::command]
#[specta::specta]
pub async fn update_workspace(
    state: State<'_, AppState>,
    id: String,
    name: String,
    description: Option<String>,
) -> Result<WorkspaceDto, AppError> {
    let use_case = UpdateWorkspace::new(state.workspace_repo());
    use_case.execute(id, name, description).await
}

/// 归档工作区
///
/// 幂等操作：已归档工作区再次归档不会损坏状态。
#[tauri::command]
#[specta::specta]
pub async fn archive_workspace(
    state: State<'_, AppState>,
    id: String,
) -> Result<WorkspaceDto, AppError> {
    let use_case = ArchiveWorkspace::new(state.workspace_repo());
    use_case.execute(id).await
}

/// 恢复工作区
///
/// 幂等操作：活跃工作区再次恢复不会损坏状态。
#[tauri::command]
#[specta::specta]
pub async fn restore_workspace(
    state: State<'_, AppState>,
    id: String,
) -> Result<WorkspaceDto, AppError> {
    let use_case = RestoreWorkspace::new(state.workspace_repo());
    use_case.execute(id).await
}

/// 删除工作区
#[tauri::command]
#[specta::specta]
pub async fn delete_workspace(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let use_case = DeleteWorkspace::new(state.workspace_repo());
    use_case.execute(id).await
}

/// 标记工作区已打开
#[tauri::command]
#[specta::specta]
pub async fn mark_workspace_opened(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let use_case = MarkWorkspaceOpened::new(state.workspace_repo());
    use_case.execute(id).await
}
