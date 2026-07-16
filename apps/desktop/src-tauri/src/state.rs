use devforge_application::app_info::AppInfo;
use devforge_application::get_app_info::GetAppInfo;
use devforge_platform::app_info::PlatformMetadata;
use devforge_storage::status::SqliteDatabaseStatus;

/// 应用全局状态。
///
/// 只暴露应用用例级接口，不向 Command 暴露 SQLite 连接池。
pub(crate) struct AppState {
    get_app_info: GetAppInfo<PlatformMetadata, SqliteDatabaseStatus>,
}

impl AppState {
    /// 使用完整初始化的平台和数据库 Provider 构造应用状态。
    pub(crate) fn new(
        platform_metadata: PlatformMetadata,
        database_status: SqliteDatabaseStatus,
    ) -> Self {
        Self {
            get_app_info: GetAppInfo::new(platform_metadata, database_status),
        }
    }

    /// 获取应用信息。
    pub(crate) async fn app_info(&self) -> AppInfo {
        self.get_app_info.execute().await
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

                let state = AppState::new(platform_metadata, database_status);

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
                        migration_version: 1,
                    }
                ),
                "数据库状态应为 Ready，migration_version 应为 1",
            );

            Ok(())
        })
    }
}
