## 第九部分：项目阶段拆分、MVP、团队分工与实施里程碑

# 1. 实施原则

DevForge 规模较大，不能按“先把所有页面搭出来，再补后台”的方式开发。

正确顺序应该是：

```text
领域边界
   ↓
数据与运行时基础
   ↓
本地知识索引
   ↓
搜索与引用
   ↓
AI 问答
   ↓
受控变更
   ↓
外部连接器
   ↓
插件与高级能力
```

每个阶段都必须形成一个可以独立运行、验证和演示的版本。

核心原则：

1. 先完成本地闭环，再接入云端和外部平台。
2. 先保证检索和引用准确，再追求 Agent 自动化。
3. 先做单工作区和单用户，再考虑团队同步。
4. 先支持少数语言，再扩展语言数量。
5. Agent 写代码必须晚于只读问答能力。
6. 插件系统不能早于核心接口稳定。
7. 每个阶段都要有明确的退出条件。
8. 不把“代码已经写完”当作阶段完成标准，必须通过测试和真实项目验证。

------

# 2. 项目拆分

整个项目建议拆成九个相对独立的子项目。

```text
DevForge
├─ P0 工程基础设施
├─ P1 工作区与本地存储
├─ P2 文档和代码索引
├─ P3 搜索与代码理解
├─ P4 AI 问答与引用
├─ P5 受控 Agent 与变更中心
├─ P6 GitHub / GitLab 连接器
├─ P7 插件、模型和高级运行时
└─ P8 发布、安全与产品化
```

这些子项目有明确依赖关系：

```text
P0
 ↓
P1
 ↓
P2
 ↓
P3
 ↓
P4
 ↓
P5

P1 ─────→ P6
P4 ─────→ P7
P5 ─────→ P8
```

------

# 3. 阶段零：工程基础设施

## 目标

建立可持续开发的大型 Monorepo，以及 Rust、React、Tauri 的基础边界。

## 主要工作

### Monorepo

```text
apps/desktop
packages/ui
packages/api-client
packages/shared
crates/*
```

### Rust Workspace

第一阶段只创建必要 crate：

```text
devforge-domain
devforge-application
devforge-runtime
devforge-storage
devforge-platform
devforge-shared
```

不要第一天就创建二十个空 crate。

随着能力落地，再逐步拆出：

```text
devforge-indexer
devforge-search
devforge-ai
devforge-agent
devforge-connectors
```

### 前端基础

完成：

- React 应用外壳
- Router
- TanStack Query
- Zustand
- IPC Client
- 主题系统
- Error Boundary
- 基础布局
- Command Palette 框架

### 工程质量

建立：

```text
Rust fmt
Clippy
TypeScript Check
ESLint
Frontend Tests
Rust Tests
Commit Hooks
CI
```

## 交付结果

应用能够启动，React 可以通过类型安全 IPC 调用 Rust，并展示：

- 应用版本
- 数据目录
- 后台健康状态
- SQLite 状态
- 基础日志

## 阶段退出条件

- Windows 开发环境可以一条命令启动。
- Rust 与 TypeScript 类型同步方案确定。
- CI 可以构建 Tauri 应用。
- SQLite Migration 能在空数据库运行。
- Rust Core 不直接依赖 React。
- Domain 层不依赖 Tauri。
- Release 构建可以安装和启动。

------

# 4. 阶段一：工作区与本地存储

## 目标

让用户可以创建真正可持续保存的开发工作区。

## 功能范围

### 工作区

- 创建工作区
- 编辑名称和描述
- 添加本地 Git 仓库
- 添加普通文档目录
- 移除数据源
- 最近工作区
- 工作区归档
- 工作区设置

### 文件发现

- 扫描目录
- 读取 `.gitignore`
- 读取 `.devforgeignore`
- 文件类型识别
- 二进制识别
- 文件大小限制
- 敏感文件识别

### 数据库

实现：

```text
workspaces
workspace_settings
sources
documents
tasks
audit_events
```

### 文件树

React 端完成：

