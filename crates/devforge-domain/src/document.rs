//! 文档领域模型

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::source::SourceId;

/// 文档 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub String);

impl DocumentId {
    /// 生成新的随机 ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 文档类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DocumentKind {
    /// 文本文件
    Text,
    /// Markdown 文件
    Markdown,
    /// 图片文件
    Image,
    /// 二进制文件
    Binary,
    /// 未知类型
    Unknown,
}

impl std::fmt::Display for DocumentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Markdown => write!(f, "markdown"),
            Self::Image => write!(f, "image"),
            Self::Binary => write!(f, "binary"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl DocumentKind {
    /// 从字符串解析文档类型
    pub fn parse_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(Self::Text),
            "markdown" => Some(Self::Markdown),
            "image" => Some(Self::Image),
            "binary" => Some(Self::Binary),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    /// 根据文件扩展名识别文档类型
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // 文本文件
            "txt" | "log" | "cfg" | "conf" | "ini" | "yml" | "yaml" | "toml" | "json" | "xml"
            | "csv" | "tsv" => Self::Text,
            // 代码文件
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "c" | "cpp" | "h"
            | "hpp" | "cs" | "rb" | "php" | "swift" | "kt" | "scala" | "lua" | "sh" | "bash"
            | "zsh" | "fish" | "ps1" | "bat" | "cmd" => Self::Text,
            // Markdown
            "md" | "markdown" | "mdx" => Self::Markdown,
            // 图片
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "ico" => Self::Image,
            // 二进制
            "exe" | "dll" | "so" | "dylib" | "bin" | "obj" | "o" | "a" | "lib" | "pdb" => {
                Self::Binary
            }
            // 其他
            _ => Self::Unknown,
        }
    }
}

/// 敏感度
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum Sensitivity {
    /// 普通文件
    Normal,
    /// 敏感文件
    Sensitive,
}

impl std::fmt::Display for Sensitivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::Sensitive => write!(f, "sensitive"),
        }
    }
}

impl Sensitivity {
    /// 从字符串解析敏感度
    pub fn parse_str(s: &str) -> Option<Self> {
        match s {
            "normal" => Some(Self::Normal),
            "sensitive" => Some(Self::Sensitive),
            _ => None,
        }
    }
}

/// 敏感文件模式列表
const SENSITIVE_PATTERNS: &[&str] = &[
    ".env",
    ".env.",
    "*.pem",
    "*.key",
    "*.pfx",
    "*.p12",
    "id_rsa",
    "id_ed25519",
    "credentials",
    "secrets",
];

/// 检查文件是否为敏感文件
pub fn is_sensitive(path: &Path) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };

    for pattern in SENSITIVE_PATTERNS {
        if let Some(ext) = pattern.strip_prefix("*.") {
            // 通配符模式：检查扩展名
            if file_name.to_lowercase().ends_with(&format!(".{ext}")) {
                return true;
            }
        } else if pattern.ends_with('.') {
            // 前缀模式：检查文件名是否以该前缀开头
            if file_name.starts_with(pattern) {
                return true;
            }
        } else {
            // 精确匹配
            if file_name == *pattern {
                return true;
            }
        }
    }

    false
}

/// 文档实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// 文档 ID
    pub id: DocumentId,
    /// 所属数据源 ID
    pub source_id: SourceId,
    /// 相对路径
    pub relative_path: PathBuf,
    /// 文档类型
    pub kind: DocumentKind,
    /// 文件大小
    pub size: u64,
    /// 修改时间
    pub modified_at: DateTime<Utc>,
    /// 敏感度
    pub sensitivity: Sensitivity,
    /// 是否可读取内容
    pub content_readable: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl Document {
    /// 创建新文档
    pub fn new(
        source_id: SourceId,
        relative_path: PathBuf,
        size: u64,
        modified_at: DateTime<Utc>,
    ) -> Self {
        let kind = relative_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(DocumentKind::from_extension)
            .unwrap_or(DocumentKind::Unknown);

        let sensitivity = if is_sensitive(&relative_path) {
            Sensitivity::Sensitive
        } else {
            Sensitivity::Normal
        };

        let content_readable = sensitivity == Sensitivity::Normal && kind != DocumentKind::Binary;

        Self {
            id: DocumentId::new(),
            source_id,
            relative_path,
            kind,
            size,
            modified_at,
            sensitivity,
            content_readable,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn document_kind_from_extension() {
        assert_eq!(DocumentKind::from_extension("rs"), DocumentKind::Text);
        assert_eq!(DocumentKind::from_extension("md"), DocumentKind::Markdown);
        assert_eq!(DocumentKind::from_extension("png"), DocumentKind::Image);
        assert_eq!(DocumentKind::from_extension("exe"), DocumentKind::Binary);
        assert_eq!(DocumentKind::from_extension("xyz"), DocumentKind::Unknown);
    }

    #[test]
    fn is_sensitive_env() {
        assert!(is_sensitive(&PathBuf::from(".env")));
        assert!(is_sensitive(&PathBuf::from(".env.production")));
        assert!(is_sensitive(&PathBuf::from(".env.local")));
    }

    #[test]
    fn is_sensitive_keys() {
        assert!(is_sensitive(&PathBuf::from("server.pem")));
        assert!(is_sensitive(&PathBuf::from("id_rsa")));
        assert!(is_sensitive(&PathBuf::from("id_ed25519")));
    }

    #[test]
    fn is_not_sensitive() {
        assert!(!is_sensitive(&PathBuf::from("README.md")));
        assert!(!is_sensitive(&PathBuf::from("src/main.rs")));
    }

    #[test]
    fn document_creation() {
        let source_id = SourceId::new();
        let doc = Document::new(source_id, PathBuf::from("src/main.rs"), 1024, Utc::now());
        assert_eq!(doc.kind, DocumentKind::Text);
        assert_eq!(doc.sensitivity, Sensitivity::Normal);
        assert!(doc.content_readable);
    }

    #[test]
    fn sensitive_document_not_readable() {
        let source_id = SourceId::new();
        let doc = Document::new(source_id, PathBuf::from(".env"), 100, Utc::now());
        assert_eq!(doc.sensitivity, Sensitivity::Sensitive);
        assert!(!doc.content_readable);
    }

    #[test]
    fn binary_document_not_readable() {
        let source_id = SourceId::new();
        let doc = Document::new(source_id, PathBuf::from("app.exe"), 1024, Utc::now());
        assert_eq!(doc.kind, DocumentKind::Binary);
        assert!(!doc.content_readable);
    }
}
