## 第二部分：技术架构与工程结构

整体采用：

> **Tauri 2 + React + TypeScript + Rust Workspace + SQLite + Tantivy + 嵌入式向量索引**

核心原则是：**Tauri 只负责桌面宿主与 IPC，真正的业务能力放在独立 Rust crate 中**。以后即使拆成后台守护进程或服务端，也不需要重写核心逻辑。

------

# 1. 总体进程架构

第一版保持单安装包，但逻辑上拆成三层：

```text
┌─────────────────────────────────────────────┐
│               React WebView                 │
│                                             │
│ 工作区 / 搜索 / AI / Diff / Git / 设置      │
└───────────────────┬─────────────────────────┘
                    │ Tauri Commands
                    │ Events
                    │ Channels
┌───────────────────▼─────────────────────────┐
│              Tauri Desktop Host             │
│                                             │
│ 窗口 / 托盘 / 文件选择 / IPC / 系统权限      │
└───────────────────┬─────────────────────────┘
                    │ Application API
┌───────────────────▼─────────────────────────┐
│               Rust Core Engine              │
│                                             │
│ 工作区 / 索引 / 搜索 / AI / Git / Agent     │
│ 任务 / 权限 / 数据库 / 连接器 / 审计         │
└─────────────────────────────────────────────┘
```

以后可以演进为：

```text
React + Tauri
      │
Named Pipe / Local Socket
      │
独立 Rust Daemon
```

因此 Rust Core 不能直接依赖 Tauri 的 `AppHandle`、窗口对象和插件 API。

------

# 2. Monorepo 结构

建议使用 `pnpm workspace + Cargo workspace`：

```text
devforge/
├─ apps/
│  └─ desktop/
│     ├─ src/                         React 主应用
│     ├─ src-tauri/                   Tauri 宿主
│     ├─ public/
│     └─ package.json
│
├─ packages/
│  ├─ ui/                             通用 UI 组件
│  ├─ editor/                         Monaco、Diff、Markdown
│  ├─ api-client/                     Rust IPC 类型封装
│  ├─ shared/                         TS 通用类型和工具
│  └─ eslint-config/
│
├─ crates/
│  ├─ devforge-domain/
│  ├─ devforge-application/
│  ├─ devforge-runtime/
│  ├─ devforge-storage/
│  ├─ devforge-indexer/
│  ├─ devforge-search/
│  ├─ devforge-code-intelligence/
│  ├─ devforge-ai/
│  ├─ devforge-agent/
│  ├─ devforge-git/
│  ├─ devforge-connectors/
│  ├─ devforge-security/
│  ├─ devforge-platform/
│  └─ devforge-shared/
│
├─ migrations/
├─ docs/
├─ scripts/
├─ Cargo.toml
├─ pnpm-workspace.yaml
└─ package.json
```

------

# 3. Rust crate 职责

## `devforge-domain`

纯领域模型，不依赖数据库、Tauri、Git 或 AI SDK。

包含：

```text
Workspace
Source
Document
CodeSymbol
CodeRelation
KnowledgeChunk
Conversation
Message
ChangeSession
ChangeSet
CommandTask
ModelProfile
Connector
AuditEvent
```

同时定义：

- 状态机
- 权限规则
- 风险等级
- 领域错误
- 领域事件
- 数据标识类型

例如不要到处使用裸 `String`：

```rust
WorkspaceId
DocumentId
ChunkId
SessionId
TaskId
ChangeSetId
```

这样可以避免把不同实体的 ID 传错。

------

## `devforge-application`

组织业务用例：

```text
CreateWorkspace
AddWorkspaceSource
StartIndexing
SearchKnowledge
AskWorkspace
CreateChangeSession
ApproveChangeSet
ExecuteCommand
CreateCheckpoint
SyncGitHubSource
```

Application 层负责业务编排，但不直接访问具体基础设施。

它依赖 Trait：

```rust
WorkspaceRepository
DocumentRepository
SearchIndex
VectorStore
ModelProvider
GitRepository
CommandExecutor
SecretStore
EventPublisher
```

------

