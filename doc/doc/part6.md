## 第六部分：GitHub/GitLab 连接器、模型 Provider、插件系统与后台调度

# 1. 本部分目标

这一层负责把 DevForge 从“只能处理本地文件的应用”扩展成一个具备持续同步、模型切换和能力扩展的长期运行平台。

整体架构：

```text
External Integrations
├─ GitHub
├─ GitLab
├─ Ollama
├─ LM Studio
├─ OpenAI
├─ Anthropic
├─ Gemini
└─ Custom Compatible API
          │
          ▼
Connector / Provider Adapters
          │
          ▼
Capability Gateway
          │
          ▼
Persistent Job Scheduler
          │
          ▼
Index / Search / AI / Agent Core
```

核心原则：

1. 外部服务不能直接影响领域层。
2. 每个连接器和模型 Provider 都必须声明能力。
3. 同步任务必须可重试、可取消、可恢复、可去重。
4. 外部服务故障不能阻止本地知识库使用。
5. 插件不能直接获得文件、网络、凭据和命令权限。
6. 本地模型不能因为失败而偷偷回退到云模型。
7. 所有同步和模型调用都需要完整的状态与审计记录。

------

# 2. 连接器统一模型

所有数据源连接器实现统一接口：

```rust
trait Connector {
    fn descriptor(&self) -> ConnectorDescriptor;

    async fn validate(
        &self,
        context: &ConnectorContext,
    ) -> Result<ConnectorValidation>;

    async fn discover(
        &self,
        context: &ConnectorContext,
    ) -> Result<Vec<DiscoveredResource>>;

    async fn sync(
        &self,
        context: &ConnectorContext,
        cursor: Option<SyncCursor>,
    ) -> Result<SyncBatch>;

    async fn fetch_content(
        &self,
        request: FetchContentRequest,
    ) -> Result<ExternalContent>;
}
```

连接器不直接写数据库或搜索索引。

正确流程：

```text
Connector 获取远程数据
   ↓
返回标准 SyncBatch
   ↓
ConnectorSyncService 校验
   ↓
写入本地快照
   ↓
转换为 Document
   ↓
提交增量索引任务
   ↓
成功后更新 Sync Cursor
```

这样可以避免某个 GitHub Connector 同时承担：

- HTTP 请求
- 数据库写入
- 文档转换
- 索引更新
- UI 通知

------

# 3. ConnectorDescriptor

每个连接器需要声明自身能力：

```text
ConnectorDescriptor
├─ connector_type
├─ display_name
├─ connector_version
├─ supported_resources
├─ authentication_methods
├─ supports_incremental_sync
├─ supports_webhooks
├─ supports_content_fetch
├─ supports_write_actions
├─ required_permissions
└─ configuration_schema
```

例如：

```text
GitHub Connector
├─ Repository
├─ Commit
├─ Pull Request
├─ Review
├─ Issue
├─ Issue Comment
├─ Release
└─ Wiki
```

第一版连接器只实现读取与同步，不实现：

- 创建 Issue
- 回复评论
- 合并 PR
- 修改远程仓库
- 创建 Release

这些属于后续远程写入工具，需要独立审批模型。

------

# 4. GitHub 连接器

## 4.1 第一版认证方式

桌面端建议支持：

```text
OAuth 用户授权
Fine-grained Personal Access Token
GitHub Enterprise Server 自定义地址
```

凭据保存流程：

```text
用户完成授权
   ↓
DevForge 获得 Token
   ↓
写入系统凭据存储
   ↓
SQLite 只保存 credential_ref
   ↓
连接器按需读取
```

后续团队版或同步中继服务可使用 GitHub App。

GitHub App 的 Installation Token 可以按安装范围获取，并进一步限制到指定仓库和权限；该 Token 当前有效期为一小时，因此运行时需要缓存过期时间并自动刷新，不能假设 Token 是永久凭据。([GitHub Docs](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-an-installation-access-token-for-a-github-app))

## 4.2 GitHub 数据范围

第一阶段同步：

