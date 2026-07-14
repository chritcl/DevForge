## 第七部分：React 桌面界面、页面交互与状态管理

# 1. 界面设计目标

DevForge 的界面不是传统的“后台管理系统”，而是接近 IDE、知识库与 AI 工作台结合的桌面应用。

核心体验目标：

1. 工作区始终是当前上下文中心。
2. 搜索、代码阅读、AI 对话和任务执行可以同时进行。
3. 用户不需要频繁切换完整页面。
4. 任何 AI 结论都可以快速跳转到源代码或文档。
5. 写文件、执行命令等高风险操作必须拥有独立、清晰的审批界面。
6. 大量实时日志和流式内容不能导致整个 React 应用频繁重渲染。
7. 同一个工作区可以保存不同的面板布局和打开状态。
8. 界面结构要支持以后增加新连接器、工具和 Agent 能力。

------

# 2. 整体桌面布局

主窗口采用类似 IDE 的多区域结构：

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ Title Bar / Workspace / Branch / Search / Agent Status / Window Controls │
├──────┬──────────────────────┬───────────────────────────┬────────────────┤
│      │                      │                           │                │
│活动栏│ 左侧边栏             │ 主编辑区                  │ AI 助手面板    │
│      │                      │                           │                │
│      │ 工作区资源           │ 文件 / 搜索 / Diff / 图谱 │ 对话 / 证据    │
│      │ 搜索                 │                           │ Agent 计划     │
│      │ Git                  │                           │ Tool Calls     │
│      │ 任务                 │                           │                │
│      │                      │                           │                │
├──────┴──────────────────────┴───────────────────────────┴────────────────┤
│ Bottom Panel：终端 / 输出 / 问题 / 索引 / 任务日志 / 审计                │
├──────────────────────────────────────────────────────────────────────────┤
│ Status Bar：工作区状态 / Git / 索引 / 模型 / 隐私 / 后台任务 / 错误      │
└──────────────────────────────────────────────────────────────────────────┘
```

主要区域：

```text
Activity Bar
Primary Sidebar
Editor Area
Assistant Sidebar
Bottom Panel
Status Bar
Command Palette
Notification Center
```

每个区域都可以：

- 折叠
- 调整宽度或高度
- 保存工作区布局
- 通过快捷键切换
- 恢复默认布局

------

# 3. 顶部标题栏

标题栏包含：

```text
应用图标
当前工作区
当前仓库
当前 Git 分支
全局搜索入口
Command Palette
Agent 当前状态
后台任务数量
窗口控制
```

示例：

```text
DevForge  |  电商平台  /  api-server  /  feature/coupon
           [ 搜索代码、文档、Issue 或提出问题…… ]
                          Agent：等待审批 2
```

点击当前工作区，可以打开快速切换器：

```text
最近工作区
已固定工作区
归档工作区
创建工作区
导入工作区
```

点击分支，可以：

- 查看当前仓库状态
- 切换分支
- 创建分支
- 打开 Git 工作台
- 查看当前 Change Session 分支

标题栏搜索框支持两种行为：

```text
普通输入
→ 打开全局搜索

以 ? 开头
→ AI 问答

以 > 开头
→ Command Palette

以 @ 开头
→ 搜索符号

以 # 开头
→ 搜索 Issue / PR

