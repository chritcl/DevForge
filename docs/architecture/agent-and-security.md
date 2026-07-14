# AI Agent、工具调用、变更审批与命令执行

## 1. 核心安全原则

1. AI 只负责提出意图，Rust 后台负责实际执行
2. 默认状态下，AI 只有读取与检索权限
3. 写文件、执行命令、访问网络、修改 Git 都是独立权限
4. AI 不能批准自己提出的操作
5. 批准对象必须是确定的文件、命令和参数，不能批准模糊意图
6. 批准后如果操作内容发生变化，原批准立即失效
7. 所有副作用操作必须进入任务时间线和审计日志
8. 用户随时可以取消任务，但不会自动重新执行有副作用的操作
9. "在临时目录运行"不等于真正的安全沙箱
10. 第一版追求强约束与可回滚，不虚构无法实现的绝对隔离

------

## 2. Agent 运行模型

一次完整的 AI 开发任务定义为 AgentRun：用户目标、工作区、仓库范围、基准版本、模型配置、权限策略、执行环境、任务计划、Tool Call 时间线、Change Session、验证结果、最终报告。

AgentRun 状态：Created → Analyzing → Planning → AwaitingPlanApproval → GatheringContext → ExecutingReadTools → PreparingChanges → AwaitingChangeApproval → ApplyingChanges → AwaitingCommandApproval → RunningCommands → Validating → Completed/PartiallyCompleted/Failed/Cancelled/Interrupted。状态转换必须由 Rust 状态机控制。

------

## 3. Agent 工作模式

- **Ask 模式**（默认）：只允许搜索、阅读、查看符号和 Git 历史、分析代码、生成解释。不能修改文件、执行命令、创建分支
- **Plan 模式**：在 Ask 基础上可生成开发计划、影响范围、文件修改清单、测试计划、风险说明。仍不能实际修改项目
- **Change 模式**：允许创建 Change Session、生成补丁、请求应用补丁、请求执行格式化/检查/测试、请求创建 Git 分支或提交。所有副作用操作必须经过策略检查和人工批准
- **Diagnose 模式**：面向 Bug 排查，允许多轮实验，每个实验仍然是可查看、可取消、可审计的任务

------

## 4. Tool Call 体系

AI 不能直接产生 PowerShell 字符串或操作系统调用，只能提交预定义工具。ToolRegistry：Knowledge Tools、File Tools、Code Intelligence Tools、Git Tools、Change Tools、Command Tools、Model Tools、Connector Tools。每个工具拥有：工具名称、用途说明、参数 Schema、返回值 Schema、所需权限、风险级别、是否需要批准、最大输入/输出、超时、审计策略。

## 5. 只读工具

通常可以自动执行：search_knowledge、search_symbols、search_git_history、get_related_symbols、get_call_hierarchy、find_references、list_directory、read_file、read_file_range、get_git_status、get_git_diff 等。即使是只读工具，也要受路径范围限制。

## 6. 写入与执行工具

Change 工具：create_change_session、propose_file_patch、propose_new_file、propose_file_delete、revise_change_set、apply_approved_change_set、rollback_change_session。

命令工具：propose_command、execute_approved_command、cancel_command、retry_command。

Git 写入工具：create_worktree、create_branch、stage_files、create_commit、restore_files。第一版不向 AI 开放 push、force_push、reset_hard、rebase、merge、delete_remote_branch。

## 7. 结构化 Tool Call

模型提交的不是自由文本命令，而是结构化请求。一个 Tool Call 只能表达一种明确操作。

## 8. Tool Schema 校验

每个请求依次经过：JSON Schema 校验 → 字段长度校验 → 枚举值校验 → 工作区存在性校验 → 路径规范化 → 权限校验 → 风险计算 → 审批状态检查。工具参数不能包含未定义字段、未解析模板变量、模型生成的伪权限、模糊路径、未知工作区。

------

## 9. Policy Engine

所有工具调用由统一的 PolicyEngine 评估。返回结果：Allow、AllowWithConstraints、RequireApproval、Deny。

PolicyContext 包含：当前工作区、当前 AgentRun、当前模式、用户配置、文件权限、命令权限、网络权限、Git 状态、执行环境、此前批准、来源风险、模型可信策略。

------

## 10. 风险等级

- **Level 0 只读**：搜索代码、读取文件、查看 Git Diff。默认自动允许
- **Level 1 低风险写入**：在会话 Worktree 中修改普通源码、创建新测试文件。需要用户批准 ChangeSet
- **Level 2 普通执行**：cargo check、cargo test、pnpm lint。默认需要批准
- **Level 3 高风险**：安装依赖、数据库迁移、删除文件、修改锁文件、创建 Git Commit。必须单独明确批准
- **Level 4 危险操作**：删除大量文件、修改 Git 历史、访问工作区外目录、读取敏感凭据。第一版默认拒绝
- **Level 5 禁止操作**：上传凭据、读取私钥、关闭安全软件、绕过审批、修改审计记录。无论模型如何请求都直接拒绝

