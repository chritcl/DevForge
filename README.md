# DevForge

面向开发者的本地知识库、跨项目代码检索与受控 AI 编程工作台。

## 技术栈

- **桌面宿主**: Tauri 2
- **前端**: React + TypeScript
- **后端**: Rust Workspace
- **数据库**: SQLite（唯一事实来源）
- **全文搜索**: Tantivy
- **代码解析**: Tree-sitter

## 项目结构

```text
devforge/
├── apps/desktop/           Tauri 桌面应用
├── packages/               前端共享包
│   ├── ui/                 通用 UI 组件
│   ├── editor/             Monaco、Diff、Markdown
│   ├── api-client/         Rust IPC 类型封装
│   └── shared/             TS 通用类型和工具
├── crates/                 Rust Workspace
│   ├── devforge-domain/    纯领域模型
│   ├── devforge-application/  业务用例
│   ├── devforge-runtime/   后台生命周期管理
│   ├── devforge-storage/   SQLite 持久化
│   ├── devforge-platform/  平台适配
│   └── devforge-shared/    跨 crate 共享
├── migrations/             数据库迁移
├── docs/                   设计文档、ADR、阶段规格
└── .claude/                AI 协作配置
```

## 开发命令

### 前端

```powershell
pnpm install          # 安装依赖
pnpm typecheck        # 类型检查
pnpm test             # 测试
pnpm lint             # Lint
```

### Rust

```powershell
cargo fmt --check     # 格式检查
cargo clippy --workspace --all-targets -- -D warnings  # Lint
cargo test --workspace  # 测试
```

## AI 协作配置

`.claude/` 目录包含 Claude Code 的项目级配置：

### 路径规则 (`.claude/rules/`)

按文件路径自动加载的规则，减少无关上下文：

| 文件 | 匹配路径 | 内容 |
|------|----------|------|
| `always.md` | 始终加载 | 全局边界、架构规则、验证底线 |
| `rust.md` | `crates/**/*.rs` | unsafe、错误处理、异步并发、API 设计 |
| `react.md` | `apps/desktop/src/**` | 状态归属、TanStack Query、Zustand、TypeScript |
| `security.md` | 安全相关 crate + src-tauri | 路径安全、审批绑定、凭据、日志脱敏 |
| `testing.md` | 测试目录和文件 | 测试原则、Property test、Fuzz、Benchmark |
| `dependencies.md` | Cargo.toml、package.json、lockfile | 依赖选择标准、推荐技术栈 |

### 工作流 Skills (`.claude/skills/`)

| Skill | 用途 |
|-------|------|
| `development-flow` | 完整开发编排（多文件、子智能体） |
| `implement-task` | 单任务 TDD 执行 |
| `verify-task` | 验证与检查 |
| `review-change` | 代码审查 |
| `systematic-debugging` | 系统调试 |

### 子智能体 (`.claude/agents/`)

专用子智能体定义，由 Skills 按需调用。

### Hooks (`.claude/hooks/`)

- `guard-dangerous-command.ps1` — 危险命令拦截
- `format-edited-file.ps1` — 编辑后自动格式化
- `notify.ps1` — 通知

## 文档

- [产品愿景](docs/product/vision.md)
- [架构总览](docs/architecture/overview.md)
- [阶段规格](docs/phases/)
- [架构决策记录](docs/adr/)
- [实施计划](docs/plans/)