以 : 开头
→ 跳转行号或文件
```

------

# 4. Activity Bar

左侧最窄的一列用于切换主要功能。

推荐入口：

```text
工作区资源
全局搜索
AI 工作台
Code Graph
Git
变更中心
任务中心
连接器
模型管理
插件
设置
```

下方固定入口：

```text
账户与凭据
通知中心
后台健康状态
帮助
```

Activity Bar 只负责切换左侧边栏内容，不应该每次都替换整个主编辑区。

例如：

- 用户正在主编辑区查看 `AuthService.rs`
- 点击 Git 图标
- 左侧切换为 Git 文件列表
- 主编辑区文件仍然保持打开

------

# 5. 左侧边栏

## 5.1 工作区资源视图

```text
WORKSPACE
├─ Repositories
│  ├─ admin-web
│  ├─ api-server
│  └─ deployment
├─ Documents
├─ External Sources
│  ├─ GitHub
│  └─ GitLab
├─ Knowledge
│  ├─ Saved Answers
│  ├─ Architecture Notes
│  └─ Decisions
└─ Recent
```

每个仓库展开后：

```text
api-server
├─ src
├─ tests
├─ Cargo.toml
├─ Symbols
├─ Git History
├─ Issues
└─ Pull Requests
```

文件节点支持状态标记：

```text
M    已修改
A    新增
D    删除
I    正在索引
V    向量待生成
!    解析失败
S    敏感文件
```

右键操作：

```text
打开
在新标签打开
在右侧打开
询问 AI
解释文件
查找引用
查看 Git 历史
复制相对路径
在资源管理器中显示
从知识索引中排除
重新索引
```

------

## 5.2 搜索视图

搜索边栏包含：

```text
查询输入
搜索范围
模式
过滤器
搜索历史
保存的查询
```

过滤器支持折叠：

```text
工作区
仓库
文件类型
语言
路径
时间
数据源
证据级别
Git 分支
```

结果列表不直接全部加载，应支持：

- 虚拟滚动
- 分组展示
- 增量加载
- 搜索取消
- 查询耗时
- 匹配原因展示

结果可以按以下方式分组：

```text
最佳匹配
符号
代码
文档
Git 历史
Issue / PR
AI 对话
```

------

## 5.3 Git 视图

```text
CHANGES
├─ Staged Changes
├─ Changes
├─ Untracked
└─ Conflicts

BRANCHES
├─ Current
├─ Local
├─ Remote
└─ Session Branches

HISTORY
├─ Recent Commits
├─ Current File
└─ Current Branch
```

Change Session 创建的 Worktree 要使用特殊图标：

```text
AI Session
├─ devforge/auth-lockout
├─ devforge/coupon-feature
└─ devforge/fix-upload-timeout
```

避免用户把 Agent 分支与普通分支混淆。

------

## 5.4 任务视图

```text
RUNNING
├─ 索引 api-server
├─ cargo test
└─ GitHub 同步

WAITING
├─ ChangeSet 审批
└─ 命令审批

RECENT
├─ Completed
├─ Failed
├─ Cancelled
└─ Interrupted
```

任务节点展示：

```text
图标
任务名称
阶段
进度
运行时间
所属工作区
取消按钮
```

点击任务后在主编辑区打开任务详情。

------

# 6. 主编辑区

主编辑区采用标签页模型。

支持的标签页类型：

```text
SourceFileTab
DocumentTab
SearchResultsTab
SymbolTab
CodeGraphTab
ConversationTab
ChangeReviewTab
TaskDetailTab
GitDiffTab
SettingsTab
DashboardTab
```

标签页需要记录：

```text
tab_id
tab_type
resource_id
workspace_id
title
icon
dirty
pinned
preview
view_state
```

## Preview Tab

单击搜索结果时使用预览标签。

特点：

- 标签标题使用斜体
- 再次单击其他结果时复用
- 双击或编辑后变成固定标签
- 适合快速浏览搜索结果

------

# 7. 编辑器布局

主编辑区支持：

```text
单列
左右两列
上下两列
三列
临时对比模式
```

常见场景：

```text
左侧：接口定义
右侧：实现代码

左侧：修改前
右侧：修改后

左侧：源代码
右侧：相关测试

