# 子计划 06：会话持久化和 E2E

## 目标

实现标签页恢复、工作区布局保存和端到端验收。

## 精确修改文件

### 新增文件

| 文件 | 职责 |
|------|------|
| apps/desktop/src-tauri/src/commands/tab.rs | 标签页 Tauri 命令 |
| apps/desktop/src/hooks/useTabs.ts | 标签页 Hook |
| apps/desktop/src/components/TabBar.tsx | 标签栏组件 |
| apps/desktop/src/stores/layoutStore.ts | 布局状态 Store |
| tests/fixtures/sample-workspace/ | 测试 Fixture |

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| apps/desktop/src-tauri/src/commands.rs | 添加 tab 命令模块 |
| apps/desktop/src-tauri/src/lib.rs | 注册新命令到 Specta |
| apps/desktop/src/pages/WorkspacePage.tsx | 添加标签栏和布局持久化 |
| apps/desktop/src/bindings.ts | 自动生成（Specta） |
| apps/desktop/src/router.tsx | 添加启动时恢复逻辑 |

## 公共接口签名

### Tauri 命令

```rust
#[tauri::command]
#[specta::specta]
pub async fn open_tab(
    state: State<'_, AppState>,
    workspace_id: String,
    document_id: String,
) -> Result<OpenTab, AppError> { ... }

#[tauri::command]
#[specta::specta]
pub async fn close_tab(
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<(), AppError> { ... }

#[tauri::command]
#[specta::specta]
pub async fn list_tabs(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<Vec<OpenTab>, AppError> { ... }

#[tauri::command]
#[specta::specta]
pub async fn set_active_tab(
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<(), AppError> { ... }

#[tauri::command]
#[specta::specta]
pub async fn save_workspace_layout(
    state: State<'_, AppState>,
    workspace_id: String,
    layout: WorkspaceLayout,
) -> Result<(), AppError> { ... }

#[tauri::command]
#[specta::specta]
pub async fn get_workspace_layout(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<Option<WorkspaceLayout>, AppError> { ... }
```

### React Hook

```typescript
// apps/desktop/src/hooks/useTabs.ts

export function useTabs(workspaceId: string) {
  const tabs = useQuery({
    queryKey: ['tabs', workspaceId],
    queryFn: () => commands.listTabs(workspaceId),
  });

  const openTab = useMutation({
    mutationFn: (documentId: string) => commands.openTab(workspaceId, documentId),
    onSuccess: () => queryClient.invalidateQueries(['tabs', workspaceId]),
  });

  const closeTab = useMutation({
    mutationFn: (tabId: string) => commands.closeTab(tabId),
    onSuccess: () => queryClient.invalidateQueries(['tabs', workspaceId]),
  });

  return { tabs, openTab, closeTab };
}
```

## 依赖关系

- 依赖子计划 05（文件树和查看器）

## 不能并行的任务

- 必须在 05 完成后执行

## 失败测试

### 标签页测试

1. 打开标签后出现在列表中
2. 重复打开同一文档不创建新标签
3. 关闭标签后从列表移除
4. 设置活动标签成功
5. 标签顺序正确

### 布局恢复测试

1. 保存布局后可以读取
2. 重启后布局恢复
3. 切换工作区后布局正确

### E2E 测试

1. 完整用户流程通过
2. 重启后数据恢复
3. 删除工作区不删除源目录

## 最小实现步骤

1. 实现 OpenTab 用例
2. 实现 CloseTab 用例
3. 实现 ListTabs 用例
4. 实现 SetActiveTab 用例
5. 实现 SaveWorkspaceLayout 用例
6. 实现 GetWorkspaceLayout 用例
7. 创建 Tauri 命令
8. 注册到 Specta
9. 实现 useTabs Hook
10. 创建 TabBar 组件
11. 实现 layoutStore
12. 集成到 WorkspacePage
13. 创建测试 Fixture
14. 编写 E2E 测试
15. 执行人工验收

## 精确验证命令

```bash
cargo test -p devforge-desktop -- tab
cargo test -p devforge-desktop -- layout
pnpm typecheck
pnpm test
pnpm check
```

## 人工验收步骤

1. 启动 DevForge
2. 创建"DevForge 测试工作区"
3. 添加三个 Source：前端 Git 仓库、后端 Git 仓库、文档目录
4. 等待扫描完成
5. 展开每个 Source
6. 打开一个 TypeScript 文件
7. 打开一个 Rust 文件
8. 打开一个 Markdown 文件
9. 尝试打开 .env，确认拒绝正文读取
10. 切换和关闭标签
11. 正常关闭应用
12. 再次启动
13. 确认工作区仍存在
14. 确认三个 Source 仍存在
15. 确认文件树仍可正常加载
16. 确认已保存的标签和活动标签恢复
17. 删除或归档工作区
18. 确认三个原始 Source 目录及文件完全未被删除

## 独立提交信息

```
feat(session): 实现标签页恢复和布局持久化

- 添加 open_tab、close_tab、list_tabs、set_active_tab 命令
- 添加 save_workspace_layout、get_workspace_layout 命令
- 实现 useTabs Hook
- 创建 TabBar 组件
- 实现 layoutStore
- 创建测试 Fixture
- 编写 E2E 测试
```

## 回滚和兼容性风险

- 删除标签命令会导致前端调用失败
- 删除布局保存不影响核心功能
