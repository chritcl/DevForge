---
paths:
  - "apps/desktop/src/**"
  - "packages/**/*.{ts,tsx}"
---

# React 与 TypeScript 规则

## 通用规则

- 使用 TanStack Query 管理 Rust 持久化数据。
- 使用 Zustand 仅管理瞬态 UI 和布局状态。
- 不得在前端存储中复制后端业务状态。
- 高频流（模型 token、命令输出）必须放在通用应用状态之外。
- 不得在全局 Zustand store 中存储完整文件、完整日志或大型搜索结果集。
- 组件不得包含后端权限或安全决策。
- 在 UI 边界校验用户输入，但将 Rust 校验视为权威。
- 大型树、结果列表、对话和日志必须使用懒加载或虚拟化。
- 不得在未经消毒的情况下渲染不受信任的模型或文档 HTML。

## 状态归属

添加状态前，将其分类为以下之一：

1. URL 或导航状态
2. Rust 持久化/服务端状态
3. 组件本地交互状态
4. 共享瞬态 UI 状态
5. 高频流状态
6. 编辑器自有状态

使用：
- React Router 或搜索参数管理可共享的导航和过滤器。
- TanStack Query 管理 Rust 持久化数据。
- 组件本地状态管理单个组件子树使用的状态。
- Zustand 管理共享瞬态 UI 和布局状态。
- 专用外部流 store 管理 AI token、日志和进度。
- Monaco 自有模型管理编辑器内容和视图状态。

禁止：
- 跨 TanStack Query、Zustand、组件状态和 URL 状态镜像同一个值。
- 存储可从现有状态廉价派生的值。
- 将 Query 数据复制到 Zustand。
- 仅为跟踪一个选中标识符而存储完整的后端实体。

正确：存储 `selectedWorkspaceId`，而非复制 `selectedWorkspace` 对象。

## TanStack Query

- 所有 Query Key 必须通过集中的类型化 Query Key Factory 创建。
- 不得在功能组件内构造临时 Query Key 数组。
- 每个 Query 必须有意定义其新鲜度和重试策略，不得盲目依赖全局默认值。
- 静态或很少变化的元数据应使用显式 `staleTime`。
- 快速变化的任务进度应使用事件或流，而非激进的轮询。
- 认证、权限、校验和安全拒绝错误不得自动重试。
- Mutation 必须无效化或更新所有受影响的 Query。
- 当 mutation 变更多个相关实体时优先使用无效化。
- 仅在结果状态确定性时使用直接缓存更新。
- 不得将 TanStack Query 用作通用事件总线。
- 不得将敏感 Query 数据持久化到浏览器存储。
- Query 函数必须调用共享 IPC API Client，不得直接调用 Tauri。
- TanStack Query 的取消必须传播到可取消的 Rust 操作。

### Query Key Factory 示例

```ts
export const workspaceKeys = {
  all: ['workspaces'] as const,
  detail: (id: string) => [...workspaceKeys.all, 'detail', id] as const,
  documents: (id: string, filters: DocumentFilters) =>
    [...workspaceKeys.detail(id), 'documents', filters] as const,
};
```

## Zustand

- Zustand store 只包含瞬态 UI 状态，不得包含 Rust 拥有的权威业务数据。
- 按职责拆分 store，而非创建一个全局 store。
- 组件必须通过聚焦的 selector 订阅，不得订阅整个 store。
- 将 action 放在它们变更的状态旁。
- 派生值属于 selector，不得重复为 store 字段。
- 只持久化稳定的用户偏好和可恢复的布局状态。
- 持久化 store 需要：显式版本、迁移函数、缺失字段的安全默认值。

不得持久化：AI 流 token、命令输出、完整对话、搜索结果、文件内容、凭据、审批对象、活跃进程状态。

避免 store 间 mutation，跨 store 操作通过功能级应用 hook 或 command 协调。

推荐的 store 拆分：`useLayoutStore`、`useEditorTabsStore`、`useCommandPaletteStore`、`useAssistantUiStore`、`StreamRegistry`。工作区、消息、任务、ChangeSet 继续归 TanStack Query。

## useEffect

- Effect 用于同步 React 与外部系统。
- 不得使用 effect 计算派生状态。
- 不得使用 effect 将 props 或 Query 数据复制到本地状态。
- 不得在 effect 中执行属于事件处理器的操作。
- 不得在组件 effect 中执行普通后端数据获取，使用 TanStack Query。
- 每个订阅 effect 必须返回清理函数。
- Effect 的设置和清理在 React Strict Mode 重复执行下必须安全。
- 不得抑制 hook 依赖 lint 警告，除非说明了生命周期需求。
- 当 effect 协调外部资源时，优先提取自定义 hook。
- 不得使用 JSON 序列化作为 effect 依赖的变通方案。
- 避免通过状态更新互相触发的 effect 链。

错误示例：

```tsx
useEffect(() => {
  setFilteredDocuments(
    documents.filter(doc => doc.language === language),
  );
}, [documents, language]);
```

正确示例：

```tsx
const filteredDocuments = useMemo(
  () => documents.filter(doc => doc.language === language),
  [documents, language],
);
```

## 高频流状态

- AI token 流、命令输出、索引进度和任务事件不得对每条消息更新大型 React 组件树。
- 缓冲高频事件并发布批量 UI 更新。
- 按 run ID 或 task ID 分区流。
- 完整日志存储在 Rust 管理的文件中，不在浏览器内存中。
- 前端只保留有界的可见窗口。
- 完成时，用持久化的权威后端结果替换瞬态流状态。
- 每个流必须支持：取消、完成、错误、重连/恢复语义、拥有视图卸载时的清理。
- 不得将 TanStack Query 缓存用作逐 token 的流缓冲区。

## TypeScript

- 启用 strict TypeScript 设置。
- 不得使用 `any` 绕过类型设计问题。
- 对不受信任的输入使用 `unknown` 并显式收窄。
- 所有 IPC、插件、模型、连接器和持久化外部数据必须在运行时校验。
- 不得使用类型断言仅为了消除编译器错误。
- 优先使用可区分联合体建模 UI 和任务状态。
- 在实际可行的情况下将不可能的 UI 状态排除在类型系统之外。
- 生成的 Rust 到 TypeScript 契约不得手动重复。
- 除非在访问前立即强制了不变量，否则不得使用非空断言。
- 穷举处理 Domain 状态值，新的 enum 或联合体变体必须在相关 switch 语句中产生编译时失败。

推荐模式：

```ts
type TaskState =
  | { status: 'queued' }
  | { status: 'running'; progress: number }
  | { status: 'failed'; error: AppError }
  | { status: 'completed'; completedAt: string };
```

优于：

```ts
interface TaskState {
  status: string;
  progress?: number;
  error?: string;
  completedAt?: string;
}
```