```text
Repository Metadata
Default Branch
Branches
Commits
Pull Requests
Pull Request Files
Pull Request Reviews
Issues
Issue Comments
Releases
Repository Languages
```

Wiki 可以作为可选数据源，因为部分仓库没有启用 Wiki。

每一种远程资源都映射成 DevForge 文档：

```text
github/repositories/{repo}
github/commits/{sha}
github/pulls/{number}
github/pulls/{number}/reviews/{id}
github/issues/{number}
github/issues/{number}/comments/{id}
```

## 4.3 GitHub 增量同步

每个资源类型拥有独立游标：

```text
Repository Cursor
├─ last_default_branch_sha
├─ last_commit_time
├─ last_pull_updated_at
├─ last_issue_updated_at
├─ last_release_updated_at
├─ etag
└─ last_full_reconciliation_at
```

不建议只保存一个全局 `last_synced_at`。

原因是：

- Commit 和 Issue 的更新时间规则不同
- PR 评论可能在代码没有变化时更新
- Release 和默认分支没有直接关系
- 部分 API 支持 ETag，部分依赖分页和时间条件
- 删除或权限变化需要定期完整核对

## 4.4 同步流程

```text
加载连接器配置和游标
   ↓
检查认证状态
   ↓
读取仓库元数据
   ↓
检测默认分支与 HEAD
   ↓
并行同步 Commit / PR / Issue
   ↓
标准化远程资源
   ↓
内容哈希去重
   ↓
保存本地快照
   ↓
提交索引任务
   ↓
更新各类游标
```

## 4.5 Webhook 策略

纯本地桌面程序通常没有可以被公网稳定访问的入口，因此第一版以：

```text
定时拉取
手动同步
应用启动后增量同步
仓库打开时优先同步
```

为主。

后续可以增加：

```text
DevForge Cloud Relay
企业自托管 Relay
用户自行配置的公网 Webhook Endpoint
```

Webhook Receiver 必须：

- 验证签名或 Secret
- 验证事件类型和 Action
- 使用 Delivery ID 去重
- 快速返回成功
- 将实际处理放入异步任务队列
- 支持遗漏事件后的重新拉取

