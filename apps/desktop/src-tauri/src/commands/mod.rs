//! Tauri 命令模块

pub mod app_info;
pub mod discovery;
pub mod document;
pub mod index;
pub mod search;
pub mod source;
pub mod tab;
pub mod workspace;

pub use app_info::*;
pub use discovery::*;
pub use document::*;
pub use index::*;
pub use search::*;
pub use source::*;
pub use tab::*;
pub use workspace::*;
