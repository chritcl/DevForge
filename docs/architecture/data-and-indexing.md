# 数据模型、索引目录与增量索引机制

## 1. 数据存储原则

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

## 2. 应用数据目录

Windows 默认建议放在：`%LOCALAPPDATA%/DevForge/`

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

每个工作区拥有独立索引目录，便于单独重建、删除、备份、迁移、控制磁盘占用、后续实现工作区加密。

SQLite 可以先采用一个全局数据库。工作区文件量特别大后，再评估是否拆成全局数据库加工作区数据库，不建议第一版直接多数据库化。

------

## 3. SQLite 核心数据模型

### 3.1 工作区

**workspaces**：id, name, description, status(active/indexing/degraded/error/archived), root_path, default_model_profile_id, privacy_policy_id, created_at, updated_at, last_opened_at, archived_at

**workspace_settings**：workspace_id, language_preferences_json, ignore_rules_json, indexing_settings_json, ai_settings_json, git_settings_json, ui_settings_json, updated_at

不建议为每一个小设置单独建表。稳定且需要查询的字段使用列，变化较频繁的配置使用 JSON。

### 3.2 数据源

**sources**：id, workspace_id, source_type(local_git/local_directory/github_repository/gitlab_repository/github_issues/gitlab_issues/wiki), name, status, config_json, sync_cursor, last_synced_at, last_error, created_at, updated_at

**source_credentials**：source_id, credential_ref, created_at, updated_at（只存储系统凭据管理器中的引用，不保存 Token 明文）

### 3.3 文档与文件

**documents**：id, workspace_id, source_id, document_type(source_code/markdown/plain_text/pdf/word/issue/pull_request/commit/wiki/note/generated_summary), logical_path, absolute_path, title, language, mime_type, size_bytes, content_hash, metadata_hash, git_blob_hash, index_status, is_deleted, created_at, updated_at, last_indexed_at

**document_versions**：id, document_id, version_kind, content_hash, source_revision, snapshot_path, created_at

### 3.4 内容块

**chunks**：id, workspace_id, document_id, chunk_type(code_symbol/code_block/document_section/paragraph/issue_body/issue_comment/pull_request_description/commit_message/diff_hunk/conversation_summary), ordinal, start_line, end_line, start_byte, end_byte, content_hash, token_count, language, symbol_id, parent_chunk_id, embedding_status, created_at, updated_at

Chunk 不建议固定按字符数切割。代码优先按函数、类、接口、模块、实现块、SQL 语句切分。文档优先按标题层级、段落、列表、表格、代码块切分。只有单个结构过大时，才按 Token 上限二次切分。

**chunk_contents**：chunk_id, content, normalized_content, summary

------

## 4. Code Graph 数据模型

### 4.1 符号表

**code_symbols**：id, workspace_id, document_id, symbol_kind(module/namespace/class/interface/trait/struct/enum/function/method/constructor/field/property/constant/type_alias/variable/endpoint/database_table/database_column), name, qualified_name, signature, language, visibility, start_line, end_line, parent_symbol_id, content_hash, evidence_level(exact/parsed/heuristic/ai_inferred), metadata_json, created_at, updated_at

### 4.2 关系表

**code_relations**：id, workspace_id, source_symbol_id, target_symbol_id, target_name, relation_type(defines/contains/imports/exports/calls/references/implements/extends/overrides/reads/writes/uses_type/routes_to/depends_on/produces/consumes), source_document_id, target_document_id, evidence_level, confidence, location_json, metadata_json, created_at, updated_at

当暂时无法解析到准确目标时，target_symbol_id = null，后续 LSP 或重新索引时再进行关系解析。

**graph_resolution_queue**：id, workspace_id, relation_id, target_name, language, status, attempt_count, next_retry_at, last_error

------

## 5. AI 对话数据模型

**conversations**：id, workspace_id, title, conversation_type(general_chat/workspace_qa/code_analysis/architecture_analysis/development_plan/change_session), model_profile_id, status, created_at, updated_at, archived_at

**messages**：id, conversation_id, role, content, content_format, model_id, token_usage_json, cost_json, status, created_at

**message_citations**：id, message_id, document_id, chunk_id, symbol_id, location_json, relevance_score, citation_order

**context_snapshots**：id, conversation_id, message_id, search_query, context_manifest_json, content_hash, privacy_result_json, created_at

