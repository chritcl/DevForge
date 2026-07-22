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
| Domain 模型 | 已实现 | crates/devforge-domain | 34 unit tests | VERIFIED |
| SQLite Schema | 已实现 | crates/devforge-storage/migrations | 5 integration tests | VERIFIED |
| Repository 层 | 已实现 | crates/devforge-storage/src/repository.rs | 4 integration tests | VERIFIED |
| Workspace CRUD 用例 | 已实现 | crates/devforge-application/src/workspace.rs | 7 unit tests | VERIFIED |
| Workspace Tauri 命令 | 已实现 | apps/desktop/src-tauri/src/commands/workspace.rs | 9 commands | VERIFIED |
| PathGuard 路径安全 | 已实现 | crates/devforge-domain/src/path_guard.rs | 15 unit tests | VERIFIED |
| Source 自动识别 | 已实现 | crates/devforge-application/src/source.rs | 10 unit tests | VERIFIED |
| Source Tauri 命令 | 已实现 | apps/desktop/src-tauri/src/commands/source.rs | 3 commands | VERIFIED |
| 文件发现用例 | 已实现 | crates/devforge-application/src/discovery.rs | 7 unit tests | VERIFIED |
| 文件发现 Tauri 命令 | 已实现 | apps/desktop/src-tauri/src/commands/discovery.rs | 1 command | VERIFIED |
| 文档查询用例 | 已实现 | crates/devforge-application/src/document.rs | 12 unit tests | VERIFIED |
| 文件树查询用例 | 已实现 | crates/devforge-application/src/document.rs | 12 unit tests | VERIFIED |
| 文档 Tauri 命令 | 已实现 | apps/desktop/src-tauri/src/commands/document.rs | 4 commands | VERIFIED |
| 标签页用例 | 已实现 | crates/devforge-application/src/tab.rs | 5 unit tests | VERIFIED |
| 标签页 Tauri 命令 | 已实现 | apps/desktop/src-tauri/src/commands/tab.rs | 4 commands | VERIFIED |
| 添加数据源 UI | 已实现 | apps/desktop/src/components/AddSourceDialog.tsx | 前端测试通过 | IMPLEMENTED |
| 文件树组件 | 已实现 | apps/desktop/src/components/FileTree.tsx | 前端测试通过 | IMPLEMENTED |
| 文件查看器组件 | 已实现 | apps/desktop/src/components/FileViewer.tsx | 前端测试通过 | IMPLEMENTED |
| 标签栏组件 | 已实现 | apps/desktop/src/components/TabBar.tsx | 前端测试通过 | IMPLEMENTED |
| 启动恢复逻辑 | 已实现 | apps/desktop/src/pages/WorkspacePage.tsx | 前端测试通过 | IMPLEMENTED |

### Phase 1A - 安全修复与深层文件支持

| 能力 | 文档状态 | 代码入口 | 测试证据 | 当前状态 |
|------|----------|----------|----------|----------|
| read_document_content 安全修复 | 已实现 | crates/devforge-application/src/document.rs | 安全测试通过 | IMPLEMENTED |
| add_local_source 自动识别 | 已实现 | crates/devforge-application/src/source.rs | 识别测试通过 | IMPLEMENTED |
| 文件树显式查询模型 | 已实现 | crates/devforge-application/src/document.rs | 文件树测试通过 | IMPLEMENTED |
| 标签批量恢复 | 已实现 | crates/devforge-application/src/document.rs | 批量查询测试通过 | IMPLEMENTED |
| Sensitivity 强类型 | 已实现 | crates/devforge-domain/src/document.rs | Specta 生成 | IMPLEMENTED |
| bindings.ts 更新 | 已实现 | apps/desktop/src/bindings.ts | 21 commands | IMPLEMENTED |

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

## Phase 1A 修复内容

### 安全修复
- 移除 `read_document_content` 的 `source_root` 参数
- 后端从数据库反查可信 `source.root_path`
- PathGuard 新增 `resolve_relative_file` 安全入口
- join 前验证路径组件（拒绝绝对路径、ParentDir、Prefix）
- join 后 canonicalize 并验证位于可信根目录内

### 数据源自动识别
- 新增 `add_local_source` 统一入口
- 后端自动识别 Git 仓库、Git Worktree、普通目录
- 删除旧的 `add_git_source` 和 `add_directory_source` IPC 命令
- 路径规范化和重复检测使用 canonicalize 后的路径

### 文件树显式查询模型
- 新增 `FileTreeEntry`、`FileTreeEntryDto`、`FileTreeEntryKind`
- 目录不是 Document，没有伪造的 document_id
- 文件条目使用真实 Document ID
- 使用 `Path::strip_prefix` 组件级判断，不使用字符串前缀匹配

### 标签批量恢复
- 新增 `get_documents_by_ids` 批量查询
- 支持任意数量数据源和任意目录深度
- 失效标签显示"文件不可用"，不自动关闭
- 空 ID 列表不发起 IPC

