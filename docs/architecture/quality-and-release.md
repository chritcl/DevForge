# 错误体系、日志、可观测性、测试与发布升级

## 1. 设计目标

1. 出错时能够明确定位到模块、工作区、任务和操作
2. 单个连接器、模型或索引损坏不会拖垮整个应用
3. 日志默认保存在本地，不无意记录代码、密钥和用户提示词
4. 用户可以导出脱敏诊断包
5. 测试覆盖领域规则、索引一致性、安全边界和完整桌面流程
6. 发布升级不能破坏工作区数据库、索引和未完成的 Change Session
7. 更新过程不能在 Agent、命令或文件写入进行中强制重启
8. 所有性能优化都必须有指标和基准

------

## 2. 统一错误体系

分层：DomainError（业务规则不允许）、ApplicationError（业务用例无法完成）、InfrastructureError（本地基础设施问题）、IntegrationError（外部服务问题）、SecurityError（安全策略阻止）、UserInputError（用户信息不合法）。

## 3. 对外错误协议

统一响应 AppErrorResponse：code（稳定机器可读编码，如 DF-WORKSPACE-001）、category、user_message、technical_message、retryable、severity、suggested_actions、correlation_id、context。错误代码一旦公开就不能随意改变含义。

## 4. Correlation ID

每个用户操作都生成关联 ID，一次操作中的 Tauri Command、Application Use Case、Database Query、Search Request、Model Run、Tool Call、Background Job、Error、UI Notification 共享关联 ID。

## 5. 错误严重等级

- **Warning**：系统已降级但主要功能可用（Reranker 不可用、GitHub 限流）
- **Error**：当前操作失败但应用整体可用（搜索任务失败、模型请求失败）
- **Critical**：核心运行条件受影响（SQLite 无法打开、核心迁移失败）
- **Security**：操作因安全策略被拒绝或检测到异常行为

## 6. 错误传播规则

基础设施层添加底层上下文但不写最终 Error 日志 → Application 层添加业务上下文 → Tauri/Job 边界记录一次最终错误日志并转换为 AppErrorResponse。保留错误链，但用户只看到最有用的一层。

## 7. 可恢复错误与不可恢复错误

每个错误声明 retryable、recovery_strategy、safe_to_continue、requires_user_action。可自动重试：临时网络错误、数据库短暂锁竞争。需要用户操作：认证失效、磁盘空间不足、文件冲突。不应自动重试：命令执行失败、Patch 基线冲突。

## 8. Panic 策略

普通错误使用 Result，Panic 只代表内部不变量被破坏。关键后台任务设置 Panic 观察边界。第一版建议保持 `panic = "unwind"`。

------

## 9. 日志架构

统一使用 tracing + tracing-subscriber。日志管线：Console Layer、Rolling File Layer、In-memory Diagnostic Layer、Optional OpenTelemetry Layer、Security Audit Layer。

## 10. 日志事件结构

每条结构化日志包含：timestamp、level、target、message、correlation_id、workspace_id、repository_id、task_id、agent_run_id、change_session_id、connector_id、provider_id、plugin_id、duration_ms、error_code。

## 11. Span 设计

重要流程创建 Span：workspace.create、workspace.open、index.workspace、index.document、search.execute、rag.build_context、model.complete、agent.run、tool.execute、change.apply、command.run、connector.sync、plugin.invoke。

## 12. 日志级别规范

- **TRACE**：极细粒度内部状态，仅开发调试开启
- **DEBUG**：开发诊断（索引任务状态变化、查询计划、缓存命中）
- **INFO**：正常业务生命周期（应用启动、索引完成、同步完成）
- **WARN**：可恢复异常或降级（Reranker 不可用、LSP 自动重启）
- **ERROR**：明确失败（任务失败、数据库写入失败）

默认 Release 日志不应开启 TRACE。

## 13. 日志过滤

开发环境：devforge=debug、tantivy=info、sqlx=warn。生产环境：devforge=info、devforge_security=warn。用户可在诊断模式中临时提高某个模块的日志等级，到期自动恢复。

## 14. 日志文件策略

按日期滚动、单文件大小限制、总保留天数限制、总磁盘容量限制、压缩旧日志。默认：普通日志 14 天、安全日志 30 天、崩溃日志最近 10 份。

## 15. 日志脱敏

日志记录前通过 RedactionLayer 处理：API Key、Bearer Token、Cookie、数据库连接字符串、SSH 私钥、环境变量、用户主目录、Prompt 中的敏感代码。AI 请求默认只记录输入 Token、上下文项数量、内容哈希、模型、耗时。

## 16. 审计日志与技术日志分离

