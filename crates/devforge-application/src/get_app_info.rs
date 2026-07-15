use crate::app_info::AppInfo;
use crate::ports::{AppMetadataProvider, DatabaseStatusProvider};

/// 获取应用信息用例
///
/// 职责：组合 AppMetadataProvider（版本、数据目录）和 DatabaseStatusProvider（数据库状态），
/// 生成最终的 AppInfo DTO。
/// Platform Adapter 不需要知道 DbStatus，Storage Adapter 不需要知道版本号。
pub struct GetAppInfo<M: AppMetadataProvider, D: DatabaseStatusProvider> {
    app_metadata: M,
    db_status: D,
}

impl<M: AppMetadataProvider, D: DatabaseStatusProvider> GetAppInfo<M, D> {
    pub fn new(app_metadata: M, db_status: D) -> Self {
        Self {
            app_metadata,
            db_status,
        }
    }

    /// 执行用例，组合元数据和数据库状态
    pub async fn execute(&self) -> AppInfo {
        let metadata = self.app_metadata.metadata();
        let db_status = self.db_status.status().await;

        AppInfo {
            version: metadata.version,
            data_dir: metadata.data_dir,
            db_status,
        }
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::app_info::{AppMetadata, DbStatus};

    struct MockAppMetadata;
    impl AppMetadataProvider for MockAppMetadata {
        fn metadata(&self) -> AppMetadata {
            AppMetadata {
                version: "0.1.0".into(),
                data_dir: "C:/test/DevForge".into(),
            }
        }
    }

    struct MockDbReady;
    #[async_trait]
    impl DatabaseStatusProvider for MockDbReady {
        async fn status(&self) -> DbStatus {
            DbStatus::Ready {
                migration_version: 1,
            }
        }
    }

    #[tokio::test]
    async fn get_app_info_composes_metadata_and_db_status() {
        let use_case = GetAppInfo::new(MockAppMetadata, MockDbReady);
        let info = use_case.execute().await;

        // 验证 AppMetadataProvider 提供版本和数据目录
        assert_eq!(info.version, "0.1.0");
        assert_eq!(info.data_dir, "C:/test/DevForge");

        // 验证 DatabaseStatusProvider 提供数据库状态
        assert!(matches!(
            info.db_status,
            DbStatus::Ready {
                migration_version: 1
            }
        ));
    }

    #[tokio::test]
    async fn platform_provider_does_not_know_db_status() {
        // MockAppMetadata 不包含任何 DbStatus 字段
        // 这证明 Platform Provider 不需要知道数据库状态
        let metadata = MockAppMetadata.metadata();
        assert_eq!(metadata.version, "0.1.0");
        assert_eq!(metadata.data_dir, "C:/test/DevForge");
    }
}
