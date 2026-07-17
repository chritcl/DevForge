# 子计划 03：Source 和 PathGuard

## 目标

实现数据源管理（Git 仓库和普通目录）和路径安全验证。

## 精确修改文件

### 新增文件

| 文件 | 职责 |
|------|------|
| crates/devforge-domain/src/source.rs | Source 领域模型（如 01 未创建） |
| crates/devforge-domain/src/path_guard.rs | PathGuard 安全验证 |
| crates/devforge-application/src/source.rs | Source 用例 |
| apps/desktop/src-tauri/src/commands/source.rs | Source Tauri 命令 |
| apps/desktop/src/components/AddSourceDialog.tsx | 添加数据源对话框 |

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| apps/desktop/src-tauri/src/commands.rs | 添加 source 命令模块 |
| apps/desktop/src-tauri/src/lib.rs | 注册新命令到 Specta |
| apps/desktop/src-tauri/src/state.rs | 添加 SourceRepository |
| apps/desktop/src/pages/WorkspacePage.tsx | 添加数据源列表 |

## 公共接口签名

### PathGuard

```rust
// crates/devforge-domain/src/path_guard.rs

pub struct PathGuard {
    source_root: PathBuf,
}

impl PathGuard {
    pub fn new(source_root: PathBuf) -> Result<Self, PathError>;
    pub fn validate(&self, path: &Path) -> Result<PathBuf, PathError>;
    pub fn is_within_root(&self, path: &Path) -> bool;
}

#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("路径逃逸：路径在源目录外")]
    PathEscape,
    #[error("路径不存在")]
    NotExists,
    #[error("路径不是文件")]
    NotFile,
    #[error("路径包含非法字符")]
    InvalidCharacters,
    #[error("IO 错误")]
    Io(#[from] std::io::Error),
}
```

### Source 用例

```rust
pub struct AddGitSource {
    repo: Arc<dyn SourceRepository>,
}

impl AddGitSource {
    pub async fn execute(&self, workspace_id: String, path: PathBuf) -> Result<Source, AppError>;
}

pub struct AddDirectorySource {
    repo: Arc<dyn SourceRepository>,
}

impl AddDirectorySource {
    pub async fn execute(&self, workspace_id: String, path: PathBuf) -> Result<Source, AppError>;
}
```

## 依赖关系

- 依赖子计划 01（Domain 和 Storage）
- 与子计划 02 可并行

## 不能并行的任务

- 与子计划 02 可并行（无文件重叠）

## 失败测试

### PathGuard 测试

1. 正常路径验证通过
2. `..` 路径逃逸被拒绝
3. 绝对路径逃逸被拒绝
4. Windows 大小写规范化
5. Symlink 解析后验证
6. Junction 解析后验证
7. 不存在路径被拒绝
8. 目录路径被拒绝（当期望文件时）

### Source 测试

1. 添加 Git 仓库成功
2. 添加普通目录成功
3. 添加不存在路径失败
4. 添加文件路径失败
5. 重复添加同一路径失败
6. 非 Git 目录作为 Git Source 失败
7. 移除 Source 不删除原目录
8. 移除 Source 后文档级联删除

## 最小实现步骤

1. 实现 PathGuard
2. 编写 PathGuard 安全测试
3. 实现 Source 领域模型
4. 实现 SourceRepository
5. 实现 AddGitSource 用例
6. 实现 AddDirectorySource 用例
7. 实现 RemoveSource 用例
8. 创建 Tauri 命令
9. 注册到 Specta
10. 创建前端对话框
11. 编写测试

## 精确验证命令

```bash
cargo test -p devforge-domain -- path_guard
cargo test -p devforge-application
cargo test -p devforge-desktop
pnpm typecheck
pnpm test
```

## 人工验收步骤

1. 创建工作区
2. 点击"添加数据源"
3. 选择 Git 仓库目录
4. 确认添加成功
5. 添加普通文档目录
6. 确认添加成功
7. 尝试添加不存在路径
8. 确认错误提示
9. 移除数据源
10. 确认原目录未被删除

## 独立提交信息

```
feat(source): 实现数据源管理和路径安全验证

- 添加 PathGuard 路径安全验证
- 实现 AddGitSource、AddDirectorySource 用例
- 添加 Source Tauri 命令
- 创建添加数据源对话框
- 编写安全测试
```

## 回滚和兼容性风险

- PathGuard 为新增模块，不影响现有功能
- 删除 Source 命令不影响原目录