普通日志用于调试程序。审计日志用于回答"谁批准了什么？AI 读取了哪些文件？哪些内容被发送到了云端？执行了什么命令？"审计记录必须结构化保存到 SQLite，技术日志被清理后关键审计事件仍然存在。

## 17. 前端日志

React 前端错误捕获：React Error Boundary、Unhandled Promise Rejection、IPC Failure、Stream Decode Failure、Monaco Failure。前端错误发送给 Rust（report_frontend_error），不能包含完整文件内容、AI 对话全文、API Token。

## 18. Error Boundary

多层 Error Boundary：AppBoundary → WorkspaceBoundary、EditorBoundary、AssistantBoundary、GraphBoundary、BottomPanelBoundary。如果 Code Graph 渲染失败，文件编辑器和 AI 对话继续工作。

------

## 19. 可观测性信号

三类核心信号：Logs、Metrics、Traces。默认模式：结构化本地日志、本地性能指标、本地任务时间线。可选模式：用户主动启用 OpenTelemetry Export。默认不能自动把遥测发送到远程服务。

## 20. 核心指标

- **应用指标**：启动时间、首屏时间、工作区打开时间、内存占用、CPU 使用率、磁盘占用
- **索引指标**：扫描文件数、解析文件数、Chunk 数、每秒解析、Embedding 处理量、索引失败率
- **搜索指标**：搜索总耗时、各 Retriever 耗时、候选数量、Rerank 耗时、缓存命中率、零结果率
- **AI 指标**：首 Token 时间、完整生成时间、输入输出 Token、工具调用次数、取消率、引用校验失败率
- **Agent 指标**：任务完成率、人工审批次数、ChangeSet 修订次数、Patch 冲突率、回滚率
- **连接器指标**：同步耗时、同步文档数、API 请求数、限流次数、认证失败

## 21. 指标基数控制

不能给指标添加无限增长的标签（file_path、query_text、prompt_text 等）。工作区 ID、文件路径和查询内容更适合进入带访问控制的日志。

------

## 22. 健康检查模型

每个核心组件实现 HealthCheck trait。组件状态：Healthy、Degraded、Unavailable、Recovering、Disabled、Unknown。检查对象：SQLite、Tantivy、Vector Store、Task Scheduler、File Watcher、Git、LSP、Model Providers、Connectors、Plugins、Updater、Disk。

## 23. 诊断中心

诊断页面展示：应用版本、构建信息、操作系统、WebView 版本、Rust Core 状态、数据库版本、索引版本、工作区健康、后台任务、模型状态、连接器状态、插件状态、磁盘使用、最近错误。提供操作：运行快速/完整诊断、打开日志目录、导出诊断包、修复索引、重置 UI 布局、验证数据库。

## 24. 诊断包

导出格式：devforge-diagnostic-{time}.zip，包含 manifest.json、system-info.json、app-config-redacted.json、health.json、recent-errors.json、logs/、database-schema.json 等。默认不包含源码正文、AI 对话正文、完整 Prompt、凭据。

## 25. 崩溃报告

应用异常退出后下次启动显示检测到的中断状态。操作：恢复工作区、查看崩溃详情、导出诊断包、暂时禁用插件启动、安全模式启动。发送远程崩溃报告必须明确征得用户同意、展示内容、完成脱敏。

## 26. 安全模式

短时间内连续崩溃 3 次建议进入安全模式：禁用第三方插件、暂停外部连接器、不自动恢复 AgentRun、不自动启动 LSP、不自动打开上次工作区、使用默认 UI 布局。

------

## 27. Rust 测试分层

Unit Tests、Domain Tests、Application Tests、Repository Tests、Adapter Contract Tests、Integration Tests、Property Tests、Fuzz Tests、End-to-End Tests、Performance Benchmarks。

## 28. Domain 测试

领域层必须完全不依赖 Tauri、SQLite 和网络。测试重点：AgentRun 状态机、ChangeSession 状态机、审批指纹、风险等级、索引版本切换。

## 29. Application 测试

使用内存或 Fake Adapter 测试业务用例。覆盖：创建工作区、首次索引、搜索、AI 问答、Change Session、批准变更、执行验证、连接器同步、任务恢复。

## 30. Repository 测试

对 SQLite Repository 使用临时数据库。覆盖：Migration、CRUD、事务回滚、唯一约束、并发读写、WAL 模式、崩溃恢复标记。每个迁移版本都应测试空数据库→最新版本、上一个正式版本→最新版本。

## 31. Adapter Contract Test

所有相同 Trait 的实现运行同一组契约测试（如 ChatProvider Contract：正常回答、流式回答、取消、超时、无效认证、Tool Call）。

## 32. Property-based Testing

适合测试：路径规范化、审批哈希稳定性、Patch 应用、索引事件合并、查询过滤、任务状态机。路径安全测试生成：..、大小写变化、Unicode、符号链接、UNC、设备路径、Alternate Data Stream、超长路径。

