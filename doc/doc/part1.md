## 第一部分：产品模块与核心流程

项目暂定名为 **DevForge**，定位是：

> 面向开发者的本地知识库、跨项目代码检索与受控 AI 编程工作台。

它不是一个简单的 AI 聊天工具，而是一套围绕“项目上下文”组织的桌面开发环境。

------

## 1. 工作区中心

一个工作区代表一个真实的软件项目，可以包含多个资料源：

```text
电商平台工作区
├─ admin-web                 React 前端仓库
├─ api-server                Rust/Java 后端仓库
├─ deployment                部署仓库
├─ docs                      本地文档目录
├─ GitHub Issues / PR
├─ GitLab Wiki
└─ 工作区笔记
```

主要能力：

- 创建、导入、归档工作区
- 添加多个本地 Git 仓库
- 添加普通文档目录
- 配置 GitHub、GitLab 数据源
- 设置忽略目录与文件规则
- 为不同工作区选择 AI 模型
- 查看索引进度和知识库健康状态
- 跨工作区全局搜索

工作区是权限、索引、对话和任务的基本隔离单位。

------

## 2. 知识资源管理器

类似 IDE 的项目资源管理器，但展示的不只是文件。

```text
工作区
├─ Sources
│  ├─ Local Repositories
│  ├─ Documentation
│  ├─ GitHub
│  └─ GitLab
├─ Symbols
│  ├─ Classes
│  ├─ Functions
│  ├─ Interfaces
│  └─ Database Entities
├─ Knowledge
│  ├─ Architecture Notes
│  ├─ Decisions
│  ├─ AI Summaries
│  └─ Saved Answers
└─ History
   ├─ Commits
   ├─ Pull Requests
   ├─ Issues
   └─ AI Tasks
```

文件详情页可以同时查看：

- 源文件内容
- 文件摘要
- 函数、类型和导出符号
- 被谁引用
- 依赖了谁
- Git 修改历史
- 相关 Issue 和 PR
- 与当前文件有关的 AI 对话

------

## 3. 全局搜索与代码图谱

提供四种搜索模式。

### 快速搜索

类似 IDE 的 `Ctrl + P`：

- 文件名
- 路径
- 类名
- 方法名
- Git 分支
- Issue 编号
- PR 标题

### 全文搜索

搜索代码、注释、文档、提交和 Issue。

### 语义搜索

允许使用自然语言：

```text
用户登录失败后在哪里记录日志？
哪个模块负责刷新 Token？
项目中有哪些地方依赖旧版支付接口？
```

### 关系搜索

基于 Code Graph 回答：

```text
谁调用了 createOrder？
UserService 实现了哪些接口？
修改 PaymentResult 会影响哪些模块？
这个函数为什么会被执行？
```

