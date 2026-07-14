# 阶段三：搜索与代码理解

## 目标

让 DevForge 从"文件全文搜索"升级为开发者知识搜索。

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

实现关系类型：

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

- 精确符号搜索稳定
- 同名符号能够按仓库和上下文区分
- Code Graph 可以展示一跳关系
- LSP 不可用时系统能降级
- 搜索结果可以跳转至准确代码行
- 搜索评估集建立完成
- Recall、MRR 和符号命中率具有基准数据
