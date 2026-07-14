## 第八部分：错误体系、日志、可观测性、测试与发布升级

# 1. 设计目标

DevForge 是一个同时包含数据库、索引、文件监听、Git、外部连接器、本地进程、AI 模型和多窗口 UI 的大型桌面应用。

这一层需要保证：

1. 出错时能够明确定位到模块、工作区、任务和操作。
2. 单个连接器、模型或索引损坏不会拖垮整个应用。
3. 日志默认保存在本地，不无意记录代码、密钥和用户提示词。
4. 用户可以导出脱敏诊断包。
5. 测试覆盖领域规则、索引一致性、安全边界和完整桌面流程。
6. 发布升级不能破坏工作区数据库、索引和未完成的 Change Session。
7. 更新过程不能在 Agent、命令或文件写入进行中强制重启。
8. 所有性能优化都必须有指标和基准，而不是凭感觉修改。

整体结构：

```text
Application Error
   ↓
Error Classification
   ↓
Structured Log / Trace
   ↓
Health State
   ↓
User-facing Recovery Action
   ↓
Optional Diagnostic Export
```

------

# 2. 统一错误体系

Rust 后台需要统一错误协议，但不能只定义一个巨大的 `AppError` 枚举，把所有错误都塞进去。

推荐分层：

```text
DomainError
ApplicationError
InfrastructureError
IntegrationError
SecurityError
UserInputError
```

## DomainError

表示业务规则不允许操作：

```text
InvalidStateTransition
WorkspaceArchived
ChangeSessionNotReviewable
ApprovalAlreadyDecided
AgentRunAlreadyCompleted
IndexVersionNotActive
```

## ApplicationError

表示业务用例无法完成：

```text
WorkspaceCreationFailed
SearchExecutionFailed
ContextBuildFailed
ChangeApplyFailed
ValidationFailed
TaskSchedulingFailed
```

## InfrastructureError

表示本地基础设施问题：

```text
DatabaseUnavailable
DatabaseMigrationFailed
FileReadFailed
FileWriteFailed
IndexCorrupted
DiskSpaceInsufficient
ProcessSpawnFailed
```

## IntegrationError

表示外部服务问题：

```text
ProviderUnavailable
AuthenticationExpired
RateLimited
RemotePermissionDenied
ConnectorProtocolChanged
ModelResponseInvalid
```

## SecurityError

表示操作被安全策略阻止：

```text
PathOutsideWorkspace
SensitiveFileAccessDenied
CommandPolicyDenied
ApprovalExpired
CredentialDisclosureDenied
PluginPermissionDenied
```

## UserInputError

表示用户提供的信息不完整或不合法：

```text
InvalidWorkspaceName
UnsupportedRepository
InvalidModelEndpoint
MalformedSearchQuery
InvalidIgnorePattern
```

------

# 3. 对外错误协议

React 不直接接收 Rust 的 Debug 输出。

统一响应：

```rust
struct AppErrorResponse {
    code: String,
    category: ErrorCategory,
    user_message: String,
    technical_message: Option<String>,
    retryable: bool,
    severity: ErrorSeverity,
    suggested_actions: Vec<SuggestedAction>,
    correlation_id: CorrelationId,
    context: ErrorContext,
}
```

## `code`

稳定的机器可读编码：

```text
DF-WORKSPACE-001
DF-INDEX-014
DF-AI-008
DF-SECURITY-003
DF-GIT-011
DF-TASK-006
```

错误代码一旦公开给前端、日志和诊断工具使用，就不能随意改变含义。

## `user_message`

面向普通用户：

```text
无法打开工作区数据库。
```

## `technical_message`

面向诊断页面：

```text
SQLite returned SQLITE_BUSY while acquiring the migration lock.
```

默认通知中不展示完整技术详情。

## `suggested_actions`

例如：

```text
重试
打开工作区健康检查
释放磁盘空间
重新授权 GitHub
切换其他模型
导出诊断报告
恢复上一个数据库备份
```

------

# 4. Correlation ID

每个用户操作都生成一个关联 ID：

```text
corr_01J...
```

一次操作中的以下内容共享关联 ID：

```text
Tauri Command
Application Use Case
Database Query
Search Request
Model Run
Tool Call
Background Job
Error
UI Notification
```

示例：

```text
用户点击“重新索引”
   ↓
Command correlation_id = corr_123
   ↓
创建 4 个后台 Job
   ↓
其中 Embedding Job 失败
   ↓
错误页面显示 corr_123
```

用户导出诊断包后，开发者可以通过关联 ID 找到完整链路。

------

# 5. 错误严重等级

