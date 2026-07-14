# ADR-0007：使用 Git Worktree 隔离 Agent 修改

## 状态

Accepted

## 背景

AI Agent 在修改代码时需要与用户当前工作区隔离，避免污染用户的分支和未提交改动。需要选择隔离方案。

## 决策

默认使用 Git Worktree 为每个 Change Session 创建独立分支和工作目录。非 Git 目录使用临时副本模式。

## 备选方案

- Git Worktree 隔离（推荐）
- 临时文件副本
- 直接修改当前工作区
- Docker 容器隔离

## 原因

- Git 官方支持 Worktree 机制，允许同一仓库同时关联多个工作目录
- 不污染用户当前分支，修改天然可比较
- 易于丢弃整个 Agent 任务
- 可以并行运行多个 Change Session
- 便于生成 Commit 和导出 Patch
- 临时副本模式作为非 Git 目录的降级方案

## 后果

- 需要 Git 仓库才能使用 Worktree 模式
- 同一分支不能同时在多个 Worktree 中检出
- 未提交改动需要额外处理（Clean HEAD 或 Current Snapshot 模式）
- 大型仓库仍会产生新的工作目录，占用磁盘空间
- 需要管理 Session Worktree 的生命周期和清理
- 应用崩溃后需要恢复或清理孤立的 Worktree

## 以后何时重新评估

- 当 Git Worktree 在特定场景下出现兼容性问题时
- 当需要支持非 Git 版本控制系统时
- 当容器化方案可以提供更好的隔离且性能可接受时
- 当用户反馈 Worktree 磁盘占用过大时
