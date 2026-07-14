---
name: verify-task
description: 任务验证流程。用于在实现完成后运行完整的格式化、Lint、类型检查、测试和构建验证。确保变更不会引入回归，并报告每项检查的精确结果。
when_to_use: 当完成代码实现后需要系统性验证变更质量时使用。也适用于提交前检查、PR 前检查或确认现有代码库的健康状态。
argument-hint: "[可选：验证范围描述，如 'Rust crate' 或 '前端包']"
---

# Verify Task

## 核心原则

不得仅基于代码检查就声称验证通过。每项检查必须包含实际执行的命令和结果。

## 1. 确认变更范围

- 运行 `git diff --name-only` 确认变更文件列表。
- 按技术栈分组：
  - Rust 文件（`.rs`）
  - TypeScript/TSX 文件（`.ts`、`.tsx`）
  - 配置文件（`Cargo.toml`、`package.json`、`tsconfig.json`）
  - 迁移文件（`migrations/`）
  - 文档文件（`docs/`）
- 确定需要运行的验证命令集。

## 2. Rust 验证

对所有变更的 Rust 文件执行：

### 格式检查

```powershell
cargo fmt --check
```

失败时运行 `cargo fmt` 修复，然后重新检查。

### Lint

```powershell
cargo clippy --workspace --all-targets -- -D warnings
```

不得使用 `#[allow(clippy::...)]` 抑制警告，除非记录了具体原因。

### 类型检查

```powershell
cargo check --workspace
```

### 单元测试

```powershell
cargo test --workspace
```

记录测试数量和结果。

### 集成测试

如果变更涉及 crate 间的交互：

```powershell
cargo test --workspace --test '*'
```

## 3. 前端验证

对所有变更的 TypeScript/TSX 文件执行：

### 类型检查

```powershell
pnpm typecheck
```

### Lint

```powershell
pnpm lint
```

### 测试

```powershell
pnpm test
```

记录测试数量和结果。

### 格式检查

```powershell
pnpm exec prettier --check "apps/desktop/src/**" "packages/**/*.{ts,tsx}"
```

## 4. 跨栈验证

当变更同时涉及 Rust 和前端时：

- 确认 IPC 契约（Tauri command 签名）两端一致。
- 确认共享类型没有手动重复。
- 如有 `pnpm build` 或 `cargo build --workspace`，运行完整构建。

## 5. 迁移验证

当变更涉及数据库迁移时：

- 在空数据库上运行迁移，确认无错误。
- 确认迁移可重复运行（幂等性）。
- 确认回滚策略存在（如适用）。

## 6. Git 状态检查

```powershell
git diff --check
```

确认：

- 没有意外的空白错误。
- 没有未跟踪的临时文件。
- 没有提交生成文件、秘密、本地路径或凭据。

## 7. 报告验证结果

### 必须包含

| 检查项 | 命令 | 结果 | 备注 |
|--------|------|------|------|
| Rust 格式化 | `cargo fmt --check` | ✅ / ❌ | |
| Rust Lint | `cargo clippy ...` | ✅ / ❌ | |
| Rust 类型 | `cargo check --workspace` | ✅ / ❌ | |
| Rust 测试 | `cargo test --workspace` | ✅ N/N | |
| TS 类型 | `pnpm typecheck` | ✅ / ❌ | |
| TS Lint | `pnpm lint` | ✅ / ❌ | |
| TS 测试 | `pnpm test` | ✅ N/N | |

### 区分状态

- **验证成功** — 实际运行并通过。
- **未运行** — 列出原因（如当前环境不支持）。
- **环境不可用** — 如缺少特定工具或服务。
- **因既有原因失败** — 非本次变更引入的失败，记录已存在的问题。

### 不得出现

- "应该可以通过" — 未实际运行不得声称通过。
- "基本验证完成" — 必须列出每项检查的实际结果。
- 跳过的检查未说明原因。

## 8. 失败处理

任何检查失败时：

1. 记录完整的错误输出。
2. 判断是否为本次变更引入。
3. 如为本次变更引入，立即修复并重新验证。
4. 如为既有问题，记录并继续其余验证。
5. 不得为通过构建而削弱或重写测试。
