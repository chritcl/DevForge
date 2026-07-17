# 子计划 02：Workspace CRUD

## 目标

实现工作区的创建、列表、更新、归档、恢复和删除功能。

## 精确修改文件

### 新增文件

| 文件 | 职责 |
|------|------|
| crates/devforge-application/src/workspace.rs | Workspace 用例 |
| apps/desktop/src-tauri/src/commands/workspace.rs | Workspace Tauri 命令 |
| apps/desktop/src/pages/WorkspaceListPage.tsx | 工作区列表页 |
| apps/desktop/src/pages/WorkspacePage.tsx | 工作区详情页 |
| apps/desktop/src/components/CreateWorkspaceDialog.tsx | 创建工作区对话框 |

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| apps/desktop/src-tauri/src/commands.rs | 添加 workspace 命令模块 |
| apps/desktop/src-tauri/src/lib.rs | 注册新命令到 Specta |
| apps/desktop/src-tauri/src/state.rs | 添加 WorkspaceRepository |
| apps/desktop/src/router.tsx | 添加工作区路由 |
| apps/desktop/src/bindings.ts | 自动生成（Specta） |

## 公共接口签名

### Application 用例

```rust
// crates/devforge-application/src/workspace.rs

pub struct CreateWorkspace {
    repo: Arc<dyn WorkspaceRepository>,
}

impl CreateWorkspace {
    pub async fn execute(&self, name: String, description: Option<String>) -> Result<Workspace, AppError>;
}

pub struct ListWorkspaces {
    repo: Arc<dyn WorkspaceRepository>,
}

impl ListWorkspaces {
    pub async fn execute(&self) -> Result<Vec<Workspace>, AppError>;
}
```

### Tauri Command

```rust
#[tauri::command]
#[specta::specta]
pub async fn create_workspace(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
) -> Result<Workspace, AppError> { ... }

#[tauri::command]
#[specta::specta]
pub async fn list_workspaces(
    state: State<'_, AppState>,
) -> Result<Vec<Workspace>, AppError> { ... }
```

## 依赖关系

- 依赖子计划 01（Domain 和 Storage）

## 不能并行的任务

- 与子计划 03 可并行（无文件重叠）

## 失败测试

1. 创建工作区后出现在列表中
2. 创建重复名称工作区失败
3. 空名称创建失败
4. 更新工作区名称成功
5. 归档后不在默认列表中
6. 恢复后重新出现在列表中
7. 删除工作区后不可查询

## 最小实现步骤

1. 实现 CreateWorkspace 用例
2. 实现 ListWorkspaces 用例
3. 实现 UpdateWorkspace 用例
4. 实现 ArchiveWorkspace 用例
5. 实现 RestoreWorkspace 用例
6. 实现 DeleteWorkspace 用例
7. 创建 Tauri 命令
8. 注册到 Specta
9. 创建前端页面
10. 编写测试

## 精确验证命令

```bash
cargo test -p devforge-application
cargo test -p devforge-desktop
pnpm typecheck
pnpm test
```

## 人工验收步骤

1. 启动 DevForge
2. 点击"创建工作区"
3. 输入名称"测试工作区"
4. 确认工作区出现在列表中
5. 点击工作区进入详情
6. 重命名工作区
7. 归档工作区
8. 确认归档工作区不在默认列表
9. 恢复工作区
10. 删除工作区

## 独立提交信息

```
feat(workspace): 实现工作区 CRUD 功能

- 添加 CreateWorkspace、ListWorkspaces 等用例
- 添加 Tauri 命令
- 创建工作区列表和详情页面
- 实现创建、更新、归档、恢复、删除功能
```

## 回滚和兼容性风险

- 删除命令会导致前端调用失败
- 数据库表已创建，删除命令不影响数据
