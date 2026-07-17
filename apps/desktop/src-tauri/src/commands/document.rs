//! 文档 Tauri 命令

use std::path::PathBuf;

use tauri::State;

use devforge_application::document::{
    DocumentDto, DocumentError, ListDocuments, ReadDocumentContent,
};

use crate::state::AppState;

/// 列出文档
#[tauri::command]
pub async fn list_documents(
    state: State<'_, AppState>,
    source_id: String,
    parent_path: Option<String>,
) -> Result<Vec<DocumentDto>, DocumentError> {
    let use_case = ListDocuments::new(state.document_repo());
    use_case.execute(source_id, parent_path).await
}

/// 读取文档内容
#[tauri::command]
pub async fn read_document_content(
    state: State<'_, AppState>,
    document_id: String,
    source_root: String,
) -> Result<String, DocumentError> {
    let use_case = ReadDocumentContent::new(state.document_repo());
    use_case
        .execute(document_id, PathBuf::from(source_root))
        .await
}
