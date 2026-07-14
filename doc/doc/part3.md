## 第三部分：数据模型、索引目录与增量索引机制

# 1. 数据存储原则

DevForge 的本地数据分成四类：

| 数据类型       | 存储位置            | 说明                                       |
| -------------- | ------------------- | ------------------------------------------ |
| 业务数据       | SQLite              | 工作区、文件、符号、对话、任务、配置       |
| 全文索引       | Tantivy             | 代码、文档、Issue、PR、Commit 的关键词检索 |
| 向量索引       | 嵌入式 Vector Store | 代码与文档的语义检索                       |
| 大型内容与缓存 | 文件系统            | 原始附件、解析结果、模型缓存、任务日志     |

核心原则：

1. SQLite 是业务元数据的唯一事实来源。
2. Tantivy 和向量索引属于可重建数据。
3. 原始代码仓库不复制，默认只保存路径和索引结果。
4. 外部同步内容需要保存本地快照，保证离线可用。
5. 索引更新必须通过统一事务任务执行。
6. 任何索引都必须带版本号与内容哈希。

------

# 2. 应用数据目录

Windows 默认建议放在：

```text
%LOCALAPPDATA%/DevForge/
```

目录结构：

```text
DevForge/
├─ config/
│  ├─ app.json
│  ├─ models.json
│  └─ ui.json
│
├─ database/
│  ├─ devforge.db
│  ├─ devforge.db-wal
│  └─ backups/
│
├─ workspaces/
│  └─ {workspace_id}/
│     ├─ workspace.json
│     ├─ indexes/
│     │  ├─ lexical/
│     │  ├─ vectors/
│     │  └─ graph/
│     ├─ extracted/
│     ├─ snapshots/
│     ├─ attachments/
│     ├─ checkpoints/
│     ├─ task-logs/
│     └─ cache/
│
├─ models/
│  ├─ embeddings/
│  ├─ rerankers/
│  └─ tokenizers/
│
├─ runtime/
│  ├─ locks/
│  ├─ sockets/
│  └─ pid/
│
├─ logs/
└─ temp/
```

每个工作区拥有独立索引目录，便于：

- 单独重建
- 单独删除
- 单独备份
- 单独迁移
- 控制磁盘占用
- 后续实现工作区加密

SQLite 可以先采用一个全局数据库。工作区文件量特别大后，再评估是否拆成全局数据库加工作区数据库，不建议第一版直接多数据库化。

------

# 3. SQLite 核心数据模型

## 3.1 工作区

### `workspaces`

```text
id
name
description
status
root_path
default_model_profile_id
privacy_policy_id
created_at
updated_at
last_opened_at
archived_at
```

`status`：

```text
active
indexing
degraded
error
archived
```

### `workspace_settings`

```text
workspace_id
language_preferences_json
ignore_rules_json
indexing_settings_json
ai_settings_json
git_settings_json
ui_settings_json
updated_at
```

不建议为每一个小设置单独建表。稳定且需要查询的字段使用列，变化较频繁的配置使用 JSON。

------

## 3.2 数据源

### `sources`

```text
id
workspace_id
source_type
name
status
config_json
sync_cursor
last_synced_at
last_error
created_at
updated_at
```

`source_type`：

```text
local_git
local_directory
github_repository
gitlab_repository
github_issues
gitlab_issues
wiki
```

### `source_credentials`

```text
source_id
credential_ref
created_at
updated_at
```

这里存储的只是系统凭据管理器中的引用，不保存 Token 明文。

------

## 3.3 文档与文件

### `documents`

```text
id
workspace_id
source_id
document_type
logical_path
absolute_path
title
language
mime_type
size_bytes
content_hash
metadata_hash
git_blob_hash
index_status
is_deleted
created_at
updated_at
last_indexed_at
```

`document_type`：

```text
source_code
markdown
plain_text
pdf
word
issue
pull_request
commit
wiki
note
generated_summary
```

`logical_path` 是工作区内部路径。

例如：

```text
backend/src/auth/service.rs
github/issues/128
docs/architecture/auth.md
```

避免前端完全依赖本地绝对路径。

### `document_versions`

保存重要内容的版本记录：

```text
id
document_id
version_kind
content_hash
source_revision
snapshot_path
created_at
```

主要用于：

