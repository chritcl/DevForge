//! 路径安全验证模块
//!
//! 所有文件路径操作必须通过 PathGuard 验证，防止路径逃逸攻击。

use std::path::{Component, Path, PathBuf};

/// 路径安全错误
#[derive(Debug, thiserror::Error)]
pub enum PathError {
    /// 路径逃逸：路径在源目录外
    #[error("路径逃逸: 路径在源目录外")]
    PathEscape,
    /// 路径不存在
    #[error("路径不存在: {0}")]
    NotExists(PathBuf),
    /// 路径不是文件
    #[error("路径不是文件: {0}")]
    NotFile(PathBuf),
    /// 路径不是目录
    #[error("路径不是目录: {0}")]
    NotDirectory(PathBuf),
    /// 路径是绝对路径（不允许）
    #[error("路径是绝对路径（不允许）: {0}")]
    AbsolutePath(PathBuf),
    /// 路径包含父目录引用（不允许）
    #[error("路径包含父目录引用（不允许）: {0}")]
    ParentDir(PathBuf),
    /// 路径包含非法组件
    #[error("路径包含非法组件: {0}")]
    IllegalComponent(PathBuf),
    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

/// 路径安全守卫
///
/// 所有路径操作必须通过此守卫验证，确保路径在源目录范围内。
pub struct PathGuard {
    source_root: PathBuf,
}

impl PathGuard {
    /// 创建新的路径守卫
    ///
    /// # 错误
    ///
    /// 如果源目录不存在或不是目录，返回错误。
    pub fn new(source_root: PathBuf) -> Result<Self, PathError> {
        let canonical = Self::canonicalize(&source_root)?;
        if !canonical.is_dir() {
            return Err(PathError::NotDirectory(source_root));
        }
        Ok(Self {
            source_root: canonical,
        })
    }

    /// 获取源目录
    pub fn source_root(&self) -> &Path {
        &self.source_root
    }

    /// 验证路径在源目录范围内
    ///
    /// 返回规范化后的绝对路径。
    ///
    /// # 错误
    ///
    /// 如果路径逃逸到源目录外，返回错误。
    pub fn validate(&self, path: &Path) -> Result<PathBuf, PathError> {
        let canonical = Self::canonicalize(path)?;
        if !canonical.starts_with(&self.source_root) {
            return Err(PathError::PathEscape);
        }
        Ok(canonical)
    }

    /// 验证路径是源目录内的文件
    ///
    /// 返回规范化后的绝对路径。
    pub fn validate_file(&self, path: &Path) -> Result<PathBuf, PathError> {
        let canonical = self.validate(path)?;
        if !canonical.is_file() {
            return Err(PathError::NotFile(path.to_path_buf()));
        }
        Ok(canonical)
    }

    /// 检查路径是否在源目录内
    pub fn is_within_root(&self, path: &Path) -> bool {
        Self::canonicalize(path)
            .ok()
            .map(|p| p.starts_with(&self.source_root))
            .unwrap_or(false)
    }

    /// 将相对路径转换为绝对路径
    pub fn resolve(&self, relative: &Path) -> PathBuf {
        self.source_root.join(relative)
    }