左侧：搜索结果
右侧：Code Graph
```

标签页可以拖拽到不同编辑组。

第一版不建议实现任意自由浮动窗口，以免布局复杂度过高。

------

# 8. 源代码查看器

使用 Monaco Editor，但默认是只读模式。

能力：

- 语法高亮
- 行号
- 折叠
- 代码导航
- 当前符号高亮
- 引用范围高亮
- Git 修改标记
- 诊断信息
- 行内 AI 操作
- 迷你地图
- Breadcrumb

顶部 Breadcrumb：

```text
api-server
> src
> auth
> service.rs
> impl AuthService
> login()
```

右键代码选区：

```text
解释这段代码
询问 AI
查找引用
查看调用者
查看被调用函数
分析影响
查看 Git Blame
添加到 AI 上下文
创建开发任务
复制引用链接
```

------

# 9. 文档查看器

Markdown 使用富文本预览与源码双模式。

PDF、Word 等文档至少支持：

- 页面导航
- 文本选取
- 搜索
- AI 询问选中内容
- 查看索引 Chunk
- 查看引用来源
- 跳转到 AI 回答引用页

文档顶部展示：

```text
来源
同步时间
索引状态
文档版本
是否会发送至云模型
```

------

# 10. 符号详情页

打开函数、类或接口时，不只是跳到源文件，还可以打开符号详情页。

```text
UserService.login
├─ 定义
├─ 签名
├─ 所属文件
├─ 调用者
├─ 被调用函数
├─ 类型依赖
├─ 实现关系
├─ 相关测试
├─ 相关 Commit
├─ 相关 Issue / PR
└─ AI 摘要
```

布局：

```text
┌──────────────────────────────┬───────────────────────┐
│ 符号源码                     │ 关系与元数据          │
│                              │                       │
│                              │ Callers               │
│                              │ Callees               │
│                              │ Tests                 │
│                              │ History               │
└──────────────────────────────┴───────────────────────┘
```

------

# 11. Code Graph 页面

Code Graph 不应只提供一张巨大关系网。

提供三种模式：

## 局部关系图

围绕当前符号展示一至两跳关系。

```text
AuthController.login
        ↓ calls
AuthService.login
        ↓ calls
TokenService.createToken
```

## 模块依赖图

展示仓库或模块级依赖。

```text
web
→ application
→ domain
→ infrastructure
```

## 影响分析图

以待修改对象为中心：

```text
PaymentResult
├─ used_by → PaymentService
├─ returned_by → PaymentController
├─ serialized_by → ApiResponse
├─ tested_by → PaymentTests
└─ consumed_by → admin-web
```

图谱操作：

```text
展开一跳
展开两跳
隐藏测试
只看精确关系
只看当前仓库
固定节点
在编辑器中打开
添加到 AI 上下文
创建影响分析
```

大型图谱采用增量加载，不能一次把整个 Code Graph 发送给前端。

------

# 12. AI 助手侧栏

右侧 AI 助手不是固定聊天窗口，而是可切换的工作模式。

```text
Chat
Context
Plan
Changes
Tools
Evidence
```

顶部显示：

```text
当前模式：Ask / Plan / Change / Diagnose
当前模型
当前工作区
当前上下文大小
隐私策略
```

## Chat

普通流式对话。

## Context

展示当前会发送给模型的内容：

```text
当前文件
当前选区
固定文件
检索证据
会话记忆
Git Diff
用户附加内容
```

用户可以手动删除某个上下文项。

## Plan

展示 Agent 生成的结构化计划：

```text
目标
理解
影响模块
预计文件
测试计划
风险
```

## Changes

展示当前 ChangeSet 摘要。

## Tools

展示 Tool Call 时间线。

## Evidence

展示 AI 回答使用的证据、引用和可信等级。

------

# 13. AI 输入框

输入框支持多种上下文引用：

```text
@文件
@符号
@仓库
@工作区
@Issue
@PR
@Commit
@当前选区
@当前 Diff
```

示例：

```text
请解释 @AuthService.login 的错误处理，
并结合 @PR#128 说明为什么这样设计。
```

输入框附件：

```text
添加文件
添加目录
添加 Git Diff
添加截图
添加日志
添加终端输出
添加数据库结构
```

模式切换：

```text
Ask
Plan
Change
Diagnose
```

用户从 Ask 切换到 Change 时，需要明显提示：

```text
Change 模式可以提出文件修改和命令执行请求，
所有实际操作仍需要你的批准。
```

------

# 14. AI 流式回答

回答状态：

```text
正在理解问题
正在搜索工作区
正在精排证据
正在构建上下文
等待云端披露确认
正在生成
正在验证引用
已完成
```

生成过程中，正文逐步显示，但引用只有在验证完成后才变成可点击状态。

回答卡片顶部：

```text
模型
模式
工作区
索引版本
可信等级
生成时间
Token
成本
```

底部操作：

```text
继续追问
打开全部引用
保存为知识
生成计划
创建变更任务
重新检索
切换模型
复制
导出 Markdown
报告问题
```

------

# 15. 引用交互

正文中的引用：

```text
登录失败时会更新失败次数，并在达到阈值后锁定账户。[1][2]
```

鼠标悬停：

```text
[1] AuthService.login
api-server/src/auth/service.rs:62-79
当前工作树
```

点击：

- 打开对应文件
- 跳转行号
- 高亮引用范围
- 右侧保留回答
- 如果内容已经变化，显示“引用可能已过期”

引用面板支持：

```text
按回答顺序
按文件分组
按仓库分组
仅显示弱证据
仅显示历史版本
```

------

# 16. 变更审批页面

Change Review 是独立主编辑页，不只放在 AI 侧栏中。

布局：

```text
┌──────────────────┬───────────────────────────────────┬──────────────────┐
│ 文件列表         │ Diff                              │ 变更说明         │
│                  │                                   │                  │
│ 4 Modified       │ 修改前 / 修改后                   │ AI 修改理由      │
│ 2 Added          │                                   │ 风险             │
│ 1 Deleted        │                                   │ 相关证据         │
│                  │                                   │ 验证计划         │
└──────────────────┴───────────────────────────────────┴──────────────────┘
```

文件列表状态：

```text
待审核
已批准
已拒绝
存在冲突
已应用
已回滚
```

顶部汇总：

```text
ChangeSet v2
7 个文件
+184 / -52
风险：普通
基准：commit 8f81c2
执行位置：Session Worktree
```

底部操作：

```text
全部批准
批准选中文件
拒绝选中文件
要求重新生成
仅导出 Patch
取消 Change Session
```

删除文件必须使用单独确认样式，不能和普通修改使用同一个批准按钮。

------

# 17. 命令审批页面

命令审批使用专门卡片：

```text
用途
运行认证模块测试