Tree-sitter 负责在代码不完整时仍提供可用的结构分析，LSP 则补充定义、引用、实现和调用层级等精确语义，两者会统一写入 Code Graph。([Tree-sitter](https://tree-sitter.github.io/tree-sitter/))

------

## 4. AI 工作台

AI 工作台不是只有一个聊天输入框，而是由多个工作模式组成。

### 项目问答

```text
解释这个项目的登录流程。
为什么订单服务使用事件总线？
找到上传文件大小限制的实现。
```

回答必须附带本地来源：

```text
结论
依据
├─ src/auth/token.rs:45-92
├─ docs/auth-design.md
└─ PR #128
```

### 代码分析

- 解释文件
- 解释函数
- 生成调用链
- 排查异常
- 分析性能问题
- 查找潜在影响范围
- 对比两个实现方案

### 架构分析

- 自动生成项目结构摘要
- 识别模块边界
- 分析循环依赖
- 提取架构决策
- 识别废弃模块
- 生成 Mermaid 架构图

### 开发规划

用户可以提出：

```text
我要给订单模块增加优惠券功能。
```

AI 输出：

```text
需求理解
影响模块
数据库调整
接口调整
前端调整
风险
测试范围
实施步骤
```

此阶段只生成方案，不修改代码。

------

## 5. 受控变更中心

当用户要求 AI 修改代码时，会创建一个独立的 **Change Session**。

```text
AI 请求
   ↓
上下文收集
   ↓
生成修改计划
   ↓
生成 Patch
   ↓
变更预览
   ↓
用户批准
   ↓
写入文件
   ↓
执行验证
   ↓
生成执行报告
```

变更中心展示：

- 修改文件列表
- 新增、修改、删除行数
- Side-by-side Diff
- AI 的修改理由
- 影响范围
- 风险提示
- 待执行命令
- 测试结果
- 回滚入口

用户可以：

- 全部批准
- 按文件批准
- 按代码块批准
- 要求 AI 重新生成
- 拒绝修改
- 仅复制 Patch
- 创建 Git 分支后应用
- 应用后自动提交

AI 不拥有直接写文件权限。只有批准后的 `ChangeSet` 才能交给 Rust 文件执行器。

------

## 6. 任务与命令中心

所有终端命令都由 Rust 后台统一执行和监督。

```text
Task
├─ Command
├─ Working Directory
├─ Environment Policy
├─ Risk Level
├─ Timeout
├─ Output Stream
├─ Exit Code
└─ Audit Record
```

风险等级：

| 等级    | 示例                       | 行为               |
| ------- | -------------------------- | ------------------ |
| Safe    | `cargo check`、`pnpm lint` | 可一次批准整组执行 |
| Normal  | 安装依赖、生成文件         | 每个任务批准       |
| High    | 删除文件、修改 Git 历史    | 强提醒并单独批准   |
| Blocked | 访问禁止目录、读取密钥     | 默认拒绝           |

任务中心支持：

- 实时标准输出
- 标准错误输出
- 停止任务
- 超时终止
- 重新执行
- 任务依赖
- 并行测试
- 执行报告
- 输出内容检索

对于普通请求响应，React 通过 Tauri Command 调用 Rust；日志和任务输出这类连续数据通过 Channel 传输，低频状态变化使用 Event。Tauri 官方接口支持异步 Command、Event 和 Channel，可以分别覆盖这些通信场景。([Tauri](https://v2.tauri.app/develop/calling-rust/))

------

## 7. Git 工作台

第一版提供：

- 仓库状态
- 分支切换
- 文件 Diff
- Commit 浏览
- Commit 语义搜索
- AI Commit 总结
- AI 生成提交说明
- 分支差异分析
- PR 变更分析
- Issue 与代码关联
- 创建安全检查点

默认不允许 AI：

- 强制推送
- 删除远程分支
- 重写公共分支历史
- 自动合并 PR
- 自动提交敏感文件

------

## 8. 模型与隐私中心

每个工作区可以单独配置：

```text
默认对话模型
代码分析模型
Embedding 模型
重排模型
本地模型优先级
云端回退策略
最大上下文预算
允许发送的文件类型
禁止发送的目录
敏感信息过滤规则
```

模型来源：

- Ollama
- LM Studio
- OpenAI
- Anthropic
- Gemini
- OpenAI Compatible API

云模型调用之前必须经过：

```text
上下文候选
   ↓
权限过滤
   ↓
敏感信息扫描
   ↓
Token 预算裁剪
   ↓
用户策略检查
   ↓
模型请求
```

系统应支持在请求发出前查看“本次将发送给云模型的内容”。

------

## 9. React 端主要页面

```text
应用框架
├─ 首页仪表盘
├─ 工作区
│  ├─ 工作区概览
│  ├─ 资源管理器
│  ├─ 索引状态
│  └─ 工作区设置
├─ 搜索
│  ├─ 全局搜索
│  ├─ 语义搜索
│  └─ Code Graph
├─ AI 工作台
│  ├─ 对话
│  ├─ 分析任务
│  ├─ 开发计划
│  └─ 历史会话
├─ 变更中心
│  ├─ 待审批
│  ├─ Diff 审查
│  └─ 检查点
├─ 任务中心
│  ├─ 正在运行
│  ├─ 执行历史
│  └─ 输出日志
├─ Git 工作台
├─ 模型管理
├─ 连接器管理
└─ 系统设置
```

------

## 10. 用户主流程

```text
创建工作区
   ↓
添加本地仓库和文档
   ↓
Rust 后台扫描并建立索引
   ↓
Tree-sitter 提取代码结构
   ↓
可用时由 LSP 增强语义关系
   ↓
生成全文索引、向量索引和 Code Graph
   ↓
用户搜索或向 AI 提问
   ↓
混合检索构建项目上下文
   ↓
AI 返回带来源的答案
   ↓
用户要求修改代码
   ↓
AI 创建变更计划和 Patch
   ↓
用户审查并批准
   ↓
Rust 应用变更并执行测试
   ↓
生成完整任务记录和回滚点
```