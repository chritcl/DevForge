# DevForge 产品范围

## 第一版明确不做的内容

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

"不做清单"需要与功能清单同等重要。

## 产品边界

DevForge 不应第一阶段试图取代 VS Code、IDEA 或 Cursor。

更合理的定位：

> 跨仓库、跨文档、跨历史的项目知识与 AI 任务工作台。

用户仍然可以使用原有 IDE 完成日常编辑。

## 第一版实施边界

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