------

## 6. 变更与审批数据模型

**change_sessions**：id, workspace_id, conversation_id, title, status(planning/generating/awaiting_review/partially_approved/approved/applying/validating/completed/failed/rolled_back/cancelled), base_git_revision, target_branch, checkpoint_id, created_at, updated_at, completed_at

**change_sets**：id, change_session_id, version, summary, risk_level, status, created_at

**file_changes**：id, change_set_id, document_id, path, change_type, base_hash, proposed_hash, patch_text, status(pending/approved/rejected/applied/conflicted/reverted), created_at, updated_at

**change_approvals**：id, change_set_id, file_change_id, approval_scope, decision, reason, created_at

------

## 7. 任务与命令数据模型

**tasks**：id, workspace_id, task_type(index_workspace/index_document/generate_embedding/sync_connector/run_command/apply_patch/run_tests/git_operation/model_download/export_workspace), title, status, progress, priority, parent_task_id, cancellation_requested, started_at, finished_at, created_at

**command_executions**：id, task_id, working_directory, shell_type, command_display, command_payload_encrypted, risk_level, approval_status, exit_code, timeout_seconds, started_at, finished_at

**task_events**：id, task_id, sequence, event_type, payload_json, created_at

任务日志不要无限写入 SQLite。建议 SQLite 保存结构化事件和日志摘要，完整 stdout、stderr 写入工作区 task-logs。

------

## 8. 模型配置数据模型

**model_providers**：id, provider_type, name, base_url, credential_ref, status, config_json, created_at, updated_at

**models**：id, provider_id, model_name, model_type(chat/embedding/rerank/vision/tool_calling), context_window, capabilities_json, pricing_json, enabled, last_checked_at

**model_profiles**：id, name, chat_model_id, fast_model_id, embedding_model_id, rerank_model_id, fallback_model_id, settings_json, created_at, updated_at

------

## 9. 审计数据模型

**audit_events**：id, workspace_id, actor_type, actor_id, action, target_type, target_id, risk_level, result, details_json, created_at

需要审计的操作包括：云模型请求、文件内容发送、文件修改、命令执行、Git 写操作、凭据读取、Connector 同步、用户批准与拒绝、回滚操作。审计记录默认不能由普通 UI 操作删除。

------

## 10. Tantivy 索引设计

每个工作区一个独立索引。建议字段：workspace_id, document_id, chunk_id, source_id, document_type, path, title, symbol_name, qualified_name, language, content, summary, git_branch, git_commit, tags, updated_at

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

搜索权重初步设为：symbol_name 5.0, qualified_name 5.0, title 3.0, path 2.5, summary 1.5, content 1.0。实际权重后续根据测试集调优。

------

## 11. 向量索引设计

每条向量记录至少包含：vector_id, workspace_id, document_id, chunk_id, embedding_model_id, embedding_version, content_hash, vector, metadata

必须记录 embedding_model_id，不同模型产生的向量不能放进同一个逻辑索引中直接比较。

目录设计：

```text
vectors/
├─ {embedding_model_id}/
│  ├─ index/
│  ├─ metadata/
│  └─ manifest.json
```

当用户切换 Embedding 模型时：旧向量索引继续可用 → 新模型建立新的索引版本 → 后台逐步重新生成向量 → 完成后切换活动版本 → 用户确认后删除旧版本。

------

## 12. 索引版本模型

**index_manifests**：id, workspace_id, index_type(lexical/vector/code_graph), engine_version, schema_version, content_version, model_id, status(building/active/stale/failed/retired), document_count, chunk_count, created_at, activated_at

一个工作区每种索引只能有一个 active 版本。新索引应先在临时目录中构建，完成校验后使用原子重命名切换。

------

## 13. 文件发现机制

本地仓库添加后，先执行完整扫描。扫描规则来源：系统默认忽略规则 + .gitignore + .devforgeignore + 工作区自定义规则。

默认忽略：.git, node_modules, target, dist, build, coverage, .idea, .vscode, .next, .nuxt, vendor, 二进制产物, 压缩文件, 超大日志, 密钥文件

支持工作区根目录下的 `.devforgeignore`。文件是否进入索引需要经过：路径过滤、文件类型识别、文件大小限制、二进制检测、敏感文件检查、编码检测。

------

## 14. 初次完整索引流程