------

## 11. 审批模型

审批范围：本次操作、本批次操作、当前 ChangeSet、当前 AgentRun、当前工作区当前会话。第一版不建议提供永久允许任意命令。

审批对象分为文件审批（全部文件/单文件/单 Patch Hunk/新增/删除）和命令审批（精确程序和参数/工作目录/环境变量/网络策略/超时时间）。

每个审批产生 approval_hash = action_type + normalized_arguments + target_hash + policy_version + execution_environment。只要参数、文件基线或执行环境发生变化，批准失效。

------

## 12. 审批界面

文件变更审批页展示：变更目标、AI 修改理由、涉及文件、新增/修改/删除行数、基准版本、当前文件是否已变化、风险等级、关联任务计划、验证计划。

命令审批页展示：程序、参数、工作目录、Shell、网络、超时、预计写入、风险等级。不能只显示"运行测试？"

------

## 13. Tauri 权限边界

React 前端不直接获得通用 Shell 权限。建议 Capability：main-window（允许基础事件/窗口操作/文件选择/自定义 Commands）、review-window（允许读取待审批内容/提交审批决定）、floating-view（只读状态）。不要给所有 windows 配置 shell:allow-execute 和 fs:allow-write。

------

## 14. 文件路径安全

所有路径操作都必须经过 PathGuard：输入逻辑路径 → 拒绝空路径和控制字符 → 与工作区根目录拼接 → 规范化路径 → 解析符号链接与 Junction → 确认最终路径位于允许根目录 → 检查敏感规则 → 执行操作。

需要防止：../、符号链接跳出工作区、Windows Junction 跳出工作区、UNC 网络路径、设备路径、大小写差异绕过、8.3 短路径绕过、Alternate Data Streams。Windows 需要特别检查 \\server\share、\\?\C:\、C:\file.txt:secret、CON/PRN/NUL/COM1。

------

## 15. 文件读取限制

读取文件时还需检查：最大文件大小、是否为二进制、是否为敏感文件、是否为生成产物、是否为压缩包、编码是否有效。默认不允许 AI 读取 .env、*.pem、*.key、id_rsa、credentials.json、secrets.*。

------

## 16. Change Session

一次代码修改必须进入独立的 ChangeSession：用户需求、基准工作区状态、目标仓库、隔离模式、修改计划、ChangeSet 版本、审批记录、应用记录、命令记录、验证结果、回滚状态。同一个 Change Session 可以生成多个 ChangeSet 版本。

------

## 17. 工作区隔离策略

- **模式 A：Git Worktree 隔离**（默认推荐）：为 Change Session 创建独立分支和独立工作目录，不污染用户当前分支，修改天然可比较，易于丢弃
- **模式 B：临时副本**：适用于非 Git 目录，选择目标文件复制到 Session Workspace，在副本中修改
- **模式 C：直接修改当前工作区**：仅作为高级选项，要求明确警告、自动创建检查点、所有目标文件记录原始哈希

------

## 18. 未提交改动处理

创建 Change Session 时，仓库可能存在未暂存/已暂存/未跟踪/冲突文件或正在 Rebase/Merge。系统必须先检测仓库状态。

- **Clean HEAD 模式**：只基于当前 HEAD 创建 Worktree
- **Current Snapshot 模式**：把当前工作区状态构造成会话基线（HEAD + staged diff + unstaged diff + 选择的 untracked files）

如果当前仓库存在冲突、未完成 Rebase 或 Merge，默认禁止创建自动变更会话。

------

## 19. ChangeSet 格式

AI 生成的修改统一保存为 ChangeSet：Base Revision、File Operations（Create/Modify/Rename/Delete）、Unified Diff、Expected File Hashes、Explanation、Risk Assessment、Validation Plan。每个文件修改必须带：目标路径、修改类型、基准内容哈希、目标内容哈希、编码、换行符、Patch、修改理由。

------

## 20. Patch 校验

Patch 应用前依次检查：路径是否合法 → 文件是否属于当前 Session → 基准哈希是否一致 → Patch 是否可解析 → Hunk 上下文是否匹配 → 文件编码是否受支持 → 是否改变换行符 → 是否涉及敏感文件 → 是否产生超大文件 → 是否存在二进制修改。如果基准哈希不同则 ChangeConflict，不能静默尝试"差不多应用"。

------

## 21. 原子文件写入