程序
cargo

参数
test -p auth-service

工作目录
session-worktree/api-server

风险
普通执行

超时
10 分钟

环境变量
8 个普通变量，5 个敏感变量已隐藏

网络
未提供强制隔离
```

按钮：

```text
批准本次
批准当前验证批次
拒绝
修改命令
复制命令
在外部终端运行
```

选择“修改命令”后，修改后的命令需要重新经过风险分析。

------

# 18. 底部面板

底部面板包含：

```text
Terminal
Output
Problems
Tasks
Index
Audit
```

## Terminal

第一版可以分为两类：

### DevForge Managed Terminal

由 Rust 受控执行器管理，适用于 Agent 和任务。

### External Terminal

打开系统终端，仅提供快捷入口，不宣称由 DevForge 审计。

必须在界面上明确区分。

## Output

按来源选择日志：

```text
DevForge Core
Indexer
Search
AI Provider
GitHub
GitLab
LSP
Plugin
```

## Problems

展示：

- LSP 诊断
- 索引错误
- Connector 错误
- ChangeSet 冲突
- 测试失败

## Index

展示当前工作区索引任务和失败文档。

## Audit

展示当前会话相关审计记录，不默认展示全部低级技术日志。

------

# 19. 状态栏

底部状态栏持续显示关键状态。

左侧：

```text
当前工作区
当前仓库
Git 分支
未提交文件数量
```

右侧：

```text
索引状态
Embedding 模型
Chat 模型
隐私模式
后台任务
连接器状态
错误与警告
```

示例：

```text
电商平台  |  api-server  |  feature/coupon*  |  Index 98%
Local AI  |  Cloud Blocked  |  Jobs 3  |  Warnings 1
```

点击状态项打开对应详情。

------

# 20. Command Palette

快捷键建议：

```text
Ctrl + Shift + P
```

命令类型：

```text
工作区命令
文件命令
搜索命令
AI 命令
Git 命令
任务命令
布局命令
设置命令
```

示例：

```text
Workspace: Switch Workspace
Workspace: Reindex Current Workspace
Search: Search Current Repository
AI: Ask About Current File
AI: Explain Selected Code
AI: Create Development Plan
AI: Start Change Session
Git: View Current File History
Git: Create Session Worktree
Tasks: Show Running Tasks
Layout: Toggle Assistant Panel
```

命令根据当前上下文启用或禁用。

例如没有选中代码时：

```text
AI: Explain Selected Code
```

显示禁用并注明：

```text
需要先选择一段代码
```

------

# 21. 快捷键体系

基础快捷键：

```text
Ctrl + P                 快速打开文件或符号
Ctrl + Shift + P         Command Palette
Ctrl + Shift + F         全局搜索
Ctrl + L                 聚焦顶部搜索
Ctrl + J                 切换底部面板
Ctrl + B                 切换左侧边栏
Ctrl + Alt + B           切换 AI 助手
Ctrl + Shift + A         打开 AI 输入
Ctrl + Enter             发送 AI 请求
Escape                   取消当前浮层或流式请求
```

上下文操作：

```text
Alt + F12                查看符号详情
Shift + F12              查找引用
Ctrl + Alt + G           打开 Code Graph
Ctrl + Alt + H           查看 Git 历史
Ctrl + Alt + D           打开当前 Diff
```

所有快捷键支持用户自定义。

------

# 22. 首页仪表盘

首页不是营销式欢迎页，而是开发工作入口。

```text
最近工作区
正在运行的 Agent
等待审批
索引状态
最近搜索
最近 AI 对话
最近 Change Session
连接器问题
模型状态
```

示例：

```text
继续工作