- 工作区资源管理器
- 目录懒加载
- 文件类型图标
- 文件基础预览
- Monaco 只读查看器
- Markdown 查看器

## 暂不实现

- 向量索引
- AI 问答
- Code Graph
- GitHub/GitLab
- Agent 修改
- 插件

## 交付结果

用户可以导入一个真实项目，并在 DevForge 中浏览代码和文档。

## 阶段退出条件

- 可导入至少三个本地仓库到同一工作区。
- 大型目录不会一次全部传给 React。
- 敏感文件默认不读取正文。
- 文件树支持增量刷新。
- 应用重启后可以恢复工作区和打开标签。
- 删除工作区不会删除原始仓库。
- 文件路径逃逸测试全部通过。

------

# 5. 阶段二：文档和代码索引

## 目标

建立完整的本地知识索引流水线。

## 功能范围

### 全文索引

- Tantivy 工作区索引
- 路径 Tokenizer
- 代码 Tokenizer
- 标题、路径、符号和正文权重
- 索引 Manifest
- Building 与 Active 版本切换

### Tree-sitter

第一批语言：

```text
TypeScript / JavaScript
Rust
Python
```

提取：

- 函数
- 类
- 接口
- Struct
- Trait
- Import
- Export
- 注释
- 基础调用关系

### Chunk

实现：

- 代码符号 Chunk
- Markdown 标题 Chunk
- 普通段落 Chunk
- 超大结构二次切分
- Token 计数
- 内容哈希

### 增量索引

- 文件监听
- Debounce
- 内容哈希判断
- 单文件索引任务
- 文件删除
- 文件重命名
- Git 分支变化感知
- 崩溃恢复

### 调度器第一版

实现：

- SQLite Job Queue
- 优先级
- 去重
- 取消
- 重试
- 进度
- Lease
- 崩溃恢复

## 交付结果

导入仓库后，用户可以看到：

```text
发现文件
解析代码
构建全文索引
提取符号
索引完成
```

并可以执行基础关键词搜索。

## 阶段退出条件

- 三种语言能够稳定提取主要符号。
- 文件修改后可以增量更新。
- 文件删除后不会残留搜索结果。
- 应用崩溃后索引任务可以恢复。
- 当前 Active Index 不会因重建失败损坏。
- 中型仓库首次索引性能达到内部目标。
- Fixture 索引回归测试稳定。

------

# 6. 阶段三：搜索与代码理解

## 目标

让 DevForge 从“文件全文搜索”升级为开发者知识搜索。

## 功能范围

### 搜索通道

实现：

```text
Exact Search
Tantivy Search
Symbol Search
Basic Graph Search
Git Commit Search
```

### 搜索融合

- 统一 SearchRequest
- 统一 SearchHit
- RRF
- 结果去重
- 查询过滤
- 当前工作区优先
- 当前文件和仓库加权
- 匹配原因解释

### Code Graph

实现：

```text
defines
contains
imports
exports
calls
references
implements
extends
uses_type
```

### LSP 第一版

优先支持：

```text
TypeScript Language Server
rust-analyzer
Pyright
```

LSP 作为可选增强：

- 定义
- 引用
- 实现
- 调用层级
- 诊断

### 界面

完成：

- 全局搜索页
- 搜索过滤器
- 符号详情页
- 文件引用列表
- 基础局部 Code Graph
- 搜索匹配原因
- 搜索结果虚拟列表

## 交付结果

用户可以搜索：

```text
UserService
token expired
在哪里刷新 Token
谁调用了 createOrder
这个接口有哪些实现
```

自然语言语义搜索暂时可以依赖关键词扩展，向量能力放在下一阶段。

## 阶段退出条件

- 精确符号搜索稳定。
- 同名符号能够按仓库和上下文区分。
- Code Graph 可以展示一跳关系。
- LSP 不可用时系统能降级。
- 搜索结果可以跳转至准确代码行。
- 搜索评估集建立完成。
- Recall、MRR 和符号命中率具有基准数据。

------

# 7. 阶段四：AI 问答与引用

## 目标

建立第一个真正可用的 AI 开发者知识问答闭环。

