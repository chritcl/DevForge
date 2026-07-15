use std::path::PathBuf;

use devforge_application::app_info::AppInfo;
use devforge_application::get_app_info::GetAppInfo;
use devforge_application::ports::NotInitializedDbStatus;
use devforge_platform::app_info::PlatformMetadata;

/// 应用全局状态
///
/// 持有 Application Use Case，Tauri Command 通过窄接口调用业务逻辑。
/// 不暴露公开字段，Command 不直接访问内部结构。
pub struct AppState {
    get_app_info: GetAppInfo<PlatformMetadata, NotInitializedDbStatus>,
}

impl AppState {
    pub fn new(version: String, data_dir: PathBuf) -> Self {
        let platform_metadata = PlatformMetadata::new(version, data_dir);
        let db_status = NotInitializedDbStatus;

        Self {
            get_app_info: GetAppInfo::new(platform_metadata, db_status),
        }
    }

    pub async fn app_info(&self) -> AppInfo {
        self.get_app_info.execute().await
    }
}
