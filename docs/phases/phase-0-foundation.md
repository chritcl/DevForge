# 阶段零：工程基础设施

## 目标

建立可持续开发的大型 Monorepo，以及 Rust、React、Tauri 的基础边界。

## 主要工作

### Monorepo

```text
apps/desktop
packages/ui
packages/api-client
packages/shared
crates/*
```

### Rust Workspace

第一阶段只创建必要 crate：

```text
devforge-domain
devforge-application
devforge-runtime
devforge-storage
devforge-platform
devforge-shared
```

不要第一天就创建二十个空 crate。随着能力落地，再逐步拆出：

```text
devforge-indexer
devforge-search
devforge-ai
devforge-agent
devforge-connectors
```

### 前端基础

完成：

- React 应用外壳
- Router
- TanStack Query
- Zustand
- IPC Client
- 主题系统
- Error Boundary
- 基础布局
- Command Palette 框架

### 工程质量

建立：

```text
Rust fmt
Clippy
TypeScript Check
ESLint
Frontend Tests
Rust Tests
Commit Hooks
CI
```

## 交付结果

应用能够启动，React 可以通过类型安全 IPC 调用 Rust，并展示：

- 应用版本
- 数据目录
- 后台健康状态
- SQLite 状态
- 基础日志

## 阶段退出条件

- Windows 开发环境可以一条命令启动
- Rust 与 TypeScript 类型同步方案确定
- CI 可以构建 Tauri 应用
- SQLite Migration 能在空数据库运行
- Rust Core 不直接依赖 React
- Domain 层不依赖 Tauri
- Release 构建可以安装和启动

## 第一版 Crate 划分

### devforge-domain

纯领域模型，不依赖数据库、Tauri、Git 或 AI SDK。包含 Workspace、Source、Document、CodeSymbol、CodeRelation、KnowledgeChunk、Conversation、Message、ChangeSession、ChangeSet、CommandTask、ModelProfile、Connector、AuditEvent 等核心实体，以及状态机、权限规则、风险等级、领域错误、领域事件和数据标识类型。

### devforge-application

组织业务用例，依赖 Trait 接口而非具体基础设施。

### devforge-runtime

管理整个后台生命周期：AppRuntime、TaskSupervisor、EventBus、JobScheduler 等。

### devforge-storage

负责 SQLite 数据持久化，使用 SQLx + SQLite WAL + Migration + Repository Pattern。

### devforge-platform

平台能力适配（Windows、macOS、Linux），核心业务代码不能出现大量条件编译。

### devforge-shared

跨 crate 共享的类型和工具。
