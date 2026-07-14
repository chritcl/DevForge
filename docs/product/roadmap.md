# DevForge 产品路线图

## MVP 定义

DevForge 的 MVP 不包含自主修改代码。

### MVP 必须包含

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

### MVP 不包含

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

### MVP 成功标准

1. 用户可以在二十分钟内完成安装、导入项目并提出第一个问题
2. 对固定真实项目问题，主要答案能够引用正确文件
3. 修改一个文件后，索引能在合理时间内更新
4. 关闭应用再打开，对话、工作区和索引仍然存在
5. 无云模型时，本地模式仍能完整工作
6. 一个 Provider 失败不会导致应用不可使用
7. 中型项目不会让 UI 明显卡死
8. 用户能理解 AI 为什么得出当前结论

## 版本路线

### DevForge 0.1：Local Workspace

```text
工作区
文件浏览
本地仓库
基础索引
关键词搜索
```

### DevForge 0.2：Code Intelligence

```text
Tree-sitter
符号搜索
Code Graph
LSP 增强
```

### DevForge 0.3：AI Knowledge（MVP 候选）

```text
本地与云模型
向量搜索
RAG
引用
隐私披露
```

这是第一个公开 MVP 候选。

### DevForge 0.4：Planning

```text
Plan 模式
影响分析
开发计划
测试计划
```

### DevForge 0.5：Controlled Changes

```text
Change Session
Diff 审批
Worktree
Patch
回滚
```

### DevForge 0.6：Agent Execution

```text
命令审批
验证流水线
任务时间线
本地 Commit
```

### DevForge 0.7：Connected Knowledge

```text
GitHub
GitLab
PR / Issue / Commit
历史原因分析
```

### DevForge 0.8：Extensibility

```text
WASM 插件
连接器 SDK
Parser SDK
高级模型路由
```

### DevForge 1.0

满足：

- Windows 正式签名发行
- 稳定升级
- 数据迁移稳定
- 中型仓库性能达标
- 问答引用准确
- Agent 审批与回滚可靠
- GitHub/GitLab 可用
- 诊断和恢复完整

## 项目子项目拆分

整个项目建议拆成九个相对独立的子项目：

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

依赖关系：

```text
P0 → P1 → P2 → P3 → P4 → P5
P1 ─────→ P6
P4 ─────→ P7
P5 ─────→ P8
```

## 推荐开发顺序

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

## 每阶段验收模板

每个阶段结束时，必须回答：

- **功能**：用户现在能够完成什么完整任务？
- **数据**：应用重启或异常退出后，数据是否仍然正确？
- **性能**：在 Small 和 Medium Fixture 中表现如何？
- **降级**：某个模块失效后，哪些能力仍然可用？
- **安全**：可以绕过权限或访问工作区外内容吗？
- **测试**：有哪些自动测试证明功能成立？
- **可观测性**：失败后能否通过关联 ID 和诊断中心定位？
- **升级**：新数据结构是否有 Migration？

## 技术风险排序

### 风险一：搜索效果看似丰富但不准确

**表现**：搜索结果很多，AI 回答很长，引用却不支持结论。

**应对**：早期建立固定评估集，引用优先于回答长度，每次修改检索策略都跑回归，允许回答"证据不足"。

### 风险二：Rust 模块过度拆分

**表现**：大量空 crate，一个功能修改需要改十几个包，类型循环转换。

**应对**：先模块化后独立 crate，只有边界稳定或编译隔离需要时才拆，保持 Domain 和 Infrastructure 分离即可。

### 风险三：AI Agent 过早实现

**表现**：功能看起来炫酷，文件修改不可靠，无法回滚，权限规则混乱。

**应对**：MVP 只读，Plan、Patch、Apply、Command 分阶段实现，安全策略在 Rust 中强制。

### 风险四：大型仓库性能不足

**表现**：初次索引过慢，文件树卡顿，内存过高，搜索阻塞 UI。

**应对**：分阶段可用，虚拟列表，任务优先级，增量索引，分资源并发，Medium Fixture 作为持续基准。

### 风险五：模型 Provider 差异

**表现**：Compatible API 实际不兼容，Tool Call 格式不同，Usage 缺失，流式事件异常。

**应对**：Provider Contract Tests，能力声明，原生 Provider，保守默认值，不根据模型名称盲目推断。

### 风险六：Windows 系统细节

**表现**：路径大小写，Junction，长路径，文件占用，进程树，WebView 和安装签名问题。

**应对**：Windows 从第一阶段进入 CI，Windows 路径安全测试，Job Object，安装与升级冒烟测试，不等到发布前再适配 Windows。

## 产品风险

### 功能过多但主线不清晰

主线必须始终是：导入项目 → 建立知识索引 → 找到答案 → 验证来源 → 安全完成开发任务。其他功能都应该服务于这条主线。

### 与 IDE 定位重叠

DevForge 不应第一阶段试图取代 VS Code、IDEA 或 Cursor。更合理的定位：跨仓库、跨文档、跨历史的项目知识与 AI 任务工作台。用户仍然可以使用原有 IDE 完成日常编辑。

### 本地优先导致安装复杂

应提供：首次设置向导、本地模型自动检测、Ollama 连接检测、云模型快速配置、示例工作区、索引进度说明。不要要求普通用户手工理解向量模型、Reranker 和 Tokenizer。

## 团队组织建议

### 单人开发

可以完成，但必须严格控制范围。单人第一目标应只做到 MVP：本地工作区、索引、搜索、AI 问答、引用、Windows 安装。不建议单人在 MVP 前同时推进 Agent、GitHub/GitLab、插件、多平台完整支持、团队协作。

单人实施时，应优先使用已有成熟库，避免自研：编辑器、Diff 算法、Git 实现、向量数据库核心、Markdown 解析器、LSP 协议模型、图形布局引擎。

### 3～4 人小组

推荐角色：Rust Core / 数据与索引、AI / 搜索 / RAG、React / Desktop UX、基础设施 / 测试 / 发布。部分角色可以合并。

### 6～8 人团队

推荐：1 名技术负责人、2 名 Rust 后台工程师、1 名搜索与 AI 工程师、2 名前端桌面工程师、1 名测试与自动化工程师、1 名产品设计或 UX。

### 模块所有权

即使人员不足，也应该保持代码所有权边界清晰：

- **Rust Core 负责人**：Domain、Application、Runtime、SQLite、Task Scheduler、Platform Adapter、错误与日志
- **搜索与 AI 负责人**：Tree-sitter、LSP、Tantivy、向量索引、RAG、Provider、搜索评估、引用校验
- **React 负责人**：App Shell、Workspace Explorer、Search、Monaco、AI Assistant、Diff Review、Task Center、状态管理
- **安全与 Agent 负责人**（阶段五后）：Policy Engine、Approval、PathGuard、Change Session、Command Runner、Worktree、Rollback

### 跨模块接口冻结顺序

以下接口应尽早稳定：WorkspaceId / DocumentId / TaskId、AppErrorResponse、DomainEvent、Job Model、Document Model、SearchRequest / SearchHit、SourceLocation、ModelStreamEvent。

以下接口不应过早冻结：Plugin API、Agent Tool API、Connector Write API、Team Sync Protocol、Remote Runner Protocol。后者需要真实使用后才能确定合理边界。
- **与 IDE 定位重叠**：定位为知识工作台，不取代 IDE
- **本地优先导致安装复杂**：提供首次设置向导、本地模型自动检测
