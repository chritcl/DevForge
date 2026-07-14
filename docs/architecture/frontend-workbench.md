# React 桌面界面、页面交互与状态管理

## 1. 界面设计目标

核心体验目标：

1. 工作区始终是当前上下文中心
2. 搜索、代码阅读、AI 对话和任务执行可以同时进行
3. 用户不需要频繁切换完整页面
4. 任何 AI 结论都可以快速跳转到源代码或文档
5. 写文件、执行命令等高风险操作必须拥有独立、清晰的审批界面
6. 大量实时日志和流式内容不能导致整个 React 应用频繁重渲染
7. 同一个工作区可以保存不同的面板布局和打开状态
8. 界面结构要支持以后增加新连接器、工具和 Agent 能力

------

## 2. 整体桌面布局

主窗口采用类似 IDE 的多区域结构：

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ Title Bar / Workspace / Branch / Search / Agent Status / Window Controls │
├──────┬──────────────────────┬───────────────────────────┬────────────────┤
│活动栏│ 左侧边栏             │ 主编辑区                  │ AI 助手面板    │
│      │ 工作区资源           │ 文件/Diff/图谱            │ 对话/证据      │
│      │ 搜索/Git/任务        │                           │ Agent 计划     │
├──────┴──────────────────────┴───────────────────────────┴────────────────┤
│ Bottom Panel：终端/输出/问题/索引/任务日志/审计                          │
├──────────────────────────────────────────────────────────────────────────┤
│ Status Bar：工作区状态/Git/索引/模型/隐私/后台任务/错误                   │
└──────────────────────────────────────────────────────────────────────────┘
```

主要区域：Activity Bar、Primary Sidebar、Editor Area、Assistant Sidebar、Bottom Panel、Status Bar、Command Palette、Notification Center。每个区域都可以折叠、调整宽度/高度、保存工作区布局、通过快捷键切换。

------

## 3. 顶部标题栏

包含：应用图标、当前工作区、当前仓库、当前 Git 分支、全局搜索入口、Command Palette、Agent 当前状态、后台任务数量、窗口控制。

标题栏搜索框支持多种行为：普通输入 → 全局搜索；以 ? 开头 → AI 问答；以 > 开头 → Command Palette；以 @ 开头 → 搜索符号；以 # 开头 → 搜索 Issue/PR；以 : 开头 → 跳转行号或文件。

------

## 4. Activity Bar

左侧最窄的一列用于切换主要功能：工作区资源、全局搜索、AI 工作台、Code Graph、Git、变更中心、任务中心、连接器、模型管理、插件、设置。下方固定：账户与凭据、通知中心、后台健康状态、帮助。Activity Bar 只负责切换左侧边栏内容，不应该每次都替换整个主编辑区。

------

## 5. 左侧边栏

### 5.1 工作区资源视图

WORKSPACE → Repositories / Documents / External Sources (GitHub/GitLab) / Knowledge (Saved Answers/Architecture Notes/Decisions) / Recent。文件节点支持状态标记：M 已修改、A 新增、D 删除、I 正在索引、V 向量待生成、! 解析失败、S 敏感文件。

### 5.2 搜索视图

查询输入、搜索范围、模式、过滤器、搜索历史、保存的查询。过滤器支持折叠：工作区、仓库、文件类型、语言、路径、时间、数据源、证据级别、Git 分支。结果支持虚拟滚动、分组展示、增量加载。

### 5.3 Git 视图

CHANGES (Staged/Changes/Untracked/Conflicts)、BRANCHES (Current/Local/Remote/Session Branches)、HISTORY (Recent Commits/Current File/Current Branch)。Change Session 创建的 Worktree 使用特殊图标。

### 5.4 任务视图

RUNNING、WAITING (ChangeSet 审批/命令审批)、RECENT (Completed/Failed/Cancelled/Interrupted)。

------

## 6. 主编辑区

主编辑区采用标签页模型。支持的标签页类型：SourceFileTab、DocumentTab、SearchResultsTab、SymbolTab、CodeGraphTab、ConversationTab、ChangeReviewTab、TaskDetailTab、GitDiffTab、SettingsTab、DashboardTab。

Preview Tab：单击搜索结果时使用预览标签（斜体标题），双击或编辑后变成固定标签。

------

## 7. 编辑器布局

支持：单列、左右两列、上下两列、三列、临时对比模式。标签页可以拖拽到不同编辑组。

------

## 8. 源代码查看器

使用 Monaco Editor（默认只读模式）。能力：语法高亮、行号、折叠、代码导航、当前符号高亮、引用范围高亮、Git 修改标记、诊断信息、行内 AI 操作、迷你地图、Breadcrumb。

右键代码选区：解释这段代码、询问 AI、查找引用、查看调用者、查看被调用函数、分析影响、查看 Git Blame、添加到 AI 上下文、创建开发任务、复制引用链接。

------

## 9. 文档查看器

Markdown 使用富文本预览与源码双模式。PDF、Word 等文档支持页面导航、文本选取、搜索、AI 询问选中内容、查看索引 Chunk、查看引用来源、跳转到 AI 回答引用页。

文档顶部展示：来源、同步时间、索引状态、文档版本、是否会发送至云模型。

------

## 10. 符号详情页

打开函数/类/接口时可打开符号详情页：定义、签名、所属文件、调用者、被调用函数、类型依赖、实现关系、相关测试、相关 Commit、相关 Issue/PR、AI 摘要。

------

## 11. Code Graph 页面

提供三种模式：
- **局部关系图**：围绕当前符号展示一至两跳关系
- **模块依赖图**：展示仓库或模块级依赖
- **影响分析图**：以待修改对象为中心展示影响范围

大型图谱采用增量加载。

------

## 12. AI 助手侧栏

右侧 AI 助手是可切换的工作模式：Chat、Context、Plan、Changes、Tools、Evidence。顶部显示当前模式（Ask/Plan/Change/Diagnose）、当前模型、当前工作区、当前上下文大小、隐私策略。

Context 面板展示当前会发送给模型的内容：当前文件、当前选区、固定文件、检索证据、会话记忆、Git Diff、用户附加内容。用户可以手动删除某个上下文项。

------

## 13. AI 输入框

支持多种上下文引用：@文件、@符号、@仓库、@工作区、@Issue、@PR、@Commit、@当前选区、@当前 Diff。附件支持：添加文件、目录、Git Diff、截图、日志、终端输出、数据库结构。

模式切换：Ask、Plan、Change、Diagnose。用户从 Ask 切换到 Change 时需要明显提示。

------

## 14. AI 流式回答

回答状态：正在理解问题 → 正在搜索工作区 → 正在精排证据 → 正在构建上下文 → 等待云端披露确认 → 正在生成 → 正在验证引用 → 已完成。生成过程中正文逐步显示，引用只有在验证完成后才变成可点击状态。

------

## 15. 引用交互

正文中的引用鼠标悬停显示来源信息，点击后打开对应文件、跳转行号、高亮引用范围。如果内容已经变化显示"引用可能已过期"。

------

## 16. 变更审批页面

Change Review 是独立主编辑页。布局：左侧文件列表、中间 Diff、右侧变更说明。文件列表状态：待审核、已批准、已拒绝、存在冲突、已应用、已回滚。删除文件必须使用单独确认样式。

------

## 17. 命令审批页面

命令审批使用专门卡片：用途、程序、参数、工作目录、风险、超时、环境变量、网络。按钮：批准本次、批准当前验证批次、拒绝、修改命令、复制命令、在外部终端运行。

------

## 18. 底部面板

Terminal（DevForge Managed Terminal 和 External Terminal 明确区分）、Output（按来源选择日志）、Problems（LSP 诊断/索引错误/Connector 错误/ChangeSet 冲突/测试失败）、Index、Audit。

------

## 19. 状态栏

左侧：当前工作区、当前仓库、Git 分支、未提交文件数量。右侧：索引状态、Embedding 模型、Chat 模型、隐私模式、后台任务、连接器状态、错误与警告。

------

## 20. Command Palette

快捷键 Ctrl + Shift + P。命令类型：工作区命令、文件命令、搜索命令、AI 命令、Git 命令、任务命令、布局命令、设置命令。命令根据当前上下文启用或禁用。

------

## 21. 快捷键体系

基础：Ctrl+P 快速打开、Ctrl+Shift+P Command Palette、Ctrl+Shift+F 全局搜索、Ctrl+J 底部面板、Ctrl+B 左侧边栏、Ctrl+Alt+B AI 助手、Ctrl+Shift+A AI 输入、Ctrl+Enter 发送 AI 请求、Escape 取消。

所有快捷键支持用户自定义。

------

## 22. 首页仪表盘

首页是开发工作入口：最近工作区、正在运行的 Agent、等待审批、索引状态、最近搜索、最近 AI 对话、最近 Change Session、连接器问题、模型状态。

------

## 23. 工作区概览页

展示：工作区名称、描述、仓库数量、文档数量、符号数量、Chunk 数量、索引体积、最近更新。主要卡片：数据源、索引健康、语言分布、模块摘要、Git 活动、AI 活动、待处理问题、磁盘使用。

------

## 24. 模型管理页面

分为 Providers、Models、Profiles、Routing、Usage。Profiles 例如：本地隐私模式、日常开发模式、深度架构分析、快速问答、低成本云端。每个 Profile 组合 Chat/Fast/Embedding/Rerank/Fallback 模型和隐私策略。

------

## 25. 连接器管理页面

状态：Healthy、Syncing、Rate Limited、Authentication Failed、Degraded、Disabled。操作：立即同步、暂停、重新授权、修改范围、查看同步日志、清除本地快照、删除连接器。

------

## 26. 插件管理页面

安装时展示权限。更新插件新增权限时必须重新确认。插件不能使用与系统审批页面完全相同的视觉样式，避免伪造系统确认。

------

## 27. 通知系统

通知分为：Info、Success、Warning、Error、ApprovalRequired、Security。普通通知使用 Toast，重要通知进入通知中心。安全通知不能自动消失。

------

## 28. React 状态划分

四类状态：
- **服务端状态**（TanStack Query）：工作区列表、文档元数据、搜索结果、对话历史、任务列表、连接器状态、模型列表、ChangeSet、Git 状态
- **UI 状态**（Zustand）：当前活动栏、侧栏宽度、编辑组、打开标签、布局模式、搜索面板状态、当前选区
- **流式状态**（独立 Store）：AI Token、命令日志、索引进度。使用外部 Store、按 ID 分片、批量提交 UI 更新
- **编辑器状态**（Monaco 自维护）：光标、选区、滚动位置、折叠状态

------

## 29. IPC Client 层

React 不直接到处调用 invoke()。统一封装：client.ts、commands.ts、channels.ts、events.ts、errors.ts、generated-types.ts。Feature 只能调用领域化 API：workspaceApi.createWorkspace()、searchApi.searchWorkspace() 等。

------

## 30. IPC 数据失效策略

Rust 状态变化后发送领域事件（workspace-updated、document-indexed、task-status-changed 等）。React 事件处理器收到事件 → 定位相关 Query Key → invalidateQueries → TanStack Query 重新获取。

------

## 31. Query Key 规范

```text
["workspaces"]
["workspace", workspaceId]
["workspace", workspaceId, "sources"]
["documents", workspaceId, filters]
["search", workspaceId, queryHash]
["conversation", conversationId]
["agent-run", runId]
["change-set", changeSetId]
["tasks", workspaceId, filters]
```

禁止在不同 Feature 中随意创建字符串 Query Key。

------

## 32. 错误处理

统一错误结构：code、message、user_message、details、retryable、suggested_action、correlation_id。React 展示分层：页面级错误、区域级错误、非阻塞错误、安全阻止。

------

## 33. Loading 与骨架屏

短请求 → 局部 Spinner；列表加载 → Skeleton；长任务 → 任务状态卡；未知总量 → Indeterminate Progress；可取消操作 → 显示取消按钮；后台操作 → 状态栏或通知中心。

------

## 34. 大数据量性能

- **文件树**：虚拟列表、懒加载目录、增量展开、Rust 后台分页
- **搜索结果**：虚拟滚动、分页、最大可见结果、结果预览延迟加载
- **日志**：环形缓冲区、批量更新、虚拟行、文件偏移读取
- **Code Graph**：局部图、按需扩展、节点数量限制、Web Worker 布局
- **对话**：消息虚拟列表、历史分页、代码块延迟渲染、Markdown 增量渲染节流

------

## 35. React 组件分层

App Shell → Feature Pages → Feature Widgets → Entity Components → Shared UI。避免一个页面文件超过数千行。

组件目录示例：

```text
features/change-review/
├─ api/
│  ├─ queries.ts
│  └─ mutations.ts
├─ components/
│  ├─ ChangeSummary.tsx
│  ├─ ChangedFileList.tsx
│  ├─ DiffReview.tsx
│  ├─ ApprovalToolbar.tsx
│  └─ RiskPanel.tsx
├─ hooks/
│  ├─ useChangeSet.ts
│  └─ useApproval.ts
├─ pages/
│  └─ ChangeReviewPage.tsx
├─ model/
│  ├─ store.ts
│  └─ selectors.ts
└─ types/
```

------

## 36. 主题系统

支持：Dark、Light、Follow System、High Contrast。Diff 颜色不能只依赖红绿，还需要图标、行前标记、文本标签。

------

## 37. 无障碍

至少保证：键盘可完成主要操作、Focus Ring 清晰、审批按钮语义明确、图标拥有文本说明、不只使用颜色表达状态、支持系统字体缩放、支持减少动画、Modal 可正确锁定焦点。

------

## 38. 多窗口策略

第一版只提供少量独立窗口：Main Window、Change Review Window、Task Log Window、Quick Capture Window。Change Review Window 只拥有读取当前 ChangeSet 和提交审批结果的权限。

------

## 39. 工作区布局持久化

每个工作区保存：打开标签、活动标签、编辑器分组、左侧视图、右侧 AI 状态、底部面板状态、各面板大小、固定上下文、最近搜索。应用异常退出后可以恢复。但命令、Agent Tool Call、待应用 Patch 只能恢复为 Interrupted/WaitingForReview 状态。

------

## 40. 搜索结果页面布局

搜索页面分三部分：

```text
┌──────────────┬──────────────────────────────┬──────────────────────┐
│ 过滤器       │ 结果列表                      │ 预览与关系           │
│              │                              │                      │
│ 工作区       │ 结果卡 1                      │ 代码片段             │
│ 仓库         │ 结果卡 2                      │ 符号详情             │
│ 文件类型     │ 结果卡 3                      │ 调用关系             │
│ 语言         │ ...                           │ 相关文件             │
│ 路径         │                              │ Git 历史             │
│ 时间         │                              │                      │
│ 数据源       │                              │                      │
│ 证据级别     │                              │                      │
│ Git 分支     │                              │                      │
└──────────────┴──────────────────────────────┴──────────────────────┘
```

结果卡展示：

```text
符号或文件名称
仓库与路径
代码片段
行号
匹配原因
数据来源
当前版本
相关性
证据等级
```

匹配原因示例：

```text
符号精确匹配
包含关键词 "refresh token"
语义匹配登录凭据刷新
被 AuthController 调用
最近在 PR #128 中修改
```

用户可以查看"为什么找到这个结果"，而不是只看到无法解释的相关性分数。

------

## 41. AI 回答页面布局

回答区域包含：

```text
回答正文
引用列表
可信等级
使用的模型
使用的工作区
检索范围
发送到云端的内容摘要
生成时间
索引版本
```

辅助操作：

```text
打开全部引用
在 Code Graph 中查看
继续追问
保存为项目知识
生成开发计划
创建变更任务
重新检索
切换模型重答
报告引用错误
```

------

## 42. Agent 页面布局

```text
┌──────────────────────────────────────────────────────────────┐
│ 任务标题 / 状态 / 模型 / 工作区 / 停止                      │
├──────────────┬───────────────────────┬───────────────────────┤
│ 任务计划     │ 当前内容              │ 上下文与证据          │
│              │                       │                       │
│ 步骤状态     │ AI 对话               │ 读取文件              │
│ Tool Calls   │ Diff                  │ 搜索结果              │
│ 审批队列     │ 命令输出              │ Git 状态              │
├──────────────┴───────────────────────┴───────────────────────┤
│ 时间线 / 终端 / 问题 / 审计                                  │
└──────────────────────────────────────────────────────────────┘
```

右上角始终显示：

```text
当前模式：Change
写权限：需批准
命令权限：需批准
网络权限：禁止
执行环境：Session Worktree
```

------

## 43. 第一版页面范围

必须完成：首页、工作区创建与管理、工作区资源管理器、文件与代码查看器、全文与语义搜索、搜索结果页、符号详情、基础 Code Graph、AI 对话、AI 证据面板、Agent Plan、Change Review、命令审批、任务中心、Git 工作台、连接器管理、模型管理、设置、健康中心、通知中心。

可简化：插件市场、复杂多窗口、自定义 Dashboard、任意 UI 插件、移动端布局、团队协作界面。

------

## 44. 典型用户流程

**项目问答**：打开工作区 → Ctrl+Shift+A → 输入问题 → 查看检索过程 → 阅读带引用回答 → 点击引用打开代码 → 查看调用关系

**开发规划**：选中订单模块 → 切换 Plan 模式 → 输入需求 → AI 生成影响范围 → 查看预计文件和风险 → 保存为开发计划

**代码修改**：从 Plan 创建 Change Session → 创建 Session Worktree → 生成 ChangeSet → 打开 Change Review → 按文件批准 → 应用修改 → 批准测试命令 → 查看实时输出 → 验证通过 → 创建本地 Commit

**Bug 排查**：粘贴错误日志 → 切换 Diagnose → AI 搜索相关代码和历史 → 提出诊断命令 → 用户批准 → 分析输出 → 生成根因与修复 Patch

------

## 45. 推荐前端依赖边界

核心依赖：React、TypeScript、Vite、React Router、TanStack Query、Zustand、Monaco Editor、Radix UI、Tailwind CSS、React Hook Form、Zod、TanStack Virtual。

图谱可选：React Flow（流程与任务图）、Cytoscape.js 或 Sigma.js（大型 Code Graph）。

Markdown：react-markdown、remark-gfm、rehype-sanitize、Shiki。任何模型返回的 HTML 都必须经过过滤。

------

## 46. 前端测试策略

### 单元测试

测试：Query Key、状态选择器、工具函数、Diff 状态、风险标签、命令格式化。

### 组件测试

测试：审批按钮、错误状态、搜索过滤器、引用卡片、任务状态、云端披露对话框。

### 集成测试

Mock Rust IPC：创建工作区、搜索、AI 流式回答、引用打开、ChangeSet 审批、命令审批、任务取消。

### 端到端测试

覆盖：导入仓库、等待基础索引、搜索符号、进行项目问答、创建 Change Session、批准 Patch、批准测试、完成任务。高风险操作必须测试：Approval Hash 失效、文件基线变化、命令参数变化、敏感文件阻止、工作区外路径阻止、任务中断恢复。

------

## 47. 前端核心原则总结

DevForge 前端不应该是"左侧菜单 + 右侧表格 + 一个 AI 聊天弹窗"，而应该是：一个以工作区为核心，可以同时完成代码阅读、知识搜索、AI 推理、变更审查和任务执行的桌面开发工作台。

前端负责：展示状态、组织工作流、提供上下文、展示风险、收集用户批准。Rust 后台负责：业务事实、权限判断、文件操作、命令执行、索引、搜索、AI 调用、审计。React 永远不能成为安全边界，UI 中隐藏按钮不等于禁止操作。