```text
Debug
Info
Warning
Error
Critical
Security
```

## Warning

系统已经降级，但主要功能仍可使用：

```text
Reranker 不可用
LSP 启动失败
GitHub 达到限流
部分 Chunk 尚未生成向量
```

## Error

当前操作失败，但应用整体仍然可用：

```text
搜索任务失败
模型请求失败
命令启动失败
单个文件索引失败
```

## Critical

核心运行条件受到影响：

```text
SQLite 无法打开
核心迁移失败
数据目录不可写
活动索引和备份索引都损坏
```

## Security

操作因安全策略被拒绝或检测到异常行为：

```text
插件试图访问未授权域名
Agent 请求读取私钥
命令审批哈希不匹配
工作区路径发生符号链接逃逸
```

安全事件不一定代表程序故障，但必须单独记录。

------

# 6. 错误传播规则

底层错误不能在每一层都重复记录。

推荐规则：

```text
基础设施层
→ 添加底层上下文，但不写最终 Error 日志

Application 层
→ 添加业务上下文

Tauri / Job 边界
→ 记录一次最终错误日志
→ 转换为 AppErrorResponse
```

错误链示例：

```text
系统错误：
Access is denied

存储层：
Failed to open workspace index manifest

应用层：
Failed to activate rebuilt lexical index

用户错误：
无法启用新的全文索引，当前工作区仍在使用旧版本。
```

保留错误链，但用户只看到最有用的一层。

------

# 7. 可恢复错误与不可恢复错误

每个错误都需要声明：

```text
retryable
recovery_strategy
safe_to_continue
requires_user_action
```

## 可自动重试

```text
临时网络错误
服务端 503
数据库短暂锁竞争
模型连接瞬时中断
连接器限流后重试
```

## 需要用户操作

```text
认证失效
磁盘空间不足
文件冲突
模型地址错误
审批过期
工作区路径被移动
```

## 不应自动重试

```text
命令执行失败
Patch 基线冲突
Git Commit 部分失败
数据库迁移脚本逻辑错误
敏感文件访问被拒绝
```

------

# 8. Panic 策略

普通错误使用 `Result`，不能依赖 Panic 处理业务失败。

Panic 只代表：

```text
内部不变量被破坏
理论上不可能发生的状态
第三方库发生未捕获严重错误
```

关键后台任务需要设置 Panic 观察边界：

```text
TaskSupervisor
   ↓
启动 Worker
   ↓
捕获 JoinError
   ↓
区分 Cancelled / Panic
   ↓
记录组件故障
   ↓
按策略重启或降级
```

不能简单地在所有地方使用：

```rust
unwrap()
expect()
```

允许使用的场景应主要限制在：

- 测试
- 编译期确定的常量
- 进程启动前无法恢复的核心配置
- 已经由前置检查证明不可能失败的局部不变量