文件写入采用：生成临时文件 → 写入并 Flush → 验证目标内容哈希 → 保留原文件元数据 → 原子替换目标文件。多文件 ChangeSet 使用 apply_journal 记录进度，如果中途失败则停止后续写入、提供回滚已写入文件、标记为 PartiallyApplied。

------

## 22. 命令执行原则

命令执行器位于 Rust 后台：CommandParser、RiskClassifier、EnvironmentBuilder、ProcessSupervisor、OutputCollector、TimeoutController、ResultRecorder。默认使用 program + args[]，而不是 shell -c "一整段字符串"。

------

## 23. 项目命令发现

系统可以读取 package.json scripts、Cargo.toml、Makefile、justfile、pyproject.toml 等识别候选命令。但"项目中存在这个脚本"不代表它一定安全，首次执行项目脚本时仍要展示实际命令。

------

## 24. 环境变量策略

环境分成：基础环境（PATH、TEMP 等）、工作区环境（用户明确配置）、敏感环境（API Token、云凭据等，默认不传递给 Agent 命令）。命令审批页要显示将传入的普通变量数和已隐藏的敏感变量数。

------

## 25. 命令工作目录

命令只能在 Session Worktree、工作区中被明确允许的子目录、DevForge 临时任务目录中执行。禁止用户主目录、系统目录、其他工作区、DevForge 凭据目录。

------

## 26. 进程监管

运行命令时需要管理整个进程树。Windows 通过 Job Object 管理同一任务产生的一组进程。任务取消时：发送优雅终止 → 等待短暂宽限期 → 终止整个进程树 → 关闭输出 Channel → 记录 cancelled。

------

## 27. 资源限制

命令任务支持：最大运行时间、最大输出大小、最大并发子进程、最大内存建议值、输出速率限制。命令产生海量输出时：完整输出写入日志文件、前端仅保留滑动窗口、数据库保存结构化摘要。

------

## 28. "命令沙箱"的真实边界

第一版的本地命令执行属于"受控执行与进程隔离，不是强安全沙箱"。普通本地进程仍可能读取当前用户可访问的其他文件、访问网络、调用其他系统程序。UI 不能误导用户显示"完全安全沙箱"，应显示"本地受控执行"。

------

## 29. 执行后端

- **Local Controlled Runner**（第一版默认）：工作目录约束、环境变量过滤、进程树监管、超时、日志、审批、风险分类。不提供真正的文件系统和网络隔离
- **Container Runner**（后续）：Docker/Podman/Dev Container
- **Remote Runner**（后续）：企业 CI、远程构建机、GPU 环境

所有 Runner 遵循统一接口 ExecutionBackend。

------

## 30. 网络访问控制

网络权限分为：DeniedByPolicy、Ask、AllowedForDomains、Allowed。工具层网络控制可以严格限制域名、方法、大小、超时。本地命令网络控制只能做到标记命令可能联网、默认不传递凭据、用户明确批准。需要真正禁止网络时应使用 Container Runner。

------

## 31. 命令风险识别

CommandRiskClassifier 综合分析：程序名称、参数、Shell 元字符、工作目录、项目脚本内容、文件系统目标、网络特征、管理员权限、Git 操作、数据库操作、包管理操作。高风险模式：rm -rf、git reset --hard、git push --force、DROP DATABASE、curl 上传等。还需识别 PowerShell AST、Shell 参数结构、脚本文件内容。

------

## 32. 验证流水线

应用 ChangeSet 后按项目配置运行验证：Syntax Check → Formatter Check → Type Check → Lint → Unit Tests → Integration Tests → Build → Custom Checks。不是所有任务都需要运行全部验证。

------

## 33. 验证计划审批

AI 可以提出验证计划，但不能自行扩大范围。用户可以给予临时规则，但仍需限制程序、参数前缀、工作目录、会话、有效时间。

------

## 34. 测试失败处理

测试失败后系统先分类：CompilationFailure、TestAssertionFailure、EnvironmentFailure、DependencyFailure、Timeout、Cancelled、UnrelatedPreExistingFailure、Unknown。非常重要的一点：在修改前尽可能运行一次基线检查，最终报告要区分"由本次修改引入"和"修改前已经存在"。

------

## 35. 检查点机制

每个 Change Session 至少保存：基准 Commit、原始 Git 状态、原始文件哈希、用户未提交 Patch、未跟踪文件清单、ChangeSet、已应用文件、命令记录、验证结果。

- **Git Worktree 模式**：回滚最简单，丢弃 Session Worktree 和 Branch
- **临时副本模式**：使用原始文件快照恢复
- **直接修改模式**：必须逐文件检查当前内容

------

## 36. 回滚类型

