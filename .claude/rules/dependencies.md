---
paths:
  - "**/Cargo.toml"
  - "Cargo.lock"
  - "**/package.json"
  - "pnpm-lock.yaml"
  - "pnpm-workspace.yaml"
  - ".npmrc"
  - "rust-toolchain.toml"
---

# 第三方依赖规则

## 选择标准

添加新依赖前必须评估：

- 维护活跃度（最近发布时间、Issue 响应速度）
- 文档质量
- Windows 支持状态
- 最低 Rust 版本 / Node 版本兼容性
- 传递依赖数量
- unsafe 代码使用情况
- 许可证兼容性
- 二进制体积影响
- 编译时间影响
- 安全历史

优先使用 Rust 标准库和现有 workspace 依赖，再考虑引入新 crate。
不得仅为避免编写小型、充分测试的辅助函数而添加依赖。
优先使用聚焦的 crate，避免大型框架依赖。
避免同时依赖两个解决同一全局关注点的 crate。
不得同时引入 `chrono` 和 `time`、多个 HTTP Client、多个异步运行时或多个错误框架，除非有 ADR 记录。

## 推荐技术栈

以下为默认选择，替换需要有理由：

| 场景 | Rust 推荐 | 前端推荐 |
|------|----------|----------|
| 序列化 | `serde` | - |
| 领域错误 | `thiserror` | - |
| 顶层错误上下文 | `anyhow` | - |
| 日志与链路 | `tracing` | - |
| 异步运行时 | `tokio` | - |
| 取消令牌 | `tokio-util` | - |
| HTTP | `reqwest` | - |
| SQLite | `sqlx` | - |
| Secret 内存包装 | `secrecy` | - |
| 临时目录 | `tempfile` | - |
| 属性测试 | `proptest` | - |
| Fuzz | `cargo-fuzz` | - |
| 编译失败测试 | `trybuild` | - |
| 数据获取 | - | TanStack Query |
| UI 状态 | - | Zustand |
| 路由 | - | React Router |
| 编辑器 | - | Monaco Editor |

## Feature 最小化

- 不得默认启用 crate 的全部 feature，只启用所需的最小 feature 集。
- Feature 变更必须在 PR 描述中说明原因。
- 不得为绕过编译问题而启用不相关的 feature。
- 当 crate 提供可选的 Windows 特定 feature 时，评估是否需要启用。

## Lockfile 规则

- Lockfile 变更必须只包含预期的依赖操作。
- 不得在实施功能时混入无关的依赖升级。
- Lockfile 必须提交到版本控制。
- 不得手动编辑 Lockfile，使用包管理器命令生成。
- 依赖升级作为独立任务执行，不与功能开发混合。

## 工具链规则

- `rust-toolchain.toml` 约束 Rust 版本和组件。
- 变更 MSRV 需要说明对所有 crate 的影响。
- CI 必须使用与 `rust-toolchain.toml` 一致的工具链。
- 不得在未更新 `rust-toolchain.toml` 的情况下使用需要更高 Rust 版本的 feature。

## JavaScript / TypeScript 依赖

- 优先使用 `pnpm` 进行包管理。
- 不得在未检查现有技术栈是否已提供该能力的情况下添加依赖。
- 重要新依赖需要在变更摘要中说明用途、维护状态、许可证影响和架构影响。
- `.npmrc` 变更需要说明对安装行为的影响。
- 不得引入与现有依赖功能重叠的新依赖。

## 新增依赖审批

显著的新依赖添加（新 crate、新的主要版本、新的全局关注点依赖）需要：

1. 说明引入原因
2. 评估上述选择标准
3. 确认无现有依赖可替代
4. 记录在变更摘要中
