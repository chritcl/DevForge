# DevForge 架构总览

## 总体架构

采用：

> **Tauri 2 + React + TypeScript + Rust Workspace + SQLite + Tantivy + 嵌入式向量索引**

核心原则：**Tauri 只负责桌面宿主与 IPC，真正的业务能力放在独立 Rust crate 中**。以后即使拆成后台守护进程或服务端，也不需要重写核心逻辑。

### 进程架构

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

## Monorepo 结构

使用 `pnpm workspace + Cargo workspace`：

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

## Rust Crate 划分原则

### devforge-domain

纯领域模型，不依赖数据库、Tauri、Git 或 AI SDK。包含 Workspace、Document、CodeSymbol、Conversation、ChangeSession、CommandTask 等核心实体，以及状态机、权限规则、风险等级、领域错误和领域事件。

使用标识类型避免裸 String：

```rust
WorkspaceId / DocumentId / ChunkId / SessionId / TaskId / ChangeSetId
```

### devforge-application

组织业务用例（CreateWorkspace、StartIndexing、SearchKnowledge、AskWorkspace 等），依赖 Trait 而非具体基础设施：

```rust
WorkspaceRepository / DocumentRepository / SearchIndex / VectorStore
ModelProvider / GitRepository / CommandExecutor / SecretStore / EventPublisher
```

### devforge-runtime

管理后台生命周期：TaskSupervisor、EventBus、JobScheduler、IndexingCoordinator、ModelRuntime、ConnectorRuntime。所有长期任务统一注册，禁止随意散落 `tokio::spawn`。

### devforge-storage

SQLite 数据持久化，使用 SQLx + WAL + Migration + Repository Pattern。大型正文、索引文件和模型缓存不全部塞进数据库。

### devforge-indexer

统一索引流水线：发现文件 → 类型判断 → 内容哈希 → 文本提取 → 代码解析 → Chunk 切分 → 全文索引 → 向量生成 → Code Graph 更新。只负责把数据转成可搜索结构。

### devforge-search

封装所有搜索能力：LexicalSearch、SemanticSearch、SymbolSearch、GraphSearch、GitSearch、HybridSearch、Reranking。输出统一 SearchHit 结构。

### devforge-code-intelligence

Tree-sitter Parser、Language Registry、Symbol Extractor、LSP Client Manager、Code Graph Builder。不同语言使用独立 Adapter（TypeScript、Rust、Python、Java、Go）。

### devforge-ai

统一管理 ChatModel、EmbeddingModel、RerankModel、ToolCallingModel。支持 Ollama、OpenAI、Anthropic、Gemini、LM Studio 等 Provider。负责 Prompt 模板、Token 预算、上下文构建、流式输出、隐私过滤。

### devforge-agent

受控 AI Agent，核心是让模型提交结构化意图（ReadFile、SearchCode、CreatePatch、RequestCommand），由 Rust 策略层决定是否执行。AI 永远不能绕过 Rust 策略层。

### devforge-git

封装 Git 操作，优先使用 `git2`，复杂兼容场景调用系统 Git，所有写操作经过权限检查。

### devforge-connectors

第一版实现 LocalDirectoryConnector、LocalGitConnector、GitHubConnector、GitLabConnector。所有连接器遵循统一生命周期。

### devforge-security

路径权限、敏感文件识别、Secret 扫描、云模型数据过滤、命令风险判断、Tool Call 权限、审批记录、审计日志。

### devforge-platform

平台能力适配，第一版 Windows 实现：Credential Manager、PowerShell 执行、文件系统监控、系统路径。核心业务代码不出现大量 `#[cfg(target_os = "windows")]`。

## 前后端通信原则

Tauri 层只暴露粗粒度用例，不暴露数据库式接口。

正确：

```text
create_workspace / start_workspace_index / search_workspace
ask_workspace / approve_change_set / run_approved_task
```

避免：

```text
insert_document / update_chunk / delete_symbol / select_all_relations
```

前端不应该知道后台数据库表结构。

对于普通请求响应，React 通过 Tauri Command 调用 Rust；日志和任务输出这类连续数据通过 Channel 传输，低频状态变化使用 Event。

## 核心数据流

### 用户主流程

```text
创建工作区
   ↓
添加本地仓库和文档
   ↓
Rust 后台扫描并建立索引
   ↓
Tree-sitter 提取代码结构
   ↓
可用时由 LSP 增强语义关系
   ↓
生成全文索引、向量索引和 Code Graph
   ↓
用户搜索或向 AI 提问
   ↓
混合检索构建项目上下文
   ↓
AI 返回带来源的答案
   ↓
用户要求修改代码
   ↓
AI 创建变更计划和 Patch
   ↓
用户审查并批准
   ↓
Rust 应用变更并执行测试
   ↓
生成完整任务记录和回滚点
```

### 启动流程

```text
Tauri 启动 → 初始化日志 → 解析数据目录 → 加载配置
→ 打开 SQLite → 运行数据库迁移 → 检查索引版本
→ 初始化搜索引擎 → 启动 TaskSupervisor → 恢复未完成任务
→ 启动文件监听 → React 请求 bootstrap → 进入应用
```

如果某个非核心模块失败（例如 GitHub 同步不可用），应用仍然可以进入，只在健康状态中标记为降级。只有数据目录无法访问、数据库无法打开、核心迁移失败或配置文件严重损坏时才阻止启动。