## 功能范围

### 模型 Provider

第一批建议实现：

```text
Ollama
OpenAI-Compatible
OpenAI
Anthropic
```

Gemini 和 LM Studio 可以作为阶段内后续任务。

### Embedding

支持：

- 本地 Embedding 模型
- 云端 Embedding Provider
- 向量版本
- 模型切换
- 后台补全
- 独立索引目录

### 混合检索

实现：

```text
全文
向量
符号
一跳 Code Graph
Git Commit
RRF
基础规则重排
```

本地 Reranker 可作为可选功能。

### RAG

实现：

- 查询意图识别
- 会话问题重写
- EvidenceBundle
- 最小代码上下文扩展
- Token 预算
- Prompt 数据边界
- 云端披露清单
- 敏感信息过滤

### 引用

实现：

- 文件和行号引用
- 引用稳定编号
- 引用点击跳转
- 引用版本
- 基础引用存在性校验
- 回答可信等级

### AI 界面

实现：

- Ask 模式
- 流式回答
- Context 面板
- Evidence 面板
- 模型切换
- 取消生成
- 会话历史
- 保存回答为知识笔记

## MVP 在此阶段形成

达到本阶段后，DevForge 已经是一个独立可用的产品：

> 用户可以导入多个代码仓库，在本机建立索引，并通过本地或云模型对项目进行带文件和行号引用的问答。

## 阶段退出条件

- AI 回答的引用可以正确跳转。
- 云模型调用前可以展示发送内容。
- 本地模型失败不会自动回退云端。
- 敏感文件默认不会进入上下文。
- Prompt Injection 内容不会获得系统权限。
- 模型响应取消有效。
- 搜索与 AI 流程具备完整关联 ID。
- 固定评估集的引用正确率达到最低门槛。

------

# 8. MVP 明确定义

DevForge 的 MVP 不包含自主修改代码。

MVP 必须包含：

```text
工作区管理
多本地 Git 仓库
文档目录
文件浏览
TypeScript、Rust、Python 索引
Tantivy 全文搜索
符号搜索
基础 Code Graph
嵌入式向量搜索
Ollama
OpenAI-Compatible
至少一个云端原生 Provider
AI 项目问答
流式回答
文件和行号引用
云端披露清单
搜索评估
索引健康检查
Windows 安装包
```

MVP 不包含：

```text
AI 自动修改代码
命令执行
GitHub/GitLab 同步
插件系统
团队协作
多设备同步
自动创建 PR
完整 LSP 多语言生态
```

MVP 成功标准：

1. 用户可以在二十分钟内完成安装、导入项目并提出第一个问题。
2. 对固定真实项目问题，主要答案能够引用正确文件。
3. 修改一个文件后，索引能在合理时间内更新。
4. 关闭应用再打开，对话、工作区和索引仍然存在。
5. 无云模型时，本地模式仍能完整工作。
6. 一个 Provider 失败不会导致应用不可使用。
7. 中型项目不会让 UI 明显卡死。
8. 用户能理解 AI 为什么得出当前结论。

------

# 9. 阶段五：受控 Agent 与变更中心

## 目标

让 AI 从“回答问题”升级为“可以提出并应用受控代码修改”。

## 功能顺序

不要直接实现完整 Agent，建议拆成四个子阶段。

## 5A：Plan 模式

实现：

- 开发计划
- 影响模块
- 预计文件
- 测试计划
- ChangeBudget
- 计划批准

此时仍不写文件。

## 5B：Patch 生成

实现：

- Change Session
- Unified Diff
- 文件级审批
- Patch 基线哈希
- Diff Review 页面
- Patch 导出

此时可以只生成 Patch，不自动应用。

## 5C：隔离应用

实现：

- Git Worktree
- 非 Git 临时副本
- 原子文件写入
- 多文件 Apply Journal
- 冲突处理
- Session Rollback

## 5D：命令执行

实现：

- 结构化命令
- Policy Engine
- 命令审批
- Windows Job Object
- 实时日志
- 超时与取消
- 验证流水线
- Git Commit 审批