## `devforge-runtime`

管理整个后台生命周期：

```text
AppRuntime
├─ TaskSupervisor
├─ EventBus
├─ JobScheduler
├─ IndexingCoordinator
├─ ModelRuntime
├─ ConnectorRuntime
└─ ShutdownCoordinator
```

所有长期任务统一注册，禁止随意散落 `tokio::spawn`。

后台任务必须具备：

- 名称
- 类型
- 当前状态
- 进度
- 可取消标志
- 重试策略
- 最近错误
- 开始与结束时间

------

## `devforge-storage`

负责 SQLite 数据持久化。

建议使用：

```text
SQLx
SQLite WAL
Migration
Repository Pattern
```

存储内容：

- 工作区和数据源
- 文档元数据
- 代码符号
- Code Graph 边
- 对话和消息
- AI 请求记录
- Change Session
- 任务记录
- 模型配置
- Connector 配置
- 审计记录

大型正文、索引文件和模型缓存不全部塞进数据库，而是保存到工作区数据目录。

------

## `devforge-indexer`

负责统一索引流水线：

```text
发现文件
  ↓
判断文件类型
  ↓
计算内容哈希
  ↓
文本提取
  ↓
代码解析
  ↓
Chunk 切分
  ↓
全文索引
  ↓
向量生成
  ↓
Code Graph 更新
  ↓
索引版本提交
```

它不负责搜索，只负责把数据转成可搜索结构。

------

## `devforge-search`

封装所有搜索能力：

```text
LexicalSearch
SemanticSearch
SymbolSearch
GraphSearch
GitSearch
HybridSearch
Reranking
```

混合检索输出统一结构：

```text
SearchHit
├─ source_type
├─ workspace_id
├─ document_id
├─ location
├─ title
├─ snippet
├─ lexical_score
├─ vector_score
├─ graph_score
├─ final_score
└─ evidence_level
```

------

## `devforge-code-intelligence`

包含：

- Tree-sitter Parser
- Language Registry
- Symbol Extractor
- Import Resolver
- Reference Analyzer
- LSP Client Manager
- Code Graph Builder
- Impact Analyzer

不同语言使用独立 Adapter：

```text
TypeScriptAdapter
RustAdapter
PythonAdapter
JavaAdapter
GoAdapter
```

第一阶段建议优先支持：

1. TypeScript / JavaScript
2. Rust
3. Python
4. Java
5. Go

------

## `devforge-ai`

统一管理不同模型：

```text
ChatModel
EmbeddingModel
RerankModel
ToolCallingModel
```

Provider：

```text
OllamaProvider
LmStudioProvider
OpenAIProvider
AnthropicProvider
GeminiProvider
OpenAICompatibleProvider
```

上层只依赖统一接口：

```rust
trait ChatModelProvider;
trait EmbeddingProvider;
trait RerankProvider;
```

AI 模块还负责：

- Prompt 模板
- Token 预算
- 上下文构建
- 消息压缩
- 流式输出
- 工具调用协议
- 模型回退
- 请求重试
- 成本统计
- 隐私过滤

------

## `devforge-agent`

负责受控 AI Agent。

核心不是让模型直接调用系统，而是让模型提交结构化意图：

```text
ReadFile
SearchCode
InspectSymbol
CreatePatch
RequestCommand
ReadGitDiff
RunTest
CreateCheckpoint
```

执行流程：

```text
AI 提交 Tool Call
   ↓
策略检查
   ↓
生成用户可读说明
   ↓
判断是否需要批准
   ↓
用户批准
   ↓
Rust 执行器执行
   ↓
结果写入任务时间线
   ↓
结果返回 AI
```

AI 永远不能绕过 Rust 策略层。

------

## `devforge-git`

封装 Git 操作：

- 仓库发现
- 状态查询
- 分支管理
- Diff
- Commit 历史
- Blame
- Checkpoint
- Worktree
- Patch 应用
- 提交创建

优先使用 `git2` 或受控调用系统 Git，二者可以组合：

- 常规读取使用 `git2`
- 复杂兼容场景调用系统 Git
- 所有写操作经过权限检查