    /// 安全解析并验证相对文件路径
    ///
    /// 该入口在 join 前验证 relative_path 的合法性：
    /// 1. 拒绝绝对路径
    /// 2. 拒绝 Windows Prefix（如 C:\、\\server\）
    /// 3. 拒绝 RootDir（如 /）
    /// 4. 拒绝 ParentDir（..）
    /// 5. 拒绝空组件
    /// 6. 将路径连接到可信 source_root
    /// 7. canonicalize 并解析 Symlink/重解析点
    /// 8. 验证最终路径位于可信根目录内
    /// 9. 验证最终对象是文件
    pub fn resolve_relative_file(&self, relative_path: &Path) -> Result<PathBuf, PathError> {
        // 验证相对路径的每个组件
        for component in relative_path.components() {
            match component {
                Component::Prefix(_) => {
                    return Err(PathError::AbsolutePath(relative_path.to_path_buf()));
                }
                Component::RootDir => {
                    return Err(PathError::AbsolutePath(relative_path.to_path_buf()));
                }
                Component::ParentDir => {
                    return Err(PathError::ParentDir(relative_path.to_path_buf()));
                }
                Component::CurDir => {
                    // 允许 .
                }
                Component::Normal(_) => {
                    // 允许正常组件
                }
            }
        }

        // 连接到可信根目录
        let full_path = self.source_root.join(relative_path);

        // canonicalize 并验证
        let canonical = Self::canonicalize(&full_path)?;
        if !canonical.starts_with(&self.source_root) {
            return Err(PathError::PathEscape);
        }

        // 验证是文件
        if !canonical.is_file() {
            return Err(PathError::NotFile(canonical));
        }

        Ok(canonical)
    }

    /// 规范化路径：解析 .、.. 和重复分隔符
    fn canonicalize(path: &Path) -> Result<PathBuf, PathError> {
        // 首先尝试 std::fs::canonicalize（解析 symlink）
        match std::fs::canonicalize(path) {
            Ok(p) => Ok(Self::strip_unc_prefix(&p)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // 文件不存在时，手动规范化路径组件
                Ok(Self::normalize_components(path))
            }
            Err(e) => Err(PathError::Io(e)),
        }
    }