```text
创建索引任务 → 加载忽略规则 → 扫描全部文件 → 建立文件清单
→ 并行计算内容哈希 → 提取文本和元数据 → Tree-sitter 解析源码
→ 生成符号与基础关系 → 结构化 Chunk → 写入 SQLite 暂存状态
→ 批量写入 Tantivy → 批量生成 Embedding → 写入向量索引
→ 启动 LSP 语义增强 → 补充 Code Graph → 校验文档数与 Chunk 数
→ 激活索引版本
```

LSP 增强不应该阻塞基础索引完成。用户在基础全文索引完成后就可以使用工作区。

------

## 15. 增量索引机制

### 15.1 文件监听

使用跨平台文件监听库监控 create/modify/rename/remove。文件变化不能立即直接触发完整解析，要先进入事件合并器（300~800ms Debounce 窗口）。

### 15.2 内容哈希判断

处理文件变化时：读取文件元数据 → 计算快速指纹（size + modified_time + 部分内容采样）→ 与数据库比较 → 必要时计算完整内容哈希。最终以内容哈希判断是否需要重建索引。

### 15.3 单文件更新事务

```text
文件变化 → 创建 IndexDocumentTask → 解析新内容
→ 生成新符号、Chunk、关系 → 写入临时数据 → 更新 Tantivy
→ 更新向量索引 → 事务切换 Document Version → 清理旧 Chunk 和旧关系
```

如果向量生成失败：新的全文索引可以正常生效，文档标记为 vector_pending，后台重试 Embedding。索引状态应细分：metadata_status, parse_status, lexical_status, vector_status, graph_status。

------

## 16. Git 感知索引

索引器需要定期检查：HEAD 是否变化、当前分支是否变化、Git Index 是否变化、工作区文件是否变化。分支切换后不要盲目全部重建，通过 Git Diff 找到变化文件，只增量更新。当变化文件超过阈值（如 40%），再退化为完整重新扫描。

------

## 17. GitHub 与 GitLab 增量同步

Connector 保存同步游标（最后更新时间、最后事件 ID、最后 Commit SHA、ETag、分页游标）。同步时：读取 Sync Cursor → 请求增量数据 → 写入本地文档快照 → 比较内容哈希 → 更新变化文档 → 提交新 Sync Cursor。只有本地事务成功后，才更新远程同步游标。

------

## 18. 索引任务队列

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

任务队列支持去重，去重键：task_type + workspace_id + document_id。

------

## 19. 崩溃恢复

应用意外退出后，启动时检查 status = running/building/applying 的任务。

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

## 20. 索引一致性检查

后台可以定期执行轻量检查：SQLite 活跃文档数、Tantivy 文档数、向量记录数、Chunk 数、索引 Manifest。发现异常时先定位问题（缺少全文索引的 Chunk、缺少向量的 Chunk、引用不存在文档的符号等），不要立即全部重建。

提供"知识库健康检查"页面，用户可以执行：修复缺失项、清理缓存、重建某类索引、完整重建、导出诊断报告。

------

## 21. 数据删除策略

删除工作区时提供三种模式：仅从 DevForge 移除 / 删除 DevForge 索引与缓存 / 删除索引、缓存和本地导入副本。绝不能默认删除原始 Git 仓库。

删除文档时：SQLite 标记 is_deleted → 从活跃搜索索引中移除 → 异步删除向量 → 清理 Code Graph 关系 → 保留短期回收窗口 → 后台执行物理清理。

------

## 22. 备份与导出

工作区导出包：workspace.devforge，包含 manifest.json、metadata.db、notes/、connector-snapshots/、conversations/、change-history/、optional-indexes/。默认不打包原始本地 Git 仓库。导出时让用户选择是否包含 AI 对话、云模型请求记录、GitHub/GitLab 快照、索引、附件、是否脱敏。

------

## 23. 推荐的第一版实现边界

第一阶段先实现：单全局 SQLite 数据库、每工作区独立 Tantivy 索引、每 Embedding 模型独立向量索引、本地目录与 Git 仓库增量监听、Tree-sitter 结构化 Chunk、TypeScript/Rust/Python 三种语言、文件内容哈希、索引任务队列、索引崩溃恢复、健康检查与手动重建。

暂缓：数据库按工作区拆分、索引分片、索引加密、分布式同步、团队共享索引、多设备实时同步、超大型 Monorepo 的远程索引节点。
