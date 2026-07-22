use std::sync::Arc;

use devforge_application::app_info::AppInfo;
use devforge_application::get_app_info::GetAppInfo;
use devforge_platform::app_info::PlatformMetadata;
use devforge_storage::indexer::WorkspaceIndex;
use devforge_storage::repository::{
    SqliteDocumentRepository, SqliteOpenTabRepository, SqliteSourceRepository,
    SqliteWorkspaceRepository,
};
use devforge_storage::status::SqliteDatabaseStatus;

/// 应用全局状态。
///
/// 只暴露应用用例级接口，不向 Command 暴露 SQLite 连接池。
pub(crate) struct AppState {
    get_app_info: GetAppInfo<PlatformMetadata, SqliteDatabaseStatus>,
    workspace_repo: Arc<SqliteWorkspaceRepository>,
    source_repo: Arc<SqliteSourceRepository>,
    document_repo: Arc<SqliteDocumentRepository>,
    tab_repo: Arc<SqliteOpenTabRepository>,
    data_dir: std::path::PathBuf,
}

impl AppState {
    /// 使用完整初始化的平台和数据库 Provider 构造应用状态。
    pub(crate) fn new(
        platform_metadata: PlatformMetadata,
        database_status: SqliteDatabaseStatus,
        workspace_repo: Arc<SqliteWorkspaceRepository>,
        source_repo: Arc<SqliteSourceRepository>,
        document_repo: Arc<SqliteDocumentRepository>,
        tab_repo: Arc<SqliteOpenTabRepository>,
        data_dir: std::path::PathBuf,
    ) -> Self {
        Self {
            get_app_info: GetAppInfo::new(platform_metadata, database_status),
            workspace_repo,
            source_repo,
            document_repo,
            tab_repo,
            data_dir,
        }
    }

    /// 获取应用信息。
    pub(crate) async fn app_info(&self) -> AppInfo {
        self.get_app_info.execute().await
    }

    /// 获取工作区索引。
    ///
    /// 如果索引目录不存在则创建新索引。
    pub(crate) fn workspace_index(
        &self,
        workspace_id: &str,
    ) -> Result<WorkspaceIndex, devforge_storage::indexer::IndexError> {
        let index_dir =
            devforge_storage::indexer::workspace_index_dir(workspace_id, &self.data_dir);
        WorkspaceIndex::open(&index_dir)
    }

    /// 获取工作区 Repository。
    pub(crate) fn workspace_repo(&self) -> Arc<SqliteWorkspaceRepository> {
        self.workspace_repo.clone()
    }

    /// 获取数据源 Repository。
    pub(crate) fn source_repo(&self) -> Arc<SqliteSourceRepository> {
        self.source_repo.clone()
    }

    /// 获取文档 Repository。
    pub(crate) fn document_repo(&self) -> Arc<SqliteDocumentRepository> {
        self.document_repo.clone()
    }

    /// 获取标签页 Repository。
    pub(crate) fn tab_repo(&self) -> Arc<SqliteOpenTabRepository> {
        self.tab_repo.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devforge_application::app_info::DbStatus;

    #[test]
    fn app_state_returns_real_database_status() -> Result<(), Box<dyn std::error::Error>> {
        tauri::async_runtime::block_on(async {
            let temp_dir = tempfile::tempdir()?;
            let data_dir = temp_dir.path().join("DevForge");
            std::fs::create_dir_all(&data_dir)?;

            let db_path = data_dir.join("devforge.db");
            let database = devforge_storage::pool::Database::open(&db_path).await?;

            let pool = database.pool().clone();

            let observed: Result<_, Box<dyn std::error::Error>> = async {
                devforge_storage::migrator::run_migrations(database.pool()).await?;

                let platform_metadata = devforge_platform::app_info::PlatformMetadata::new(
                    "test-version".to_owned(),
                    data_dir.clone(),
                );

                let database_status =
                    devforge_storage::status::SqliteDatabaseStatus::new(pool.clone());

                let workspace_repo = Arc::new(SqliteWorkspaceRepository::new(pool.clone()));
                let source_repo = Arc::new(SqliteSourceRepository::new(pool.clone()));
                let document_repo = Arc::new(SqliteDocumentRepository::new(pool.clone()));
                let tab_repo = Arc::new(SqliteOpenTabRepository::new(pool.clone()));

                let state = AppState::new(
                    platform_metadata,
                    database_status,
                    workspace_repo,
                    source_repo,
                    document_repo,
                    tab_repo,
                    data_dir.clone(),
                );

                let info = state.app_info().await;

                Ok((state, info))
            }
            .await;

            let (state, info) = match observed {
                Ok(value) => value,
                Err(error) => {
                    drop(database);
                    pool.close().await;
                    return Err(error);
                }
            };

            drop(state);
            drop(database);
            pool.close().await;

            let expected_data_dir = data_dir.to_string_lossy().into_owned();

            assert_eq!(info.version, "test-version");
            assert_eq!(info.data_dir, expected_data_dir);
            assert!(
                matches!(
                    info.db_status,
                    DbStatus::Ready {
                        migration_version: 5,
                    }
                ),
                "数据库状态应为 Ready，migration_version 应为 5",
            );

            Ok(())
        })
    }
}