### 敏感度强类型
- `DocumentKind` 和 `Sensitivity` 枚举添加 Specta + serde(rename_all = "snake_case")
- 前端不再手写字符串枚举
- 修复 Sensitivity 大小写不匹配 Bug

## 已修复的问题

### P0 - 数据损坏（已修复）
- modified_at 计算逻辑错误：使用 DateTime::from_timestamp 替代错误的 Utc::now() - duration
- removed 计数不更新数据库：已删除文件现在标记为 content_readable = false

### P1 - 主流程不可用（已修复）
- Specta 绑定不完整：所有 21 个命令已注册到 Specta
- 无添加数据源 UI：已创建 AddSourceDialog 组件
- 文件树非懒加载：已重写为使用 parentPath 懒加载
- 无启动恢复逻辑：WorkspacePage 现在在挂载时恢复标签
- 标签活跃状态切换为空操作：已修复 OpenTab::execute
- 无标签栏：已创建 TabBar 组件
- 不记录 last_opened_at：已添加 mark_workspace_opened 命令

### P2 - 测试和可维护性（已修复）
- list_documents 查询逻辑缺陷：已实现 list_by_source_and_parent

### P3 - 安全和深层文件（Phase 1A 已修复）
- source_root 参数不受信：已移除，后端从数据库反查
- 深层文件无法导航：已实现文件树显式查询模型
- 标签恢复依赖根目录列表：已实现批量查询
- 硬编码三个数据源：已移除限制
- Sensitivity 大小写不匹配：已修复为 snake_case
- bindings.ts 过期：已重新生成

## v0.1 实施状态

### Phase A - 基线收敛（DONE）

| 工作包 | 状态 | 提交 |
|--------|------|------|
| WP-A01 创建 docs/GOAL.md | DONE | 4018dc3 |
| WP-A02 创建 docs/STATE.json | DONE | 4018dc3 |
| WP-A03 修复关闭活动标签持久化 | DONE | 4018dc3 |
| WP-A04 修复 removeSource 缓存一致性 | DONE | 4018dc3 |
| WP-A05 删除死文件 types.ts | DONE | 4018dc3 |
| WP-A06 修复 GetDocumentsByIds 错误吞没 | DONE | 4018dc3 |
| WP-A07 更新 implementation-status.md | DONE | 4018dc3 |

### Phase B - 文件查看（DONE）

| 工作包 | 状态 | 提交 |
|--------|------|------|
| WP-B01 Monaco Editor 代码查看器 | DONE | 0a759c5 |
| WP-B02 Markdown 安全渲染器 | DONE | 0a759c5 |
| WP-B03 集成到 FileViewer | DONE | 0a759c5 |

### Phase C - 基础全文索引（DONE）

| 工作包 | 状态 | 提交 |
|--------|------|------|
| WP-C01 Tantivy 索引基础设施 | DONE | 323e867 |
| WP-C02 集成到 Source 扫描流程 | DONE | 78f5868 |
| WP-C03 暴露索引状态和管理命令 | DONE | 750c58f |

### Phase D - 关键词搜索（DONE）

| 工作包 | 状态 | 提交 |
|--------|------|------|
| WP-D01 搜索 Tauri 命令和前端 UI | DONE | 2c57429 |

### Phase E - v0.1 收口（IN PROGRESS）

| 工作包 | 状态 | 说明 |
|--------|------|------|
| WP-E01 最终验证和文档更新 | IN PROGRESS | 全量验证、文档更新 |

## 验证命令记录

```
cargo fmt --all -- --check
- 通过

cargo clippy --workspace --all-targets -- -D warnings
- 通过

cargo test --workspace
- 通过（105 tests）

pnpm typecheck
- 通过

pnpm test
- 通过（5 tests）

pnpm lint
- 通过
```

## 已确认实现的功能

### 核心工作流

- 工作区创建、编辑、归档、恢复和删除 UI
- 添加数据源（自动识别 Git/目录）
- 文件树懒加载浏览
- 代码查看器（Monaco Editor，语法高亮，只读）
- Markdown 安全渲染（react-markdown + rehype-sanitize）
- 敏感文件和二进制文件拦截
- 全文索引（Tantivy，自动索引扫描文件）
- 关键词搜索（工作区级，搜索文件名和内容）
- 标签栏和标签管理
- 启动恢复标签和工作区状态
- 移除数据源时级联清理索引和缓存

### 安全特性

- PathGuard 路径安全（9层防御，symlink 解析）
- 文件系统根路径由后端获取
- 删除元数据不触碰本地文件
- Markdown 禁止脚本和不安全 HTML
- 敏感文件不读取正文

## 剩余缺口

以下功能尚未实现，不影响 v0.1 核心流程：

- 文件监听与增量刷新（手动触发扫描）
- workspace_settings 的真实使用
- 大目录分页（当前不设上限，性能风险已记录）
- 搜索结果高亮（snippet）

## 基线信息

- 分支：main
- 当前 HEAD：2c57429