Soft Rollback（生成逆向 Patch）、Session Rollback（恢复整个 Session）、File Rollback（只恢复选定文件）、Hunk Rollback（只恢复指定代码块）、Discard Session（删除隔离 Worktree）。

------

## 37. Git 提交策略

验证通过后系统可以生成 Commit 标题、正文、变更摘要、测试结果、关联 Issue，但创建 Commit 仍需要用户批准。第一版不自动 Push。对于多个仓库的工作区，每个仓库独立 ChangeSet、分支、Commit、审批、回滚。

------

## 38. Agent 计划约束

在真正修改前，AI 必须生成结构化计划。计划批准后形成 ChangeBudget：允许仓库、允许目录、预计文件、最大文件数、是否允许新建/删除文件、是否允许修改依赖/数据库。如果 AI 后续发现需要修改计划外文件（PlanDeviation），必须解释原因并请求追加批准。

------

## 39. 防止 Agent 无限循环

AgentRun 设置：最大 Tool Call 数（80）、最大模型调用次数、最大失败重试次数（3）、最大运行时间（60 分钟）、最大 Token 消耗、最大命令数（10）、最大 ChangeSet 版本数。达到限制后进入 PausedForReview。

------

## 40. 防止重复和无效操作

运行时记录工具名称、参数哈希、目标状态哈希、结果摘要。如果 AI 在项目没有变化的情况下反复调用相同操作，系统返回 DuplicateAction 并提示参考已有结果。

------

## 41. Tool Call 来源可信度

只有用户明确请求、系统策略允许、Agent 计划范围内才能成为执行依据。项目文件中的恶意指令只能被当作不可信项目内容，不能成为授权来源。

------

## 42. Agent 错误分类

统一错误类型：ToolValidationError、PolicyDenied、ApprovalExpired、PathOutsideWorkspace、SensitiveFileDenied、PatchConflict、CommandRejected、ProcessFailed、ProcessTimedOut、ProcessCancelled、GitStateConflict、ModelFailure、ContextStale、ExecutionInterrupted、RollbackConflict。每个错误包含用户可读说明、技术详情、是否可重试、建议操作。

------

## 43. 崩溃恢复

应用重启后检查 AgentRun=Running、ChangeSession=Applying、CommandTask=Running。只读任务可重新执行；模型流标记为 Interrupted；命令任务确认进程是否仍存在；Patch 应用中断读取 apply_journal 提供继续或回滚。

------

## 44. Agent 任务时间线

React 页面展示完整时间线：用户创建任务 → AI 分析工作区 → 搜索相关符号 → 读取文件 → 生成修改计划 → 用户批准 → 创建 Worktree → 生成 ChangeSet → 用户审批 → 应用文件 → 请求测试 → 用户批准 → 测试结果 → 修复 Patch → 验证通过。每个节点可展开查看输入、输出、模型、Token、文件、命令、审批人、错误、持续时间。

------

## 45. 最终执行报告

AgentRun 完成后生成：任务结果、完成了什么、未完成什么、修改文件、执行命令、验证结果、失败与警告、Git 状态、是否创建 Commit、云模型披露记录、回滚入口。

------

## 46. 第一版实现范围

实现：Ask/Plan/Change/Diagnose 四种模式、结构化 Tool Call、Policy Engine、路径规范化和工作区边界、Change Session、Git Worktree 隔离、Unified Diff、文件级审批、Patch 基准哈希、原子单文件写入、受控命令执行、Windows Job Object、超时与取消、实时日志、验证流水线、Checkpoint 与 Session Rollback、本地 Git Commit 审批、完整审计时间线、崩溃恢复。

暂缓：AI 自主 Push、AI 自主合并、AI 自动执行数据库生产迁移、无审批的持续自治模式。

------

## 47. 完整 Agent 时序

```text
用户创建开发任务 → AgentRun 进入 Analyzing
→ 检索代码/文档/Git/测试 → 生成结构化计划
→ 用户批准计划 → 确定 ChangeBudget → 检测 Git 工作区状态
→ 创建隔离 Worktree 或临时副本 → AI 生成 Tool Call
→ Tool Schema 校验 → Policy Engine 评估 → 执行只读工具
→ 生成 ChangeSet → Patch/路径/基准哈希校验
→ 用户审查文件 Diff → 批准后应用到 Session Workspace
→ AI 提出验证命令 → 命令风险分析 → 用户批准
→ Process Supervisor 执行 → 实时返回日志
→ 验证结果反馈给 AI → 必要时生成下一版 ChangeSet
→ 达到完成条件 → 用户选择 Commit/导出 Patch/保留 Worktree
→ 生成执行报告/审计记录/回滚入口
```

核心：模型可以主动思考和提出操作，但文件系统、命令、Git、网络和凭据始终掌握在 Rust Policy Engine 与用户审批手中。