## 33. Fuzz Testing

优先 Fuzz 的入口：Unified Diff Parser、.gitignore/.devforgeignore Parser、Model Tool Call JSON、Connector Webhook Payload、Document Metadata Parser、Plugin Manifest、Search Query Parser。

## 34. 索引测试

建立小型固定测试仓库 fixtures：typescript-project、rust-project、python-project、mixed-monorepo、malformed-project。覆盖：符号提取、Chunk 边界、Import 关系、增量修改、文件重命名/删除、分支切换、索引重建、崩溃中断。

## 35. 搜索质量回归测试

建立标准问题集（问题、预期文件、预期符号、预期答案要点、不应出现的结果）。每次调整 Tokenizer、Chunk 策略、搜索权重、Embedding、Reranker、RRF 参数、Graph 扩展、查询改写时运行。构建门槛：Exact Symbol Hit Rate 不得下降、Recall@10 下降不得超过阈值、引用正确率必须达到最低线。

## 36. 安全测试

必须覆盖：工作区路径逃逸、符号链接逃逸、Junction 逃逸、敏感文件读取、过期审批重放、命令参数变更、插件权限升级、云模型隐私策略绕过、Prompt Injection、日志凭据泄漏。安全测试必须直接调用 Rust Application API。

## 37. React 测试

- **单元与组件测试**（Vitest + Testing Library）：状态选择器、Query Key、错误展示、引用交互、审批卡片
- **IPC 集成测试**：Mock Tauri Command/Event/Channel
- **桌面端到端测试**：通过 WebDriverIO 驱动

## 38. CI 流程

PR 流程：格式化检查 → Rust Clippy → TypeScript 类型检查 → Rust 单元与集成测试 → React 测试 → 安全规则测试 → Fixture 索引测试 → 构建桌面应用 → Windows E2E。

主分支额外：跨平台构建、搜索质量回归、性能基准、依赖漏洞检查、许可证检查。

Release Tag：干净环境构建、生成 SBOM、签名安装包、生成更新包、签名更新元数据、安装升级测试。

------

## 39. 性能预算

- **应用启动**：冷启动到主窗口可操作目标 2~4 秒
- **工作区打开**：读取元数据 <500ms、文件树首屏 <1 秒
- **搜索**：精确文件与符号 <100ms、全文搜索首批 <300ms、混合检索 <1 秒
- **AI**：检索状态反馈立即展示、首 Token 延迟由模型决定但必须可观测
- **前端**：滚动保持流畅、日志和结果列表必须虚拟化、Code Graph 默认只加载局部节点

## 40. 性能基准场景

- **Small**：1 个仓库、5,000 个文件、500 万字符
- **Medium**：3 个仓库、50,000 个文件、500 MB 可索引文本
- **Large**：大型 Monorepo、200,000 个文件、数百万 Chunk

第一版可以正式支持 Small 与 Medium，Large 作为压力测试和优化方向。

## 41. 性能分析工具

Rust：tracing Span、CPU Profiler、Heap Profiler、Criterion Benchmark、Tokio Task Diagnostics。React：React Profiler、浏览器 Performance、Long Task 监控、Bundle Analyzer。数据库：慢查询记录、EXPLAIN QUERY PLAN。搜索：各 Retriever 耗时、Tantivy/向量/Rerank 时间。

## 42. Rust Release Profile

第一版建议保守配置：opt-level=3、lto="thin"、codegen-units=8、panic="unwind"、strip="debuginfo"。不建议第一天就开启 fat LTO、codegen-units=1、panic=abort。

------

## 43. 数据库迁移策略

使用单向版本迁移。每次迁移包含：迁移版本、SQL、前置条件、后置校验、是否需要备份、预计耗时。启动流程：检测数据库版本 → 创建迁移前备份 → 获取迁移锁 → 运行迁移 → 执行完整性校验 → 记录新版本。迁移失败：保留失败数据库、保留备份、不自动反复运行、进入恢复界面。

## 44. 长时间数据库迁移

大型迁移拆成：启动期必须迁移（只执行快速 Schema 变更）+ 后台数据回填（应用启动后进入兼容模式，后台逐步完成）。业务代码在回填完成前兼容旧数据。

## 45. 索引迁移

索引 Schema 升级时：保留旧 Active Index → 后台建立新版本 → 校验搜索结果 → 原子切换 → 延迟删除旧版本。应用升级后不能直接删除所有索引。

## 46. 配置迁移

配置文件需要 config_version。迁移时：读取旧配置 → 转换为新结构 → 验证 → 原子写入 → 保留旧配置备份。未知字段应尽量保留。

## 47. 工作区备份

