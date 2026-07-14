# ADR-0004：使用 Tree-sitter + LSP 进行代码理解

## 状态

Accepted

## 背景

需要从源代码中提取符号、调用关系和类型信息，支持 Code Graph 构建。

## 决策

Tree-sitter 负责基础结构解析（在代码不完整时仍可用），LSP 作为可选增强补充精确定义、引用、实现和调用层级。两者统一写入 Code Graph。

## 备选方案

- 纯正则表达式提取
- 纯 LSP
- Tree-sitter + LSP（推荐）
- 自研解析器

## 原因

- Tree-sitter 语法容错性好，适合不完整代码
- LSP 提供精确语义信息
- 两者互补，LSP 不可用时系统仍能降级工作
- Tree-sitter 支持多语言，每种语言独立 Adapter

## 后果

- 第一阶段优先支持 TypeScript/JavaScript、Rust、Python
- LSP 增强不应阻塞基础索引完成
- 需要管理 LSP 进程生命周期
- 符号证据分为四级：exact、parsed、heuristic、ai_inferred

## 以后何时重新评估

- 当 Tree-sitter 对某语言的语法支持不够准确时
- 当 LSP 协议发生重大变更时
- 当需要支持的编程语言数量超过 10 种，维护成本过高时
- 当纯 LSP 方案的启动速度和资源占用可以接受时