------

## `devforge-connectors`

第一版实现：

```text
LocalDirectoryConnector
LocalGitConnector
GitHubConnector
GitLabConnector
```

后续扩展：

```text
OpenApiConnector
DatabaseSchemaConnector
WebConnector
JiraConnector
ConfluenceConnector
TerminalHistoryConnector
```

所有连接器遵循统一生命周期：

```text
configure
authenticate
validate
sync
pause
resume
remove
```

------

## `devforge-security`

负责：

- 路径权限
- 敏感文件识别
- Secret 扫描
- 云模型数据过滤
- 命令风险判断
- Tool Call 权限
- 审批记录
- 审计日志
- 凭据引用

敏感凭据不存 SQLite 明文，Windows 优先使用系统凭据管理器；跨平台通过 `SecretStore` Trait 抽象。

------

## `devforge-platform`

平台能力适配：

```text
WindowsPlatform
MacOsPlatform
LinuxPlatform
```

第一版 Windows 实现：

- Windows Credential Manager
- PowerShell 执行
- 文件系统监控
- 系统路径
- 默认终端
- Explorer 打开文件
- 进程管理
- 系统通知

核心业务代码不能出现大量：

```rust
#[cfg(target_os = "windows")]
```

条件编译集中放在平台 crate。

------

# 4. React 前端架构

建议采用：

```text
React
TypeScript
React Router
TanStack Query
Zustand
Monaco Editor
Tailwind CSS
Radix UI 或 shadcn/ui
```

职责划分：

- **TanStack Query**：Rust 后台数据和请求缓存
- **Zustand**：窗口状态、当前选区、面板布局等纯 UI 状态
- **React Router**：页面导航
- **Monaco Editor**：代码查看和 Diff
- **Tauri Channel**：流式聊天和终端输出
- **Tauri Event**：索引完成、任务状态、文件变化等通知

避免把所有数据都放进一个全局 Store。

------

# 5. React 功能目录

```text
src/
├─ app/
│  ├─ router/
│  ├─ providers/
│  ├─ layouts/
│  └─ bootstrap/
│
├─ features/
│  ├─ workspaces/
│  ├─ explorer/
│  ├─ search/
│  ├─ code-graph/
│  ├─ ai-chat/
│  ├─ change-review/
│  ├─ task-center/
│  ├─ git/
│  ├─ connectors/
│  ├─ models/
│  └─ settings/
│
├─ entities/
│  ├─ workspace/
│  ├─ document/
│  ├─ symbol/
│  ├─ conversation/
│  ├─ task/
│  └─ change-set/
│
├─ shared/
│  ├─ api/
│  ├─ components/
│  ├─ hooks/
│  ├─ lib/
│  └─ types/
│
└─ widgets/
   ├─ sidebar/
   ├─ command-palette/
   ├─ assistant-panel/
   └─ bottom-panel/
```

每个 Feature 自己维护：

```text
api
components
hooks
model
pages
types
utils
```

------

# 6. IPC 边界

Tauri 层只暴露粗粒度用例，不暴露数据库式接口。

正确：

```text
create_workspace
start_workspace_index
search_workspace
ask_workspace
approve_change_set
run_approved_task
```

避免：

```text
insert_document
update_chunk
delete_symbol
select_all_relations
```

前端不应该知道后台数据库表结构。

------

# 7. 启动流程

```text
Tauri 启动
   ↓
初始化日志
   ↓
解析数据目录
   ↓
加载配置
   ↓
打开 SQLite
   ↓
运行数据库迁移
   ↓
检查索引版本
   ↓
初始化搜索引擎
   ↓
启动 TaskSupervisor
   ↓
恢复未完成任务
   ↓
启动文件监听
   ↓
React 请求 bootstrap
   ↓
进入应用
```

如果某个非核心模块失败，例如 GitHub 同步不可用，应用仍然可以进入，只在健康状态中标记为降级。

只有以下问题阻止启动：

- 数据目录无法访问
- 数据库无法打开
- 核心迁移失败
- 配置文件严重损坏且无法恢复