## 交付结果

用户可以：

```text
提出修改需求
→ 查看计划
→ 审批计划
→ 查看 Diff
→ 审批文件
→ 在隔离 Worktree 应用
→ 审批测试命令
→ 查看结果
→ 创建本地 Commit
```

## 阶段退出条件

- AI 无法绕过审批修改文件。
- Approval Hash 参数变化后会失效。
- 工作区外路径操作全部被拦截。
- Patch 冲突不会静默覆盖文件。
- 命令取消可以终止进程树。
- Session Worktree 不污染用户当前分支。
- 崩溃后可以识别部分应用状态。
- 回滚测试全部通过。
- 安全测试直接调用 Rust API 仍然无法绕过策略。

------

# 10. 阶段六：GitHub 与 GitLab 连接器

## 目标

将开发过程中的历史讨论和远程协作信息加入知识库。

## 第一批功能

### GitHub

- OAuth 或 Fine-grained Token
- 仓库选择
- Commit
- Pull Request
- Review
- Issue
- Comment
- Release

### GitLab

- GitLab.com
- Self-Managed
- OAuth 或 Access Token
- Commit
- Merge Request
- Note
- Issue
- Wiki
- Pipeline 摘要

### 同步能力

- 增量游标
- 内容哈希
- 本地快照
- 限流
- 认证失效
- 定期完整核对
- 手动同步
- 工作区打开时快速同步

### 搜索整合

支持回答：

```text
为什么这里要兼容旧字段？
这个 Bug 以前是否出现过？
PR 中为什么拒绝了这个方案？
这个函数最近是谁修改的？
```

## 暂不实现

- 远程写入
- 自动评论
- 自动合并
- 自动创建 PR
- 本地直接接收公网 Webhook

## 阶段退出条件

- 增量同步不会重复产生文档。
- 认证失效后不会无限重试。
- 本地快照可以离线搜索。
- GitHub/GitLab 数据能与代码引用关联。
- 远程历史不会覆盖当前工作树事实。
- 删除和权限变化能通过定期核对发现。

------

# 11. 阶段七：插件、模型与高级运行时

## 目标

开放可控扩展能力，而不破坏核心安全边界。

## 功能范围

### 模型增强

- Gemini
- LM Studio
- Reranker
- 模型路由
- 显式回退链
- Provider 熔断
- Token 和成本中心
- 模型性能对比

### WASM 插件

先完成：

```text
Plugin Manifest
安装与卸载
权限展示
WASM Runtime
资源限制
插件存储
HTTP Host Capability
Connector Plugin
Parser Plugin
Read-only Tool Plugin
```

### 插件 SDK

提供：

- WIT 接口
- 示例插件
- Rust SDK
- TypeScript 或其他语言绑定
- 本地测试工具
- 插件签名工具

### 高级调度

- 电池模式
- 计量网络
- CPU 与内存压力
- GPU 任务
- 模型下载
- 任务依赖可视化
- Dead Letter 管理

## 阶段退出条件

- 插件无法直接读取宿主文件系统。
- 插件无法获得凭据明文。
- 插件权限增加需要重新审批。
- 插件故障不会导致主应用崩溃。
- Provider 回退不会跨越用户隐私策略。
- 插件 SDK 有至少两个真实示例。

------

# 12. 阶段八：发布、安全与产品化

## 目标

将工程原型转化为可以长期升级和分发的桌面产品。

## 功能范围

- Stable、Beta、Nightly 渠道
- Windows Code Signing
- Tauri Updater Signing
- 数据库升级备份
- 索引后台迁移
- 更新安全重启
- 崩溃恢复
- 安全模式
- 诊断中心
- 脱敏诊断包
- SBOM
- 依赖审计
- 性能回归
- 安装与升级测试
- 隐私说明
- 本地数据管理页面

## 退出条件

- 上一个正式版本可以直接升级。
- 升级不会删除旧索引，除非新索引成功。
- 更新期间不会中断关键写操作。
- 安装包和更新包签名验证通过。
- 诊断包默认不包含源码和 Prompt。
- 连续崩溃可以进入安全模式。
- Release CI 具备完整可重复流程。

