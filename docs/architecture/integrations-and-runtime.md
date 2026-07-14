# GitHub/GitLab 连接器、模型 Provider、插件系统与后台调度

## 1. 本部分目标

这一层负责把 DevForge 从"只能处理本地文件的应用"扩展成一个具备持续同步、模型切换和能力扩展的长期运行平台。

核心原则：

1. 外部服务不能直接影响领域层。
2. 每个连接器和模型 Provider 都必须声明能力。
3. 同步任务必须可重试、可取消、可恢复、可去重。
4. 外部服务故障不能阻止本地知识库使用。
5. 插件不能直接获得文件、网络、凭据和命令权限。
6. 本地模型不能因为失败而偷偷回退到云模型。
7. 所有同步和模型调用都需要完整的状态与审计记录。

------

## 2. 连接器统一模型

所有数据源连接器实现统一接口（Connector Trait）：descriptor、validate、discover、sync、fetch_content。连接器不直接写数据库或搜索索引，正确流程：Connector 获取远程数据 → 返回标准 SyncBatch → ConnectorSyncService 校验 → 写入本地快照 → 转换为 Document → 提交增量索引任务 → 成功后更新 Sync Cursor。

## 3. ConnectorDescriptor

每个连接器需要声明自身能力：connector_type、display_name、supported_resources、authentication_methods、supports_incremental_sync、supports_webhooks、supports_content_fetch、supports_write_actions、required_permissions、configuration_schema。

第一版连接器只实现读取与同步，不实现远程写入（创建 Issue、回复评论、合并 PR 等）。

## 4. GitHub 连接器

### 4.1 认证方式

桌面端支持：OAuth 用户授权、Fine-grained Personal Access Token、GitHub Enterprise Server 自定义地址。凭据保存流程：用户完成授权 → DevForge 获得 Token → 写入系统凭据存储 → SQLite 只保存 credential_ref。

### 4.2 数据范围

第一阶段同步：Repository Metadata、Default Branch、Branches、Commits、Pull Requests、PR Files、PR Reviews、Issues、Issue Comments、Releases、Repository Languages。每一种远程资源都映射成 DevForge 文档。

### 4.3 增量同步

每个资源类型拥有独立游标（last_default_branch_sha、last_commit_time、last_pull_updated_at、last_issue_updated_at、last_release_updated_at、etag、last_full_reconciliation_at）。不建议只保存一个全局 last_synced_at。

### 4.4 Webhook 策略

第一版以定时拉取、手动同步、应用启动后增量同步为主。后续可增加 DevForge Cloud Relay。Webhook Receiver 必须验证签名、验证事件类型、使用 Delivery ID 去重、快速返回成功、将实际处理放入异步任务队列。

## 5. GitLab 连接器

### 5.1 多实例支持

从第一版就必须支持：GitLab.com、GitLab Self-Managed、自定义 Base URL、自签名证书策略、不同 GitLab 版本。

### 5.2 认证方式

支持：OAuth 2.0 Token、Personal Access Token、Project Access Token、Group Access Token。根据用户连接范围推荐最小权限。

### 5.3 同步资源

Project、Repository Branch、Commit、Merge Request、MR Diff、MR Note、Issue、Issue Note、Release、Wiki Page、Pipeline Summary。Pipeline 第一版只同步状态、分支、Commit、开始结束时间、失败 Job 摘要。

### 5.4 Webhook

支持 Signing Token（HMAC-SHA256 签名）和旧版 X-Gitlab-Token 回退。

## 6. 远程内容标准化

GitHub 与 GitLab 数据使用统一模型：ExternalRepository、ExternalCommit、ExternalChangeRequest（统一表示 PR 和 MR）、ExternalIssue、ExternalComment、ExternalReview、ExternalRelease、ExternalPipeline、ExternalWikiPage。Provider 特有内容放入 provider_metadata_json。

## 7. 连接器数据一致性

三层保证：增量游标（快速获取新增和更新）→ 内容哈希（避免重复更新）→ 定期 Reconciliation（发现删除和权限变化）。

## 8. Connector Runtime

ConnectorRuntime 包含：ConnectorRegistry、AuthenticationManager、SyncPlanner、RateLimitManager、WebhookInbox、ContentNormalizer、SnapshotWriter、ConnectorHealthMonitor。RateLimitManager 遇到限流后暂停低优先级同步、保留用户主动请求、等待服务端指定时间。

## 9. 模型 Provider 总体架构

模型层分成两层：Provider Protocol Adapter → DevForge Model Capability。统一能力接口：ChatProvider、EmbeddingProvider、RerankProvider、ModelDiscoveryProvider、TokenCounter。

## 10. 模型能力声明

每个模型记录 ModelCapabilities：chat、streaming、tool_calling、parallel_tool_calling、structured_output、embeddings、reranking、vision、reasoning、context_window、max_output_tokens 等。能力来源按优先级：用户手动覆盖 > Provider 实时探测 > Provider 官方元数据 > DevForge 内置模型目录 > 保守默认值。