电商平台
最后打开：20 分钟前
Git：3 个未提交文件
索引：正常
Agent：1 个任务等待审批

数据平台
GitLab 认证已过期
向量索引：72%
```

首页快速操作：

```text
打开工作区
创建工作区
导入仓库
询问全局知识库
查看待审批
继续 Agent 任务
```

------

# 23. 工作区概览页

```text
工作区名称
描述
仓库数量
文档数量
符号数量
Chunk 数量
索引体积
最近更新
```

主要卡片：

```text
数据源
索引健康
语言分布
模块摘要
Git 活动
AI 活动
待处理问题
磁盘使用
```

工作区操作：

```text
添加仓库
添加文档目录
连接 GitHub
连接 GitLab
重新索引
导出工作区
归档
删除
```

------

# 24. 模型管理页面

分为：

```text
Providers
Models
Profiles
Routing
Usage
```

## Providers

展示：

```text
Ollama
LM Studio
OpenAI
Anthropic
Gemini
Compatible API
```

Provider 卡片：

```text
连接状态
地址
认证状态
平均延迟
最近错误
已发现模型
```

## Profiles

例如：

```text
本地隐私模式
日常开发模式
深度架构分析
快速问答
低成本云端
```

每个 Profile 组合：

```text
Chat Model
Fast Model
Embedding Model
Rerank Model
Fallback
隐私策略
Token 预算
```

------

# 25. 连接器管理页面

连接器列表：

```text
GitHub
GitLab
Local Directory
Local Git
未来插件连接器
```

状态：

```text
Healthy
Syncing
Rate Limited
Authentication Failed
Degraded
Disabled
```

连接器详情：

```text
连接配置
授权范围
同步资源
上次同步
下次同步
同步游标
最近错误
数据量
索引状态
审计记录
```

操作：

```text
立即同步
暂停
重新授权
修改范围
查看同步日志
清除本地快照
删除连接器
```

------

# 26. 插件管理页面

插件卡片：

```text
名称
发布者
版本
插件类型
状态
权限
最近运行
错误次数
```

安装时展示权限：

```text
该插件请求：
- 访问 api.example.com
- 使用一个凭据引用
- 写入自己的插件存储
- 向知识库提交文档
```

更新插件新增权限时，必须重新确认。

插件不能使用与系统审批页面完全相同的视觉样式，避免伪造系统确认。

------

# 27. 通知系统

通知分为：

```text
Info
Success
Warning
Error
ApprovalRequired
Security
```

普通通知使用 Toast：

```text
工作区索引完成
GitHub 同步完成
模型连接恢复
```

重要通知进入通知中心：

```text
命令等待批准
ChangeSet 等待审核
GitLab 授权失效
插件被隔离
索引检测到损坏
```

安全通知不能自动消失。

------

# 28. React 状态划分

不能把所有数据放进 Zustand。

建议分成四类状态。

## 28.1 服务端状态

虽然后台在本机，但 Rust Core 对 React 来说仍然属于服务端。

使用 TanStack Query 管理：

```text
工作区列表
文档元数据
搜索结果
对话历史
任务列表
连接器状态
模型列表
ChangeSet
Git 状态
```

负责：

- 请求缓存
- 去重
- 失效
- 重试
- Loading
- Error
- 分页
- Infinite Query

------

## 28.2 UI 状态

使用 Zustand：

```text
当前活动栏
左侧边栏宽度
右侧面板宽度
底部面板高度
当前编辑组
打开标签
固定标签
布局模式
搜索面板状态
当前选区
临时浮层
```

不要在 Zustand 中保存：

```text
完整搜索结果
完整文件内容
完整任务日志
完整 AI 对话
数据库实体全集
```

------

## 28.3 流式状态

AI Token、命令日志和索引进度属于高频状态。

建议独立维护：

```text
StreamRegistry
├─ AIStreamStore
├─ CommandOutputBuffer
├─ IndexProgressStore
└─ TaskEventBuffer
```

特点：

- 使用外部 Store
- 按 Run ID 或 Task ID 分片
- 批量提交 UI 更新
- 只保留前端可视窗口
- 完整日志仍写入 Rust 文件
- 任务完成后转为普通查询数据

避免每个 Token 都更新全局 React Context。

------

## 28.4 编辑器状态

Monaco 自己维护：

```text
光标
选区
滚动位置
折叠状态
模型内容
Diff 状态
```

React 只保存必要引用：

```text
editor_id
document_id
view_state_key
```

不要把整个文件内容复制到多个 Store 中。

------

# 29. IPC Client 层

React 不直接到处调用 `invoke()`。

统一封装：

```text
src/shared/api/
├─ client.ts
├─ commands.ts
├─ channels.ts
├─ events.ts
├─ errors.ts
└─ generated-types.ts
```

Feature 只能调用领域化 API：

```ts
workspaceApi.createWorkspace()
searchApi.searchWorkspace()
aiApi.startRun()
changeApi.approveChangeSet()
taskApi.cancelTask()
```

而不是：

```ts
invoke("some_rust_command", ...)
```

这样方便：

- 类型检查
- Mock
- 测试
- 错误处理
- 未来切换到 Daemon IPC

------

# 30. IPC 数据失效策略

Rust 状态变化后发送领域事件：

```text
workspace-updated
document-indexed
task-status-changed
change-set-updated
connector-health-changed
model-health-changed
```

React 事件处理器不直接修改大量缓存数据，而是：

```text
收到领域事件
   ↓