------

# 13. 推荐开发顺序

完整顺序：

```text
1. Monorepo 和 Tauri Shell
2. SQLite 与工作区
3. 本地仓库和文件树
4. 后台任务调度
5. Tantivy 全文索引
6. Tree-sitter 符号提取
7. 增量文件监听
8. 搜索页面
9. Code Graph 基础关系
10. 模型 Provider
11. Embedding 与向量检索
12. RAG 上下文
13. AI 流式回答
14. 引用验证
15. Plan 模式
16. Change Session
17. Diff 审批
18. Worktree 隔离
19. 命令审批与执行
20. GitHub / GitLab
21. 插件系统
22. 发布与升级
```

不要采用：

```text
先做完整 UI
→ 再补 Rust
→ 最后接索引和 AI
```

也不要采用：

```text
先实现自主 Agent
→ 后补权限、安全和回滚
```

这两条路线都会导致大规模返工。

------

# 14. 前十个开发迭代

每个迭代应尽量形成可演示结果。

## Iteration 1：应用骨架

完成：

- Tauri + React
- Rust Workspace
- SQLite
- IPC
- 日志
- 主布局

演示：

```text
启动应用
查看版本
查看健康状态
创建空工作区
```

## Iteration 2：本地工作区

完成：

- 添加仓库
- 添加文档目录
- 文件树
- Monaco
- Markdown

演示：

```text
导入三个仓库
浏览代码
恢复上次标签页
```

## Iteration 3：文件扫描

完成：

- Ignore 规则
- Document 表
- 文件哈希
- 文件监听
- 索引任务

演示：

```text
修改文件后，后台识别变化
```

## Iteration 4：全文索引

完成：

- Tantivy
- 关键词搜索
- 路径搜索
- 搜索结果页面

演示：

```text
搜索错误文本和配置项
```

## Iteration 5：代码结构

完成：

- Tree-sitter
- 符号提取
- 符号搜索
- 符号详情

演示：

```text
搜索 UserService
打开定义和所在文件
```

## Iteration 6：Code Graph

完成：

- Import
- Call
- Implements
- Reference
- 局部关系图

演示：

```text
查看函数调用者和被调用函数
```

## Iteration 7：本地模型

完成：

- Ollama
- Chat Streaming
- 基础上下文
- 会话记录

演示：

```text
询问当前文件
```

## Iteration 8：混合检索

完成：

- Embedding
- 向量索引
- RRF
- EvidenceBundle
- Token 预算

演示：

```text
使用自然语言搜索不同命名的代码
```

## Iteration 9：引用问答

完成：

- 工作区问答
- 文件行号引用
- 引用跳转
- 可信等级
- 云端披露

演示：

```text
解释登录流程，并打开每个引用
```

## Iteration 10：MVP 稳定化

完成：

- 索引健康检查
- Provider 错误降级
- 性能测试
- Windows 安装包
- 数据库迁移
- 搜索质量回归

演示：

```text
安装正式包
导入真实项目
完成完整问答流程
```

------

# 15. 团队规模建议

## 单人开发

可以完成，但必须严格控制范围。

单人第一目标应只做到 MVP：

```text
本地工作区
索引
搜索
AI 问答
引用
Windows 安装
```

不建议单人在 MVP 前同时推进：

- Agent
- GitHub/GitLab
- 插件
- 多平台完整支持
- 团队协作

单人实施时，应优先使用已有成熟库，避免自研：

- 编辑器
- Diff 算法
- Git 实现
- 向量数据库核心
- Markdown 解析器
- LSP 协议模型
- 图形布局引擎

## 3～4 人小组

推荐角色：

```text
Rust Core / 数据与索引
AI / 搜索 / RAG
React / Desktop UX
基础设施 / 测试 / 发布
```

部分角色可以合并。

## 6～8 人团队

推荐：

```text
1 名技术负责人
2 名 Rust 后台工程师
1 名搜索与 AI 工程师
2 名前端桌面工程师
1 名测试与自动化工程师
1 名产品设计或 UX
```

