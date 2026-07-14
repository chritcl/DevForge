# DevForge Project Instructions

## 产品

DevForge — 面向开发者的本地知识库、跨项目代码检索与受控 AI 编程工作台。

产品规格：[docs/product/vision.md](docs/product/vision.md)

## 技术栈

- 桌面宿主：Tauri 2
- 前端：React + TypeScript
- 后端：Rust Workspace
- 数据库：SQLite（唯一事实来源）
- 全文搜索：Tantivy
- 代码解析：Tree-sitter
- 语义增强：可选 LSP

架构总览：[docs/architecture/overview.md](docs/architecture/overview.md)

## 命令

| 操作 | 命令 |
|------|------|
| 前端安装 | `pnpm install` |
| 前端类型检查 | `pnpm typecheck` |
| 前端测试 | `pnpm test` |
| Rust 格式检查 | `cargo fmt --check` |
| Rust Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Rust 测试 | `cargo test --workspace` |

## 当前阶段

当前实施阶段：[docs/phases/phase-0-foundation.md](docs/phases/phase-0-foundation.md)

当前已批准计划：[docs/plans/phase-0-foundation-plan.md](docs/plans/phase-0-foundation-plan.md)

## 规则与 Skills

全局规则和路径规则在 `.claude/rules/` 中，按文件路径自动加载。

工作流 Skills 在 `.claude/skills/` 中，按需调用。

详见 [README.md](README.md) 的「AI 协作配置」章节。