## 11. Provider 实现

- **OpenAI-Compatible Provider**：统一处理 OpenAI Compatible API、Ollama、LM Studio、企业代理等，但必须保留独立能力探测
- **Anthropic Provider**：使用原生 Messages 与 Tool Use 语义
- **OpenAI Provider**：使用原生 Provider，支持流式事件、工具调用、结构化输出、Reasoning
- **Gemini Provider**：独立处理 Content Part、Tool/Function Call、Safety Result
- **Ollama Provider**：同时支持 Ollama Native API 和 OpenAI-Compatible API
- **LM Studio Provider**：通过本地兼容 API 接入

## 12. 模型配置层级

Provider → Model → Model Preset → Workspace Model Profile → AgentRun Override

## 13. 模型路由

模型路由器根据任务选择模型：简单查询改写 → Fast Model；复杂架构分析 → Primary Reasoning Model；敏感代码 → Local Model；Embedding → 固定 Embedding Model。

## 14. 模型回退策略

回退必须是显式策略：NoFallback、SameProviderFallback、LocalOnlyFallback、CloudFallbackWithApproval、ConfiguredChain。禁止本地模型失败后后台自动把代码发送到云端。

## 15. Provider 健康与熔断

每个 Provider 保存 HealthStatus（Healthy/Degraded/RateLimited/AuthenticationFailed/Unreachable/Misconfigured/Disabled）和统计信息。连续失败后进入熔断（Closed → Open → HalfOpen → Closed）。

## 16. 流式事件统一

不同模型返回的流式格式不同，但 React 只消费统一事件：RunStarted、TextDelta、ReasoningDelta、ToolCallStarted、ToolCallArgumentsDelta、ToolCallCompleted、UsageUpdated、Warning、RunCompleted、RunFailed。

## 17. Token 与成本统计

每次模型调用保存：provider_id、model_id、input_tokens、output_tokens、cache_read_tokens、reasoning_tokens、estimated_cost、latency、first_token_latency。本地模型不显示虚假的货币成本，而显示运行时间、Token 数、内存占用。

## 18. 插件扩展方案

最终采用：**内置核心模块 + WASM 扩展插件 + 后续可选子进程插件**。

WASM 插件优点：跨平台、接口明确、不直接共享宿主内存、可控制暴露能力、可设置资源限制。

## 19. 插件类型

第一版支持三类：Connector Plugin（连接外部知识源）、Parser Plugin（处理新文档格式）、Read-only Tool Plugin（为 AI 提供新只读工具）。第一版不允许第三方插件提供任意命令执行工具。

## 20. 不支持 UI 插件

第一版不允许插件注入任意 React 或 JavaScript 页面。插件可以提供配置 JSON Schema、展示字段定义、图标、Markdown 描述。

## 21. 插件 Manifest

plugin_id、name、version、publisher、description、plugin_type、api_version、entry_component、permissions、configuration_schema、supported_platforms、minimum_devforge_version、signature、content_hash。权限示例：read_connector_config、http_request、emit_documents、write_plugin_storage。不提供模糊权限（full_access、system_access、all_files）。

## 22. 插件 Host Capability

插件通过宿主函数访问外部能力：host.http.request、host.documents.emit、host.secrets.use_reference、host.plugin_storage.get/set、host.logging.write、host.tasks.report_progress。Credential 永远不以明文直接交给插件。

## 23. 插件资源限制

每次插件调用限制：最大内存、最大执行时间、最大返回大小、最大 HTTP 请求数、允许域名、最大日志量、最大文档数量、最大并发。超限后终止当前调用、记录 PluginViolation、增加故障计数、必要时自动禁用。

## 24. 插件生命周期

Discovered → Installed → Disabled → Enabled → Updating → Incompatible → Quarantined → Failed → Removed。安装时校验 Manifest、哈希、签名、API Version、展示权限、用户批准。更新插件时权限增加必须重新确认。

## 25. 持久化任务调度器

后台所有长任务统一进入 JobScheduler：JobRepository、JobPlanner、Dispatcher、WorkerPools、ResourceGovernor、RetryManager、LeaseManager、DependencyResolver、RecoveryManager。不能为每个功能随意创建独立循环和 tokio::spawn。

## 26. Job 数据模型

jobs 表：id、workspace_id、job_type、payload_json、status(Queued/Scheduled/Blocked/Leased/Running/Pausing/Paused/RetryWaiting/Succeeded/Failed/Cancelled/Interrupted/DeadLetter)、priority、resource_class、deduplication_key、attempt、max_attempts、scheduled_at、lease_owner、lease_expires_at、heartbeat_at、progress、parent_job_id、created_at、started_at、finished_at

## 27. 任务资源分类

IoLight、IoHeavy、CpuLight、CpuHeavy、Network、Embedding、Gpu、ExternalProcess、DatabaseExclusive、IndexWriter。调度器按资源类别限制并发，不是只限制总并发数。

## 28. 并发预算

