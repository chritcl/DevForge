# 搜索、RAG、上下文构建与引用机制

## 1. 设计目标

DevForge 的搜索与 AI 问答必须解决四个问题：

1. **找得到**：文件名、符号名、自然语言描述都能召回正确内容
2. **排得准**：真正相关的代码和文档排在前面
3. **看得懂**：模型获得完整但不过量的项目上下文
4. **可验证**：回答中的关键结论能够定位到具体文件、代码行、Commit、Issue 或 PR

完整链路：

```text
用户问题 → 查询理解 → 权限与工作区范围确定 → 多通道候选召回
→ 结果融合 → 精排与去重 → 关系扩展 → 证据质量评估
→ 上下文预算分配 → 敏感信息与指令注入过滤 → 模型生成
→ 引用校验 → 回答与证据展示
```

------

## 2. 搜索请求模型

所有搜索请求统一转换为 SearchRequest：query、workspace_ids、mode(Auto/Keyword/Semantic/Symbol/Graph/GitHistory/Exact)、filters、scope(CurrentFile/CurrentRepository/CurrentWorkspace/SelectedWorkspaces/AllWorkspaces)、limit、include_related、conversation_context。

SearchFilters 支持：文件类型、编程语言、仓库、目录、Git 分支、数据源、时间范围、符号类型、Issue/PR 状态、是否包含生成内容、证据可信等级。

------

## 3. 查询理解

在真正搜索前，系统先生成结构化查询计划。例如"登录失败后，系统在哪里记录日志？"解析为：intent=code_behavior、entities=[登录/失败/日志]、probable_symbols=[login/authenticate/signin/logger]、retrieval_channels=[lexical/semantic/symbol/graph]。

查询类型：
- **导航型**：UserService 在哪里？→ 主要使用精确文件名、路径、符号索引
- **解释型**：这个项目如何刷新 Token？→ 主要使用语义搜索、符号关系、调用链
- **影响分析型**：修改 UserDto 会影响哪里？→ 主要使用引用关系、类型使用、Import
- **历史原因型**：为什么这里不再使用 Redis？→ 主要使用 Commit、PR、Issue、Blame
- **修改型**：帮我给接口增加分页 → 先执行检索与影响分析，再创建 ChangeSession

------

## 4. 查询重写

对连续对话中的问题，需要将其转成可以独立检索的查询。例如上一轮"解释 UserService 的登录逻辑"，当前问题"它失败后会怎样"重写为"UserService 登录流程失败后的错误处理、日志记录和返回结果"。

只有复杂问题才生成多个检索变体（最多 3~4 个）。多查询并不应该默认开启。DevForge 采用：简单查询单查询、复杂查询有限扩展、低召回查询二次扩展。

------

## 5. 多通道候选召回

系统并行执行多个 Retriever：ExactRetriever、LexicalRetriever(Tantivy)、VectorRetriever、SymbolRetriever、GraphRetriever、GitRetriever、ConversationRetriever。

### 5.1 精确召回

针对完整路径、文件名、符号名、Qualified Name、Issue 编号、PR 编号、Commit SHA、API 路径、配置键、数据库表名。精确命中应拥有较高优先级，但不能直接视为最终答案。

### 5.2 Tantivy 全文召回

用于关键词、错误信息、日志文本、注释、文档内容、Commit Message、Issue 和 PR、SQL、配置内容。支持短语搜索、前缀搜索、布尔搜索、字段限定、路径限定、语言限定、时间过滤、模糊匹配。代码搜索需要配置专用 Tokenizer（驼峰、下划线分词）。

### 5.3 向量语义召回

用于自然语言与代码表达不一致的场景。向量召回必须先按 workspace_id、embedding_model_id、document_type、language、source_id、is_deleted、active_index_version 过滤，不能先全库搜索再过滤。

### 5.4 符号召回

专门处理函数、方法、类、接口、Trait、Struct、数据库表、接口路由、配置项。同时比较 name、qualified_name、signature、别名、所在路径、所在模块。

### 5.5 Code Graph 召回

从已知实体向外扩展（默认不超过两跳）。支持关系查询：调用者、被调用者、实现类、父类与子类、字段读写、类型使用、API 到 Service、代码到测试、代码到 Commit、Commit 到 PR、PR 到 Issue。