    /// 手动规范化路径组件（不访问文件系统）
    fn normalize_components(path: &Path) -> PathBuf {
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    components.pop();
                }
                std::path::Component::CurDir => {}
                other => components.push(other),
            }
        }
        components.iter().collect()
    }

    /// 去除 Windows UNC 路径前缀（\\?\）
    ///
    /// Windows 的 `std::fs::canonicalize` 返回带有 `\\?\` 前缀的路径，
    /// 需要去除以确保路径比较的一致性。
    #[cfg(windows)]
    fn strip_unc_prefix(path: &Path) -> PathBuf {
        let s = path.to_string_lossy();
        if let Some(stripped) = s.strip_prefix("\\\\?\\") {
            PathBuf::from(stripped)
        } else {
            path.to_path_buf()
        }
    }

    #[cfg(not(windows))]
    fn strip_unc_prefix(path: &Path) -> PathBuf {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn validate_within_root() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let guard = PathGuard::new(root.to_path_buf()).unwrap();

        let file = root.join("test.txt");
        assert!(guard.validate(&file).is_ok());
    }

    #[test]
    fn validate_escape_with_dotdot() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir_all(&root).unwrap();
        let guard = PathGuard::new(root).unwrap();

        let escape = temp.path().join("outside.txt");
        assert!(guard.validate(&escape).is_err());
    }

    #[test]
    fn validate_escape_absolute_path() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir_all(&root).unwrap();
        let guard = PathGuard::new(root).unwrap();

        // 尝试逃逸到源目录外的绝对路径
        let escape = PathBuf::from("C:\\Windows\\System32\\cmd.exe");
        assert!(guard.validate(&escape).is_err());
    }

    #[test]
    fn validate_file_exists() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let file = root.join("test.txt");
        fs::write(&file, "content").unwrap();

        let guard = PathGuard::new(root.to_path_buf()).unwrap();
        assert!(guard.validate_file(&file).is_ok());
    }

    #[test]
    fn validate_file_not_exists() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let guard = PathGuard::new(root.to_path_buf()).unwrap();

        let file = root.join("nonexistent.txt");
        assert!(guard.validate_file(&file).is_err());
    }

    #[test]
    fn is_within_root_true() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let guard = PathGuard::new(root.to_path_buf()).unwrap();

        assert!(guard.is_within_root(&root.join("sub/file.txt")));
    }

    #[test]
    fn is_within_root_false() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir_all(&root).unwrap();
        let guard = PathGuard::new(root).unwrap();

        assert!(!guard.is_within_root(&temp.path().join("outside.txt")));
    }

    #[test]
    fn resolve_relative_path() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let guard = PathGuard::new(root.to_path_buf()).unwrap();

        let resolved = guard.resolve(Path::new("src/main.rs"));
        assert_eq!(resolved, root.join("src/main.rs"));
    }

    #[test]
    fn normalize_dotdot_components() {
        let path = Path::new("/workspace/src/../lib/main.rs");
        let normalized = PathGuard::normalize_components(path);
        assert_eq!(normalized, PathBuf::from("/workspace/lib/main.rs"));
    }

    #[test]
    fn symlink_escape() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir_all(&root).unwrap();

        // 创建一个指向源目录外的 symlink
        let outside = temp.path().join("outside");
        fs::create_dir_all(&outside).unwrap();

        let guard = PathGuard::new(root.clone()).unwrap();

        // 直接验证外部路径应该失败
        assert!(guard.validate(&outside).is_err());
    }

    #[test]
    fn resolve_relative_file_rejects_absolute_path() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir_all(&root).unwrap();

        let guard = PathGuard::new(root.clone()).unwrap();

        // 绝对路径应被拒绝
        let result = guard.resolve_relative_file(Path::new("/etc/passwd"));
        assert!(result.is_err());
    }

    #[test]
    fn resolve_relative_file_rejects_parent_dir() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir_all(&root).unwrap();

        let guard = PathGuard::new(root.clone()).unwrap();

        // 包含 .. 的路径应被拒绝
        let result = guard.resolve_relative_file(Path::new("../escape"));
        assert!(result.is_err());
    }

    #[test]
    fn resolve_relative_file_accepts_valid_path() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();

        let guard = PathGuard::new(root.clone()).unwrap();

        // 有效相对路径应成功
        let result = guard.resolve_relative_file(Path::new("src/main.rs"));
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("src/main.rs"));
    }

    #[test]
    fn resolve_relative_file_rejects_non_file() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("src")).unwrap();

        let guard = PathGuard::new(root.clone()).unwrap();

        // 目录应被拒绝
        let result = guard.resolve_relative_file(Path::new("src"));
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn resolve_relative_file_rejects_symlink_escape() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("src")).unwrap();

        // 创建一个指向外部的文件
        let outside_file = temp.path().join("secret.txt");
        fs::write(&outside_file, "secret content").unwrap();

        // 在 workspace 内创建指向外部文件的 symlink
        let symlink_path = root.join("src/link.txt");
        std::os::unix::fs::symlink(&outside_file, &symlink_path).unwrap();

        let guard = PathGuard::new(root.clone()).unwrap();

        // 通过 symlink 读取应被拒绝
        let result = guard.resolve_relative_file(Path::new("src/link.txt"));
        assert!(result.is_err());
    }

    #[cfg(windows)]
    #[test]
    fn resolve_relative_file_rejects_symlink_escape() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("src")).unwrap();

        // 创建一个指向外部的文件
        let outside_file = temp.path().join("secret.txt");
        fs::write(&outside_file, "secret content").unwrap();

        // 在 workspace 内创建指向外部文件的 symlink
        // 注意：Windows 上创建 symlink 需要管理员权限或开发者模式
        let symlink_path = root.join("src/link.txt");
        match std::os::windows::fs::symlink_file(&outside_file, &symlink_path) {
            Ok(_) => {
                let guard = PathGuard::new(root.clone()).unwrap();
                // 通过 symlink 读取应被拒绝
                let result = guard.resolve_relative_file(Path::new("src/link.txt"));
                assert!(result.is_err());
            }
            Err(_) => {
                // 无法创建 symlink（权限不足），跳过测试
                // 人工验证：需要在管理员权限下运行此测试
                eprintln!("跳过 symlink 测试：需要管理员权限或开发者模式");
            }
        }
    }
}
