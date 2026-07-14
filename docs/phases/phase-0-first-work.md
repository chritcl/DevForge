# 正式编码前的第一项工作

## 概述

正式编码前，第一项工作不是创建 React 页面，而是编写设计文档并实现一个纵向切片验证。

## 设计文档

需要先完成以下 8 份设计文档：

```text
01-product-scope.md        产品范围与不做什么
02-domain-model.md         核心领域实体与状态机
03-rust-boundaries.md      Rust Crate 划分与依赖方向
04-data-model.md           SQLite 表结构与索引设计
05-indexing-pipeline.md    文件发现、解析与索引流程
06-ipc-contract.md         Tauri Command 与事件协议
07-security-boundaries.md  路径安全、权限与审批规则
08-mvp-acceptance.md       MVP 验收标准与测试策略
```

这些文档大部分内容已经存在于 `docs/architecture/` 下的专题文档中，需要从中提炼出可执行的实现规格。

## 纵向切片

设计文档完成后，实现一个端到端纵向切片：

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

## 纵向切片验证目标

这个切片可以验证以下技术风险：

| 验证点 | 说明 |
| ------ | ---- |
| Tauri IPC | React 与 Rust 之间的类型安全通信 |
| Rust 分层 | Domain、Application、Infrastructure 依赖方向正确 |
| SQLite | Migration 能在空数据库运行，WAL 模式正常 |
| 路径安全 | PathGuard 能阻止工作区外路径访问 |
| React Query | 服务端状态管理与缓存失效 |
| Zustand 布局 | UI 状态与面板布局持久化 |
| 文件树性能 | 大型目录懒加载与虚拟列表 |
| 错误协议 | 统一错误结构从 Rust 到 React 的完整链路 |
| 日志 | 结构化日志与 Correlation ID |

## 完成标准

纵向切片完成后，应该能够：

1. 一条命令启动 Windows 开发环境
2. 创建工作区并添加至少一个本地仓库
3. 文件树正确展示目录结构
4. 点击文件可以在 Monaco 中查看源码
5. 应用重启后工作区和标签页恢复
6. CI 可以构建并运行基础测试

完成纵向切片后，再进入索引模块的正式开发。