- **自动轻量备份**：SQLite、工作区配置、用户笔记、审批和任务记录
- **可选完整备份**：连接器快照、附件、索引、模型配置

不默认备份：原始 Git 仓库、本地模型文件、可重新下载缓存。备份前需检查当前是否存在数据库迁移、ChangeSet 应用、Git 写操作。

## 48. 更新渠道

Stable（经过完整升级测试）、Beta（提前获得新能力）、Nightly（面向开发者，不保证数据格式长期兼容）。切换到 Beta 或 Nightly 时明确提示"建议先备份工作区数据"。

## 49. 应用更新流程

检查更新 → 获取更新元数据 → 验证版本与渠道 → 展示更新说明 → 下载更新包 → 验证签名 → 等待安全安装时机 → 刷新任务状态与数据库 → 退出并安装 → 重启 → 运行迁移 → 启动后健康检查。

## 50. 安全安装时机

存在以下状态时默认不允许立即重启安装：ChangeSet 正在应用、命令正在执行、Git Commit 正在创建、数据库正在迁移、索引正在切换 Active 版本、插件正在安装。可暂停或等待的任务：普通索引、Embedding、连接器同步、模型下载。

## 51. 更新签名与 Windows 签名

Tauri 更新签名用于验证更新包来源。Windows Code Signing 用于验证可执行文件发布者身份。发布密钥必须不进入 Git、不进入普通构建日志、由 CI Secret 管理。Tauri 更新私钥一旦丢失，已安装版本将无法验证后续更新。

## 52. 更新回滚

索引最容易回滚（重新激活旧 Index Manifest）。数据库迁移通常不能简单反向执行，推荐升级前数据库快照 + 迁移完成后健康检查。升级后启动失败则进入恢复程序。

## 53. 构建信息

应用内"关于"页面显示：App Version、Git Commit、Build Time、Release Channel、Rust Version、Tauri Version、Frontend Build ID、Database Schema Version、Index Schema Version、Plugin API Version。

诊断信息中还可以记录：Debug/Release、Target Triple、Architecture、Installer Type。

## 54. 发布版本规则

使用语义化版本 MAJOR.MINOR.PATCH。同时记录内部格式版本：database_schema_version、workspace_format_version、index_schema_version、plugin_api_version、ipc_protocol_version。

## 55. 软件供应链

发布流程生成：依赖锁文件、SBOM、安装包哈希、更新包哈希、签名、发布说明、许可证清单。CI 中检查：Cargo.lock/pnpm-lock.yaml 变化、已知漏洞、许可证是否允许发布。

## 56. 发布门槛

正式版本必须通过：全部单元测试、数据库升级测试、索引升级测试、Windows E2E、安装包安装测试、旧版本升级测试、更新签名验证、安全路径测试、审批重放测试、诊断包脱敏测试、性能回归阈值。

------

## 57. 完整故障处理时序

```text
底层操作失败
   ↓
转换为模块错误
   ↓
添加工作区、任务和关联 ID
   ↓
判断可恢复性
   ↓
记录结构化日志与 Span
   ↓
更新组件健康状态
   ↓
任务进入 Retry / Failed / Interrupted
   ↓
React 接收领域事件
   ↓
显示用户错误与恢复操作
   ↓
必要时生成诊断包
```

------

## 58. 完整发布升级时序

```text
CI 构建正式版本
   ↓
运行全部发布门槛
   ↓
生成安装包、SBOM 和更新包
   ↓
Windows Code Signing
   ↓
Tauri Updater Signing
   ↓
发布更新元数据
   ↓
客户端检查更新
   ↓
用户确认下载
   ↓
验证更新签名
   ↓
等待关键任务结束
   ↓
创建数据库与配置备份
   ↓
安装并重启
   ↓
运行快速 Schema Migration
   ↓
启动核心服务
   ↓
后台建立新索引版本
   ↓
执行升级健康检查
   ↓
标记升级完成
```

------

## 59. 第一版实现范围

必须完成：统一错误协议、稳定错误代码、Correlation ID、结构化 tracing 日志、日志滚动和清理、Secret Redaction、React Error Boundary、本地诊断中心、诊断包导出、组件健康检查、核心性能指标、Domain/Application/Repository 测试、Provider 和 Connector 契约测试、路径与审批安全测试、Fixture 索引测试、Windows E2E、SQLite 迁移、索引版本迁移、Stable/Beta 更新渠道、Tauri 签名更新、Windows 签名发布流程、安全更新重启。

暂缓：默认上传遥测、默认上传崩溃报告、自动收集源码和 Prompt、升级过程中强制终止用户任务、无法验证签名的旁路更新。

核心原则：错误必须可定位、故障必须可隔离、数据必须可恢复、更新必须可验证、性能必须可度量。