定位相关 Query Key
   ↓
invalidateQueries
   ↓
TanStack Query 重新获取
```

只有简单且确定的数据，可以直接使用 `setQueryData`。

例如：

```text
任务进度 61% → 62%
```

可以直接更新。

但涉及多表业务变化时，应重新查询，避免前端自己复制 Rust 业务逻辑。

------

# 31. Query Key 规范

```text
["workspaces"]
["workspace", workspaceId]
["workspace", workspaceId, "sources"]
["documents", workspaceId, filters]
["document", documentId]
["symbols", workspaceId, filters]
["search", workspaceId, queryHash]
["conversation", conversationId]
["agent-run", runId]
["change-set", changeSetId]
["tasks", workspaceId, filters]
```

禁止在不同 Feature 中随意创建字符串 Query Key。

------

# 32. 错误处理

统一错误结构：

```text
code
message
user_message
details
retryable
suggested_action
correlation_id
```

React 展示分层：

## 页面级错误

例如工作区不存在。

## 区域级错误

例如 Code Graph 加载失败，但文件仍能阅读。

## 非阻塞错误

例如后台同步失败。

## 安全阻止

例如试图读取敏感文件。

安全阻止提示需要明确：

```text
该请求被阻止，因为目标文件位于工作区外：
C:\Users\...\ .ssh\id_rsa