### 5.6 Git 历史召回

搜索对象：Commit Message、Commit Diff、PR 标题与描述、Review Comment、Issue、Blame、分支差异。Git 历史候选必须明确标记时间和版本。

------

## 6. 候选融合

不同检索器的分数不可直接比较，第一版使用基于排名的融合（RRF）：RRF(candidate) = Σ channel_weight / (k + rank_in_channel)。

建议初始权重：精确匹配 2.0、符号搜索 1.8、全文搜索 1.4、向量搜索 1.3、Code Graph 1.2、Git 历史 1.0、对话记忆 0.7。权重根据查询意图调整。

------

## 7. 候选去重与聚合

同一段代码可能从多个通道重复命中。聚合键：document_id + chunk_id、symbol_id、document_id + line_range、content_hash。多通道共同命中的候选可以获得额外置信加成。

------

## 8. 候选精排

融合后先保留约 50~100 个候选，再精排到 10~30 个。精排器分三层：

- **第一层：规则重排**（不调用模型）：当前文件/仓库/分支优先、精确符号优先、已删除/旧版本降权
- **第二层：本地 Reranker**：使用本地 Cross-Encoder 判断相关性，只对前 30~50 个候选运行
- **第三层：任务相关重排**：根据任务类型（代码解释/Bug 分析/影响分析/历史原因）加入专用信号

------

## 9. 证据集合构建

精排结果先构建 EvidenceBundle：primary_evidence（直接支持答案）、supporting_evidence（解释背景）、conflicting_evidence（文档与代码不一致、分支实现不同等）、historical_evidence、unresolved_questions、confidence。

发现冲突时必须单独标记，AI 不应该擅自把冲突内容融合成一个确定结论。

------

## 10. 代码上下文扩展

找到目标符号后，系统按需补充：所属模块、父级类型、函数签名、调用者、直接被调用函数、相关类型定义、配置项、错误类型、测试、相关 Commit。原则：能够解释行为的最小完整上下文，而不是可能相关的最大上下文。

------

## 11. 上下文预算管理

每个模型拥有最大上下文窗口、最大输入预算、保留输出预算、系统指令预算、会话历史预算、检索证据预算、工具结果预算。上下文构建按预算分层，实际预算根据任务动态调整。

------

## 12. 上下文压缩策略

当证据超出预算时按顺序处理：删除重复候选 → 删除低相关候选 → 缩小代码行范围 → 保留签名压缩无关实现 → 将次要文档转换成抽取式摘要 → 合并相邻 Chunk → 对历史讨论生成摘要 → 最后才减少主要证据。不得仅因为预算不足而截断代码中间部分。

------

## 13. Prompt 上下文结构

发送给模型的上下文分为明确区域：[SYSTEM POLICY]、[USER TASK]、[WORKSPACE CONTEXT]、[PRIMARY EVIDENCE]、[SUPPORTING EVIDENCE]、[CONFLICTS]、[CONVERSATION CONTEXT]、[OUTPUT CONTRACT]。每个证据块使用稳定编号（E1、E2），模型引用编号，最终由 Rust 后台转换为真实文件和行号。

------

## 14. 引用机制

回答中的关键结论必须绑定引用。点击引用后：打开文件、跳转到对应行、高亮引用范围、显示引用时的版本、当前内容变化时提示引用已过期。

------

## 15. 引用校验

模型生成完成后先由 CitationValidator 检查：引用编号是否存在、是否属于本次上下文、文件路径是否真实存在、行号是否在合法范围、引用内容是否支持附近的结论、是否引用了旧版本。校验结果：Valid、WeaklySupported、Unsupported、Stale、Missing。

------

## 16. 回答可信等级

- **High**：多个直接证据一致，关键关系由 LSP 确认，引用完整
- **Medium**：主要结论有代码支持，部分关系来自启发式推断
- **Low**：证据较少，依赖命名推断，存在冲突
- **InsufficientEvidence**：直接回答"当前知识库中没有足够证据确定这一点"，不能编造结论

------

## 17. 事实、推测和建议分离

回答结构明确区分：已确认（根据当前代码可以确认的事实）、可能情况（从命名或不完整证据推断）、风险（当前实现可能存在的问题）、建议（AI 提出的改进方案）。

------

## 18. 版本与时间意识

