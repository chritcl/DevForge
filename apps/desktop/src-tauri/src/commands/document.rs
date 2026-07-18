//! 文档 Tauri 命令

use tauri::State;

use devforge_application::document::{
    DocumentDto, DocumentError, DocumentLookupDto, FileTreeEntryDto, GetDocumentsByIds,
    ListDocuments, ListFileTree, ReadDocumentContent,
};

use crate::state::AppState;

/// 列出文档
#[tauri::command]
#[specta::specta]
pub async fn list_documents(
    state: State<'_, AppState>,
    source_id: String,
    parent_path: Option<String>,
) -> Result<Vec<DocumentDto>, DocumentError> {
    let use_case = ListDocuments::new(state.document_repo());
    use_case.execute(source_id, parent_path).await
}

/// 列出文件树条目
#[tauri::command]
#[specta::specta]
pub async fn list_file_tree(
    state: State<'_, AppState>,
    source_id: String,
    parent_path: Option<String>,
) -> Result<Vec<FileTreeEntryDto>, DocumentError> {
    let use_case = ListFileTree::new(state.document_repo());
    use_case.execute(source_id, parent_path).await
}

/// 读取文档内容（不需要 source_root，后端从数据库反查可信根目录）
#[tauri::command]
#[specta::specta]
pub async fn read_document_content(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<String, DocumentError> {
    let use_case = ReadDocumentContent::new(state.document_repo(), state.source_repo());
    use_case.execute(document_id).await
}

/// 批量获取文档信息
#[tauri::command]
#[specta::specta]
pub async fn get_documents_by_ids(
    state: State<'_, AppState>,
    document_ids: Vec<String>,
) -> Result<Vec<DocumentLookupDto>, DocumentError> {
    let use_case = GetDocumentsByIds::new(state.document_repo(), state.source_repo());
    use_case.execute(document_ids).await
}