- GitHub/GitLab 同步内容
- 用户笔记
- AI 生成文档
- 受控变更前检查点

普通 Git 文件不需要每次都复制完整内容，因为 Git 仓库本身已经保存历史。

------

## 3.4 内容块

### `chunks`

```text
id
workspace_id
document_id
chunk_type
ordinal
start_line
end_line
start_byte
end_byte
content_hash
token_count
language
symbol_id
parent_chunk_id
embedding_status
created_at
updated_at
```

`chunk_type`：

```text
code_symbol
code_block
document_section
paragraph
issue_body
issue_comment
pull_request_description
commit_message
diff_hunk
conversation_summary
```

Chunk 不建议固定按字符数切割。

代码优先按：

- 函数
- 类
- 接口
- 模块
- 实现块
- SQL 语句

文档优先按：

- 标题层级
- 段落
- 列表
- 表格
- 代码块

只有单个结构过大时，才按 Token 上限二次切分。

### `chunk_contents`

```text
chunk_id
content
normalized_content
summary
```

内容可以先保存在 SQLite。若后期数据库体积明显膨胀，再把大型正文迁移到压缩文件中。

第一版不要过早做复杂的 Blob 分层。

------

# 4. Code Graph 数据模型

## 4.1 符号表

### `code_symbols`

```text
id
workspace_id
document_id
symbol_kind
name
qualified_name
signature
language
visibility
start_line
end_line
parent_symbol_id
content_hash
evidence_level
metadata_json
created_at
updated_at
```

`symbol_kind`：

```text
module
namespace
class
interface
trait
struct
enum
function
method
constructor
field
property
constant
type_alias
variable
endpoint
database_table
database_column
```

`evidence_level`：

```text
exact
parsed
heuristic
ai_inferred
```

------

## 4.2 关系表

### `code_relations`

```text
id
workspace_id
source_symbol_id
target_symbol_id
target_name
relation_type
source_document_id
target_document_id
evidence_level
confidence
location_json
metadata_json
created_at
updated_at
```

`relation_type`：

```text
defines
contains
imports
exports
calls
references
implements
extends
overrides
reads
writes
uses_type
routes_to
depends_on
produces
consumes
```

当暂时无法解析到准确目标时：

```text
target_symbol_id = null
target_name = "UserService"
```

后续 LSP 或重新索引时再进行关系解析。

### `graph_resolution_queue`

```text
id
workspace_id
relation_id
target_name
language
status
attempt_count
next_retry_at
last_error
```

用于异步处理尚未解析的跨文件关系，避免阻塞基础索引。

------

# 5. AI 对话数据模型

## 5.1 会话

### `conversations`

```text
id
workspace_id
title
conversation_type
model_profile_id
status
created_at
updated_at
archived_at
```

`conversation_type`：

```text
general_chat
workspace_qa
code_analysis
architecture_analysis
development_plan
change_session
```

### `messages`

```text
id
conversation_id
role
content
content_format
model_id
token_usage_json
cost_json
status
created_at
```

### `message_citations`

```text
id
message_id
document_id
chunk_id
symbol_id
location_json
relevance_score
citation_order
```

每条知识型回答必须能够关联到具体来源。

### `context_snapshots`

保存某次模型调用实际使用的上下文：

```text
id
conversation_id
message_id
search_query
context_manifest_json
content_hash
privacy_result_json
created_at
```

这样可以回答：

- 当时 AI 看到了哪些文件？
- 为什么得出这个结论？
- 哪些内容被发送到了云端？
- 后来文件变化后，原回答是否已经过期？

------

# 6. 变更与审批数据模型

## 6.1 变更会话

### `change_sessions`

```text
id
workspace_id
conversation_id
title
status
base_git_revision
target_branch
checkpoint_id
created_at
updated_at
completed_at
```

`status`：

```text
planning
generating
awaiting_review
partially_approved
approved
applying
validating
completed
failed
rolled_back
cancelled
```

### `change_sets`

```text
id
change_session_id
version
summary
risk_level
status
created_at
```

### `file_changes`

```text
id
change_set_id
document_id
path
change_type
base_hash
proposed_hash
patch_text
status
created_at
updated_at
```

`status`：

```text
pending
approved
rejected
applied
conflicted
reverted
```

### `change_approvals`

