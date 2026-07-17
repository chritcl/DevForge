# 子计划 05：文件树和查看器

## 目标

实现文件树懒加载展示和文件内容查看器。

## 精确修改文件

### 新增文件

| 文件 | 职责 |
|------|------|
| apps/desktop/src/components/FileTree.tsx | 文件树组件 |
| apps/desktop/src/components/FileTreeNode.tsx | 文件树节点 |
| apps/desktop/src/components/FileViewer.tsx | 文件查看器容器 |
| apps/desktop/src/components/TextViewer.tsx | 文本文件查看器 |
| apps/desktop/src/components/MarkdownViewer.tsx | Markdown 查看器 |
| apps/desktop/src/components/SensitiveFileBlocked.tsx | 敏感文件拒绝提示 |
| apps/desktop/src/hooks/useFileTree.ts | 文件树 Hook |
| apps/desktop/src/hooks/useDocument.ts | 文档内容 Hook |

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| apps/desktop/src/pages/WorkspacePage.tsx | 添加文件树和查看器 |
| apps/desktop/src-tauri/src/commands/document.rs | 添加文档命令 |
| apps/desktop/src-tauri/src/lib.rs | 注册新命令到 Specta |
| apps/desktop/src/bindings.ts | 自动生成（Specta） |

## 公共接口签名

### Tauri 命令

```rust
#[tauri::command]
#[specta::specta]
pub async fn list_documents(
    state: State<'_, AppState>,
    source_id: String,
    parent_path: Option<String>,
) -> Result<Vec<Document>, AppError> { ... }

#[tauri::command]
#[specta::specta]
pub async fn read_document_content(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<String, AppError> { ... }
```

### React Hook

```typescript
// apps/desktop/src/hooks/useFileTree.ts

export function useFileTree(sourceId: string, parentPath?: string) {
  return useQuery({
    queryKey: ['documents', sourceId, parentPath],
    queryFn: () => commands.listDocuments(sourceId, parentPath),
  });
}

// apps/desktop/src/hooks/useDocument.ts

export function useDocument(documentId: string) {
  return useQuery({
    queryKey: ['document', documentId],
    queryFn: () => commands.readDocumentContent(documentId),
  });
}
```

## 依赖关系

- 依赖子计划 02（Workspace CRUD）
- 依赖子计划 04（文件发现）

## 不能并行的任务

- 必须在 02 和 04 完成后执行

## 失败测试

### 文件树测试

1. 空目录显示空状态
2. 文件和目录正确显示
3. 目录展开加载子项
4. 文件图标正确
5. 敏感文件图标标识
6. 切换 Source 时重置状态

### 文件查看器测试

1. 文本文件正确显示
2. Markdown 渲染
3. 敏感文件显示拒绝提示
4. 大文件拒绝读取
5. 不存在文件显示错误

## 最小实现步骤

1. 创建 list_documents 命令
2. 创建 read_document_content 命令
3. 注册到 Specta
4. 实现 useFileTree Hook
5. 实现 useDocument Hook
6. 创建 FileTree 组件
7. 创建 FileTreeNode 组件
8. 创建 FileViewer 组件
9. 创建 TextViewer 组件
10. 创建 MarkdownViewer 组件
11. 创建 SensitiveFileBlocked 组件
12. 集成到 WorkspacePage
13. 编写测试

## 精确验证命令

```bash
cargo test -p devforge-desktop -- document
pnpm typecheck
pnpm test
```

## 人工验收步骤

1. 创建工作区并添加数据源
2. 展开文件树
3. 点击目录展开子项
4. 点击文本文件查看内容
5. 点击 Markdown 文件查看渲染
6. 点击 .env 文件
7. 确认显示敏感文件拒绝提示
8. 切换标签
9. 关闭标签

## 独立提交信息

```
feat(viewer): 实现文件树和文件查看器

- 添加 list_documents、read_document_content 命令
- 实现 FileTree 组件
- 实现 TextViewer、MarkdownViewer 组件
- 实现 SensitiveFileBlocked 组件
- 集成到 WorkspacePage
```

## 回滚和兼容性风险

- 删除组件会导致页面空白
- 删除命令会导致前端调用失败