GitHub 官方建议只订阅必要事件、使用 Webhook Secret、验证 HTTPS，并在十秒内返回响应；还建议通过 `X-GitHub-Delivery` 识别重复投递。因此 Webhook 只能作为“加速同步的触发器”，不能成为数据一致性的唯一来源。([GitHub Docs](https://docs.github.com/en/webhooks/using-webhooks/best-practices-for-using-webhooks))

------

# 5. GitLab 连接器

## 5.1 多实例支持

GitLab Connector 从第一版就必须支持：

```text
GitLab.com
GitLab Self-Managed
自定义 Base URL
自签名证书策略
不同 GitLab 版本
```

连接器配置：

```text
base_url
api_version
authentication_type
credential_ref
project_ids
group_ids
ssl_policy
sync_settings
```

不能把 API 地址写死为 `gitlab.com`。

## 5.2 认证方式

第一版可支持：

```text
OAuth 2.0 Token
Personal Access Token
Project Access Token
Group Access Token
```

GitLab REST API 官方支持 OAuth、个人访问令牌、项目访问令牌和组访问令牌等认证方式；个人、项目和组令牌推荐通过 `PRIVATE-TOKEN` 请求头发送。([GitLab Docs](https://docs.gitlab.com/api/rest/authentication/))

DevForge 应根据用户连接范围推荐最小权限：

```text
单个项目
→ Project Access Token

多个个人项目
→ OAuth 或 Personal Access Token

整个团队 Group
→ Group Access Token
```

## 5.3 GitLab 同步资源

第一版同步：

```text
Project
Repository Branch
Commit
Merge Request
Merge Request Diff
Merge Request Note
Issue
Issue Note
Release
Wiki Page
Pipeline Summary
```

Pipeline 第一版只同步：

- 状态
- 分支
- Commit
- 开始结束时间
- 失败 Job 摘要

不下载所有 CI 日志，除非用户主动请求。

## 5.4 GitLab Webhook

新版本 GitLab 支持使用 Signing Token 对请求体生成 HMAC-SHA256 签名，并提供 `webhook-id`、时间戳和签名头；官方推荐新 Webhook 使用 Signing Token，而不是只依赖明文 `X-Gitlab-Token`。([GitLab Docs](https://docs.gitlab.com/user/project/integrations/webhooks/))

Receiver 应执行：

```text
验证 webhook-signature
验证 webhook-timestamp 新鲜度
使用 webhook-id 去重
检查 GitLab 实例地址
检查事件类型
写入 Webhook Inbox
立即返回
异步调度增量同步
```

GitLab 的 `webhook-id` 在重试中保持一致，可作为幂等键。([GitLab Docs](https://docs.gitlab.com/user/project/integrations/webhooks/))

由于需要兼容旧版自托管 GitLab，连接器还应支持：

```text
优先验证 Signing Token
旧版本回退到 X-Gitlab-Token
记录当前安全等级
```

UI 中明确显示：

```text
Webhook 安全等级：HMAC 签名
```

或者：

```text
Webhook 安全等级：旧版 Secret Header
建议升级 GitLab 或更换验证方式
```

------

# 6. 远程内容标准化

GitHub 与 GitLab 数据不能使用完全不同的上层模型。

统一模型：

```text
ExternalRepository
ExternalCommit
ExternalChangeRequest
ExternalIssue
ExternalComment
ExternalReview
ExternalRelease
ExternalPipeline
ExternalWikiPage
```

其中：

```text
ExternalChangeRequest
```

统一表示：

```text
GitHub Pull Request
GitLab Merge Request
```

标准字段：

```text
provider
remote_id
canonical_url
repository_ref
title
body
author
state
labels
base_revision
head_revision
created_at
updated_at
closed_at
merged_at
metadata
```

Provider 特有内容放入：

```text
provider_metadata_json
```

避免领域层到处出现：

```rust
if provider == GitHub
else if provider == GitLab
```

------

# 7. 连接器数据一致性

远程同步需要三层保证。

## 7.1 增量游标

快速获取新增和更新内容。

## 7.2 内容哈希

避免重复更新没有变化的文档。

## 7.3 定期 Reconciliation

定期重新获取远程资源清单，用于发现：

- 已删除 Issue
- 仓库权限被取消
- PR 被强制更新
- 分支被删除
- Wiki 被禁用
- 用户 Token 权限变化

推荐：

```text
普通增量同步：15～60 分钟
打开工作区时：快速同步
完整核对：每天或每周
```

具体频率由用户和网络策略决定。

------

# 8. Connector Runtime

```text
ConnectorRuntime
├─ ConnectorRegistry
├─ AuthenticationManager
├─ SyncPlanner
├─ RateLimitManager
├─ WebhookInbox
├─ ContentNormalizer
├─ SnapshotWriter
└─ ConnectorHealthMonitor
```

## RateLimitManager

每个连接器记录：

```text
当前剩余请求额度
重置时间
最近一次限流
服务端 Retry-After
并发请求数
最近错误率
```

遇到限流后：

```text
暂停低优先级同步
保留用户主动请求
等待服务端指定时间
减少并发
显示下一次重试时间
```

禁止无限快速重试。

------

# 9. 模型 Provider 总体架构

模型层不能以“OpenAI SDK 作为所有模型的事实标准”。

建议分成两层：

```text
Provider Protocol Adapter
        ↓
DevForge Model Capability
```

统一能力接口：

```rust
trait ChatProvider;
trait EmbeddingProvider;
trait RerankProvider;
trait ModelDiscoveryProvider;
trait TokenCounter;
```

Chat 接口：

```rust
trait ChatProvider {
    async fn capabilities(
        &self,
        model: &ModelId,
    ) -> Result<ModelCapabilities>;

    async fn complete(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse>;

    async fn stream(
        &self,
        request: ChatRequest,
    ) -> Result<ChatStream>;
}
```

------

# 10. 模型能力声明

不能只根据模型名称猜测能力。

每个模型记录：

```text
ModelCapabilities
├─ chat
├─ streaming
├─ tool_calling
├─ parallel_tool_calling
├─ structured_output
├─ embeddings
├─ reranking
├─ vision
├─ reasoning
├─ context_window
├─ max_output_tokens
├─ supports_system_messages
├─ supports_temperature
├─ supports_seed
└─ supports_usage_reporting
```

能力来源按优先级：

```text
用户手动覆盖
Provider 实时探测
Provider 官方元数据
DevForge 内置模型目录
保守默认值
```

Provider 返回不支持的字段时，应在发送前移除或返回明确错误，不要反复盲目重试。

------

# 11. Provider 实现

## 11.1 OpenAI-Compatible Provider

统一处理：

```text
OpenAI Compatible API
Ollama OpenAI Compatibility
LM Studio Compatible API
部分企业代理服务
其他自托管网关
```

Ollama 当前提供 OpenAI 兼容的 `/v1/chat/completions`、`/v1/responses`、模型查询和 `/v1/embeddings` 等接口，因此可以复用一部分 OpenAI 协议适配逻辑。([Ollama](https://docs.ollama.com/openai))

但必须保留独立能力探测，因为“OpenAI Compatible”不代表：

- 所有字段都支持
- Tool Call 行为完全一致
- Usage 一定返回
- JSON Schema 一定严格
- 流式事件完全一致
- Embedding 维度固定
- Responses API 一定存在

因此配置需要包含：

```text
compatibility_mode
supported_endpoints
custom_headers
request_field_overrides
response_mapping
```

## 11.2 Anthropic Provider

Anthropic Provider 使用其原生 Messages 与 Tool Use 语义。

Anthropic 的 Client Tool 模式是：模型返回结构化 `tool_use`，应用程序实际执行工具，再把 `tool_result` 发回模型。这与 DevForge“模型提出意图、Rust 执行工具”的安全架构一致。([Claude Platform Docs](https://docs.anthropic.com/en/docs/agents-and-tools/tool-use/overview))

适配层负责将：

```text
DevForge ToolDefinition
```

转换成：

```text
Anthropic Tool Schema
```

并将 Tool Use 结果重新转换成 DevForge 统一事件。

## 11.3 OpenAI Provider

使用原生 Provider，而不是永远走 Compatible Adapter。

原因是原生 Provider 可以更准确地支持：

- 流式事件
- 工具调用
- 结构化输出
- Reasoning 信息
- Usage
- Provider 特有错误
- 新接口能力

## 11.4 Gemini Provider

独立处理：

- Content Part
- Tool / Function Call
- Safety Result
- Usage
- 多模态内容
- Provider 错误类型

## 11.5 Ollama Provider

同时支持：

```text
Ollama Native API
Ollama OpenAI-Compatible API
```

Native API 用于：

- 模型发现
- 模型状态
- 模型下载
- 模型删除
- 本地运行状态
- Ollama 特有参数

OpenAI-Compatible API 用于复用聊天和 Embedding 流程。

## 11.6 LM Studio Provider

主要通过本地兼容 API 接入，并额外实现：

```text
服务健康检查
模型列表
当前加载模型
本地地址检测
```

------

# 12. 模型配置层级

```text
Provider
   └─ Model
       └─ Model Preset
           └─ Workspace Model Profile
               └─ AgentRun Override
```

示例：

```text
Provider：Ollama Local
Model：qwen-coder
Preset：代码分析低温度
Workspace Profile：本地隐私模式
AgentRun Override：最大输出 8K
```

避免把所有参数直接塞进工作区设置。

------

# 13. 模型路由

模型路由器根据任务选择模型：

```text
Task Type
├─ Query Rewrite
├─ Fast Summary
├─ Workspace Q&A
├─ Code Analysis
├─ Change Planning
├─ Patch Generation
├─ Embedding
└─ Reranking
```

路由规则：

```text
简单查询改写
→ Fast Model

复杂架构分析
→ Primary Reasoning Model

敏感代码
→ Local Model

Embedding
→ 固定 Embedding Model

候选精排
→ Rerank Model
```

------

# 14. 模型回退策略

回退必须是显式策略：

```text
NoFallback
SameProviderFallback
LocalOnlyFallback
CloudFallbackWithApproval
ConfiguredChain
```

示例：

```text
Primary：本地 Qwen
Fallback 1：另一个本地模型
Fallback 2：云端 Claude，需要确认
```

禁止：

```text
本地模型失败
   ↓
后台自动把代码发送到云端
```

发生本地到云端的边界变化时，必须重新生成 `DisclosureManifest`。

------

# 15. Provider 健康与熔断

每个 Provider 保存：

```text
HealthStatus
├─ Healthy
├─ Degraded
├─ RateLimited
├─ AuthenticationFailed
├─ Unreachable
├─ Misconfigured
└─ Disabled
```

统计：

```text
请求成功率
平均首 Token 时间
平均完整响应时间
最近错误
连续失败次数
限流状态
并发请求数
```

连续失败后进入熔断：

```text
Closed
→ Open
→ HalfOpen
→ Closed
```

Provider 熔断只影响该 Provider，不影响本地检索和其他模型。

------

# 16. 流式事件统一

不同模型返回的流式格式不同，但 React 只消费统一事件：

```text
ModelStreamEvent
├─ RunStarted
├─ TextDelta
├─ ReasoningDelta
├─ ToolCallStarted
├─ ToolCallArgumentsDelta
├─ ToolCallCompleted
├─ UsageUpdated
├─ Warning
├─ RunCompleted
└─ RunFailed
```

所有 Provider Adapter 都转换为该格式。

React 不应该解析 Anthropic、OpenAI 或 Gemini 原始流。

------

# 17. Token 与成本统计

每次模型调用保存：

```text
provider_id
model_id
input_tokens
output_tokens
cache_read_tokens
cache_write_tokens
reasoning_tokens
estimated_cost
reported_cost
latency
first_token_latency
```

成本分为：

```text
ProviderReported
DevForgeEstimated
Unknown
```

本地模型不显示虚假的货币成本，而显示：

```text
运行时间
输入输出 Token
内存占用
可选 GPU 使用
```

------

# 18. 插件扩展方案比较

## 方案 A：Rust 原生动态库

优点：

- 性能高
- 能访问完整 Rust API

缺点：

- ABI 稳定性差
- 插件崩溃可能导致主进程崩溃
- 拥有宿主进程权限
- 跨平台发布复杂
- 不适合第三方不可信插件

第一版不采用。

## 方案 B：独立子进程插件

优点：

- 故障隔离较好
- 可使用任意语言
- 容易实现复杂连接器

缺点：

- 需要管理进程生命周期
- IPC 协议复杂
- 插件包体积可能较大
- 权限仍需操作系统级控制

适合后续高级插件。

## 方案 C：WebAssembly Component 插件

这是推荐方案。

Wasmtime 可以作为 Rust 库嵌入应用并执行 WebAssembly 模块或组件；Component Model 通过明确接口描述组件与宿主之间的能力，WASI 则提供可选择暴露的系统接口。([Wasmtime](https://docs.wasmtime.dev/lang.html))

优点：

- 跨平台
- 接口明确
- 不直接共享宿主内存
- 可控制暴露给插件的宿主能力
- 可设置执行时间和资源限制
- 插件崩溃通常不会直接崩溃主应用

限制：

- 生态仍不如普通原生插件成熟
- 复杂语言运行时可能增大插件体积
- 不适合直接构建复杂 React 页面
- 高性能解析器仍优先内置 Rust 实现

最终采用：

> **内置核心模块 + WASM 扩展插件 + 后续可选子进程插件。**

------

# 19. 插件类型

第一版支持三类插件。

## 19.1 Connector Plugin

连接外部知识源：

```text
Jira
Confluence
自定义文档平台
内部研发平台
数据库元数据服务
```

## 19.2 Parser Plugin

处理新的文档格式：

```text
自定义日志格式
专有配置格式
行业文档
内部 DSL
```

输出统一：

```text
DocumentMetadata
ExtractedText
ChunkHints
Symbols
Relations
```

## 19.3 Read-only Tool Plugin

为 AI 提供新的只读工具：

```text
查询内部 API 文档
读取测试报告
查询构建状态
读取项目指标
```

第一版不允许第三方插件提供任意命令执行工具。

------

# 20. 暂不支持 UI 插件

第一版不允许插件注入任意 React 或 JavaScript 页面。

原因：

- WebView 内代码权限难以控制
- 容易访问应用状态
- 容易伪造审批界面
- UI 版本兼容复杂
- 插件可能读取敏感数据

插件可以提供：

```text
配置 JSON Schema
展示字段定义
图标
Markdown 描述
状态信息
```

React 使用系统内置组件渲染这些内容。

------

# 21. 插件 Manifest

```text
plugin_id
name
version
publisher
description
plugin_type
api_version
entry_component
permissions
configuration_schema
supported_platforms
minimum_devforge_version
signature
content_hash
```

权限示例：

```text
read_connector_config
read_credential_reference
http_request
emit_documents
emit_symbols
read_selected_documents
write_plugin_storage
```

不提供模糊权限：

```text
full_access
system_access
all_files
```

------

# 22. 插件 Host Capability

插件不能直接调用操作系统网络或文件系统，而是通过宿主函数：

```text
host.http.request
host.documents.emit
host.secrets.use_reference
host.plugin_storage.get
host.plugin_storage.set
host.logging.write
host.tasks.report_progress
```

例如插件请求 HTTP：

```text
Plugin
   ↓
HttpRequestSpec
   ↓
PluginPolicyEngine
   ↓
检查域名、方法、大小和凭据
   ↓
DevForge 发出请求
   ↓
过滤响应
   ↓
返回插件
```

Credential 永远不以明文直接交给插件。

可以提供：

```text
credential_ref
```

由宿主在请求阶段注入认证头。

------

# 23. 插件资源限制

每次插件调用限制：

```text
最大内存
最大执行时间
最大返回大小
最大 HTTP 请求数
允许域名
最大日志量
最大文档数量
最大并发
```

插件超限后：

```text
终止当前调用
记录 PluginViolation
增加故障计数
必要时自动禁用
```

WASM 插件不能无限占用后台线程。

------

# 24. 插件生命周期

```text
Discovered
Installed
Disabled
Enabled
Updating
Incompatible
Quarantined
Failed
Removed
```

安装流程：

```text
读取 Manifest
   ↓
校验包哈希
   ↓
校验签名
   ↓
检查 API Version
   ↓
展示权限
   ↓
用户批准安装
   ↓
保存到插件目录
   ↓
初始化插件存储
   ↓
执行健康检查
   ↓
启用
```

更新插件时，权限增加必须重新确认。

------

# 25. 插件目录

```text
plugins/
├─ installed/
│  └─ {plugin_id}/
│     ├─ {version}/
│     │  ├─ manifest.json
│     │  ├─ plugin.wasm
│     │  └─ assets/
│     └─ active-version
├─ storage/
│  └─ {plugin_id}/
├─ cache/
└─ quarantine/
```

插件不能直接读取其他插件的存储。

------

# 26. 持久化任务调度器

后台所有长任务统一进入 `JobScheduler`。

```text
JobScheduler
├─ JobRepository
├─ JobPlanner
├─ Dispatcher
├─ WorkerPools
├─ ResourceGovernor
├─ RetryManager
├─ LeaseManager
├─ DependencyResolver
└─ RecoveryManager
```

不能为每个功能随意创建独立循环和 `tokio::spawn`。

------

# 27. Job 数据模型

```text
jobs
├─ id
├─ workspace_id
├─ job_type
├─ payload_json
├─ status
├─ priority
├─ resource_class
├─ deduplication_key
├─ attempt
├─ max_attempts
├─ scheduled_at
├─ lease_owner
├─ lease_expires_at
├─ heartbeat_at
├─ progress
├─ parent_job_id
├─ created_at
├─ started_at
└─ finished_at
```

状态：

```text
Queued
Scheduled
Blocked
Leased
Running
Pausing
Paused
RetryWaiting
Succeeded
Failed
Cancelled
Interrupted
DeadLetter
```

------

# 28. 任务资源分类

```text
IoLight
IoHeavy
CpuLight
CpuHeavy
Network
Embedding
Gpu
ExternalProcess
DatabaseExclusive
IndexWriter
```

任务示例：

| 任务             | 资源分类          |
| ---------------- | ----------------- |
| 文件扫描         | IoHeavy           |
| Tree-sitter 解析 | CpuHeavy          |
| Tantivy 写入     | IndexWriter       |
| Embedding        | Embedding / Gpu   |
| GitHub 同步      | Network           |
| LSP 启动         | ExternalProcess   |
| 模型下载         | Network + IoHeavy |
| SQLite Migration | DatabaseExclusive |

调度器不是只限制“总并发数”，而是按资源类别限制。

------

# 29. 并发预算

示例默认值：

```text
IoLight             8
IoHeavy             2
CpuLight            CPU 核心数的一半
CpuHeavy            1～2
Network             4
Embedding Local     1
ExternalProcess     2
IndexWriter         每工作区 1
DatabaseExclusive   全局 1
```

用户开始 AI 对话时，可以暂时降低后台索引和 Embedding 优先级，保证交互响应。

------

# 30. 任务优先级

```text
P0：用户当前等待
P1：用户主动操作
P2：当前工作区增量更新
P3：连接器同步
P4：Embedding 补全
P5：LSP 语义增强
P6：摘要与缓存预热
P7：清理与维护
```

例子：

```text
用户打开刚修改的文件
→ P0 索引

GitHub 定时同步
→ P3

旧文档重新生成 Embedding
→ P4

清理三十天前缓存
→ P7
```

------

# 31. 去重与合并

去重键：

```text
job_type
+ workspace_id
+ resource_id
+ relevant_version
```

例如：

```text
index_document:ws_1:doc_28:hash_abc
```

同一文件重复变化时：

```text
旧任务尚未开始
→ 替换为最新版本

旧任务正在运行
→ 标记后续重新检查

旧任务已经完成且哈希相同
→ 不再创建任务
```

------

# 32. Job Lease

任务执行前获得租约：

```text
lease_owner
lease_expires_at
heartbeat_at
```

Worker 定期续约。

应用崩溃后：

```text
Lease 过期
   ↓
RecoveryManager 检查任务类型
   ↓
无副作用任务重新排队
   ↓
有副作用任务标记 Interrupted
```

适合自动重试：

```text
文件扫描
解析
Embedding
远程读取同步
搜索索引构建
```

不适合自动重试：

```text
命令执行
文件变更应用
Git Commit
远程写操作
数据库迁移
```

------

# 33. 重试策略

```text
RetryPolicy
├─ max_attempts
├─ initial_delay
├─ max_delay
├─ backoff_factor
├─ jitter
└─ retryable_errors
```

错误分类：

```text
TransientNetworkError
RateLimited
ServiceUnavailable
AuthenticationFailed
PermissionDenied
InvalidConfiguration
InvalidData
LocalResourceExhausted
PermanentFailure
```

只有可恢复错误进入指数退避。

认证失败应：

```text
暂停连接器
提示重新登录
不反复请求
```

------

# 34. 任务依赖

```text
Workspace Initial Index
├─ Scan Files
├─ Parse Documents
├─ Build Lexical Index
├─ Generate Embeddings
└─ Enhance Code Graph
```

依赖支持：

```text
AllSucceeded
AnySucceeded
Always
```

基础全文索引完成后，即使 Embedding 仍在运行，工作区也可以进入“可搜索”状态。

------

# 35. 进度模型

统一进度不能只返回一个百分比。

```text
JobProgress
├─ phase
├─ current
├─ total
├─ unit
├─ message
├─ estimated_remaining_items
└─ child_progress
```

示例：

```text
阶段：生成向量
进度：1,842 / 6,310 Chunks
当前模型：bge-m3
速度：18 Chunks/s
失败：3
```

对于无法确定总量的任务：

```text
progress_mode = Indeterminate
```

不要显示虚假的 `63%`。

------

# 36. 调度策略与设备状态

Windows 第一版可以根据以下状态调整后台任务：

```text
是否使用电池
是否处于节能模式
是否为计量网络
剩余磁盘空间
当前 CPU 使用率
当前内存压力
本地模型是否正在生成
用户是否正在交互
```

策略示例：

```text
电池模式
→ 暂停低优先级 Embedding

磁盘不足
→ 阻止模型下载和完整索引

AI 正在生成
→ 降低 CPU 重型解析并发

计量网络
→ 暂停模型下载和完整远程同步
```

这些策略应可关闭，不能强制替用户做决定。

------

# 37. 故障隔离

## Connector 故障

```text
标记单个 Connector Degraded
保留最后一次快照
暂停其同步
本地搜索继续工作
```

## Model Provider 故障

```text
停止当前模型请求
保留 EvidenceBundle
允许用户切换模型
搜索与索引继续工作
```

## Plugin 故障

```text
终止当前插件调用
记录 Trap
增加失败计数
必要时隔离插件
主应用继续运行
```

## Index 故障

```text
保留当前 Active Index
丢弃 Building Index
允许使用旧索引
后台重新构建
```

## LSP 故障

```text
重启对应 Language Server
降级到 Tree-sitter
不影响基础搜索
```

------

# 38. 运行时健康中心

健康状态按组件展示：

```text
Core Database             Healthy
Tantivy Index             Healthy
Vector Index              Building
GitHub Connector          Rate Limited
GitLab Connector          Authentication Failed
Ollama                    Healthy
Anthropic                 Disabled
TypeScript LSP            Healthy
Rust Analyzer             Restarting
Plugin: Jira              Quarantined
```

每个组件提供：

```text
当前状态
最近成功时间
最近错误
下一次重试
任务数量
诊断详情
修复操作
```

------

# 39. 第一版实现范围

## GitHub / GitLab

第一版实现：

- GitHub OAuth 或 Token
- GitLab OAuth 或 Access Token
- GitHub Enterprise / GitLab Self-Managed 地址
- 仓库发现
- Commit、PR/MR、Issue、评论同步
- 本地快照
- 增量游标
- 内容哈希
- 定时完整核对
- 限流处理
- 连接器健康状态

暂缓：

- 本地直接接收公网 Webhook
- 远程写入
- PR/MR 创建
- 自动评论
- 自动合并

## 模型

第一版实现：

- OpenAI
- Anthropic
- Gemini
- OpenAI-Compatible
- Ollama
- LM Studio
- Chat Streaming
- Tool Calling
- Embedding
- 模型发现
- 能力声明
- Provider 健康检查
- 显式回退策略
- Token 与成本统计

## 插件

第一版实现：

- 内置 Connector Registry
- 插件 Manifest 规范
- WASM Runtime 原型
- Connector Plugin
- Parser Plugin
- Read-only Tool Plugin
- 权限声明
- 资源限制
- 插件隔离和禁用

不实现任意 React UI 插件和第三方命令执行插件。

## 调度器

第一版实现：

- SQLite 持久化任务队列
- 优先级
- 资源分类
- 去重
- 依赖
- Lease 与 Heartbeat
- 指数退避
- 任务取消
- 崩溃恢复
- Dead Letter
- 进度事件
- 设备资源策略

------

# 40. 完整同步时序

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

# 41. 完整模型调用时序

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

这层设计的核心是：

> **连接器只负责获取外部知识，Provider 只负责调用模型，插件只获得明确能力，调度器统一控制所有后台工作；任何一个外部组件失效，都不能破坏本地知识库和核心工作台。**

