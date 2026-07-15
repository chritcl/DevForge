use std::path::PathBuf;

use devforge_application::app_info::AppMetadata;
use devforge_application::ports::AppMetadataProvider;

/// 平台元数据提供者
///
/// 只提供版本和数据目录，不感知数据库状态。
/// version 和 data_dir 均由 Composition Root 注入，
/// 不在 Provider 内部自行决定版本号或重新调用 dirs::data_local_dir()。
pub struct PlatformMetadata {
    version: String,
    data_dir: PathBuf,
}

impl PlatformMetadata {
    pub fn new(version: String, data_dir: PathBuf) -> Self {
        Self { version, data_dir }
    }
}

impl AppMetadataProvider for PlatformMetadata {
    fn metadata(&self) -> AppMetadata {
        AppMetadata {
            version: self.version.clone(),
            data_dir: self.data_dir.to_string_lossy().into_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injected_version_and_data_dir_are_returned() {
        let meta = PlatformMetadata::new(
            "1.2.3".to_owned(),
            PathBuf::from("C:/Users/test/AppData/Local/DevForge"),
        );
        let info = meta.metadata();
        assert_eq!(info.version, "1.2.3");
        assert_eq!(info.data_dir, "C:/Users/test/AppData/Local/DevForge");
    }
}