```text
id
change_set_id
file_change_id
approval_scope
decision
reason
created_at
```

支持：

- 整个 ChangeSet 审批
- 单文件审批
- 单代码块审批

------

# 7. 任务与命令数据模型

### `tasks`

```text
id
workspace_id
task_type
title
status
progress
priority
parent_task_id
cancellation_requested
started_at
finished_at
created_at
```

`task_type`：

```text
index_workspace
index_document
generate_embedding
sync_connector
run_command
apply_patch
run_tests
git_operation
model_download
export_workspace
```

### `command_executions`

```text
id
task_id
working_directory
shell_type
command_display
command_payload_encrypted
risk_level
approval_status
exit_code
timeout_seconds
started_at
finished_at
```

### `task_events`

```text
id
task_id
sequence
event_type
payload_json
created_at
```

任务日志不要无限写入 SQLite。

建议：

- SQLite 保存结构化事件和日志摘要
- 完整 stdout、stderr 写入工作区 `task-logs`
- 数据库保存日志文件路径与偏移信息

------

# 8. 模型配置数据模型

### `model_providers`

```text
id
provider_type
name
base_url
credential_ref
status
config_json
created_at
updated_at
```

### `models`

```text
id
provider_id
model_name
model_type
context_window
capabilities_json
pricing_json
enabled
last_checked_at
```

`model_type`：

```text
chat
embedding
rerank
vision
tool_calling
```

### `model_profiles`

一个 Profile 可以组合多个模型：

```text
id
name
chat_model_id
fast_model_id
embedding_model_id
rerank_model_id
fallback_model_id
settings_json
created_at
updated_at
```

例如：

```text
本地隐私模式
├─ Chat：Qwen 本地模型
├─ Fast：轻量本地模型
├─ Embedding：本地 BGE
└─ Rerank：本地重排模型
```

------

# 9. 审计数据模型

### `audit_events`

```text
id
workspace_id
actor_type
actor_id
action
target_type
target_id
risk_level
result
details_json
created_at
```

需要审计的操作包括：

- 云模型请求
- 文件内容发送
- 文件修改
- 命令执行
- Git 写操作
- 凭据读取
- Connector 同步
- 用户批准与拒绝
- 回滚操作

审计记录默认不能由普通 UI 操作删除。

------

# 10. Tantivy 索引设计

每个工作区一个独立索引。

建议字段：

```text
workspace_id
document_id
chunk_id
source_id
document_type
path
title
symbol_name
qualified_name
language
content
summary
git_branch
git_commit
tags
updated_at
```

字段属性：

| 字段             | 索引方式           |
| ---------------- | ------------------ |
| `content`        | 全文分词           |
| `title`          | 全文分词并提高权重 |
| `path`           | 路径 Tokenizer     |
| `symbol_name`    | 精确值与分词双索引 |
| `qualified_name` | 精确值             |
| `language`       | Facet 或 Keyword   |
| `document_type`  | Facet              |
| `updated_at`     | Fast Field         |

搜索权重可以初步设为：

```text
symbol_name       5.0
qualified_name    5.0
title             3.0
path              2.5
summary           1.5
content           1.0
```

实际权重后续根据测试集调优，不能写死在领域层。

------

# 11. 向量索引设计

每条向量记录至少包含：

```text
vector_id
workspace_id
document_id
chunk_id
embedding_model_id
embedding_version
content_hash
vector
metadata
```

必须记录 `embedding_model_id`。

不同模型产生的向量不能放进同一个逻辑索引中直接比较。

目录可以设计为：

```text
vectors/
├─ {embedding_model_id}/
│  ├─ index/
│  ├─ metadata/
│  └─ manifest.json
```

当用户切换 Embedding 模型时：

1. 旧向量索引继续可用。
2. 新模型建立新的索引版本。
3. 后台逐步重新生成向量。
4. 完成后切换活动版本。
5. 用户确认后删除旧版本。

避免切换模型后整个知识库突然不可搜索。

------

# 12. 索引版本模型

### `index_manifests`

```text
id
workspace_id
index_type
engine_version
schema_version
content_version
model_id
status
document_count
chunk_count
created_at
activated_at
```

`index_type`：

```text
lexical
vector
code_graph
```

`status`：

```text
building
active
stale
failed
retired
```