Release 构建是否使用 `panic = "abort"` 需要谨慎。Cargo 支持 `unwind` 和 `abort` 两种策略；测试工具链仍依赖 unwind。对 DevForge 这种需要保存任务状态、刷新日志和处理数据库的桌面应用，第一版建议保持 `unwind`，通过任务边界和崩溃恢复控制影响，而不是为了减小体积直接改成 `abort`。([Rust 文档](https://doc.rust-lang.org/cargo/reference/profiles.html))

------

# 9. 日志架构

Rust 日志建议统一使用：

```text
tracing
tracing-subscriber
```

`tracing` 适合记录结构化、带作用域且能理解异步上下文的诊断数据；`tracing-subscriber` 的 Layer 可以组合多个输出与过滤策略，也支持按层过滤和 JSON 格式输出。([Docs.rs](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/))

日志管线：

```text
tracing events / spans
   ├─ Console Layer
   ├─ Rolling File Layer
   ├─ In-memory Diagnostic Layer
   ├─ Optional OpenTelemetry Layer
   └─ Security Audit Layer
```

------

# 10. 日志事件结构

每条结构化日志建议包含：

```text
timestamp
level
target
message
correlation_id
workspace_id
repository_id
task_id
agent_run_id
change_session_id
connector_id
provider_id
plugin_id
duration_ms
error_code
```

示例：

```json
{
  "level": "INFO",
  "target": "devforge_indexer",
  "message": "document indexed",
  "workspace_id": "ws_123",
  "document_id": "doc_876",
  "task_id": "task_438",
  "duration_ms": 43,
  "chunk_count": 12
}
```

不要把所有字段都拼进字符串：

```text
indexed document ws_123 doc_876 task_438 in 43ms chunks=12
```

结构化字段更适合过滤、聚合和导出诊断。

------

# 11. Span 设计

对重要流程创建 Span：

```text
workspace.create
workspace.open
index.workspace
index.document
search.execute
rag.build_context
model.complete
agent.run
tool.execute
change.apply
command.run
connector.sync
plugin.invoke
```

示例：

```text
agent.run
├─ search.execute
├─ rag.build_context
├─ model.complete
├─ tool.execute.read_file
├─ change.generate
├─ change.apply
└─ command.run
```

Span 中记录：

```text
开始时间
结束时间
结果
关键计数
关联 ID
失败阶段
```

不记录完整代码、完整 Prompt 和密钥。

------

# 12. 日志级别规范

## TRACE

极细粒度内部状态，仅开发调试开启：

```text
每个检索器候选
每个 IPC 分片
每个 Token Stream 事件
```

## DEBUG

开发诊断：

```text
索引任务状态变化
查询计划
缓存命中
调度器决策
```

## INFO

正常业务生命周期：

```text
应用启动
工作区打开
索引完成
同步完成
模型请求完成
任务完成
```

## WARN

可恢复异常或降级：

```text
Reranker 不可用
索引部分缺失
连接器接近限流
LSP 自动重启
```

## ERROR

明确失败：

```text
任务失败
数据库写入失败
模型响应无法解析
Patch 应用失败
```

默认 Release 日志不应开启 TRACE。

------

# 13. 日志过滤

开发环境：

```text
devforge=debug
tantivy=info
sqlx=warn
hyper=warn
```

生产环境：

```text
devforge=info
devforge_security=warn
第三方依赖=warn
```

用户可以在诊断模式中临时提高某个模块的日志等级：

```text
Indexer：DEBUG，持续 30 分钟
GitHub Connector：TRACE，持续 10 分钟
```

到期自动恢复，避免长期生成大量敏感和高容量日志。

------

# 14. 日志文件策略

目录：

```text
logs/
├─ app/
│  ├─ devforge-2026-07-14.jsonl
│  └─ ...
├─ security/
├─ crash/
└─ diagnostic/
```

建议：

```text
按日期滚动
单文件大小限制
总保留天数限制
总磁盘容量限制
压缩旧日志
启动时清理超期日志
```

默认策略示例：

```text
普通日志：保留 14 天
安全日志：保留 30 天
崩溃日志：保留最近 10 份
任务完整输出：按工作区策略保留
```

用户可调整，但不能让日志无限增长。

------

# 15. 日志脱敏

日志记录前通过 `RedactionLayer`。

需要处理：

```text
API Key
Bearer Token
Cookie
Authorization Header
数据库连接字符串
SSH 私钥内容
环境变量
用户主目录
远程仓库私有 URL 中的凭据
Prompt 中的敏感代码
```

示例：

```text
Authorization: Bearer sk-abc123
```

记录为：

```text
Authorization: Bearer [REDACTED]
```

路径可以根据隐私策略记录为：

```text
<USER_HOME>/Projects/project-a
```

AI 请求默认只记录：

```text
输入 Token
上下文项数量
内容哈希
模型
耗时
披露策略结果
```

不记录完整 Prompt 和完整项目代码。

------

# 16. 审计日志与技术日志分离

普通日志用于调试程序。

审计日志用于回答：

```text
谁批准了什么？
AI 读取了哪些文件？
哪些内容被发送到了云端？
执行了什么命令？
哪个插件使用了哪个权限？
```

审计记录必须结构化保存到 SQLite：

```text
actor
action
target
decision
risk_level
approval_id
result
timestamp
```

技术日志被清理后，关键审计事件仍然存在。

------

# 17. 前端日志

React 前端不应该把全部 `console.log` 保留在 Release 构建中。

前端错误捕获分为：

```text
React Error Boundary
Unhandled Promise Rejection
IPC Failure
Stream Decode Failure
Monaco Failure
Rendering Performance Warning
```

前端错误发送给 Rust：

```text
report_frontend_error
```

数据包含：

```text
错误类型
组件位置
路由
当前标签类型
应用版本
前端构建版本
关联 ID
经过脱敏的堆栈
```

不能包含：

- 完整文件内容
- AI 对话全文
- API Token
- 用户剪贴板
- 未经允许的项目路径

------

# 18. Error Boundary

建议设置多层 Error Boundary。

```text
AppBoundary
├─ WorkspaceBoundary
├─ EditorBoundary
├─ AssistantBoundary
├─ GraphBoundary
└─ BottomPanelBoundary
```

如果 Code Graph 渲染失败：

```text
Code Graph 区域显示错误
文件编辑器和 AI 对话继续工作
```

如果 AI 助手发生错误：

```text
右侧助手区域重置
工作区和代码阅读不受影响
```

只有 App Shell 本身无法运行时才显示全屏恢复页面。

------

# 19. 可观测性信号

DevForge 需要三类核心信号：

```text
Logs
Metrics
Traces
```

OpenTelemetry Rust 支持采集日志、指标和 Trace，但截至当前官方文档，这三个主要组件仍标记为 Beta。第一版因此建议把 OpenTelemetry 作为可选导出层，而不是本地诊断能力的唯一依赖。([OpenTelemetry](https://opentelemetry.io/docs/languages/rust/))

默认模式：

```text
结构化本地日志
本地性能指标
本地任务时间线
```

可选模式：

```text
用户主动启用 OpenTelemetry Export
企业自托管 Collector
开发调试 Collector
```

默认不能自动把遥测发送到远程服务。

------

# 20. 核心指标

## 应用指标

```text
启动时间
首屏时间
工作区打开时间
内存占用
CPU 使用率
磁盘占用
异常退出次数
```

## 索引指标

```text
扫描文件数
解析文件数
Chunk 数
每秒解析文件
Embedding 每秒处理量
全文索引提交耗时
索引失败率
索引队列长度
```

## 搜索指标

```text
搜索总耗时
各 Retriever 耗时
候选数量
Rerank 耗时
缓存命中率
结果点击率
零结果率
```

## AI 指标

```text
首 Token 时间
完整生成时间
输入输出 Token
工具调用次数
取消率
模型失败率
引用校验失败率
```

## Agent 指标

```text
任务完成率
人工审批次数
ChangeSet 修订次数
Patch 冲突率
验证通过率
回滚率
平均任务时长
```

## 连接器指标

```text
同步耗时
同步文档数
API 请求数
限流次数
认证失败
游标滞后时间
```

------

# 21. 指标基数控制

不能给指标添加无限增长的标签：

错误：

```text
file_path
query_text
prompt_text
user_message
commit_sha
```

正确：

```text
document_type
language
task_type
provider_type
error_code
workspace_size_bucket
```

工作区 ID、文件路径和查询内容更适合进入带访问控制的日志，而不是高基数指标。

------

# 22. 健康检查模型

每个核心组件实现：

```rust
trait HealthCheck {
    async fn check(&self) -> ComponentHealth;
}
```

组件状态：

```text
Healthy
Degraded
Unavailable
Recovering
Disabled
Unknown
```

检查对象：

```text
SQLite
Tantivy
Vector Store
Task Scheduler
File Watcher
Git
LSP
Model Providers
Connectors
Plugins
Updater
Disk
```

健康检查不能都在固定时间同时执行，避免周期性高峰。

------

# 23. 诊断中心

诊断页面展示：

```text
应用版本
构建信息
操作系统
WebView 版本
Rust Core 状态
数据库版本
索引版本
工作区健康
后台任务
模型状态
连接器状态
插件状态
磁盘使用
最近错误
```

提供操作：

```text
运行快速诊断
运行完整诊断
打开日志目录
导出诊断包
修复索引
重置 UI 布局
验证数据库
检查更新
```

------

# 24. 诊断包

导出格式：

```text
devforge-diagnostic-{time}.zip
├─ manifest.json
├─ system-info.json
├─ app-config-redacted.json
├─ health.json
├─ recent-errors.json
├─ logs/
├─ database-schema.json
├─ index-manifests/
├─ task-summaries/
└─ user-description.txt
```

默认不包含：

```text
源码正文
AI 对话正文
完整 Prompt
凭据
环境变量明文
任务完整输出
数据库内容
```

导出前显示清单，让用户决定是否额外包含：

```text
特定任务日志
特定 AI 请求上下文
特定索引错误文件
```

------

# 25. 崩溃报告

应用异常退出后，下次启动显示：

```text
DevForge 上次运行意外结束。

已检测到：
- 1 个被中断的 AgentRun
- 1 个未完成的索引任务
- 1 个仍保留的 Session Worktree
```

操作：

```text
恢复工作区
查看崩溃详情
导出诊断包
暂时禁用插件启动
安全模式启动
```

崩溃报告默认保存在本地。

发送远程崩溃报告必须：

- 明确征得用户同意
- 展示将发送的内容
- 完成脱敏
- 允许关闭
- 不包含源码和 Prompt

------

# 26. 安全模式

启动时检测连续崩溃：

```text
短时间内连续崩溃 3 次
```

建议进入安全模式：

```text
禁用第三方插件
暂停外部连接器
不自动恢复 AgentRun
不自动启动 LSP
不自动打开上次工作区
使用默认 UI 布局
```

用户可以逐项恢复组件，定位问题来源。

------

# 27. Rust 测试分层

```text
Unit Tests
Domain Tests
Application Tests
Repository Tests
Adapter Contract Tests
Integration Tests
Property Tests
Fuzz Tests
End-to-End Tests
Performance Benchmarks
```

------

# 28. Domain 测试

领域层必须完全不依赖 Tauri、SQLite 和网络。

测试重点：

```text
AgentRun 状态机
ChangeSession 状态机
审批指纹
风险等级
索引版本切换
工作区状态
任务状态转换
隐私策略
```

示例：

```text
AwaitingChangeApproval
→ ApplyingChanges
```

必须要求存在有效批准。

错误转换：

```text
AwaitingChangeApproval
→ Completed
```

必须被拒绝。

------

# 29. Application 测试

使用内存或 Fake Adapter 测试业务用例：

```text
FakeWorkspaceRepository
FakeSearchIndex
FakeModelProvider
FakeCommandExecutor
FakeSecretStore
FakeEventPublisher
```

覆盖：

```text
创建工作区
首次索引
搜索
AI 问答
Change Session
批准变更
执行验证
连接器同步
任务恢复
```

测试重点是业务编排，不测试具体 SQLite 或 HTTP 实现。

------

# 30. Repository 测试

对 SQLite Repository 使用临时数据库。

覆盖：

```text
Migration
CRUD
事务回滚
唯一约束
并发读写
WAL 模式
崩溃恢复标记
JSON 字段兼容
旧版本迁移
```

每个迁移版本都应测试：

```text
空数据库 → 最新版本
上一个正式版本 → 最新版本
具有真实边界数据的旧数据库 → 最新版本
```

------

# 31. Adapter Contract Test

所有相同 Trait 的实现运行同一组契约测试。

例如：

```text
ChatProvider Contract
├─ 正常回答
├─ 流式回答
├─ 取消
├─ 超时
├─ 无效认证
├─ Tool Call
├─ Usage 缺失
└─ Provider 错误映射
```

运行对象：

```text
OpenAIProvider
AnthropicProvider
GeminiProvider
OllamaProvider
CompatibleProvider
```

Connector 也使用相同思路：

```text
增量游标
内容去重
认证失败
限流
分页
删除检测
```

------

# 32. Property-based Testing

适合测试：

```text
路径规范化
审批哈希稳定性
Patch 应用
索引事件合并
查询过滤
任务状态机
```

路径安全测试生成：

```text
..
大小写变化
Unicode
符号链接
UNC
设备路径
Alternate Data Stream
超长路径
```

需要验证最终结果永远不会越过工作区允许根目录。

------

# 33. Fuzz Testing

优先 Fuzz 的入口：

```text
Unified Diff Parser
.gitignore / .devforgeignore Parser
Model Tool Call JSON
Connector Webhook Payload
Document Metadata Parser
Custom Plugin Manifest
Search Query Parser
```

这些入口会接触：

- 模型生成数据
- 第三方服务数据
- 用户文件
- 插件数据

都不能假设输入永远合法。

------

# 34. 索引测试

建立小型固定测试仓库：

```text
fixtures/
├─ typescript-project/
├─ rust-project/
├─ python-project/
├─ mixed-monorepo/
└─ malformed-project/
```

覆盖：

```text
符号提取
Chunk 边界
Import 关系
增量修改
文件重命名
文件删除
分支切换
索引重建
崩溃中断
向量待补全
```

每个 Fixture 保留预期：

```text
documents.json
symbols.json
relations.json
search-cases.json
```

------

# 35. 搜索质量回归测试

建立标准问题集：

```text
问题
预期文件
预期符号
预期答案要点
不应出现的结果
```

每次调整以下内容时运行：

```text
Tokenizer
Chunk 策略
搜索权重
Embedding
Reranker
RRF 参数
Graph 扩展
查询改写
```

构建门槛示例：

```text
Exact Symbol Hit Rate 不得下降
Recall@10 下降不得超过约定阈值
引用正确率必须达到最低线
平均搜索耗时不得明显恶化
```

------

# 36. 安全测试

必须覆盖：

```text
工作区路径逃逸
符号链接逃逸
Junction 逃逸
敏感文件读取
过期审批重放
命令参数变更
插件权限升级
云模型隐私策略绕过
Prompt Injection
日志凭据泄漏
诊断包泄漏
```

安全测试不能只验证 UI 按钮隐藏，必须直接调用 Rust Application API。

------

# 37. React 测试

## 单元与组件测试

```text
Vitest
Testing Library
```

覆盖：

- 状态选择器
- Query Key
- 错误展示
- 引用交互
- 审批卡片
- Diff 状态
- 模型披露清单

## IPC 集成测试

Mock Tauri Command、Event 和 Channel：

```text
流式 AI 回答
命令日志
索引进度
任务取消
审批过期
后端重连
```

Tauri 官方提供 Mock Runtime 用于单元和集成测试，在该模式下不会启动原生 WebView；同时提供基于 WebDriver 的端到端测试能力。([Tauri](https://v2.tauri.app/develop/tests/))

------

# 38. 桌面端到端测试

关键流程：

```text
启动应用
创建工作区
导入 Fixture 仓库
等待基础索引
搜索符号
发起 AI 问答
打开引用
创建 Change Session
批准 Patch
执行测试
回滚任务
重启恢复
```

Tauri 当前支持通过 WebDriverIO 在 Windows、Linux 和 macOS 上进行 Tauri 端到端测试；直接驱动 `tauri-driver` 时主要支持 Windows 和 Linux。Windows 环境需要匹配 Edge 与 Edge Driver 版本，否则测试连接可能异常。([Tauri](https://v2.tauri.app/develop/tests/))

CI 中至少保留：

```text
Windows 主 E2E
Linux 基础构建与 E2E
macOS 构建和关键流程
```

Windows 是产品第一优先平台，因此完整 Agent、Worktree、命令监管和升级流程以 Windows CI 为发布门槛。

------

# 39. CI 流程

Pull Request 流程：

```text
格式化检查
   ↓
Rust Clippy
   ↓
TypeScript 类型检查
   ↓
Rust 单元与集成测试
   ↓
React 单元与组件测试
   ↓
安全规则测试
   ↓
Fixture 索引测试
   ↓
构建桌面应用
   ↓
Windows E2E
```

主分支额外执行：

```text
跨平台构建
搜索质量回归
性能基准
依赖漏洞检查
许可证检查
安装包冒烟测试
数据库升级测试
```

Release Tag 执行：

```text
干净环境构建
生成 SBOM
签名安装包
生成更新包
签名更新元数据
安装升级测试
发布到指定渠道
```

Tauri 的官方文档提供了在 Linux 和 Windows CI 中运行 WebDriver 测试的方式，并建议在桌面 E2E 前先运行 Rust 测试，避免对已经损坏的核心构建继续执行 UI 测试。([Tauri](https://v2.tauri.app/develop/tests/webdriver/ci/))

------

# 40. 性能预算

第一版建议设立可测量目标，而不是硬性承诺所有电脑都达到相同速度。

## 应用启动

```text
冷启动到主窗口可操作：目标 2～4 秒
恢复最近工作区：后台渐进执行
不得等待全部索引和连接器同步完成
```

## 工作区打开

```text
读取元数据：目标小于 500 ms
文件树首屏：目标小于 1 秒
大型目录按需加载
```

## 搜索

```text
精确文件与符号：目标小于 100 ms
全文搜索首批结果：目标小于 300 ms
混合检索：目标小于 1 秒
Rerank：单独显示阶段
```

## AI

```text
检索状态反馈：立即展示
首 Token 延迟：由模型决定，但必须可观测
取消操作：快速停止前端流，并终止可取消后台任务
```

## 前端

```text
滚动保持流畅
单次批量状态更新不阻塞主线程
日志和结果列表必须虚拟化
Code Graph 默认只加载局部节点
```

这些目标需要按：

```text
小工作区
中型工作区
大型 Monorepo
低配置电脑
高配置电脑
```

分别测试。

------

# 41. 性能基准场景

## Small

```text
1 个仓库
5,000 个文件
500 万字符
```

## Medium

```text
3 个仓库
50,000 个文件
500 MB 可索引文本
```

## Large

```text
大型 Monorepo
200,000 个文件
数百万 Chunk
多语言
```

每个场景测量：

```text
首次扫描
增量索引
冷搜索
热搜索
内存
磁盘
启动
工作区切换
取消恢复
```

第一版可以正式支持 Small 与 Medium。

Large 作为压力测试和优化方向，不应在没有测试数据时宣传“无限规模”。

------

# 42. 性能分析工具

Rust：

```text
tracing Span
CPU Profiler
Heap / Allocation Profiler
Criterion Benchmark
Tokio Task Diagnostics
```

React：

```text
React Profiler
浏览器 Performance
Long Task 监控
组件渲染次数
Bundle Analyzer
```

数据库：

```text
慢查询记录
EXPLAIN QUERY PLAN
事务持续时间
WAL 大小
连接池等待时间
```

搜索：

```text
各 Retriever 耗时
Tantivy Query 时间
向量查询时间
Rerank 时间
上下文构建时间
```

------

# 43. Rust Release Profile

第一版建议使用保守 Release 配置：

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 8
panic = "unwind"
strip = "debuginfo"
```

Cargo 文档说明 Thin LTO 相比 Fat LTO 构建时间明显更低，同时仍能获得接近的优化收益；`strip = "debuginfo"` 可以移除调试信息而保留符号，具体选择仍应以崩溃诊断和安装包大小测试结果为准。([Rust 文档](https://doc.rust-lang.org/cargo/reference/profiles.html))

不建议第一天就开启：

```text
fat LTO
codegen-units = 1
panic = abort
彻底 strip symbols
```

这些设置可能显著增加构建时间或降低问题诊断能力。

------

# 44. 数据库迁移策略

数据库 Schema 使用单向版本迁移。

```text
v1
→ v2
→ v3
```

每次迁移包含：

```text
迁移版本
SQL
前置条件
后置校验
是否需要备份
预计耗时
是否允许回滚
```

应用启动流程：

```text
检测数据库版本
   ↓
创建迁移前备份
   ↓
获取迁移锁
   ↓
运行迁移
   ↓
执行完整性校验
   ↓
记录新版本
   ↓
启动应用
```

迁移失败：

```text
保留失败数据库
保留备份
不自动反复运行
进入恢复界面
```

------

# 45. 长时间数据库迁移

大型迁移不能在主窗口无反馈地阻塞。

例如：

```text
为所有 Chunk 补充新字段
迁移大量消息引用
重构 Code Graph 表
```

应拆成：

## 启动期必须迁移

只执行快速 Schema 变更。

## 后台数据回填

应用启动后进入兼容模式，后台逐步完成。

```text
schema_version = 12
data_backfill_version = 9
```

业务代码在回填完成前兼容旧数据。

------

# 46. 索引迁移

全文索引、向量索引和 Code Graph 都属于可重建数据。

当索引 Schema 升级时：

```text
保留旧 Active Index
   ↓
后台建立新版本
   ↓
校验搜索结果
   ↓
原子切换
   ↓
延迟删除旧版本
```

应用升级后不能直接删除所有索引，迫使用户等待数小时。

如果新版本构建失败，继续使用旧索引并进入降级状态。

------

# 47. 配置迁移

配置文件需要：

```text
config_version
```

迁移时：

```text
读取旧配置
   ↓
转换为新结构
   ↓
验证
   ↓
原子写入
   ↓
保留旧配置备份
```

未知字段应尽量保留，避免降级或插件配置被无意删除。

------

# 48. 工作区备份

备份层级：

## 自动轻量备份

```text
SQLite
工作区配置
用户笔记
审批和任务记录
```

## 可选完整备份

```text
连接器快照
附件
索引
模型配置
```

不默认备份：

```text
原始 Git 仓库
本地模型文件
可重新下载缓存
大型任务输出
```

备份前需检查当前是否存在：

```text
数据库迁移
ChangeSet 应用
Git 写操作
```

避免获得不一致快照。

------

# 49. 更新渠道

建议支持：

```text
Stable
Beta
Nightly
```

## Stable

经过完整升级测试和数据迁移测试。

## Beta

提前获得新模型、插件和索引能力。

## Nightly

面向开发者和内部测试，不保证数据格式长期兼容。

普通用户默认 Stable。

切换到 Beta 或 Nightly 时明确提示：

```text
建议先备份工作区数据。
```

------

# 50. 应用更新流程

```text
检查更新
   ↓
获取更新元数据
   ↓
验证版本与渠道
   ↓
展示更新说明
   ↓
下载更新包
   ↓
验证签名
   ↓
等待安全安装时机
   ↓
刷新任务状态与数据库
   ↓
退出并安装
   ↓
重启
   ↓
运行迁移
   ↓
启动后健康检查
```

Tauri Updater 支持静态 JSON 或动态更新服务；其更新包签名校验是强制要求，不能关闭。生产环境更新地址默认要求 TLS，Updater 也支持在下载过程中返回进度。([Tauri](https://v2.tauri.app/plugin/updater/))

------

# 51. 安全安装时机

存在以下状态时，默认不允许立即重启安装：

```text
ChangeSet 正在应用
命令正在执行
Git Commit 正在创建
数据库正在迁移
索引正在切换 Active 版本
插件正在安装
```

可暂停或等待的任务：

```text
普通索引
Embedding
连接器同步
模型下载
```

用户点击“立即更新”时显示：

```text
正在等待 2 个关键操作完成：
- 应用 ChangeSet
- 创建数据库备份
```

------

# 52. 更新签名与 Windows 签名

需要区分两种签名：

## Tauri 更新签名

用于验证下载的更新包来自可信发布者。

## Windows Code Signing

用于验证 Windows 可执行文件和安装包的发布者身份，并减少系统信任警告。

Windows 应用即使不签名也可能运行，但通过浏览器下载时容易触发 SmartScreen 未受信任提示；正式发布版本应配置 Windows 代码签名。([Tauri](https://v2.tauri.app/distribute/sign/windows/))

发布密钥必须：

```text
不进入 Git
不进入普通构建日志
由 CI Secret 或硬件/云端签名服务管理
限制发布环境访问
提供密钥轮换方案
```

Tauri 更新私钥一旦丢失，已安装版本将无法验证后续使用新密钥签发的普通更新，因此必须作为关键发布资产备份。([Tauri](https://v2.tauri.app/plugin/updater/))

------

# 53. 更新回滚

桌面应用更新的回滚需要区分：

```text
应用二进制回滚
数据库回滚
索引回滚
配置回滚
```

索引最容易回滚：

```text
重新激活旧 Index Manifest
```

配置可通过备份恢复。

数据库迁移通常不能简单反向执行，因此推荐：

```text
升级前数据库快照
+
迁移完成后健康检查
```

如果升级后启动失败：

```text
进入恢复程序
提供恢复旧数据库和旧应用版本的指引
```

不能在不知道数据是否兼容时自动把旧程序直接指向新数据库。

------

# 54. 发布版本规则

使用语义化版本：

```text
MAJOR.MINOR.PATCH
```

## PATCH

```text
Bug 修复
性能优化
不破坏数据格式
```

## MINOR

```text
新功能
可向前迁移的数据变化
新插件 API 能力
```

## MAJOR

```text
重大架构变化
插件 API 不兼容
工作区格式不兼容
```

同时记录内部格式版本：

```text
database_schema_version
workspace_format_version
index_schema_version
plugin_api_version
ipc_protocol_version
```

应用版本不能替代这些独立版本。

------

# 55. 构建信息

应用内“关于”页面显示：

```text
App Version
Git Commit
Build Time
Release Channel
Rust Version
Tauri Version
Frontend Build ID
Database Schema Version
Index Schema Version
Plugin API Version
```

诊断信息中还可以记录：

```text
Debug / Release
Target Triple
Architecture
Installer Type
```

------

# 56. 软件供应链

发布流程建议生成：

```text
依赖锁文件
SBOM
安装包哈希
更新包哈希
签名
发布说明
许可证清单
```

CI 中检查：

```text
Cargo.lock 是否变化
pnpm-lock.yaml 是否变化
已知漏洞
被撤回依赖
许可证是否允许发布
重复依赖和体积异常
```

第三方插件包也需要：

```text
Manifest
内容哈希
发布者签名
权限清单
```

------

# 57. 发布门槛

正式版本必须通过：

```text
全部单元测试
数据库升级测试
索引升级测试
Windows E2E
安装包安装测试
旧版本升级测试
更新签名验证
安全路径测试
审批重放测试
诊断包脱敏测试
性能回归阈值
```

禁止因为“只是小修复”跳过迁移和更新测试。

------

# 58. 第一版实现范围

第一版必须完成：

```text
统一错误协议
稳定错误代码
Correlation ID
结构化 tracing 日志
日志滚动和清理
Secret Redaction
React Error Boundary
本地诊断中心
诊断包导出
组件健康检查
核心性能指标
Domain / Application / Repository 测试
Provider 和 Connector 契约测试
路径与审批安全测试
Fixture 索引测试
Windows E2E
SQLite 迁移
索引版本迁移
Stable / Beta 更新渠道
Tauri 签名更新
Windows 签名发布流程
安全更新重启
```

第二阶段实现：

```text
OpenTelemetry 可选导出
企业遥测策略
自动性能回归 Dashboard
Crash Symbol Server
差分更新
后台恢复程序
更完善的数据库恢复工具
插件自动化兼容测试
Large Monorepo 压力平台
```

暂缓：

```text
默认上传遥测
默认上传崩溃报告
自动收集源码和 Prompt
升级过程中强制终止用户任务
无法验证签名的旁路更新
无备份数据库迁移
```

------

# 59. 完整故障处理时序

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

# 60. 完整发布升级时序

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

这一层的核心原则是：

> **错误必须可定位、故障必须可隔离、数据必须可恢复、更新必须可验证、性能必须可度量。**