每条证据必须带版本信息。当用户问"登录逻辑现在是怎样的"优先使用当前工作树；当问"这个逻辑之前为什么修改"则同时使用修改前后代码、Commit、PR、Issue。

------

## 19. 对话记忆

三层：短期记忆（当前会话最近几轮原始消息）、会话摘要（对较早内容生成结构化摘要）、工作区知识记忆（只有用户明确保存后才进入长期知识库）。AI 生成且未经确认的内容不能获得高可信等级。

------

## 20. 知识库 Prompt Injection 防护

代码注释、README、Issue、网页快照和第三方文档都属于不可信内容。防护原则：

- **指令与数据隔离**：所有检索内容放在明确的数据边界中
- **内容扫描**：检测忽略系统指令、请求读取密钥、请求上传文件、伪造 Tool Call 等
- **来源风险等级**：本地用户代码 Medium、GitHub Issue High、外部网页 High
- **权限在模型外执行**：即使模型被诱导，仍然不能读取禁止路径、获取凭据、修改文件等，由 Rust Policy Engine 控制

------

## 21. 云模型上下文审查

调用云模型前生成 DisclosureManifest：展示将发送的内容、已移除的敏感信息、预计输入 Token。用户可以选择继续发送、删除某条证据、切换本地模型、仅发送摘要、取消请求。

------

## 22. 搜索与问答缓存

搜索缓存键：workspace_index_version、query_hash、filters_hash、search_mode、reranker_version。AI 回答不直接缓存最终文本，可以缓存检索结果、EvidenceBundle、文档摘要、Embedding、Rerank 结果。

------

## 23. 流式回答

AI 回答分阶段推送：AnalyzingQuery → Searching → Reranking → BuildingContext → WaitingForDisclosureApproval → Generating → ValidatingCitations → Completed。流式正文通过 Tauri Channel 发送，结构化状态使用 Event。用户可以随时取消。

------

## 24. 搜索质量评估

项目内置本地评估集。检索指标：Recall@5/10、MRR、nDCG@10、Exact Symbol Hit Rate、Citation Coverage。回答指标：关键事实覆盖率、无依据断言数量、引用正确率、过期引用率、回答拒绝准确率。每次修改 Chunk 策略、搜索权重、Embedding 模型、Reranker、查询扩展、Prompt 模板都应该运行固定评估集。

------

## 25. 搜索错误与降级

系统按模块降级：向量模型不可用 → 继续使用全文/符号/Code Graph；Reranker 不可用 → 使用 RRF 排名；LSP 未启动 → 使用 Tree-sitter 关系；GitHub 同步失败 → 使用上次快照；检索结果不足 → 不生成确定答案。

------

## 26. 推荐的第一版范围

第一版实现：查询类型识别、当前工作区搜索、Tantivy 全文搜索、嵌入式向量搜索、符号搜索、一跳 Code Graph 扩展、Git Commit 搜索、RRF 融合、本地可选 Reranker、候选去重、Token 预算分配、代码上下文扩展、行级引用、引用编号校验、可信等级、云端内容审查、Prompt Injection 基础扫描、流式回答、检索过程可视化、基础搜索评估集。

第二阶段：多查询扩展、两跳图谱推理、PR/Issue 与代码联合搜索、跨工作区搜索、自动冲突检测、Corrective Retrieval。

暂缓：基于用户行为训练排序模型、自动微调 Embedding、分布式检索、完全自主的 Agentic RAG。

------

## 27. 完整问答时序

```text
用户输入问题 → React 创建 AI Run → Rust 验证工作区与权限
→ Query Analyzer 识别意图 → 生成 Search Plan
→ 全文/向量/符号/图谱/Git 并行召回 → RRF 融合
→ 去重与规则重排 → 可选本地 Reranker 精排
→ 建立 EvidenceBundle → 补充类型/调用者/测试和配置
→ 执行敏感信息与 Prompt Injection 扫描 → 计算 Token 预算
→ 需要时展示云端披露清单 → 调用本地或云端模型
→ 流式返回正文 → 解析引用标记 → Citation Validator 校验
→ 生成可信等级 → 保存回答/证据快照与审计记录
```

核心：在明确的工作区、版本、权限和 Token 预算内，找到足够支持结论的最小证据集合，并让用户能直接验证每一个关键结论。