一个工作区每种索引只能有一个 `active` 版本。

新索引应先在临时目录中构建：

```text
indexes/
├─ lexical-active/
├─ lexical-building-{version}/
├─ vector-active/
└─ vector-building-{version}/
```

完成校验后使用原子重命名切换，避免重建过程中破坏当前可用索引。

------

# 13. 文件发现机制

本地仓库添加后，先执行完整扫描。

扫描规则来源：

```text
系统默认忽略规则
      +
.gitignore
      +
.devforgeignore
      +
工作区自定义规则
```

默认忽略：

```text
.git
node_modules
target
dist
build
coverage
.idea
.vscode
.next
.nuxt
vendor
二进制产物
压缩文件
超大日志
密钥文件
```

建议支持工作区根目录下的 `.devforgeignore`：

```gitignore
generated/
public/assets/
**/*.min.js
**/*.map
private-docs/**
```

文件是否进入索引需要经过：

```text
路径过滤
文件类型识别
文件大小限制
二进制检测
敏感文件检查
编码检测
```

------

# 14. 初次完整索引流程

```text
创建索引任务
   ↓
加载忽略规则
   ↓
扫描全部文件
   ↓
建立文件清单
   ↓
并行计算内容哈希
   ↓
提取文本和元数据
   ↓
Tree-sitter 解析源码
   ↓
生成符号与基础关系
   ↓
结构化 Chunk
   ↓
写入 SQLite 暂存状态
   ↓
批量写入 Tantivy
   ↓
批量生成 Embedding
   ↓
写入向量索引
   ↓
启动 LSP 语义增强
   ↓
补充 Code Graph
   ↓
校验文档数与 Chunk 数
   ↓
激活索引版本
```

LSP 增强不应该阻塞基础索引完成。

索引状态可以分为：

```text
可搜索
语义索引处理中
代码关系增强中
全部完成
```

用户在基础全文索引完成后就可以使用工作区。

------

# 15. 增量索引机制

## 15.1 文件监听

使用跨平台文件监听库监控：

```text
create
modify
rename
remove
```

文件变化不能立即直接触发完整解析，要先进入事件合并器。

例如编辑器保存一个文件时，可能产生：

```text
modify
remove
create
rename
modify
```

合并后只生成一个稳定事件：

```text
DocumentChanged(path)
```

建议设置 300 至 800 毫秒的 Debounce 窗口。

------

## 15.2 内容哈希判断

处理文件变化时：

```text
读取文件元数据
   ↓
计算快速指纹
   ↓
与数据库比较
   ↓
必要时计算完整内容哈希
```

快速指纹可以包含：

```text
size
modified_time
部分内容采样
```

只有快速指纹变化后才计算完整哈希。

最终以内容哈希判断是否需要重建索引，不能只依赖修改时间。

------

## 15.3 单文件更新事务

```text
文件变化
   ↓
创建 IndexDocumentTask
   ↓
解析新内容
   ↓
生成新符号、Chunk、关系
   ↓
写入临时数据
   ↓
更新 Tantivy
   ↓
更新向量索引
   ↓
事务切换 Document Version
   ↓
清理旧 Chunk 和旧关系
```

如果向量生成失败：

- 新的全文索引可以正常生效
- 文档标记为 `vector_pending`
- 后台重试 Embedding
- 不需要回滚整个文档更新

所以索引状态应细分：

```text
metadata_status
parse_status
lexical_status
vector_status
graph_status
```

不能只使用一个笼统的 `indexed = true`。

------

# 16. Git 感知索引

文件系统监听只代表工作区当前状态，但 Git 仓库还有：

- 当前分支
- HEAD Commit
- 未提交修改
- 暂存区
- 分支切换
- Rebase
- Merge

索引器需要定期检查：

```text
HEAD 是否变化
当前分支是否变化
Git Index 是否变化
工作区文件是否变化
```

分支切换后不要盲目全部重建。

可以通过 Git Diff 找到：

```text
added
modified
deleted
renamed
```

只增量更新变化文件。

当变化文件超过阈值，例如工作区的 40%，再退化为完整重新扫描。

------

# 17. GitHub 与 GitLab 增量同步

Connector 保存同步游标：

```text
最后更新时间
最后事件 ID
最后 Commit SHA
ETag
分页游标
```

同步时：