------

# 16. 模块所有权

## Rust Core 负责人

负责：

- Domain
- Application
- Runtime
- SQLite
- Task Scheduler
- Platform Adapter
- 错误与日志

## 搜索与 AI 负责人

负责：

- Tree-sitter
- LSP
- Tantivy
- 向量索引
- RAG
- Provider
- 搜索评估
- 引用校验

## React 负责人

负责：

- App Shell
- Workspace Explorer
- Search
- Monaco
- AI Assistant
- Diff Review
- Task Center
- 状态管理

## 安全与 Agent 负责人

项目进入阶段五后，可以独立负责：

- Policy Engine
- Approval
- PathGuard
- Change Session
- Command Runner
- Worktree
- Rollback

即使人员不足，也应该保持代码所有权边界清晰。

------

# 17. 跨模块接口冻结顺序

以下接口应尽早稳定：

```text
WorkspaceId / DocumentId / TaskId
AppErrorResponse
DomainEvent
Job Model
Document Model
SearchRequest / SearchHit
SourceLocation
ModelStreamEvent
```

以下接口不应过早冻结：

```text
Plugin API
Agent Tool API
Connector Write API
Team Sync Protocol
Remote Runner Protocol
```

原因是后者需要真实使用后才能确定合理边界。

------

# 18. 技术风险排序

## 风险一：搜索效果看似丰富但不准确

表现：

- 搜索结果很多
- AI 回答很长
- 引用却不支持结论

应对：

- 早期建立固定评估集
- 引用优先于回答长度
- 每次修改检索策略都跑回归
- 允许回答“证据不足”

## 风险二：Rust 模块过度拆分

表现：

- 大量空 crate
- 一个功能修改需要改十几个包
- 类型循环转换

应对：

- 先模块化，后独立 crate
- 只有边界稳定或编译隔离需要时才拆
- 保持 Domain 和 Infrastructure 分离即可

## 风险三：AI Agent 过早实现

表现：

- 功能看起来炫酷
- 文件修改不可靠
- 无法回滚
- 权限规则混乱

应对：

- MVP 只读
- Plan、Patch、Apply、Command 分阶段实现
- 安全策略在 Rust 中强制

## 风险四：大型仓库性能不足

表现：

- 初次索引过慢
- 文件树卡顿
- 内存过高
- 搜索阻塞 UI

应对：

- 分阶段可用
- 虚拟列表
- 任务优先级
- 增量索引
- 分资源并发
- Medium Fixture 作为持续基准

## 风险五：模型 Provider 差异

表现：

- Compatible API 实际不兼容
- Tool Call 格式不同
- Usage 缺失
- 流式事件异常

应对：

- Provider Contract Tests
- 能力声明
- 原生 Provider
- 保守默认值
- 不根据模型名称盲目推断

## 风险六：Windows 系统细节

表现：

- 路径大小写
- Junction
- 长路径
- 文件占用
- 进程树
- WebView 和安装签名问题

应对：

- Windows 从第一阶段进入 CI
- Windows 路径安全测试
- Job Object
- 安装与升级冒烟测试
- 不等到发布前再适配 Windows

------

# 19. 产品风险

## 功能过多但主线不清晰

主线必须始终是：

```text
导入项目
→ 建立知识索引
→ 找到答案
→ 验证来源
→ 安全完成开发任务
```

其他功能都应该服务于这条主线。

## 与 IDE 定位重叠

DevForge 不应第一阶段试图取代 VS Code、IDEA 或 Cursor。

更合理的定位：

> 跨仓库、跨文档、跨历史的项目知识与 AI 任务工作台。

用户仍然可以使用原有 IDE 完成日常编辑。

## 本地优先导致安装复杂

应提供：

- 首次设置向导
- 本地模型自动检测
- Ollama 连接检测
- 云模型快速配置
- 示例工作区
- 索引进度说明

不要要求普通用户手工理解向量模型、Reranker 和 Tokenizer。

------

# 20. 第一版明确不做的内容

为了避免项目失控，第一版明确不做：

