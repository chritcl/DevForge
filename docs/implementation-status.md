# DevForge 实施状态矩阵

## 状态定义

| 状态 | 含义 |
|------|------|
| NOT STARTED | 尚未开始实施 |
| IN PROGRESS | 正在实施中 |
| IMPLEMENTED | 代码已实现但未独立验证 |
| VERIFIED | 代码已实现且通过独立验证 |
| BLOCKED | 被其他任务阻塞 |

## 实施状态

### Phase 0 - Foundation

| 能力 | 文档状态 | 代码入口 | 测试证据 | 当前状态 |
|------|----------|----------|----------|----------|
| 工程基础设施 | 已完成 | apps/desktop, crates/* | 9 tests pass | VERIFIED |
| SQLite Bootstrap | 已完成 | crates/devforge-storage | 5 integration tests | VERIFIED |
| Tauri IPC (Specta) | 已完成 | apps/desktop/src-tauri | 19 commands with Specta | VERIFIED |
| React Router | 已完成 | apps/desktop/src/router.tsx | 2 test files | VERIFIED |
| 主题系统 | 已完成 | apps/desktop/src/styles | UI tests | VERIFIED |
| CI 流水线 | 已完成 | .github/workflows | pnpm check | VERIFIED |
| NSIS 安装包 | 已完成 | scripts/ | smoke test | VERIFIED |

### Phase 1 - Local Workspace

| 能力 | 文档状态 | 代码入口 | 测试证据 | 当前状态 |
|------|----------|----------|----------|----------|
| Domain 模型 | 已实现 | crates/devforge-domain | 29 unit tests | VERIFIED |
| SQLite Schema | 已实现 | crates/devforge-storage/migrations | 9 tests | VERIFIED |
| Repository 层 | 已实现 | crates/devforge-storage/src/repository.rs | 4 integration tests | VERIFIED |
| Workspace CRUD 用例 | 已实现 | crates/devforge-application/src/workspace.rs | 7 unit tests | VERIFIED |
| Workspace Tauri 命令 | 已实现 | apps/desktop/src-tauri/src/commands/workspace.rs | 9 commands | VERIFIED |
| PathGuard 路径安全 | 已实现 | crates/devforge-domain/src/path_guard.rs | 10 unit tests | VERIFIED |
| Source CRUD 用例 | 已实现 | crates/devforge-application/src/source.rs | 7 unit tests | VERIFIED |
| Source Tauri 命令 | 已实现 | apps/desktop/src-tauri/src/commands/source.rs | 4 commands | VERIFIED |
| 文件发现用例 | 已实现 | crates/devforge-application/src/discovery.rs | 7 unit tests | VERIFIED |
| 文件发现 Tauri 命令 | 已实现 | apps/desktop/src-tauri/src/commands/discovery.rs | 1 command | VERIFIED |
| 文档查询用例 | 已实现 | crates/devforge-application/src/document.rs | 3 unit tests | VERIFIED |
| 文档 Tauri 命令 | 已实现 | apps/desktop/src-tauri/src/commands/document.rs | 2 commands | VERIFIED |
| 标签页用例 | 已实现 | crates/devforge-application/src/tab.rs | 5 unit tests | VERIFIED |
| 标签页 Tauri 命令 | 已实现 | apps/desktop/src-tauri/src/commands/tab.rs | 4 commands | VERIFIED |
| 添加数据源 UI | 已实现 | apps/desktop/src/components/AddSourceDialog.tsx | 前端测试通过 | IMPLEMENTED |
| 懒加载文件树 | 已实现 | apps/desktop/src/components/FileTree.tsx | 前端测试通过 | IMPLEMENTED |
| 标签栏组件 | 已实现 | apps/desktop/src/components/TabBar.tsx | 前端测试通过 | IMPLEMENTED |
| 启动恢复逻辑 | 已实现 | apps/desktop/src/pages/WorkspacePage.tsx | 前端测试通过 | IMPLEMENTED |

### Phase 2+ - 未开始

| 阶段 | 名称 | 状态 | 说明 |
|------|------|------|------|
| Phase 2 | Indexing | NOT STARTED | Tantivy、Tree-sitter、Code Graph |
| Phase 3 | Search | NOT STARTED | 全文搜索、语义搜索 |
| Phase 4 | AI Knowledge | NOT STARTED | AI 问答、RAG、引用验证 |
| Phase 5 | Agent | NOT STARTED | AI Agent、Patch、命令执行 |
| Phase 6 | Connectors | NOT STARTED | GitHub/GitLab 集成 |
| Phase 7 | Plugins | NOT STARTED | 插件系统 |
| Phase 8 | Release | NOT STARTED | 发布、自动更新、代码签名 |

## 已修复的问题

### P0 - 数据损坏（已修复）
- modified_at 计算逻辑错误：使用 DateTime::from_timestamp 替代错误的 Utc::now() - duration
- removed 计数不更新数据库：已删除文件现在标记为 content_readable = false

### P1 - 主流程不可用（已修复）
- Specta 绑定不完整：所有 19 个命令已注册到 Specta
- 无添加数据源 UI：已创建 AddSourceDialog 组件
- 文件树非懒加载：已重写为使用 parentPath 懒加载
- 无启动恢复逻辑：WorkspacePage 现在在挂载时恢复标签
- 标签活跃状态切换为空操作：已修复 OpenTab::execute
- 无标签栏：已创建 TabBar 组件
- 不记录 last_opened_at：已添加 mark_workspace_opened 命令

### P2 - 测试和可维护性（已修复）
- list_documents 查询逻辑缺陷：已实现 list_by_source_and_parent

## 验证命令记录

```
cargo fmt --all -- --check
- 通过

cargo clippy --workspace --all-targets -- -D warnings
- 通过

cargo test --workspace
- 通过（42 tests）

pnpm typecheck
- 通过

pnpm test
- 通过（5 tests）
```

## 基线信息

- 分支：fix/phase-1-vertical-slice
- 起始 HEAD：0e9aeb3
- 当前 HEAD：b24dbf4
- 基线检查：全部通过
