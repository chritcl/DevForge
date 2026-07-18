//! 数据源用例

use std::path::PathBuf;
use std::sync::Arc;

use devforge_domain::error::DomainError;
use devforge_domain::path_guard::PathError;
use devforge_domain::source::{Source, SourceId, SourceKind};
use devforge_domain::workspace::WorkspaceId;

/// 数据源 DTO（用于 IPC 传输）
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct SourceDto {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub root_path: String,
    pub kind: SourceKind,
    pub created_at: String,
}

impl From<&Source> for SourceDto {
    fn from(source: &Source) -> Self {
        Self {
            id: source.id.0.clone(),
            workspace_id: source.workspace_id.0.clone(),
            name: source.name.clone(),
            root_path: source.root_path.to_string_lossy().to_string(),
            kind: source.kind.clone(),
            created_at: source.created_at.to_rfc3339(),
        }
    }
}

/// Source Repository Trait（应用层端口）
#[async_trait::async_trait]
pub trait SourceRepository: Send + Sync {
    async fn create(&self, source: &Source) -> Result<(), DomainError>;
    async fn get(&self, id: &SourceId) -> Result<Option<Source>, DomainError>;
    async fn list_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<Source>, DomainError>;
    async fn delete(&self, id: &SourceId) -> Result<(), DomainError>;
}

/// 应用层错误
#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum SourceError {
    #[error("路径错误: {0}")]
    Path(String),
    #[error("数据源不存在")]
    SourceNotFound,
    #[error("路径已存在")]
    PathAlreadyExists,
    #[error("路径不存在: {0}")]
    PathNotExists(String),
    #[error("路径不是目录: {0}")]
    NotDirectory(String),
    #[error("无权限访问: {0}")]
    PermissionDenied(String),
    #[error("不是 Git 仓库: {0}")]
    NotGitRepository(String),
    #[error("领域错误: {0}")]
    Domain(String),
}

impl From<DomainError> for SourceError {
    fn from(err: DomainError) -> Self {
        SourceError::Domain(err.to_string())
    }
}

impl From<PathError> for SourceError {
    fn from(err: PathError) -> Self {
        SourceError::Path(err.to_string())
    }
}

/// 规范化路径用于比较和存储
///
/// 使用 std::fs::canonicalize 解析 symlink 和相对路径。
/// Windows 下统一处理 \\?\ 前缀。
/// 返回的路径用于数据库存储和重复检测。
pub fn normalize_path(path: &std::path::Path) -> Result<PathBuf, SourceError> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| SourceError::Path(format!("无法规范化路径 {}: {}", path.display(), e)))?;

    // Windows 下去除 \\?\ 前缀
    #[cfg(windows)]
    let canonical = {
        let s = canonical.to_string_lossy();
        if let Some(stripped) = s.strip_prefix("\\\\?\\") {
            PathBuf::from(stripped)
        } else {
            canonical
        }
    };

    Ok(canonical)
}

/// 生成路径的比较键（Windows 大小写不敏感）
///
/// 用于重复路径检测，不用于存储。
pub fn path_comparison_key(path: &std::path::Path) -> String {
    path.to_string_lossy().to_lowercase()
}

/// 检测路径对应的数据源类型
///
/// 1. .git 是目录 → 普通 Git 仓库
/// 2. .git 是文件且包含合法 gitdir: 指向 → Git Worktree
/// 3. 否则 → 普通目录
pub fn detect_source_kind(root_path: &std::path::Path) -> SourceKind {
    let git_path = root_path.join(".git");
    if !git_path.exists() {
        return SourceKind::Directory;
    }
    if git_path.is_dir() {
        return SourceKind::Git;
    }
    // .git 是文件，检查是否为 Worktree
    if let Ok(content) = std::fs::read_to_string(&git_path) {
        for line in content.lines() {
            if let Some(gitdir_value) = line.strip_prefix("gitdir:") {
                let gitdir_value = gitdir_value.trim();
                // 解析 gitdir 路径（可能是相对或绝对）
                let gitdir_path = if std::path::Path::new(gitdir_value).is_absolute() {
                    std::path::PathBuf::from(gitdir_value)
                } else {
                    // 相对路径以 Worktree 根目录为基准
                    root_path.join(gitdir_value)
                };
                // 验证目标存在且为目录
                if gitdir_path.exists() && gitdir_path.is_dir() {
                    return SourceKind::Git;
                }
            }
        }
    }
    // .git 文件无效或 gitdir 目标不存在，视为普通目录
    SourceKind::Directory
}

/// 添加本地数据源用例（统一入口）
///
/// 后端自动识别 Git 仓库、Git Worktree 或普通目录。
/// 添加前 canonicalize 路径，防止重复添加。
pub struct AddLocalSource {
    source_repo: Arc<dyn SourceRepository>,
}

impl AddLocalSource {
    pub fn new(source_repo: Arc<dyn SourceRepository>) -> Self {
        Self { source_repo }
    }

