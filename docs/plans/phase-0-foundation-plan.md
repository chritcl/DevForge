# 阶段零实施计划：应用骨架

## 目标

完成 Monorepo + Tauri + React + Rust 基础骨架，应用能够启动并展示基础信息。

## 迭代 1：应用骨架

### 工作内容

#### 1.1 Monorepo 初始化

- 初始化 pnpm workspace
- 初始化 Cargo workspace
- 创建 apps/desktop 目录结构
- 创建 packages/ui、packages/api-client、packages/shared
- 创建 crates/devforge-domain、devforge-application、devforge-runtime、devforge-storage、devforge-platform、devforge-shared

#### 1.2 Tauri 应用创建

- 使用 Tauri 2 模板创建应用
- 配置基本窗口属性
- 配置日志
- 配置数据目录

#### 1.3 React 前端基础

- 创建 React 应用外壳
- 配置 React Router
- 配置 TanStack Query
- 配置 Zustand
- 配置主题系统（Dark/Light）
- 配置 Error Boundary
- 创建基础布局（Activity Bar + Sidebar + Editor + Status Bar）
- 创建 IPC Client 层

#### 1.4 Rust Core 基础

- 创建 devforge-domain 基础类型（WorkspaceId、DocumentId 等标识类型）
- 创建 devforge-storage SQLite 连接和基础 Migration
- 创建 devforge-platform 基础适配
- 创建 devforge-shared 错误类型

#### 1.5 IPC 基础

- 定义第一个 Tauri Command：get_app_info
- 返回应用版本、数据目录、SQLite 状态
- React 端调用并展示

#### 1.6 工程质量

- 配置 Rust fmt
- 配置 Clippy
- 配置 TypeScript 类型检查
- 配置 ESLint
- 配置基础 CI（GitHub Actions）

### 交付验证

```text
启动应用
→ 查看版本信息
→ 查看数据目录
→ 查看 SQLite 状态
→ 查看基础健康状态
```

### 验收标准

- Windows 开发环境可以一条命令启动（pnpm tauri dev）
- Rust 与 TypeScript 类型同步方案确定
- CI 可以构建 Tauri 应用
- SQLite Migration 能在空数据库运行
- Rust Core 不直接依赖 React
- Domain 层不依赖 Tauri
- Release 构建可以安装和启动