```text
多人实时协作
云端知识库同步
移动端
浏览器版
完整 IDE 编译调试器
远程 SSH 开发
任意终端自动化
自主 Push
自主合并 PR
生产数据库迁移
无限制 Agent
任意第三方 UI 插件
所有编程语言
大型企业权限系统
云端计费系统
```

“不做清单”需要与功能清单同等重要。

------

# 21. 产品版本路线

## DevForge 0.1：Local Workspace

```text
工作区
文件浏览
本地仓库
基础索引
关键词搜索
```

## DevForge 0.2：Code Intelligence

```text
Tree-sitter
符号搜索
Code Graph
LSP 增强
```

## DevForge 0.3：AI Knowledge

```text
本地与云模型
向量搜索
RAG
引用
隐私披露
```

这是第一个公开 MVP 候选。

## DevForge 0.4：Planning

```text
Plan 模式
影响分析
开发计划
测试计划
```

## DevForge 0.5：Controlled Changes

```text
Change Session
Diff 审批
Worktree
Patch
回滚
```

## DevForge 0.6：Agent Execution

```text
命令审批
验证流水线
任务时间线
本地 Commit
```

## DevForge 0.7：Connected Knowledge

```text
GitHub
GitLab
PR / Issue / Commit
历史原因分析
```

## DevForge 0.8：Extensibility

```text
WASM 插件
连接器 SDK
Parser SDK
高级模型路由
```

## DevForge 1.0

满足：

- Windows 正式签名发行
- 稳定升级
- 数据迁移稳定
- 中型仓库性能达标
- 问答引用准确
- Agent 审批与回滚可靠
- GitHub/GitLab 可用
- 诊断和恢复完整

------

# 22. 每阶段验收模板

每个阶段结束时，必须回答：

## 功能

```text
用户现在能够完成什么完整任务？
```

## 数据

```text
应用重启或异常退出后，数据是否仍然正确？
```

## 性能

```text
在 Small 和 Medium Fixture 中表现如何？
```

## 降级

```text
某个模块失效后，哪些能力仍然可用？
```

## 安全

```text
可以绕过权限或访问工作区外内容吗？
```

## 测试

```text
有哪些自动测试证明功能成立？
```

## 可观测性

```text
失败后能否通过关联 ID 和诊断中心定位？
```

## 升级

```text
新数据结构是否有 Migration？
```

只有这些问题都能回答，阶段才算完成。

------

# 23. 项目管理建议

建议使用 Epic 组织：

```text
EPIC-001 Workspace Foundation
EPIC-002 Indexing Pipeline
EPIC-003 Code Intelligence
EPIC-004 Hybrid Search
EPIC-005 AI Workspace Q&A
EPIC-006 Controlled Agent
EPIC-007 GitHub and GitLab
EPIC-008 Plugin Runtime
EPIC-009 Release Engineering
```

每个 Epic 下拆：

```text
Architecture
Domain
Infrastructure
IPC
Frontend
Testing
Diagnostics
Documentation
```

任务不能只写：

```text
完成搜索功能
```

应该写成：

```text
实现 Tantivy 路径和符号 Tokenizer
实现工作区全文索引 Manifest
实现搜索分页 IPC
实现搜索结果虚拟列表
实现文件删除后的索引清理
增加搜索 Fixture 回归测试
```

------

# 24. 架构决策记录

建议在项目中维护 ADR：

```text
docs/adr/
├─ 0001-use-tauri-2.md
├─ 0002-modular-rust-core.md
├─ 0003-sqlite-as-source-of-truth.md
├─ 0004-tantivy-for-lexical-search.md
├─ 0005-embedded-vector-index.md
├─ 0006-tree-sitter-plus-lsp.md
├─ 0007-human-approved-agent.md
├─ 0008-git-worktree-isolation.md
└─ 0009-wasm-plugin-system.md
```

每份 ADR 包含：

```text
背景
决策
备选方案
原因
后果
以后何时重新评估
```

这能防止半年后团队不知道“为什么当初这样设计”。

------

# 25. 文档体系

建议维护：

