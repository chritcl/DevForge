use serde::Serialize;
use specta::Type;

/// 应用元数据（由 Platform Adapter 提供）
///
/// 只包含版本和数据目录，不包含数据库状态。
/// Platform Adapter 不需要感知数据库的存在。
#[derive(Debug, Clone)]
pub struct AppMetadata {
    pub version: String,
    pub data_dir: String,
}

/// 应用基础信息（诊断 DTO，IPC 输出）
///
/// 由 GetAppInfo Use Case 组合 AppMetadata + DbStatus 生成。
/// derive Type 用于 specta 自动生成 TypeScript 类型。
/// 仅派生 Serialize（IPC 输出），不派生 Deserialize。
#[derive(Debug, Clone, Serialize, Type)]
pub struct AppInfo {
    pub version: String,
    pub data_dir: String,
    pub db_status: DbStatus,
}

/// 数据库状态
///
/// `#[serde(tag = "type")]` 使 serde 生成内部标签表示：
/// `{ "type": "NotInitialized" } | { "type": "Ready", "migration_version": 1 } | ...`
/// specta 尊重 serde 标签策略，生成对应的 TypeScript tagged union。
/// 仅派生 Serialize（IPC 输出），不无意义地派生 Deserialize。
#[derive(Debug, Clone, Serialize, Default, Type)]
#[serde(tag = "type")]
pub enum DbStatus {
    #[default]
    NotInitialized,
    Ready {
        migration_version: u32,
    },
    Error {
        message: String,
    },
}
