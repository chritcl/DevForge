# ADR-0006：采用模块化 Rust Core 架构

## 状态

Accepted

## 背景

DevForge 的 Rust 后端需要支撑工作区管理、代码索引、搜索、AI 问答、Agent 执行、连接器同步等多种能力。需要决定 Rust 代码的组织方式。

## 决策

采用 Rust Workspace 多 crate 架构，按领域职责划分 crate。Domain 层不依赖 Tauri、SQLite 和网络；Application 层依赖 Trait 而非具体基础设施；基础设施层实现具体存储、搜索和外部集成。

## 备选方案

- 单一 crate 内按模块划分
- Rust Workspace 多 crate（推荐）
- 微服务拆分

## 原因

- 编译隔离：修改一个 crate 不需要重新编译所有代码
- 依赖方向清晰：Domain → Application → Infrastructure，防止循环依赖
- 测试友好：Domain 层可以完全不依赖 Tauri、SQLite 和网络进行测试
- 未来可演进：Rust Core 可以脱离 Tauri 独立运行，甚至拆为后台守护进程
- 第一阶段只创建必要 crate（domain、application、runtime、storage、platform、shared），随能力落地再逐步拆出 indexer、search、ai、agent、connectors

## 后果

- 需要维护 Cargo Workspace 配置和 crate 间依赖关系
- 跨 crate 共享类型需要独立的 shared crate
- 不应过早拆分：先模块化后独立 crate，只有边界稳定或编译隔离需要时才拆
- 保持 Domain 和 Infrastructure 分离即可，不需要第一天就创建二十个空 crate

## 以后何时重新评估

- 当 crate 数量超过 15 个，依赖管理变得复杂时
- 当需要将 Rust Core 拆分为独立微服务时
- 当编译时间因 Workspace 结构反而变慢时
- 当团队规模显著增大，需要更细粒度的所有权边界时