    /// 添加本地数据源
    pub async fn execute(
        &self,
        workspace_id: String,
        path: PathBuf,
    ) -> Result<SourceDto, SourceError> {
        let workspace_id = WorkspaceId(workspace_id);

        // 验证路径存在且是目录
        if !path.exists() {
            return Err(SourceError::PathNotExists(path.display().to_string()));
        }
        if !path.is_dir() {
            return Err(SourceError::NotDirectory(path.display().to_string()));
        }

        // 规范化路径
        let canonical_path = normalize_path(&path)?;

        // 检查是否已存在（使用规范化后的路径比较）
        let existing = self.source_repo.list_by_workspace(&workspace_id).await?;
        let new_key = path_comparison_key(&canonical_path);
        for src in &existing {
            let existing_key = path_comparison_key(&src.root_path);
            if existing_key == new_key {
                return Err(SourceError::PathAlreadyExists);
            }
        }

        // 自动识别数据源类型
        let kind = detect_source_kind(&canonical_path);

        // 从路径提取名称
        let name = canonical_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "未命名".to_owned());

        let source = match kind {
            SourceKind::Git => Source::new_git(workspace_id, name, canonical_path),
            SourceKind::Directory => Source::new_directory(workspace_id, name, canonical_path),
        };
        self.source_repo.create(&source).await?;
        Ok(SourceDto::from(&source))
    }
}

/// 添加 Git 数据源用例（已废弃，请使用 AddLocalSource）
#[deprecated(note = "请使用 AddLocalSource")]
pub struct AddGitSource {
    source_repo: Arc<dyn SourceRepository>,
}

#[allow(deprecated)]
impl AddGitSource {
    pub fn new(source_repo: Arc<dyn SourceRepository>) -> Self {
        Self { source_repo }
    }

    pub async fn execute(
        &self,
        workspace_id: String,
        path: PathBuf,
    ) -> Result<SourceDto, SourceError> {
        let use_case = AddLocalSource::new(self.source_repo.clone());
        let result = use_case.execute(workspace_id, path).await?;
        // 验证确实是 Git 类型
        if result.kind != SourceKind::Git {
            return Err(SourceError::NotGitRepository(result.root_path));
        }
        Ok(result)
    }
}

/// 添加目录数据源用例（已废弃，请使用 AddLocalSource）
#[deprecated(note = "请使用 AddLocalSource")]
pub struct AddDirectorySource {
    source_repo: Arc<dyn SourceRepository>,
}

#[allow(deprecated)]
impl AddDirectorySource {
    pub fn new(source_repo: Arc<dyn SourceRepository>) -> Self {
        Self { source_repo }
    }

    pub async fn execute(
        &self,
        workspace_id: String,
        path: PathBuf,
    ) -> Result<SourceDto, SourceError> {
        // 直接使用 AddLocalSource，不限制类型
        let use_case = AddLocalSource::new(self.source_repo.clone());
        use_case.execute(workspace_id, path).await
    }
}

/// 列出数据源用例
pub struct ListSources {
    source_repo: Arc<dyn SourceRepository>,
}

impl ListSources {
    pub fn new(source_repo: Arc<dyn SourceRepository>) -> Self {
        Self { source_repo }
    }

    pub async fn execute(&self, workspace_id: String) -> Result<Vec<SourceDto>, SourceError> {
        let workspace_id = WorkspaceId(workspace_id);
        let sources = self.source_repo.list_by_workspace(&workspace_id).await?;
        Ok(sources.iter().map(SourceDto::from).collect())
    }
}

/// 移除数据源用例
///
/// 注意：移除数据源只删除元数据，不删除源目录。
pub struct RemoveSource {
    source_repo: Arc<dyn SourceRepository>,
}

impl RemoveSource {
    pub fn new(source_repo: Arc<dyn SourceRepository>) -> Self {
        Self { source_repo }
    }