示例默认值：IoLight 8、IoHeavy 2、CpuLight CPU核心数一半、CpuHeavy 1~2、Network 4、Embedding Local 1、ExternalProcess 2、IndexWriter 每工作区 1、DatabaseExclusive 全局 1。

## 29. 任务优先级

P0 用户当前等待 → P1 用户主动操作 → P2 当前工作区增量更新 → P3 连接器同步 → P4 Embedding 补全 → P5 LSP 语义增强 → P6 摘要与缓存预热 → P7 清理与维护

## 30. 去重与合并

去重键：job_type + workspace_id + resource_id + relevant_version。同一文件重复变化时：旧任务尚未开始则替换为最新版本；旧任务正在运行则标记后续重新检查；旧任务已完成且哈希相同则不再创建。

## 31. Job Lease

任务执行前获得租约，Worker 定期续约。应用崩溃后 Lease 过期 → RecoveryManager 检查任务类型 → 无副作用任务重新排队 → 有副作用任务标记 Interrupted。

## 32. 重试策略

RetryPolicy：max_attempts、initial_delay、max_delay、backoff_factor、jitter、retryable_errors。只有可恢复错误进入指数退避。认证失败应暂停连接器、提示重新登录、不反复请求。

## 33. 任务依赖

支持 AllSucceeded、AnySucceeded、Always。基础全文索引完成后，即使 Embedding 仍在运行，工作区也可以进入"可搜索"状态。

## 34. 进度模型

JobProgress：phase、current、total、unit、message、estimated_remaining_items、child_progress。对于无法确定总量的任务，progress_mode = Indeterminate，不要显示虚假的百分比。

## 35. 调度策略与设备状态

Windows 第一版可根据电池、节能模式、计量网络、磁盘空间、CPU 使用率、内存压力、本地模型状态、用户交互状态调整后台任务。策略可关闭。

## 36. 故障隔离

Connector 故障 → 标记 Degraded、保留快照、本地搜索继续。Model Provider 故障 → 停止当前请求、保留 EvidenceBundle、允许切换模型。Plugin 故障 → 终止调用、记录 Trap、必要时隔离。Index 故障 → 保留 Active Index、丢弃 Building Index。LSP 故障 → 重启 Language Server、降级到 Tree-sitter。

## 37. 运行时健康中心

按组件展示健康状态：Core Database、Tantivy Index、Vector Index、GitHub Connector、GitLab Connector、Ollama、Anthropic、TypeScript LSP、Rust Analyzer、Plugin 等。每个组件提供当前状态、最近成功时间、最近错误、下一次重试、任务数量、诊断详情、修复操作。

## 38. 第一版实现范围

**GitHub/GitLab**：OAuth/Token 认证、仓库发现、Commit/PR/Issue/评论同步、本地快照、增量游标、内容哈希、定时核对、限流处理、健康状态。暂缓：Webhook、远程写入、自动评论。

**模型**：OpenAI、Anthropic、Gemini、OpenAI-Compatible、Ollama、LM Studio、Chat Streaming、Tool Calling、Embedding、模型发现、能力声明、Provider 健康检查、显式回退策略、Token 与成本统计。

**插件**：内置 Connector Registry、插件 Manifest 规范、WASM Runtime 原型、Connector/Parser/Read-only Tool Plugin、权限声明、资源限制、隔离和禁用。

**调度器**：SQLite 持久化任务队列、优先级、资源分类、去重、依赖、Lease/Heartbeat、指数退避、任务取消、崩溃恢复、Dead Letter、进度事件、设备资源策略。

------

## 39. 完整同步时序

```text
调度器创建 Connector Sync Job
   ↓
ConnectorRuntime 检查健康状态
   ↓
SecretStore 提供凭据引用
   ↓
Connector 获取远程增量数据
   ↓
RateLimitManager 更新额度
   ↓
标准化为 External Documents
   ↓
比较远程 ID 和内容哈希
   ↓
写入本地快照
   ↓
SQLite 更新 Document Metadata
   ↓
创建索引子任务
   ↓
全文、向量和 Code Graph 更新
   ↓
所有必要事务成功
   ↓
提交 Sync Cursor
   ↓
更新连接器健康状态
```

------

## 40. 完整模型调用时序

```text
AI Application Service 创建 Model Run
   ↓
Model Router 选择 Provider 和 Model
   ↓
检查能力与隐私策略
   ↓
生成 Disclosure Manifest
   ↓
必要时等待用户批准
   ↓
Provider Health 检查
   ↓
Rate Limit 与并发检查
   ↓
Provider Adapter 转换请求
   ↓
流式返回统一 ModelStreamEvent
   ↓
Tool Call 交给 Rust Agent Runtime
   ↓
保存 Usage、成本和审计记录
   ↓
成功或明确进入配置好的回退路径
```

核心：连接器只负责获取外部知识，Provider 只负责调用模型，插件只获得明确能力，调度器统一控制所有后台工作；任何一个外部组件失效，都不能破坏本地知识库和核心工作台。