此限制不能由 AI 自动解除。
```

------

# 33. Loading 与骨架屏

不同操作使用不同反馈：

```text
短请求
→ 局部 Spinner

列表加载
→ Skeleton

长任务
→ 任务状态卡

未知总量
→ Indeterminate Progress

可取消操作
→ 显示取消按钮

后台操作
→ 状态栏或通知中心
```

不能让整个应用因为一个 GitHub 同步任务进入全屏 Loading。

------

# 34. 大数据量性能

## 文件树

使用：

- 虚拟列表
- 懒加载目录
- 增量展开
- Rust 后台分页

## 搜索结果

使用：

- 虚拟滚动
- 分页
- 最大可见结果
- 结果预览延迟加载

## 日志

使用：

- 环形缓冲区
- 批量更新
- 虚拟行
- 文件偏移读取

## Code Graph

使用：

- 局部图
- 按需扩展
- 节点数量限制
- Web Worker 布局

## 对话

长对话采用：

- 消息虚拟列表
- 历史分页
- 代码块延迟渲染
- Markdown 增量渲染节流

------

# 35. React 组件分层

```text
App Shell
   ↓
Feature Pages
   ↓
Feature Widgets
   ↓
Entity Components
   ↓
Shared UI
```

示例：

```text
ChangeReviewPage
├─ ChangeSummary
├─ ChangedFileList
├─ DiffViewer
├─ ChangeExplanation
├─ ApprovalToolbar
└─ ValidationPlan
```

避免一个页面文件超过数千行。

------

# 36. 组件目录示例

```text
features/change-review/
├─ api/
│  ├─ queries.ts
│  └─ mutations.ts
├─ components/
│  ├─ ChangeSummary.tsx
│  ├─ ChangedFileList.tsx
│  ├─ DiffReview.tsx
│  ├─ ApprovalToolbar.tsx
│  └─ RiskPanel.tsx
├─ hooks/
│  ├─ useChangeSet.ts
│  └─ useApproval.ts
├─ pages/
│  └─ ChangeReviewPage.tsx
├─ model/
│  ├─ store.ts
│  └─ selectors.ts
└─ types/
```

------

# 37. 主题系统

支持：

```text
Dark
Light
Follow System
High Contrast
```

主题变量：

```text
背景层级
编辑器背景
边框
主文字
次要文字
强调色
成功
警告
错误
审批
敏感信息
代码 Diff
```

Diff 颜色不能只依赖红绿，还需要：

- 图标
- 行前标记
- 文本标签
- 高对比度模式

------

# 38. 无障碍

至少保证：

- 键盘可完成主要操作
- Focus Ring 清晰
- 审批按钮语义明确
- 图标拥有文本说明
- 不只使用颜色表达状态
- 支持系统字体缩放
- 支持减少动画
- Modal 可正确锁定焦点
- Screen Reader 可以读取任务状态

高风险审批不使用含糊按钮：

```text
确定
好的
继续
```

而应使用：

```text
批准并应用 4 个文件
批准执行 cargo test
拒绝删除 3 个文件
```

------

# 39. 多窗口策略

第一版建议只提供少量独立窗口：

```text
Main Window
Change Review Window
Task Log Window
Quick Capture Window
```

## Change Review Window

适合用户在独立显示器上审查 Diff。

只拥有：

- 读取当前 ChangeSet
- 提交审批结果
- 打开来源文件

没有：

- 通用 Shell
- 插件管理
- 模型凭据管理

## Quick Capture

用于快速：

- 记录开发笔记
- 保存剪贴板内容
- 把一段内容加入工作区
- 创建稍后处理的问题

------

# 40. 工作区布局持久化

每个工作区保存：

```text
打开标签
活动标签
编辑器分组
左侧视图
右侧 AI 状态
底部面板状态
各面板大小
固定上下文
最近搜索
```

应用异常退出后可以恢复。

但以下内容不能自动恢复执行：

```text
命令
Agent Tool Call
待应用 Patch
远程写操作
```

这些只能恢复为：

```text
Interrupted
WaitingForReview
```

------

# 41. 第一版页面范围

第一版必须完成：

```text
首页
工作区创建与管理
工作区资源管理器
文件与代码查看器
全文与语义搜索
搜索结果页
符号详情
基础 Code Graph
AI 对话
AI 证据面板
Agent Plan
Change Review
命令审批
任务中心
Git 工作台
连接器管理
模型管理
设置
健康中心
通知中心
```

第一版可简化：

```text
插件市场
复杂多窗口
自定义 Dashboard
任意 UI 插件
移动端布局
团队协作界面
```

------

# 42. 典型用户流程

## 项目问答

```text
打开工作区
   ↓