```text
docs/
├─ product/
│  ├─ vision.md
│  ├─ scope.md
│  └─ roadmap.md
├─ architecture/
│  ├─ overview.md
│  ├─ data-flow.md
│  ├─ security.md
│  └─ runtime.md
├─ adr/
├─ api/
├─ plugin-sdk/
├─ testing/
├─ release/
└─ operations/
```

最重要的文档：

```text
系统边界
数据目录
错误代码
IPC 接口
索引格式
权限规则
插件权限
数据库迁移
发布流程
```

------

# 26. 最终推荐落地范围

从实际可实施角度，建议把首个完整周期限定为：

```text
Windows
单用户
本地优先
多工作区
多本地仓库
TypeScript / Rust / Python
SQLite
Tantivy
嵌入式向量索引
Tree-sitter
可选 LSP
Ollama
OpenAI-Compatible
Anthropic 或 OpenAI
AI 问答
可靠引用
Plan 模式
```

第一轮不急着完成：

```text
代码自动应用
命令执行
GitHub/GitLab
插件系统
```

先验证核心假设：

> 开发者是否愿意导入真实项目，并持续使用跨仓库检索与带引用 AI 问答？

如果这个假设成立，再进入 Agent 和连接器阶段。

------

# 27. 最终项目结构演进建议

初始阶段：

```text
crates/
├─ domain
├─ application
├─ infrastructure
├─ runtime
└─ platform
```

MVP 阶段：

```text
crates/
├─ domain
├─ application
├─ runtime
├─ storage
├─ indexing
├─ search
├─ code-intelligence
├─ ai
├─ platform
└─ shared
```

Agent 阶段：

```text
crates/
├─ agent
├─ security
├─ git
└─ execution
```

连接器与插件阶段：

```text
crates/
├─ connectors
├─ plugin-host
├─ plugin-sdk
└─ integration-models
```

不要在项目创建当天直接建立最终所有 crate。

------

# 28. 第一项真正应该开始的工作

正式编码前，第一项工作不是创建 React 页面，而是编写以下设计文档：

```text
01-product-scope.md
02-domain-model.md
03-rust-boundaries.md
04-data-model.md
05-indexing-pipeline.md
06-ipc-contract.md
07-security-boundaries.md
08-mvp-acceptance.md
```

然后实现一个纵向切片：

```text
创建工作区
   ↓
添加本地仓库
   ↓
扫描文件
   ↓
写入 SQLite
   ↓
在 React 文件树展示
   ↓
打开一个源码文件
```

这个纵向切片可以验证：

- Tauri IPC
- Rust 分层
- SQLite
- 路径安全
- React Query
- Zustand 布局
- 文件树性能
- 错误协议
- 日志

完成后再开始索引模块。

------

# 29. 项目最终验收目标

DevForge 1.0 应能完成下面这条完整流程：

```text
用户安装 DevForge
   ↓
创建开发工作区
   ↓
添加多个本地代码仓库
   ↓
系统增量建立代码、文档、向量和图谱索引
   ↓
用户用自然语言询问项目问题
   ↓
AI 返回带文件、符号、行号和版本的答案
   ↓
用户查看来源并继续追问
   ↓
AI 生成开发计划和影响分析
   ↓
用户批准计划
   ↓
系统创建隔离 Worktree
   ↓
AI 生成可审查 Patch
   ↓
用户批准文件变更
   ↓
Rust 安全应用修改
   ↓
用户批准格式化、检查和测试
   ↓
系统监管命令并记录结果
   ↓
用户创建本地 Commit
   ↓
需要时关联 GitHub PR 或 GitLab MR 历史
   ↓
所有过程可审计、可取消、可恢复、可回滚
```

最终产品定位可以概括为：

> **DevForge 是一个本地优先、证据驱动、人工审批的开发者知识与 AI 任务工作台。**

它的竞争力不应该是“AI 能自动做更多事情”，而应该是：

```text
更懂整个项目
答案可以验证
上下文由用户控制
代码默认留在本地
修改前必须审批
操作能够回滚
历史和当前实现统一检索
```