```text
读取 Sync Cursor
   ↓
请求增量数据
   ↓
写入本地文档快照
   ↓
比较内容哈希
   ↓
更新变化文档
   ↓
提交新 Sync Cursor
```

只有本地事务成功后，才更新远程同步游标。

否则可能出现：

> 游标已经前进，但本地数据没有成功保存。

------

# 18. 索引任务队列

索引任务按优先级处理：

| 优先级 | 任务                     |
| ------ | ------------------------ |
| P0     | 用户当前打开文件发生变化 |
| P1     | 用户主动要求重新索引     |
| P2     | 工作区普通文件变化       |
| P3     | GitHub/GitLab 同步       |
| P4     | Embedding 补全           |
| P5     | LSP 关系增强             |
| P6     | 摘要与辅助信息生成       |

任务队列还需要支持去重。

例如同一个文件在短时间内触发五次更新，只保留最后一个任务。

去重键：

```text
task_type + workspace_id + document_id
```

------

# 19. 崩溃恢复

应用意外退出后，启动时检查：

```text
status = running
status = building
status = applying
```

恢复策略：

| 任务类型       | 恢复方式                       |
| -------------- | ------------------------------ |
| 文件索引       | 重新执行                       |
| Embedding      | 从未完成 Chunk 继续            |
| Connector 同步 | 使用旧游标重新同步             |
| 索引重建       | 删除 building 目录后重建       |
| 命令执行       | 标记为 interrupted，不自动重跑 |
| 文件变更应用   | 检查 Checkpoint 后提示用户     |
| Git 写操作     | 检查仓库真实状态后人工处理     |

不能自动重新执行具有副作用的命令。

------

# 20. 索引一致性检查

后台可以定期执行轻量检查：

```text
SQLite 活跃文档数
Tantivy 文档数
向量记录数
Chunk 数
索引 Manifest
```

发现异常时，不要立即全部重建，而是先定位：

```text
缺少全文索引的 Chunk
缺少向量的 Chunk
引用不存在文档的符号
失效的 Code Relation
孤立的附件
```

提供“知识库健康检查”页面：

```text
工作区状态：存在 12 个待修复项目

全文索引：正常
向量索引：8 个 Chunk 待生成
代码图谱：3 条关系目标失效
GitHub 同步：认证过期
磁盘缓存：2.4 GB
```

用户可以执行：

- 修复缺失项
- 清理缓存
- 重建某类索引
- 完整重建
- 导出诊断报告

------

# 21. 数据删除策略

删除工作区时提供三种模式：

```text
仅从 DevForge 移除
删除 DevForge 索引与缓存
删除索引、缓存和本地导入副本
```

绝不能默认删除原始 Git 仓库。

删除文档时：

1. SQLite 标记 `is_deleted`。
2. 从活跃搜索索引中移除。
3. 异步删除向量。
4. 清理 Code Graph 关系。
5. 保留短期回收窗口。
6. 后台执行物理清理。

------

# 22. 备份与导出

工作区导出包：

```text
workspace.devforge
├─ manifest.json
├─ metadata.db
├─ notes/
├─ connector-snapshots/
├─ conversations/
├─ change-history/
└─ optional-indexes/
```

默认不打包原始本地 Git 仓库，只保存仓库路径和远程地址。

导出时让用户选择：

- 是否包含 AI 对话
- 是否包含云模型请求记录
- 是否包含 GitHub/GitLab 快照
- 是否包含索引
- 是否包含附件
- 是否脱敏

全文和向量索引属于可选项，因为它们可以重新生成。

------

# 23. 推荐的第一版实现边界

第一阶段先实现：

- 单全局 SQLite 数据库
- 每工作区独立 Tantivy 索引
- 每 Embedding 模型独立向量索引
- 本地目录与 Git 仓库增量监听
- Tree-sitter 结构化 Chunk
- TypeScript、Rust、Python 三种语言
- 文件内容哈希
- 索引任务队列
- 索引崩溃恢复
- 健康检查与手动重建

暂缓：

- 数据库按工作区拆分
- 索引分片
- 索引加密
- 分布式同步
- 团队共享索引
- 多设备实时同步
- 超大型 Monorepo 的远程索引节点

这套设计足以支持数十个工作区和中大型本地代码仓库，同时保留后续扩展空间。