    pub async fn execute(&self, id: String) -> Result<(), SourceError> {
        let source_id = SourceId(id);
        // 验证数据源存在
        self.source_repo
            .get(&source_id)
            .await?
            .ok_or(SourceError::SourceNotFound)?;

        self.source_repo.delete(&source_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct InMemorySourceRepository {
        sources: Mutex<HashMap<String, Source>>,
    }

    impl InMemorySourceRepository {
        fn new() -> Self {
            Self {
                sources: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl SourceRepository for InMemorySourceRepository {
        async fn create(&self, source: &Source) -> Result<(), DomainError> {
            let mut sources = self.sources.lock().unwrap();
            sources.insert(source.id.0.clone(), source.clone());
            Ok(())
        }

        async fn get(&self, id: &SourceId) -> Result<Option<Source>, DomainError> {
            let sources = self.sources.lock().unwrap();
            Ok(sources.get(&id.0).cloned())
        }

        async fn list_by_workspace(
            &self,
            workspace_id: &WorkspaceId,
        ) -> Result<Vec<Source>, DomainError> {
            let sources = self.sources.lock().unwrap();
            Ok(sources
                .values()
                .filter(|s| s.workspace_id == *workspace_id)
                .cloned()
                .collect())
        }

        async fn delete(&self, id: &SourceId) -> Result<(), DomainError> {
            let mut sources = self.sources.lock().unwrap();
            sources.remove(&id.0);
            Ok(())
        }
    }

    #[test]
    fn detect_kind_git_repo() {
        let temp = tempfile::tempdir().unwrap();
        let git_dir = temp.path().join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();

        let kind = detect_source_kind(temp.path());
        assert_eq!(kind, SourceKind::Git);
    }

    #[test]
    fn detect_kind_directory() {
        let temp = tempfile::tempdir().unwrap();

        let kind = detect_source_kind(temp.path());
        assert_eq!(kind, SourceKind::Directory);
    }

    #[test]
    fn detect_kind_git_worktree() {
        let temp = tempfile::tempdir().unwrap();
        let git_dir = temp.path().join(".git");

        // 创建一个有效的 gitdir 目标
        let worktree_target = temp.path().join("worktree.git");
        std::fs::create_dir_all(&worktree_target).unwrap();

        // 写入 .git 文件（Worktree 格式）
        std::fs::write(&git_dir, format!("gitdir: {}", worktree_target.display())).unwrap();

        let kind = detect_source_kind(temp.path());
        assert_eq!(kind, SourceKind::Git);
    }

    #[test]
    fn detect_kind_git_file_invalid_gitdir() {
        let temp = tempfile::tempdir().unwrap();
        let git_dir = temp.path().join(".git");

        // 写入 .git 文件但 gitdir 目标不存在
        std::fs::write(&git_dir, "gitdir: /nonexistent/path").unwrap();

        let kind = detect_source_kind(temp.path());
        assert_eq!(kind, SourceKind::Directory);
    }

    #[test]
    fn detect_kind_git_file_no_gitdir() {
        let temp = tempfile::tempdir().unwrap();
        let git_dir = temp.path().join(".git");

        // 写入 .git 文件但没有 gitdir 行
        std::fs::write(&git_dir, "some other content").unwrap();

        let kind = detect_source_kind(temp.path());
        assert_eq!(kind, SourceKind::Directory);
    }

    #[tokio::test]
    async fn add_local_source_detects_git() {
        let temp = tempfile::tempdir().unwrap();
        let git_dir = temp.path().join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();

        let repo = Arc::new(InMemorySourceRepository::new());
        let use_case = AddLocalSource::new(repo.clone());

        let source = use_case
            .execute("workspace-1".to_owned(), temp.path().to_path_buf())
            .await
            .unwrap();

        assert_eq!(source.kind, SourceKind::Git);
    }

    #[tokio::test]
    async fn add_local_source_detects_directory() {
        let temp = tempfile::tempdir().unwrap();

        let repo = Arc::new(InMemorySourceRepository::new());
        let use_case = AddLocalSource::new(repo.clone());

        let source = use_case
            .execute("workspace-1".to_owned(), temp.path().to_path_buf())
            .await
            .unwrap();

        assert_eq!(source.kind, SourceKind::Directory);
    }

    #[tokio::test]
    async fn add_local_source_duplicate_path() {
        let temp = tempfile::tempdir().unwrap();

        let repo = Arc::new(InMemorySourceRepository::new());
        let use_case = AddLocalSource::new(repo.clone());

        // 第一次添加成功
        use_case
            .execute("workspace-1".to_owned(), temp.path().to_path_buf())
            .await
            .unwrap();

        // 第二次添加失败
        let result = use_case
            .execute("workspace-1".to_owned(), temp.path().to_path_buf())
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn add_local_source_not_exists() {
        let repo = Arc::new(InMemorySourceRepository::new());
        let use_case = AddLocalSource::new(repo.clone());

        let result = use_case
            .execute("workspace-1".to_owned(), PathBuf::from("/nonexistent/path"))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_sources() {
        let temp1 = tempfile::tempdir().unwrap();
        let temp2 = tempfile::tempdir().unwrap();

        let repo = Arc::new(InMemorySourceRepository::new());
        let add = AddLocalSource::new(repo.clone());
        let list = ListSources::new(repo.clone());

        add.execute("workspace-1".to_owned(), temp1.path().to_path_buf())
            .await
            .unwrap();
        add.execute("workspace-1".to_owned(), temp2.path().to_path_buf())
            .await
            .unwrap();

        let sources = list.execute("workspace-1".to_owned()).await.unwrap();
        assert_eq!(sources.len(), 2);
    }

    #[tokio::test]
    async fn remove_source() {
        let temp = tempfile::tempdir().unwrap();

        let repo = Arc::new(InMemorySourceRepository::new());
        let add = AddLocalSource::new(repo.clone());
        let remove = RemoveSource::new(repo.clone());
        let list = ListSources::new(repo.clone());

        let source = add
            .execute("workspace-1".to_owned(), temp.path().to_path_buf())
            .await
            .unwrap();

        remove.execute(source.id.clone()).await.unwrap();

        let sources = list.execute("workspace-1".to_owned()).await.unwrap();
        assert_eq!(sources.len(), 0);
    }
}