Ctrl + Shift + A
   ↓
输入“登录失败后会发生什么？”
   ↓
查看检索过程
   ↓
阅读带引用回答
   ↓
点击引用打开代码
   ↓
查看调用关系
```

## 开发规划

```text
选中订单模块
   ↓
切换 Plan 模式
   ↓
输入“增加优惠券功能”
   ↓
AI 生成影响范围
   ↓
查看预计文件和风险
   ↓
保存为开发计划
```

## 代码修改

```text
从 Plan 创建 Change Session
   ↓
创建 Session Worktree
   ↓
生成 ChangeSet
   ↓
打开 Change Review
   ↓
按文件批准
   ↓
应用修改
   ↓
批准测试命令
   ↓
查看实时输出
   ↓
验证通过
   ↓
创建本地 Commit
```

## Bug 排查

```text
粘贴错误日志
   ↓
切换 Diagnose
   ↓
AI 搜索相关代码和历史
   ↓
提出诊断命令
   ↓
用户批准
   ↓
分析输出
   ↓
生成根因与修复 Patch
```

------

# 43. 推荐前端依赖边界

建议核心依赖：

```text
React
TypeScript
Vite
React Router
TanStack Query
Zustand
Monaco Editor
Radix UI
Tailwind CSS
React Hook Form
Zod
TanStack Virtual
```

图谱可选：

```text
React Flow
Cytoscape.js
Sigma.js
```

更推荐：

- 流程与任务图使用 React Flow
- 大型 Code Graph 使用 Cytoscape.js 或 Sigma.js
- 不强行使用一个图形库解决所有问题

Markdown：

```text
react-markdown
remark-gfm
rehype-sanitize
Shiki 或 Monaco Code Block
```

任何模型返回的 HTML 都必须经过过滤，不能直接使用不受控制的 `dangerouslySetInnerHTML`。

------

# 44. 前端测试策略

## 单元测试

测试：

- Query Key
- 状态选择器
- 工具函数
- Diff 状态
- 风险标签
- 命令格式化

## 组件测试

测试：

- 审批按钮
- 错误状态
- 搜索过滤器
- 引用卡片
- 任务状态
- 云端披露对话框

## 集成测试

Mock Rust IPC：

```text
创建工作区
搜索
AI 流式回答
引用打开
ChangeSet 审批
命令审批
任务取消
```

## 端到端测试

覆盖：

```text
导入仓库
等待基础索引
搜索符号
进行项目问答
创建 Change Session
批准 Patch
批准测试
完成任务
```

高风险操作必须测试：

- Approval Hash 失效
- 文件基线变化
- 命令参数变化
- 敏感文件阻止
- 工作区外路径阻止
- 任务中断恢复

------

# 45. 前端核心原则总结

DevForge 前端不应该是：

```text
左侧菜单
+
右侧表格
+
一个 AI 聊天弹窗
```

而应该是：

> 一个以工作区为核心，可以同时完成代码阅读、知识搜索、AI 推理、变更审查和任务执行的桌面开发工作台。

前端负责：

```text
展示状态
组织工作流
提供上下文
展示风险
收集用户批准
```

Rust 后台负责：

```text
业务事实
权限判断
文件操作
命令执行
索引
搜索
AI 调用
审计
```

React 永远不能成为安全边界，UI 中隐藏按钮不等于禁止操作。