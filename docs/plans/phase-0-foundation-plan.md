# 阶段零实施计划：应用骨架（可执行版）

## 目标

完成 Monorepo + Tauri + React + Rust 基础骨架，应用能启动并通过类型安全 IPC 展示基础信息。

## 前置条件

- Windows 11 开发环境
- Node.js 22+、pnpm 10+
- Rust 1.96+（stable）
- Git 已配置

## 全局约束

- 所有文本文件使用 UTF-8 without BOM
- 中文注释，英文标识符
- 每个 Task 完成后独立提交
- 每个 Task 结束时运行指定验证命令，记录结果

---

## Task 1：初始化 pnpm workspace 和根配置

**依赖**：无

**目标**：建立 monorepo 根目录，pnpm workspace 可用。

### 精确文件

| 文件 | 职责 |
|------|------|
| `package.json` | 根 package.json，private: true，定义 workspace scripts |
| `pnpm-workspace.yaml` | 声明 apps/* 和 packages/* 为 workspace 成员 |
| `.npmrc` | 配置 pnpm 行为（shamefully-hoist=false, strict-peer-deps=true） |
| `.gitignore` | 补充 node_modules, dist, target 等忽略规则 |

### 根 package.json 内容

```json
{
  "name": "devforge",
  "private": true,
  "scripts": {
    "typecheck": "pnpm -r typecheck",
    "lint": "pnpm -r lint",
    "test": "pnpm -r test",
    "dev:desktop": "pnpm --filter @devforge/desktop dev",
    "build:desktop": "pnpm --filter @devforge/desktop build",
    "bindings:generate": "cargo run -p devforge-desktop --bin export_bindings"
  },
  "engines": {
    "node": ">=22.0.0",
    "pnpm": ">=10.0.0"
  },
  "packageManager": "pnpm@10.33.2"
}
```

### pnpm-workspace.yaml 内容

```yaml
packages:
  - "apps/*"
  - "packages/*"
```

### 验证命令

```powershell
pnpm install
pnpm list -r --depth 0
```

**预期结果**：pnpm 不报错，`pnpm list -r` 显示根 package。

### 提交信息

```
chore: 初始化 pnpm workspace
```

---

## Task 2：创建 Rust workspace 和最小 crate 骨架

**依赖**：Task 1

**目标**：`cargo build --workspace` 通过。只创建当前纵向切片有实际需求的 crate。

**设计决策**：
- Phase 0 只创建 3 个 crate：`devforge-application`、`devforge-storage`、`devforge-platform`
- `devforge-domain` 延后到 Phase 1，等真正的领域实体（Workspace、Document）出现时再创建
- `AppInfo`、`DbStatus` 是应用诊断 DTO，放在 `devforge-application`
- `AppMetadata`（版本、数据目录）由 Platform Adapter 提供，`DbStatus` 由 Storage Adapter 提供，`GetAppInfo` Use Case 负责组合最终 `AppInfo`
- 不提前创建无实际用途的错误抽象（`ApplicationError`、`StorageError`、`PlatformError`），等出现真实失败路径时再添加

### 精确文件

| 文件 | 职责 |
|------|------|
| `Cargo.toml` | workspace 根，定义 members、workspace.dependencies |
| `crates/devforge-application/Cargo.toml` | 应用服务 crate |
| `crates/devforge-application/src/lib.rs` | crate 入口 |
| `crates/devforge-application/src/app_info.rs` | AppInfo、AppMetadata、DbStatus 类型定义 |
| `crates/devforge-application/src/ports.rs` | Port trait 定义（AppMetadataProvider、DatabaseStatusProvider） |
| `crates/devforge-application/src/get_app_info.rs` | GetAppInfo 用例（组合 AppMetadata + DbStatus） |
| `crates/devforge-storage/Cargo.toml` | 存储 crate（Task 2 为骨架，Task 6 补充依赖） |
| `crates/devforge-storage/src/lib.rs` | crate 入口 |
| `crates/devforge-platform/Cargo.toml` | 平台适配 crate（Task 2 为骨架，Task 4 补充依赖） |
| `crates/devforge-platform/src/lib.rs` | crate 入口 |

### 根 Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/devforge-application",
    "crates/devforge-storage",
    "crates/devforge-platform",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
specta = { version = "=2.0.0-rc.25", features = ["derive"] }
async-trait = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

# 内部 crate
devforge-application = { path = "crates/devforge-application" }
devforge-storage = { path = "crates/devforge-storage" }
devforge-platform = { path = "crates/devforge-platform" }
```

**依赖延迟策略**：以下依赖推迟到实际使用的 Task 才加入 workspace.dependencies：
- `tauri`、`tauri-build`：Task 3
- `specta-typescript`、`tauri-specta`：Task 4
- `sqlx`、`tracing`、`tempfile`、`thiserror`：Task 6
- `serde_json`：出现真实用途时再加入

### devforge-application/Cargo.toml

```toml
[package]
name = "devforge-application"
version.workspace = true
edition.workspace = true

[dependencies]
async-trait = { workspace = true }
serde = { workspace = true }
specta = { workspace = true }

[dev-dependencies]
tokio = { workspace = true }
```

### devforge-application/src/lib.rs

```rust
pub mod app_info;
pub mod get_app_info;
pub mod ports;
```

### devforge-application/src/app_info.rs

```rust
use serde::Serialize;
use specta::Type;

/// 应用元数据（由 Platform Adapter 提供）
///
/// 只包含版本和数据目录，不包含数据库状态。
/// Platform Adapter 不需要感知数据库的存在。
#[derive(Debug, Clone)]
pub struct AppMetadata {
    pub version: String,
    pub data_dir: String,
}

/// 应用基础信息（诊断 DTO，IPC 输出）
///
/// 由 GetAppInfo Use Case 组合 AppMetadata + DbStatus 生成。
/// derive Type 用于 specta 自动生成 TypeScript 类型。
/// 仅派生 Serialize（IPC 输出），不派生 Deserialize。
#[derive(Debug, Clone, Serialize, Type)]
pub struct AppInfo {
    pub version: String,
    pub data_dir: String,
    pub db_status: DbStatus,
}

/// 数据库状态
///
/// `#[serde(tag = "type")]` 使 serde 生成内部标签表示：
/// `{ "type": "NotInitialized" } | { "type": "Ready", "migration_version": 1 } | ...`
/// specta 尊重 serde 标签策略，生成对应的 TypeScript tagged union。
/// 仅派生 Serialize（IPC 输出），不无意义地派生 Deserialize。
#[derive(Debug, Clone, Serialize, Default, Type)]
#[serde(tag = "type")]
pub enum DbStatus {
    #[default]
    NotInitialized,
    Ready { migration_version: u32 },
    Error { message: String },
}
```

### devforge-application/src/ports.rs

```rust
use async_trait::async_trait;
use crate::app_info::{AppMetadata, DbStatus};

/// 应用元数据查询端口
///
/// 由 Platform Adapter 实现，提供版本和数据目录。
/// 不感知数据库状态。
pub trait AppMetadataProvider: Send + Sync {
    fn metadata(&self) -> AppMetadata;
}

/// 数据库状态查询端口（异步）
///
/// 使用 async 因为 SQLx 查询天然是异步的。
#[async_trait]
pub trait DatabaseStatusProvider: Send + Sync {
    async fn status(&self) -> DbStatus;
}

/// 默认数据库状态（Task 4 使用，Task 7 替换为真实实现）
pub struct NotInitializedDbStatus;

#[async_trait]
impl DatabaseStatusProvider for NotInitializedDbStatus {
    async fn status(&self) -> DbStatus {
        DbStatus::NotInitialized
    }
}
```

### devforge-application/src/get_app_info.rs

```rust
use crate::app_info::AppInfo;
use crate::ports::{AppMetadataProvider, DatabaseStatusProvider};

/// 获取应用信息用例
///
/// 职责：组合 AppMetadataProvider（版本、数据目录）和 DatabaseStatusProvider（数据库状态），
/// 生成最终的 AppInfo DTO。
/// Platform Adapter 不需要知道 DbStatus，Storage Adapter 不需要知道版本号。
pub struct GetAppInfo<M: AppMetadataProvider, D: DatabaseStatusProvider> {
    app_metadata: M,
    db_status: D,
}

impl<M: AppMetadataProvider, D: DatabaseStatusProvider> GetAppInfo<M, D> {
    pub fn new(app_metadata: M, db_status: D) -> Self {
        Self { app_metadata, db_status }
    }

    /// 执行用例，组合元数据和数据库状态
    pub async fn execute(&self) -> AppInfo {
        let metadata = self.app_metadata.metadata();
        let db_status = self.db_status.status().await;

        AppInfo {
            version: metadata.version,
            data_dir: metadata.data_dir,
            db_status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_info::{AppMetadata, DbStatus};
    use async_trait::async_trait;

    struct MockAppMetadata;
    impl AppMetadataProvider for MockAppMetadata {
        fn metadata(&self) -> AppMetadata {
            AppMetadata {
                version: "0.1.0".into(),
                data_dir: "C:/test/DevForge".into(),
            }
        }
    }

    struct MockDbReady;
    #[async_trait]
    impl DatabaseStatusProvider for MockDbReady {
        async fn status(&self) -> DbStatus {
            DbStatus::Ready { migration_version: 1 }
        }
    }

    #[tokio::test]
    async fn get_app_info_composes_metadata_and_db_status() {
        let use_case = GetAppInfo::new(MockAppMetadata, MockDbReady);
        let info = use_case.execute().await;

        // 验证 AppMetadataProvider 提供版本和数据目录
        assert_eq!(info.version, "0.1.0");
        assert_eq!(info.data_dir, "C:/test/DevForge");

        // 验证 DatabaseStatusProvider 提供数据库状态
        assert!(matches!(
            info.db_status,
            DbStatus::Ready { migration_version: 1 }
        ));
    }

    #[tokio::test]
    async fn platform_provider_does_not_know_db_status() {
        // MockAppMetadata 不包含任何 DbStatus 字段
        // 这证明 Platform Provider 不需要知道数据库状态
        let metadata = MockAppMetadata.metadata();
        assert_eq!(metadata.version, "0.1.0");
        assert_eq!(metadata.data_dir, "C:/test/DevForge");
    }
}
```

### devforge-storage/Cargo.toml

```toml
[package]
name = "devforge-storage"
version.workspace = true
edition.workspace = true

[dependencies]
# Task 6 补充：sqlx, tokio, tracing, thiserror, async-trait, devforge-application
```

Task 2 只创建 Workspace 边界骨架，不添加未使用依赖和占位错误。`StorageError` 在 Task 6 首次出现真实 SQLx 失败时创建。

### devforge-storage/src/lib.rs

```rust
// Task 6 补充模块：error, pool, migrator, status
```

### devforge-platform/Cargo.toml

```toml
[package]
name = "devforge-platform"
version.workspace = true
edition.workspace = true

[dependencies]
# Task 4 补充：devforge-application
```

Task 2 只创建 Workspace 边界骨架。`PlatformError` 在真正存在可恢复的平台失败路径时创建。

### devforge-platform/src/lib.rs

```rust
// Task 4 补充模块：app_info
```

### 验证命令

```powershell
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

**预期结果**：全部通过，0 error, 0 warning。

### 提交信息

```
feat(rust): 创建 Rust workspace 和最小 crate 骨架（无 domain 层）
```

---

## Task 3：创建最小 Tauri + React 应用

**依赖**：Task 1, Task 2

**目标**：`pnpm tauri dev` 能启动空白窗口。

### 精确文件

| 文件 | 职责 |
|------|------|
| `Cargo.toml` | 根 Cargo.toml，members 增加 `apps/desktop/src-tauri` |
| `apps/desktop/package.json` | 前端包定义，name: @devforge/desktop |
| `apps/desktop/tsconfig.json` | TypeScript 配置 |
| `apps/desktop/tsconfig.node.json` | Node 侧 TS 配置 |
| `apps/desktop/vite.config.ts` | Vite 配置（固定端口 1420） |
| `apps/desktop/index.html` | HTML 入口 |
| `apps/desktop/src/main.tsx` | React 入口 |
| `apps/desktop/src/App.tsx` | 根组件（占位） |
| `apps/desktop/src/vite-env.d.ts` | Vite 类型声明 |
| `apps/desktop/src-tauri/Cargo.toml` | Tauri 宿主 crate |
| `apps/desktop/src-tauri/build.rs` | Tauri 构建脚本 |
| `apps/desktop/src-tauri/tauri.conf.json` | Tauri 配置 |
| `apps/desktop/src-tauri/src/main.rs` | Tauri 入口 |
| `apps/desktop/src-tauri/src/lib.rs` | Tauri lib 入口 |
| `apps/desktop/src-tauri/capabilities/default.json` | 默认 capability |
| `apps/desktop/src-tauri/icons/` | 应用图标（Tauri 生成） |

### apps/desktop/package.json

```json
{
  "name": "@devforge/desktop",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "typecheck": "tsc --noEmit",
    "tauri": "tauri"
  },
  "dependencies": {
    "react": "^19.1.0",
    "react-dom": "^19.1.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@types/node": "^22",
    "@types/react": "^19",
    "@types/react-dom": "^19",
    "@vitejs/plugin-react": "^4",
    "typescript": "~5.8",
    "vite": "^7"
  }
}
```

**依赖说明**：
- `@tauri-apps/api` 推迟到 Task 4（Specta 生成的绑定实际调用 `@tauri-apps/api/core`）
- `react-router` 推迟到 Task 8（Router）
- `vitest` 推迟到 Task 9（测试基础设施）
- `@types/node` 提供 `process.env` 类型（`vite.config.ts` 使用）
- Task 3 只包含实际使用的依赖

### apps/desktop/tsconfig.json

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedSideEffectImports": true
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

### apps/desktop/tsconfig.node.json

```json
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true,
    "types": ["node"]
  },
  "include": ["vite.config.ts"]
}
```

**配置说明**：
- `tsconfig.node.json` 专门用于 `vite.config.ts`
- `"types": ["node"]` 确保 `process.env` 类型可用
- `composite: true` 支持 TypeScript 项目引用

### apps/desktop/src-tauri/Cargo.toml

```toml
[package]
name = "devforge-desktop"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
publish = false

[lib]
name = "devforge_desktop_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
```

**依赖延迟策略**：
- `serde`：Task 4（IPC 序列化）
- `devforge-application`：Task 4（Application Use Case）
- `devforge-platform`：Task 4（Platform Adapter）
- Task 3 的 Tauri runtime 只需要 `tauri` 本身

**crate-type 说明**：
- `staticlib`：Windows 静态库（Tauri Windows 构建必需）
- `cdylib`：动态库（Tauri 插件系统使用）
- `rlib`：Rust 标准库（cargo test 使用）

### apps/desktop/src-tauri/capabilities/default.json

```json
{
  "identifier": "default",
  "description": "默认窗口能力",
  "windows": ["main"],
  "permissions": [
    "core:default"
  ]
}
```

### apps/desktop/src-tauri/src/lib.rs

```rust
#![forbid(unsafe_code)]

/// Tauri 应用入口（Phase 0 最小版本）
///
/// Task 4 会添加 commands 和 state。
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
```

### apps/desktop/src-tauri/src/main.rs

```rust
#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    devforge_desktop_lib::run();
}
```

### 根 Cargo.toml 修改

在 Task 2 基础上，将 `apps/desktop/src-tauri` 加入 Cargo Workspace members：

```toml
[workspace]
resolver = "2"
members = [
    "crates/devforge-application",
    "crates/devforge-storage",
    "crates/devforge-platform",
    "apps/desktop/src-tauri",
]
```

### apps/desktop/src-tauri/build.rs

```rust
fn main() {
    tauri_build::build()
}
```

### apps/desktop/vite.config.ts

```typescript
import process from "node:process";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
});
```

**关键配置**：
- `port: 1420` + `strictPort: true`：固定端口，与 `tauri.conf.json` 的 `devUrl` 一致
- `watch.ignored: ["**/src-tauri/**"]`：避免 Rust 文件变更触发前端全量刷新
- `TAURI_DEV_HOST`：支持 Tauri 远程开发模式
- `import process from "node:process"`：显式导入，不依赖未声明的全局类型

### apps/desktop/src-tauri/tauri.conf.json

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "DevForge",
  "version": "0.1.0",
  "identifier": "com.devforge.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build"
  },
  "app": {
    "windows": [
      {
        "title": "DevForge",
        "width": 1280,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {}
  }
}
```

### 验证命令

```powershell
pnpm install
pnpm --filter @devforge/desktop typecheck
pnpm --filter @devforge/desktop build
cargo check -p devforge-desktop
pnpm --filter @devforge/desktop tauri dev
```

### 验证要求

- [ ] TypeScript 类型检查通过（`pnpm typecheck`）
- [ ] Vite 生产构建通过（`pnpm build`）
- [ ] Tauri Rust crate 编译通过（`cargo check -p devforge-desktop`）
- [ ] 窗口成功启动并显示 React 占位页
- [ ] Vite 运行在固定端口 1420
- [ ] Ctrl+C 后前端开发服务器和 Tauri 进程都正常退出
- [ ] 不创建 Command、State、IPC bindings、SQLite 或 Router
- [ ] 不提前执行 Task 4
- [ ] 所有新增依赖均有实际使用者，无未来任务的提前依赖

### 提交信息

```
feat(desktop): 创建最小 Tauri + React 应用
```

---

## Task 4：建立 IPC 层 — get_app_info 命令（specta 类型生成）

**依赖**：Task 2, Task 3

**目标**：建立 `get_app_info` Tauri Command 和 Rust → TypeScript bindings 生成管线，证明生成的 `commands.getAppInfo()` 可以调用后端并返回 `AppInfo`。React 页面展示和字段消费留到 Task 5。

**架构决策**：
- Tauri Command 只调用 Application Use Case，不自行构造业务逻辑
- TypeScript 类型由 specta 从 Rust `#[derive(Type)]` 自动生成，不是手写 DTO
- `apps/desktop/src/bindings.ts` 必须完全由 Specta 生成，不得手写或修改
- 前端通过 specta 生成的绑定调用命令，类型在编译期保证一致
- `PlatformMetadata` 接受版本号和数据目录，版本号由 Composition Root 注入，不在 platform crate 中使用 `env!("CARGO_PKG_VERSION")`
- `AppState` 只保存当前实际使用的 Application Use Case，不提前保存未来字段
- 启动和诊断路径使用 `anyhow::Result` 和 `anyhow::Context`，不使用 `expect()`

### 调用链

```text
Tauri get_app_info Command（#[specta::specta]）
        ↓
GetAppInfo Application Use Case
        ↓
AppMetadataProvider + DatabaseStatusProvider
        ↓
PlatformMetadata    + NotInitializedDbStatus
        ↓
AppInfo（组合后的最终 DTO）
        ↓
specta 自动生成 TypeScript 类型和命令绑定
```

> **注意**：Task 4 使用 `NotInitializedDbStatus` 占位实现，真实的 Storage Adapter（`SqliteDatabaseStatus`）到 Task 7 才接入。

不得让 Tauri Command、Platform Adapter 或 Storage Adapter 自行组合最终 `AppInfo`。

### 类型生成流程

```text
devforge-application/src/app_info.rs
  #[derive(specta::Type)] on AppInfo, DbStatus
        ↓
apps/desktop/src-tauri/src/commands.rs
  #[specta::specta] on get_app_info
        ↓
apps/desktop/src-tauri/src/lib.rs
  Builder::new().commands(collect_commands![...]).export() → src/bindings.ts
        ↓
前端直接 import { commands } from "./bindings"
  类型安全，无需手写 invokeCommand<T>()
```

### 精确文件

| 文件 | 职责 |
|------|------|
| `Cargo.toml` | 根 Cargo.toml，补充 workspace.dependencies |
| `Cargo.lock` | cargo 更新 |
| `crates/devforge-platform/Cargo.toml` | 添加 devforge-application 依赖 |
| `crates/devforge-platform/src/lib.rs` | 添加安全属性、导出 app_info 模块 |
| `crates/devforge-platform/src/app_info.rs` | PlatformMetadata 实现 AppMetadataProvider |
| `apps/desktop/src-tauri/Cargo.toml` | 合并补充 specta、anyhow 等依赖 |
| `apps/desktop/src-tauri/src/commands.rs` | Tauri Command 定义（带 specta 注解） |
| `apps/desktop/src-tauri/src/state.rs` | 管理 Application Service |
| `apps/desktop/src-tauri/src/lib.rs` | 注册 command、state、specta 导出 |
| `apps/desktop/src-tauri/src/main.rs` | Tauri 入口，保留安全属性 |
| `apps/desktop/src-tauri/src/bin/export_bindings.rs` | 独立绑定生成入口 |
| `apps/desktop/src/bindings.ts` | specta 生成的类型和命令绑定（提交到 git） |
| `apps/desktop/package.json` | 添加 `@tauri-apps/api` 依赖 |
| `pnpm-lock.yaml` | pnpm install 更新 |

### crates/devforge-platform/src/app_info.rs

```rust
use std::path::PathBuf;
use devforge_application::ports::AppMetadataProvider;
use devforge_application::app_info::AppMetadata;

/// 平台元数据提供者
///
/// 只提供版本和数据目录，不感知数据库状态。
/// version 和 data_dir 均由 Composition Root 注入，
/// 不在 Provider 内部自行决定版本号或重新调用 dirs::data_local_dir()。
pub struct PlatformMetadata {
    version: String,
    data_dir: PathBuf,
}

impl PlatformMetadata {
    pub fn new(version: String, data_dir: PathBuf) -> Self {
        Self { version, data_dir }
    }
}

impl AppMetadataProvider for PlatformMetadata {
    fn metadata(&self) -> AppMetadata {
        AppMetadata {
            version: self.version.clone(),
            data_dir: self.data_dir.to_string_lossy().into_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injected_version_and_data_dir_are_returned() {
        let meta = PlatformMetadata::new(
            "1.2.3".to_owned(),
            PathBuf::from("C:/Users/test/AppData/Local/DevForge"),
        );
        let info = meta.metadata();
        assert_eq!(info.version, "1.2.3");
        assert_eq!(info.data_dir, "C:/Users/test/AppData/Local/DevForge");
    }
}
```

### crates/devforge-platform/Cargo.toml（补充依赖）

```toml
[dependencies]
devforge-application = { workspace = true }
```

### crates/devforge-platform/src/lib.rs（更新）

```rust
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod app_info;
```

### apps/desktop/src-tauri/src/state.rs

```rust
use std::path::PathBuf;
use devforge_application::app_info::AppInfo;
use devforge_application::get_app_info::GetAppInfo;
use devforge_platform::app_info::PlatformMetadata;
use devforge_application::ports::NotInitializedDbStatus;

/// 应用全局状态
///
/// 持有 Application Use Case，Tauri Command 通过窄接口调用业务逻辑。
/// 不暴露公开字段，Command 不直接访问内部结构。
pub struct AppState {
    get_app_info: GetAppInfo<PlatformMetadata, NotInitializedDbStatus>,
}

impl AppState {
    pub fn new(version: String, data_dir: PathBuf) -> Self {
        let platform_metadata = PlatformMetadata::new(version, data_dir);
        let db_status = NotInitializedDbStatus;

        Self {
            get_app_info: GetAppInfo::new(platform_metadata, db_status),
        }
    }

    pub async fn app_info(&self) -> AppInfo {
        self.get_app_info.execute().await
    }
}
```

### apps/desktop/src-tauri/src/commands.rs

```rust
use tauri::{AppHandle, Manager};
use devforge_application::app_info::AppInfo;
use crate::state::AppState;

/// 获取应用信息
///
/// AppHandle 由 Tauri 注入，不出现在生成的 TypeScript 参数中。
/// AppState 已由 Composition Root 注册为 managed state。
///
/// 使用 AppHandle 而非 State<'_, AppState>，是因为 Tauri 要求
/// 包含带生命周期引用参数的异步 Command 必须返回 Result，
/// 而此 Command 运行时不会失败，不应引入虚假错误类型。
/// AppHandle 不带生命周期参数，可直接返回 AppInfo。
#[tauri::command]
#[specta::specta]
pub async fn get_app_info(app: AppHandle) -> AppInfo {
    app.state::<AppState>().app_info().await
}
```

### apps/desktop/src-tauri/src/lib.rs（更新）

```rust
#![forbid(unsafe_code)]

mod commands;
mod state;

use anyhow::Context;
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};
use state::AppState;

/// 创建 specta Builder（绑定生成和 run 共用同一个 Builder）
fn create_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::get_app_info,
        ])
}

/// 导出 specta 生成的 TypeScript 绑定
///
/// 输出路径基于 CARGO_MANIFEST_DIR 构造，不依赖进程当前工作目录。
/// 输出文件：apps/desktop/src/bindings.ts
/// 该文件提交到 git，前端直接 import 使用。
///
/// # Errors
///
/// 当无法导出 Specta TypeScript bindings 时返回错误。
pub fn export_bindings() -> anyhow::Result<()> {
    let builder = create_builder();
    let out_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("src")
        .join("bindings.ts");
    builder
        .export(Typescript::default(), &out_path)
        .context("无法导出 TypeScript bindings")?;
    Ok(())
}

/// 启动 Tauri 应用
///
/// Composition Root 统一解析 data_dir 和版本号，注入 PlatformMetadata。
///
/// # Errors
///
/// 当无法解析本地数据目录或无法启动 Tauri 应用时返回错误。
pub fn run() -> anyhow::Result<()> {
    let builder = create_builder();

    let data_dir = dirs::data_local_dir()
        .context("无法解析本地数据目录")?
        .join("DevForge");
    let app_version = env!("CARGO_PKG_VERSION").to_owned();
    let app_state = AppState::new(app_version, data_dir);

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .context("无法启动 Tauri 应用")?;

    Ok(())
}
```

### apps/desktop/src-tauri/src/main.rs

```rust
#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> anyhow::Result<()> {
    devforge_desktop_lib::run()
}
```

### apps/desktop/src-tauri/src/bin/export_bindings.rs（新增）

独立的绑定生成入口，供 CI 和本地开发使用。输出路径基于 `CARGO_MANIFEST_DIR`，不依赖 cwd。

```rust
/// 独立绑定生成入口
///
/// 运行方式：cargo run -p devforge-desktop --bin export_bindings
/// 输出：apps/desktop/src/bindings.ts（基于 CARGO_MANIFEST_DIR 构造）
fn main() -> anyhow::Result<()> {
    devforge_desktop_lib::export_bindings()?;
    println!("绑定已导出");
    Ok(())
}
```

### apps/desktop/package.json（补充依赖）

在 Task 3 基础上，`dependencies` 中增加：

```json
"@tauri-apps/api": "^2"
```

原因：Specta 生成的绑定会 `import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core"`，必须声明为直接依赖。

### 根 Cargo.toml（补充 workspace.dependencies）

```toml
[workspace.dependencies]
# 已有：serde, specta, async-trait, tokio
# Task 4 新增：
specta-typescript = "=0.0.12"
tauri-specta = { version = "=2.0.0-rc.25", features = ["typescript"] }
```

### apps/desktop/src-tauri/Cargo.toml（合并更新）

以下依赖合并到 Task 3 已有的 `[dependencies]` 表中，不得创建第二个重复的 `[dependencies]` 段：

```toml
[dependencies]
# Task 3 已有：tauri = { version = "2", features = [] }
# Task 4 新增：
anyhow = "1"
dirs = "6"
devforge-application = { workspace = true }
devforge-platform = { workspace = true }
specta = { workspace = true }
specta-typescript = { workspace = true }
tauri-specta = { workspace = true }
```

### apps/desktop/src/bindings.ts（specta 生成）

此文件完全由 specta 自动生成，提交到 git 作为类型契约。前端直接 import 使用。**不得手写或修改此文件。**

实际文件由 `pnpm bindings:generate`（即 `cargo run -p devforge-desktop --bin export_bindings`）显式生成。生成的绑定会导入：

```typescript
import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";
```

因此 `apps/desktop/package.json` 的 `dependencies` 中必须包含 `@tauri-apps/api`。如果生成结果与本文档示意不同，以 specta 输出为准。

### 验证命令

从仓库根目录依次运行：

```powershell
pnpm install
cargo test -p devforge-application
cargo test -p devforge-platform
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p devforge-desktop
pnpm bindings:generate
pnpm --filter @devforge/desktop typecheck
pnpm --filter @devforge/desktop build
pnpm dev:desktop
```

### 人工验证

1. `apps/desktop/src/bindings.ts` 由 Specta 生成
2. bindings 导入 `@tauri-apps/api/core`
3. 没有 `window.__TAURI__`
4. 在 Tauri 开发者工具中执行：
   ```javascript
   const { commands } = await import("/src/bindings.ts");
   await commands.getAppInfo();
   ```
5. 返回值包含：
   - `version = 0.1.0`
   - `data_dir` 以 `DevForge` 结尾
   - `db_status.type = NotInitialized`
6. Ctrl+C 后相关进程正常退出

### 提交信息

```
feat(ipc): 使用 specta 建立 Rust → TypeScript 类型生成管线
```

---

## Task 5：React 展示 AppInfo 和健康状态

**依赖**：Task 4

**目标**：应用启动后自动获取并展示 AppInfo，类型来自 specta 生成的绑定。

### 精确文件

| 文件 | 职责 |
|------|------|
| `apps/desktop/src/App.tsx` | 根组件，展示应用信息 |
| `apps/desktop/src/components/HealthStatus.tsx` | 健康状态组件 |
| `apps/desktop/src/hooks/useAppInfo.ts` | AppInfo 查询 hook |
| `apps/desktop/src/queryKeys.ts` | 集中式类型化 Query Key 工厂 |
| `apps/desktop/src/main.tsx` | 更新：挂载 QueryClientProvider |
| `apps/desktop/package.json` | 添加 @tanstack/react-query 依赖 |
| `pnpm-lock.yaml` | pnpm install 更新 |

### 类型来源

所有 TypeScript 类型从 `src/bindings.ts` 导入，由 specta 从 Rust 自动生成。
不使用手写 DTO，不使用 `invokeCommand<T>()` 类型断言。

### 依赖安装

从仓库根目录执行，固定主版本号：

```powershell
pnpm --filter @devforge/desktop add @tanstack/react-query@^5
```

不得提前添加 TanStack Query Devtools、Vitest、Testing Library、Router、Zustand 或 CSS 框架。
测试基础设施由 Task 9 建立。

### apps/desktop/src/queryKeys.ts

```typescript
/**
 * 集中式类型化 Query Key 工厂
 *
 * 所有 Query Key 必须通过此工厂创建，确保：
 * - 全局唯一，避免 key 冲突
 * - 类型安全，IDE 自动补全
 * - 重命名时全局编译报错
 */
export const appKeys = {
  all: ["app"] as const,
  info: () => [...appKeys.all, "info"] as const,
};
```

Task 5 所有 AppInfo 查询必须使用 `appKeys.info()`，不得散落手写字符串数组。

### apps/desktop/src/hooks/useAppInfo.ts

```typescript
import { useQuery } from "@tanstack/react-query";

import { commands } from "../bindings";
import { appKeys } from "../queryKeys";

export function useAppInfo() {
  return useQuery({
    queryKey: appKeys.info(),
    queryFn: commands.getAppInfo,
    // 本地 Tauri IPC，不依赖互联网
    networkMode: "always",
    // 配置或 IPC 错误不应盲目重试
    retry: false,
    staleTime: 30_000,
  });
}
```

要求：
- 不显式写 `useQuery<AppInfo>`，类型完全从 Specta 生成的 Command 推导；
- 不重复导入 `type AppInfo`；
- 不使用类型断言；
- 不包装成手写 Promise 类型；
- 必须使用 `networkMode: "always"`，因为这是本地 Tauri IPC，不依赖互联网。

### apps/desktop/src/components/HealthStatus.tsx

```typescript
import type { DbStatus } from "../bindings";

interface HealthStatusProps {
  dbStatus: DbStatus;
}

function assertNever(value: never): never {
  throw new Error(`未知数据库状态：${JSON.stringify(value)}`);
}

function getStatusLabel(dbStatus: DbStatus): string {
  switch (dbStatus.type) {
    case "NotInitialized":
      return "未初始化";
    case "Ready":
      return `就绪（migration v${dbStatus.migration_version}）`;
    case "Error":
      return `错误：${dbStatus.message}`;
    default:
      return assertNever(dbStatus);
  }
}

export function HealthStatus({ dbStatus }: HealthStatusProps) {
  return (
    <span role="status" data-status={dbStatus.type}>
      {getStatusLabel(dbStatus)}
    </span>
  );
}
```

要求：
- 使用穷举 `switch` 处理 `DbStatus`，不使用连续三元表达式；
- 新增 Rust 状态变体后必须触发 TypeScript 编译错误（通过 `assertNever`）；
- 不添加 `data-testid`；
- 使用语义化 `role="status"`。

### apps/desktop/src/App.tsx

```typescript
import { useAppInfo } from "./hooks/useAppInfo";
import { HealthStatus } from "./components/HealthStatus";

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  try {
    return JSON.stringify(error) ?? "未知错误";
  } catch {
    return "未知错误";
  }
}

export default function App() {
  const appInfoQuery = useAppInfo();

  if (appInfoQuery.isPending) {
    return <div>加载中...</div>;
  }

  if (appInfoQuery.isError) {
    return (
      <div role="alert">
        <p>加载应用信息失败：{getErrorMessage(appInfoQuery.error)}</p>
        <button
          type="button"
          disabled={appInfoQuery.isFetching}
          onClick={() => void appInfoQuery.refetch()}
        >
          {appInfoQuery.isFetching ? "正在重试..." : "重试"}
        </button>
      </div>
    );
  }

  const data = appInfoQuery.data;

  return (
    <div style={{ padding: 24, fontFamily: "sans-serif" }}>
      <h1>DevForge</h1>
      <p>开发者知识库与 AI 工作台</p>
      <dl>
        <dt>版本</dt>
        <dd>{data.version}</dd>
        <dt>数据目录</dt>
        <dd>{data.data_dir}</dd>
        <dt>数据库状态</dt>
        <dd><HealthStatus dbStatus={data.db_status} /></dd>
      </dl>
    </div>
  );
}
```

要求：
- 查询状态必须显式处理：`isPending`、`isError`、成功；
- 错误状态使用 `role="alert"` 和重试按钮；
- 不得保留 `if (!data) return null`；
- 不得产生无提示空白页面。

### apps/desktop/src/main.tsx（更新）

```typescript
import React from "react";
import ReactDOM from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import App from "./App";

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>
  </React.StrictMode>,
);
```

要求：
- 在模块级创建唯一 QueryClient，不得在 React 组件内部创建；
- 保留 `React.StrictMode`。

### 验证命令

从仓库根目录执行：

```powershell
pnpm install
pnpm bindings:generate
git diff --exit-code -- apps/desktop/src/bindings.ts
pnpm --filter @devforge/desktop typecheck
pnpm --filter @devforge/desktop build
cargo check -p devforge-desktop
pnpm dev:desktop
```

### 人工验证

确认：
1. 窗口显示 `DevForge`；
2. 显示"开发者知识库与 AI 工作台"；
3. 版本显示为 `0.1.0`；
4. 数据目录以 `DevForge` 结尾；
5. 数据库状态显示"未初始化"；
6. 页面没有空白状态；
7. 控制台没有 React、TanStack Query 或 IPC 异常；
8. 不出现 `typedError` 包装；
9. Ctrl+C 后 Tauri 和 Vite 进程正常退出。

### 提交信息

```
feat(ui): React 使用 specta 生成的绑定展示 AppInfo
```

---

## Task 6：SQLite Bootstrap 和第一条 Migration

**依赖**：Task 2

**目标**：`devforge-storage` 能使用 SQLx 打开本地 SQLite 文件，以确定性配置创建连接池，执行嵌入式 Migration，并通过 `SqliteDatabaseStatus` 返回真实数据库健康状态。

**范围约束**：Task 6 只负责 Storage crate 的独立能力。不得修改 Tauri AppState、Tauri Command、React，不得接入桌面应用启动流程，不得提前执行 Task 7，不得创建 Workspace 等 Phase 1 领域结构。

**架构决策**：使用 SQLx + SqlitePool（符合架构文档规定），而非 rusqlite 单连接模型。SqlitePool 可克隆、支持异步、天然适合 Tauri State 持有。

### 精确文件

| 文件 | 职责 |
|------|------|
| `Cargo.toml` | 根 Cargo.toml，补充 workspace.dependencies |
| `Cargo.lock` | cargo 更新 |
| `.gitattributes` | 强制 SQL 文件 LF 换行 |
| `crates/devforge-storage/Cargo.toml` | 更新依赖 |
| `crates/devforge-storage/build.rs` | Migration 构建跟踪 |
| `crates/devforge-storage/src/lib.rs` | 导出模块 |
| `crates/devforge-storage/src/error.rs` | StorageError（thiserror，保留原始错误链） |
| `crates/devforge-storage/src/pool.rs` | SQLite 连接池管理 |
| `crates/devforge-storage/src/migrator.rs` | Migration 运行器（静态 Migrator） |
| `crates/devforge-storage/src/status.rs` | SqliteDatabaseStatus 实现 |
| `crates/devforge-storage/migrations/0001_create_app_meta.sql` | 第一条 migration |
| `crates/devforge-storage/tests/sqlite_bootstrap.rs` | 集成测试 |

### 根 Cargo.toml 补充 workspace.dependencies

```toml
sqlx = {
    version = "=0.9.0",
    default-features = false,
    features = [
        "runtime-tokio",
        "sqlite-bundled",
        "macros",
        "migrate",
    ],
}
thiserror = "2"
tempfile = "3"
```

使用 SQLx `0.9.0`，因为项目 Rust `1.96` 满足 SQLx `0.9.0` 的 Rust `1.94` 最低要求。

不得使用 `features = ["sqlite"]`，因为 SQLx 0.9 的 `sqlite` 总功能会启用当前不需要的扩展加载等能力。只启用最小的 `sqlite-bundled`。

### crates/devforge-storage/Cargo.toml

```toml
[dependencies]
async-trait = { workspace = true }
devforge-application = { workspace = true }
sqlx = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
tokio = { workspace = true }
```

正常生产代码不直接使用 Tokio API，因此 `tokio` 仅作为测试依赖。不得添加未使用的 `tracing`。

### .gitattributes

```gitattributes
*.sql text eol=lf
```

防止 Windows CRLF 与 Linux LF 造成 SQLx migration 哈希不一致。

### crates/devforge-storage/build.rs

```rust
#![forbid(unsafe_code)]

fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
```

确保仅新增或修改 migration 文件时，Cargo 也会重新编译嵌入式 migration。

### migrations/0001_create_app_meta.sql

```sql
CREATE TABLE app_meta (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

要求：
- 不创建 `workspace` 表，不创建任何 Phase 1 领域表；
- 不插入 `schema_version`，schema version 由 `_sqlx_migrations` 管理；
- 不使用 `CREATE TABLE IF NOT EXISTS`，不使用 `INSERT OR IGNORE`；
- migration 冲突必须明确失败，不能被静默掩盖；
- SQL 文件必须使用 LF 换行。

### crates/devforge-storage/src/lib.rs

```rust
#![forbid(unsafe_code)]

pub mod error;
pub mod migrator;
pub mod pool;
pub mod status;
```

不得导出测试辅助代码。

### crates/devforge-storage/src/error.rs

```rust
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("无法打开 SQLite 数据库：{path}")]
    OpenDatabase {
        path: PathBuf,
        #[source]
        source: sqlx::Error,
    },

    #[error("无法执行 SQLite migration")]
    Migration {
        #[source]
        source: sqlx::migrate::MigrateError,
    },

    #[error("无法读取 SQLite schema 版本")]
    SchemaVersion {
        #[source]
        source: sqlx::Error,
    },

    #[error("migration 版本超出应用支持范围：{version}")]
    MigrationVersionOutOfRange {
        version: i64,
    },
}
```

要求：
- 不使用 `Connection(String)`、`Migration(String)`、`Query(String)`；
- 不丢失 `sqlx::Error` 或 `MigrateError` 的 source；
- 只有转换为最终用户诊断 DTO 时才允许调用 `to_string()`。

### crates/devforge-storage/src/pool.rs

```rust
use std::path::Path;
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

use crate::error::StorageError;

/// SQLite 连接池管理
///
/// 使用 SqlitePool 实现连接池化，支持并发访问和异步操作。
/// SqlitePool 可克隆，适合放入 Tauri State 供多个 Command 共享。
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// 打开数据库并配置 SQLite 参数
    ///
    /// 通过 SqliteConnectOptions 配置，确保连接池中每个连接都使用一致的配置：
    /// - WAL 模式：提升并发读写性能
    /// - foreign_keys：启用外键约束
    /// - busy_timeout：避免锁竞争时立即失败（通过 C API 设置，非 PRAGMA）
    /// - create_if_missing：数据库文件不存在时自动创建
    ///
    /// # Errors
    ///
    /// 当数据库文件无法打开、路径无效或连接池创建失败时返回错误。
    pub async fn open(db_path: &Path) -> Result<Self, StorageError> {
        let opts = SqliteConnectOptions::new()
            .filename(db_path)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5))
            .foreign_keys(true)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect_with(opts)
            .await
            .map_err(|source| StorageError::OpenDatabase {
                path: db_path.to_path_buf(),
                source,
            })?;

        Ok(Self { pool })
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
```

要求：
- `busy_timeout` 处理 SQLite 文件锁等待；
- `acquire_timeout` 处理连接池获取连接等待；
- 不使用 `connect_lazy_with`，打开时必须至少建立一个真实连接；
- `Database::open()` 不负责创建父目录，父目录由 Composition Root 在 Task 7 创建；
- 错误中保存数据库路径和原始 `sqlx::Error`；
- `Database::open()` Rustdoc 增加 `# Errors`；
- 保留 `#![forbid(unsafe_code)]`。
- 测试统一使用 `tempfile` 创建文件数据库，不得使用多连接的普通 `:memory:` 数据库。

### crates/devforge-storage/src/migrator.rs

```rust
use sqlx::migrate::Migrator;
use sqlx::SqlitePool;

use crate::error::StorageError;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// 运行所有待执行迁移
///
/// 使用静态 Migrator 加载 migrations/ 目录下的 SQL 文件。
/// SQLx 自动创建 `_sqlx_migrations` 表记录已执行的迁移。
///
/// # Errors
///
/// 当 migration 文件损坏、SQL 语法错误或数据库锁定时返回错误。
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), StorageError> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(|source| StorageError::Migration { source })
}

/// 获取当前 schema 版本
///
/// 从 `_sqlx_migrations` 表查询最后一个成功版本。
/// 空结果按版本 0 处理。
///
/// # Errors
///
/// 当无法查询 `_sqlx_migrations` 表或版本号超出 `u32` 范围时返回错误。
pub async fn schema_version(pool: &SqlitePool) -> Result<u32, StorageError> {
    let row: (Option<i64>,) = sqlx::query_as(
        "SELECT MAX(version) FROM _sqlx_migrations WHERE success = TRUE",
    )
    .fetch_one(pool)
    .await
    .map_err(|source| StorageError::SchemaVersion { source })?;

    let version = row.0.unwrap_or(0);

    u32::try_from(version).map_err(|_| StorageError::MigrationVersionOutOfRange { version })
}
```

要求：
- 定义单一静态 `Migrator`；
- 为公共可失败函数增加 Rustdoc `# Errors`；
- `schema_version()` 使用 `Option<i64>` 接收查询结果，空结果按 0 处理；
- `i64` → `u32` 使用 `u32::try_from()`，失败时返回 `MigrationVersionOutOfRange`；
- 禁止 `version as u32`。

### crates/devforge-storage/src/status.rs

```rust
use async_trait::async_trait;
use devforge_application::app_info::DbStatus;
use devforge_application::ports::DatabaseStatusProvider;
use sqlx::SqlitePool;

use crate::migrator;

/// 基于 SQLx 的数据库状态提供者
///
/// 只表示已经成功建立的 SQLite 数据源。
/// 不允许构造内部没有 Pool 的实例，
/// 未初始化状态由 Application 层的 `NotInitializedDbStatus` 表示。
pub struct SqliteDatabaseStatus {
    pool: SqlitePool,
}

impl SqliteDatabaseStatus {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DatabaseStatusProvider for SqliteDatabaseStatus {
    async fn status(&self) -> DbStatus {
        match migrator::schema_version(&self.pool).await {
            Ok(version) => DbStatus::Ready { migration_version: version },
            Err(e) => DbStatus::Error { message: e.to_string() },
        }
    }
}
```

要求：
- 不使用 `Option<SqlitePool>`，Storage Adapter 只表示已成功建立的数据源；
- `schema_version()` 成功返回 `DbStatus::Ready`，失败在最终诊断边界转换为 `DbStatus::Error`；
- 不记录并重复返回同一个错误；
- 不 panic，不使用 `unwrap()` 或 `expect()`。

### crates/devforge-storage/tests/sqlite_bootstrap.rs

使用 `tempfile::TempDir` 和文件数据库。每个测试结束前执行 `pool.close().await` 确保 Windows 文件句柄释放。

至少实现以下测试：

#### 1. 数据库打开与连接配置

验证数据库文件被创建，以及：
- `PRAGMA journal_mode = wal`
- `PRAGMA foreign_keys = 1`
- `PRAGMA busy_timeout = 5000`

不要通过检查 `-wal` 文件是否存在来判断 WAL，因为 WAL 文件可能在 checkpoint 或关闭后被删除。

#### 2. Migration 在空数据库执行成功

验证：
- `run_migrations()` 成功
- `app_meta` 表存在
- `schema_version() = 1`

#### 3. Migration 重复执行保持幂等

对同一个数据库连续调用两次 `run_migrations()`，确认：
- 没有错误
- `schema_version()` 仍为 1
- `app_meta` 表没有重复或损坏

幂等性来自 SQLx migration 追踪，不通过 `IF NOT EXISTS` 掩盖错误。

#### 4. 健康状态

Migration 完成后构造 `SqliteDatabaseStatus::new(pool.clone())`，调用 `status()` 确认返回 `DbStatus::Ready { migration_version: 1 }`。

#### 5. 未迁移数据库错误状态

打开新的文件数据库但不运行 migration，构造 `SqliteDatabaseStatus` 后调用 `status()`，确认返回 `DbStatus::Error { .. }`，不得 panic。

#### 测试约束

- 不得写入用户真实数据目录；
- 不得依赖系统已安装 SQLite；
- 不得依赖网络；
- 不得并行共享同一个数据库文件；
- 不得使用 `unwrap()` 或 `expect()`，测试函数使用 `Result` 和 `?`。

### 验证命令

从仓库根目录依次运行：

```powershell
cargo test -p devforge-storage
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace
git diff --check
git status --short
```

人工确认：
1. `Cargo.lock` 中解析 SQLx `0.9.0`；
2. SQLx 只启用了所需 SQLite、Tokio、macro 和 migration 能力；
3. Migration 文件为 LF；
4. 没有 `workspace` 领域表；
5. 没有未经检查的 `as u32`；
6. 没有把 SQLx source error 提前转换为 String；
7. 没有未使用的 `tracing` 依赖；
8. 没有修改 Tauri 或 React 文件。

### 提交信息

```
feat(storage): 建立 SQLite 连接池和第一条 migration
```

---

## Task 7：串联 Storage 到 Tauri Command

**依赖**：Task 4, Task 6

**目标**：在桌面 Composition Root 中创建数据目录，打开 SQLite 文件并执行 Migration，使用真实 SqliteDatabaseStatus 构造 AppState，使现有 get_app_info Command 返回 Ready 数据库状态。

**范围约束**：Task 7 只负责已有模块之间的组合，不修改 Application 或 Storage 内部契约。

### 调用链

```text
React useAppInfo
        ↓
现有 commands.getAppInfo()
        ↓
现有 Tauri get_app_info(AppHandle) -> AppInfo
        ↓
AppHandle.state::<AppState>()
        ↓
AppState.app_info()
        ↓
GetAppInfo<PlatformMetadata, SqliteDatabaseStatus>
        ↓
_sqlx_migrations
        ↓
DbStatus::Ready { migration_version: 1 }
```

### 精确文件

Task 7 只允许修改：

| 文件 | 职责 |
|------|------|
| `Cargo.lock` | cargo 更新 |
| `apps/desktop/src-tauri/Cargo.toml` | 添加 devforge-storage 依赖 |
| `apps/desktop/src-tauri/src/state.rs` | 使用真实 SqliteDatabaseStatus 构造 AppState |
| `apps/desktop/src-tauri/src/lib.rs` | 组合初始化：创建目录、打开数据库、执行 Migration |

**不得修改**：

- `apps/desktop/src/bindings.ts`
- `apps/desktop/src/**`（除 bindings.ts 外的前端文件）
- `crates/devforge-application/**`
- `crates/devforge-storage/**`
- `crates/devforge-platform/**`

**原因**：
- Application Trait 已经是 async
- GetAppInfo 已经是 async
- SqliteDatabaseStatus 已经实现
- Command 已经使用 AppHandle 并直接返回 AppInfo
- main 已经正确返回 anyhow::Result
- Task 7 不改变 IPC 类型契约

### Cargo 依赖

在 `apps/desktop/src-tauri/Cargo.toml` 的现有 `[dependencies]` 中只增加：

```toml
devforge-storage = { workspace = true }
```

为聚焦测试增加：

```toml
[dev-dependencies]
tempfile = { workspace = true }
```

**要求**：
- 不增加生产依赖 `tokio`
- 不使用 `#[tokio::main]`
- 不创建重复的 `[dependencies]`
- 保留现有 `anyhow`、`dirs`、Specta 和 Tauri 依赖
- `Cargo.lock` 根据依赖关系正常更新

### AppState 设计

更新 `apps/desktop/src-tauri/src/state.rs`：

```rust
use devforge_application::app_info::AppInfo;
use devforge_application::get_app_info::GetAppInfo;
use devforge_platform::app_info::PlatformMetadata;
use devforge_storage::status::SqliteDatabaseStatus;

/// 应用全局状态。
///
/// 只暴露应用用例级接口，不向 Command 暴露 SqlitePool。
pub(crate) struct AppState {
    get_app_info: GetAppInfo<PlatformMetadata, SqliteDatabaseStatus>,
}

impl AppState {
    pub(crate) fn new(
        platform_metadata: PlatformMetadata,
        database_status: SqliteDatabaseStatus,
    ) -> Self {
        Self {
            get_app_info: GetAppInfo::new(platform_metadata, database_status),
        }
    }

    pub(crate) async fn app_info(&self) -> AppInfo {
        self.get_app_info.execute().await
    }
}
```

**要求**：
- 字段保持私有
- 不公开 `SqlitePool`
- 不保存公开的 `data_dir`
- 不使用 `Option<SqlitePool>`
- 不使用 `NotInitializedDbStatus`
- 不提供 `init_db(&mut self)`
- 不在初始化后替换 Use Case
- 构造完成的 AppState 必须始终是有效状态
- 使用尽可能窄的可见性

### Composition Root 初始化

更新 `apps/desktop/src-tauri/src/lib.rs`：

保留：
- `#![forbid(unsafe_code)]`
- 现有 `create_builder()`
- 现有 `export_bindings()`
- 基于 `CARGO_MANIFEST_DIR` 的 bindings 输出
- `anyhow::Result`
- Specta Builder 运行时和导出共用

新增私有异步初始化函数，职责仅为组合已有模块：

```rust
async fn initialize_app_state(
    version: String,
    data_dir: std::path::PathBuf,
) -> anyhow::Result<AppState> {
    let db_path = data_dir.join("devforge.db");

    let database = devforge_storage::pool::Database::open(&db_path)
        .await
        .with_context(|| {
            format!("无法打开 SQLite 数据库：{}", db_path.display())
        })?;

    devforge_storage::migrator::run_migrations(database.pool())
        .await
        .context("无法执行 SQLite migration")?;

    let platform_metadata =
        devforge_platform::app_info::PlatformMetadata::new(
            version,
            data_dir,
        );

    let database_status =
        devforge_storage::status::SqliteDatabaseStatus::new(
            database.pool().clone(),
        );

    Ok(AppState::new(
        platform_metadata,
        database_status,
    ))
}
```

**说明**：
- `Database` 包装器在函数结束时可以被释放
- `SqliteDatabaseStatus` 持有克隆的 `SqlitePool`
- 不向 AppState 或 Command 暴露 Pool
- Migration 失败属于核心启动失败，必须阻止应用启动
- 不把错误转换为 String
- 允许根据 rustfmt 调整排版，但不得改变边界

### 同步文件系统操作位置

`std::fs::create_dir_all()` 必须在进入异步数据库初始化之前执行，避免在 Tokio worker 上执行同步文件系统操作。

`run()` 采用：

```rust
pub fn run() -> anyhow::Result<()> {
    let data_dir = dirs::data_local_dir()
        .context("无法解析本地数据目录")?
        .join("DevForge");

    std::fs::create_dir_all(&data_dir)
        .with_context(|| {
            format!("无法创建本地数据目录：{}", data_dir.display())
        })?;

    let app_version = env!("CARGO_PKG_VERSION").to_owned();

    let app_state = tauri::async_runtime::block_on(
        initialize_app_state(app_version, data_dir),
    )
    .context("无法初始化桌面应用状态")?;

    let builder = create_builder();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .context("无法启动 Tauri 应用")?;

    Ok(())
}
```

**要求**：
- 保持 `run()` 同步
- 使用 Tauri 自己的 `tauri::async_runtime::block_on()`
- 不使用 `#[tokio::main]`
- 不调用 `tauri::async_runtime::set()`
- 不创建第二个 Tokio Runtime
- 不在 `setup` 回调中进行两阶段 State 替换
- AppState 必须在 `.manage(app_state)` 前完整初始化

更新 `run()` Rustdoc 的 `# Errors`，明确包括：
- 无法解析数据目录
- 无法创建数据目录
- 数据库无法打开
- Migration 失败
- Tauri 应用无法启动

### Command 保持不变

`apps/desktop/src-tauri/src/commands.rs` 不修改。

必须继续保持：

```rust
#[tauri::command]
#[specta::specta]
pub async fn get_app_info(app: AppHandle) -> AppInfo {
    app.state::<AppState>().app_info().await
}
```

不得改为：
- `State<'_, AppState>`
- `Result<AppInfo, String>`
- 不得重新生成 `typedError`

### main.rs 保持不变

`apps/desktop/src-tauri/src/main.rs` 不修改：

```rust
#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> anyhow::Result<()> {
    devforge_desktop_lib::run()
}
```

不得添加：
- `#[tokio::main]`
- `async fn main()`

### 聚焦测试

在 `state.rs` 的 `#[cfg(test)]` 模块中增加真实 Storage 串联测试。

测试使用：
- tempfile 临时目录
- Database::open()
- run_migrations()
- PlatformMetadata
- SqliteDatabaseStatus
- AppState

测试流程：

```text
创建临时文件数据库
→ 执行 Migration
→ 构造真实 SqliteDatabaseStatus
→ 构造 AppState
→ 调用 app_info()
→ 验证版本、数据目录和 Ready v1
→ drop AppState
→ 显式关闭 Pool
→ 最终断言
```

使用 `tauri::async_runtime::block_on()`，不得添加 Tokio 测试 Runtime。

测试至少验证：

```rust
info.version == "test-version"
info.data_dir == 注入的临时目录
matches!(
    info.db_status,
    DbStatus::Ready {
        migration_version: 1
    }
)
```

**资源清理要求**：
1. 保存独立的 `SqlitePool` clone
2. 构造 AppState
3. 获取 AppInfo
4. `drop(state)`
5. `drop(database)`
6. 调用 `tauri::async_runtime::block_on(pool.close());`
7. 最后执行断言

**测试约束**：
- 不得使用真实用户数据目录
- 不得使用普通 `:memory:` 数据库
- 不得使用 `unwrap()` 或 `expect()`
- 不得直接 `panic!()`
- 不得为测试修改 Application DTO derive
- 不得导出测试辅助代码

### Bindings 契约

Task 7 不改变：
- Tauri Command 名称
- Command 参数
- Command 返回类型
- AppInfo
- DbStatus

因此重新生成 bindings 后，`apps/desktop/src/bindings.ts` 必须保持无差异。

仍应是：

```typescript
getAppInfo: () =>
  __TAURI_INVOKE<AppInfo>("get_app_info")
```

不得出现 `typedError`。

`export_bindings()` 不得打开数据库、创建数据目录或运行 Migration。

### 验证命令

从仓库根目录执行：

```powershell
cargo test -p devforge-desktop
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
pnpm bindings:generate
git diff --exit-code -- apps/desktop/src/bindings.ts
pnpm --filter @devforge/desktop typecheck
pnpm --filter @devforge/desktop build
git diff --check
git status --short
pnpm dev:desktop
```

### 人工验证

确认：
1. 桌面窗口正常打开
2. 页面显示 `DevForge`
3. 版本显示 `0.1.0`
4. 数据目录以 `DevForge` 结尾
5. 数据库状态显示 `就绪（migration v1）`
6. 数据目录中存在 `devforge.db`
7. 不要求 `devforge.db-wal` 必然存在
8. 关闭并重新启动应用后仍显示 Ready v1
9. 控制台没有 React、TanStack Query、Tauri 或 IPC 异常
10. `commands.getAppInfo()` 仍直接返回 AppInfo
11. bindings 中没有 `typedError`
12. Ctrl+C 后 Tauri 和 Vite 进程正常退出

### 范围限制

不得：
- 修改 Application crate
- 修改 Storage crate
- 修改 Platform crate
- 修改 Command
- 修改 main.rs
- 修改 React
- 手动修改 bindings
- 添加 Router、Theme 或 ErrorBoundary
- 执行 Task 8
- 添加无关抽象
- 提交或 push

### 提交信息

```
feat(storage): 串联 SQLite 到 Tauri get_app_info 命令
```

---

## Task 8：前端基础设施 — Router、Theme、ErrorBoundary

**依赖**：Task 5, Task 7

**目标**：建立适合 Tauri 桌面静态壳的 Hash Router，保留 Task 5 的 AppInfo 三态页面，实现可持久化且跟随系统变化的主题系统，建立基础导航布局、路由级错误边界和应用级错误边界。

**不实现**：Command Palette、Vitest、Testing Library、CI、Tauri 或 Rust 修改、新 IPC、Task 9。

### 精确文件

| 文件 | 职责 |
|------|------|
| `apps/desktop/package.json` | 添加 react-router 和 zustand 依赖 |
| `pnpm-lock.yaml` | pnpm install 更新 |
| `apps/desktop/src/main.tsx` | 更新：添加 AppErrorBoundary 和 RouterProvider |
| `apps/desktop/src/App.tsx` | 更新：根路由组件，同步主题，渲染 AppLayout |
| `apps/desktop/src/router.tsx` | Hash Router 配置 |
| `apps/desktop/src/layouts/AppLayout.tsx` | 导航布局，使用 sidebarCollapsed 和 toggleSidebar |
| `apps/desktop/src/pages/HomePage.tsx` | 首页（从 App.tsx 迁移 Task 5 内容） |
| `apps/desktop/src/pages/SettingsPage.tsx` | 设置页（主题选择） |
| `apps/desktop/src/pages/RouteErrorPage.tsx` | 路由级错误页 |
| `apps/desktop/src/pages/NotFoundPage.tsx` | 404 页面 |
| `apps/desktop/src/components/AppErrorBoundary.tsx` | 应用级错误边界 |
| `apps/desktop/src/hooks/useThemeSync.ts` | 主题同步 Hook |
| `apps/desktop/src/stores/ui.ts` | Zustand UI 状态（persist） |
| `apps/desktop/src/styles/global.css` | 全局样式和主题变量 |

**不得修改**：
- `apps/desktop/src/hooks/useAppInfo.ts`
- `apps/desktop/src/components/HealthStatus.tsx`
- `apps/desktop/src/bindings.ts`
- `apps/desktop/src/queryKeys.ts`
- `apps/desktop/src-tauri/**`
- `crates/**`

### 依赖安装

从仓库根目录运行：

```powershell
pnpm --filter @devforge/desktop add react-router@^7.18.1 zustand@^5.0.14
```

**版本选择原因**：
- 当前项目使用 React `19.1.0` 和 React DOM `19.1.0`
- React Router 8.2.0 要求更高版本的 React 和 React DOM
- Task 8 不应顺带升级 React 核心版本
- React Router 7.18.1 与当前 React 版本兼容
- Task 8 所需的 `createHashRouter`、Route Object `Component`、`ErrorBoundary`、`RouterProvider`、`useRouteError` 等能力在 7.18.1 中均可使用

**要求**：
- 同时更新 `apps/desktop/package.json` 和 `pnpm-lock.yaml`
- 不使用未声明依赖
- 不安装 `react-router-dom` 包
- `RouterProvider` 从 `react-router/dom` 导入
- 其余 Router API 从 `react-router` 导入
- 不安装测试依赖、UI 组件库、CSS 框架、图标库
- 不添加当前任务没有使用者的依赖
- 不升级 `react`、`react-dom`、`@types/react`、`@types/react-dom`

### apps/desktop/src/router.tsx

使用 `createHashRouter`，不得使用 `createBrowserRouter`。

Router 在模块顶层只创建一次，不放在 React state 或组件渲染函数中。

```typescript
import { createHashRouter } from "react-router";

import App from "./App";
import { HomePage } from "./pages/HomePage";
import { NotFoundPage } from "./pages/NotFoundPage";
import { RouteErrorPage } from "./pages/RouteErrorPage";
import { SettingsPage } from "./pages/SettingsPage";

export const router = createHashRouter([
  {
    path: "/",
    Component: App,
    ErrorBoundary: RouteErrorPage,
    children: [
      {
        index: true,
        Component: HomePage,
      },
      {
        path: "settings",
        Component: SettingsPage,
      },
      {
        path: "*",
        Component: NotFoundPage,
      },
    ],
  },
]);
```

**要求**：
- 使用 Route Object 的 `Component`
- 根路由配置 `ErrorBoundary`
- 增加 `path: "*"` 处理 404
- 设置页地址为 `#/settings`
- 不使用 Browser History
- 不创建自定义 History
- 不加入 loader 或 action
- 不改变 IPC 调用方式

### apps/desktop/src/App.tsx

App.tsx 保留为根路由组件，职责：

```text
同步当前主题到 DOM
→ 渲染 AppLayout
→ AppLayout 内渲染 Outlet
```

```typescript
import { AppLayout } from "./layouts/AppLayout";
import { useThemeSync } from "./hooks/useThemeSync";

export default function App() {
  useThemeSync();

  return <AppLayout />;
}
```

**要求**：
- 不在 App 中重新调用 `useAppInfo()`
- 不在 App 中直接访问 Router API

### apps/desktop/src/pages/HomePage.tsx

将当前 `App.tsx` 中 Task 5 的内容完整迁移到 `HomePage.tsx`。

必须保留：
- `useAppInfo()`
- `isPending` 加载状态
- `isError` 错误状态
- `role="alert"`
- 安全的 `getErrorMessage()`
- 手动重试
- 重试期间禁用按钮
- AppInfo 成功展示
- `HealthStatus`
- 版本、数据目录、数据库状态

迁移过程中不得：
- 删除错误重试
- 删除加载状态
- 添加重复类型
- 手写 AppInfo
- 直接调用 Tauri invoke
- 修改 `useAppInfo()`
- 修改 bindings

### apps/desktop/src/stores/ui.ts

使用 Zustand `persist`。

```typescript
import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ThemePreference = "light" | "dark" | "system";

interface UIState {
  theme: ThemePreference;
  sidebarCollapsed: boolean;
  setTheme: (theme: ThemePreference) => void;
  toggleSidebar: () => void;
}

export const useUIStore = create<UIState>()(
  persist(
    (set) => ({
      theme: "system",
      sidebarCollapsed: false,
      setTheme: (theme) => set({ theme }),
      toggleSidebar: () =>
        set((state) => ({
          sidebarCollapsed: !state.sidebarCollapsed,
        })),
    }),
    {
      name: "devforge-ui",
      partialize: (state) => ({
        theme: state.theme,
        sidebarCollapsed: state.sidebarCollapsed,
      }),
    },
  ),
);
```

**要求**：
- 只持久化用户偏好
- 不持久化函数
- 不持久化计算后的 resolved theme
- 不直接在 Store 创建时修改 DOM
- 不读取 Tauri 文件系统
- 不创建第二套主题状态

### apps/desktop/src/hooks/useThemeSync.ts

职责：
1. 从 `useUIStore` 读取 theme
2. system 模式通过 `window.matchMedia("(prefers-color-scheme: dark)")` 解析实际主题
3. 设置 `document.documentElement.dataset.theme` 和 `document.documentElement.style.colorScheme`
4. system 模式监听系统主题变化
5. effect cleanup 时移除监听

实际 DOM 主题只能是 `light` 或 `dark`，不得设置 `data-theme="system"`。

**首次渲染要求**：
- 优先使用 `useLayoutEffect` 同步主题
- 在浏览器绘制前设置 `data-theme` 和 `color-scheme`
- 减少持久化暗色主题启动时先显示浅色再切换的闪烁

**要求**：
- 不每次 render 重复注册 listener
- 不忘记 cleanup
- 不把 MediaQueryList 放进 Zustand
- 不在模块加载阶段直接访问 `window` 或 `document`
- 不创建 MutationObserver
- 不使用轮询
- system 模式仍需注册 `matchMedia` change listener
- effect cleanup 时仍需移除 listener

### apps/desktop/src/layouts/AppLayout.tsx

`AppLayout` 必须真正使用 `sidebarCollapsed` 和 `toggleSidebar`。

布局至少包括：
- `<aside aria-label="主导航">`
- 首页 `NavLink`
- 设置 `NavLink`
- Active 样式
- 折叠按钮
- `aria-expanded`
- `<main>`
- `<Outlet />`

```typescript
import { NavLink, Outlet } from "react-router";

import { useUIStore } from "../stores/ui";

export function AppLayout() {
  const sidebarCollapsed = useUIStore(
    (state) => state.sidebarCollapsed,
  );
  const toggleSidebar = useUIStore(
    (state) => state.toggleSidebar,
  );

  return (
    <div
      className="app-layout"
      data-sidebar-collapsed={sidebarCollapsed}
    >
      <aside
        className="activity-bar"
        aria-label="主导航"
      >
        <button
          type="button"
          aria-expanded={!sidebarCollapsed}
          onClick={toggleSidebar}
        >
          {sidebarCollapsed ? "展开" : "收起"}
        </button>

        <nav>
          <NavLink to="/" end>
            首页
          </NavLink>
          <NavLink to="/settings">
            设置
          </NavLink>
        </nav>
      </aside>

      <main className="main-content">
        <Outlet />
      </main>
    </div>
  );
}
```

**要求**：
- 允许优化标签和类名
- 必须满足可访问性和实际使用 Store

### apps/desktop/src/pages/SettingsPage.tsx

设置页不能只是空占位。至少提供三个主题选项：浅色、深色、跟随系统。

使用 radio group 或原生 select。

**要求**：
- 当前选项可见
- 选择后立即生效
- 重启应用后保持
- system 模式随系统变化
- 使用 `fieldset` 和 `legend`，或带 label 的 select
- 不使用不可访问的 div 模拟按钮
- 不添加无实现的设置项

### apps/desktop/src/pages/RouteErrorPage.tsx

必须使用 `useRouteError()` 和 `isRouteErrorResponse()`。

处理：
- Route Error Response
- Error 实例
- string
- 任意未知值

提供：
- 返回首页
- 重新加载应用

不得仅依赖外层 Class ErrorBoundary 处理 Data Router 错误。

### apps/desktop/src/pages/NotFoundPage.tsx

至少包含：
- "页面不存在"
- 返回首页链接
- 不抛异常
- 不自动重定向
- 不显示调试堆栈

### apps/desktop/src/components/AppErrorBoundary.tsx

普通 React Class Error Boundary，包裹：

```tsx
<QueryClientProvider>
  <RouterProvider />
</QueryClientProvider>
```

职责：
- 捕获 Provider 或 Router 初始化层渲染错误
- `role="alert"`
- 安全展示错误消息
- 提供"重新加载应用"按钮
- `componentDidCatch()` 只记录一次错误
- 不显示生产 stack
- 不吞掉错误后渲染空页面

### apps/desktop/src/main.tsx

保留模块级 QueryClient：

```typescript
import React from "react";
import ReactDOM from "react-dom/client";
import {
  QueryClient,
  QueryClientProvider,
} from "@tanstack/react-query";
import { RouterProvider } from "react-router/dom";

import { AppErrorBoundary } from "./components/AppErrorBoundary";
import { router } from "./router";
import "./styles/global.css";

const queryClient = new QueryClient();

ReactDOM.createRoot(
  document.getElementById("root")!,
).render(
  <React.StrictMode>
    <AppErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </AppErrorBoundary>
  </React.StrictMode>,
);
```

**要求**：
- 保留 StrictMode
- QueryClient 只创建一次
- 不在组件中创建 Router
- 不在组件中创建 QueryClient
- 全局 CSS 只导入一次
- 不删除 QueryClientProvider
- `RouterProvider` 从 `react-router/dom` 导入
- 其余 Router API（如 `createHashRouter`、`NavLink`、`Outlet`）从 `react-router` 导入

### apps/desktop/src/styles/global.css

必须定义完整主题变量：

```css
:root,
html[data-theme="light"] {
  color-scheme: light;
  --bg-primary: #ffffff;
  --bg-secondary: #f5f5f5;
  --text-primary: #1a1a1a;
  --text-secondary: #666666;
  --border: #e0e0e0;
  --interactive-hover: #f0f0f0;
  --interactive-active: #e0e0e0;
  --focus-ring: #0066cc;
  --danger: #dc3545;
}

html[data-theme="dark"] {
  color-scheme: dark;
  --bg-primary: #1a1a1a;
  --bg-secondary: #2a2a2a;
  --text-primary: #e0e0e0;
  --text-secondary: #999999;
  --border: #404040;
  --interactive-hover: #333333;
  --interactive-active: #404040;
  --focus-ring: #4d9fff;
  --danger: #ff6b6b;
}
```

**要求**：
- `html`、`body`、`#root` 高度为 100%
- body 使用主题背景和文字颜色
- AppLayout 不出现页面级水平滚动
- MainContent 可独立滚动
- NavLink 有 active 状态
- 键盘 focus 可见
- 侧栏折叠状态有明确布局变化
- 错误页有可读布局
- 不依赖变量 fallback 掩盖变量遗漏
- 不使用内联 style 作为主要布局方案
- 不引入 CSS-in-JS

### 验证命令

从仓库根目录运行：

```powershell
pnpm install
pnpm --filter @devforge/desktop typecheck
pnpm --filter @devforge/desktop build
git diff --check
git status --short
pnpm dev:desktop
```

### 人工验证

确认：
1. 首页地址为 `#/`
2. 设置页地址为 `#/settings`
3. 点击导航能切换页面
4. 设置页刷新仍正常
5. 不存在路由显示 NotFoundPage
6. 首页仍完整显示 AppInfo
7. 加载状态仍存在
8. IPC 错误仍有重试按钮
9. Light 主题立即生效
10. Dark 主题立即生效
11. System 主题跟随当前系统
12. System 模式下切换系统主题后页面实时变化
13. 主题选择在应用重启后保持
14. 侧栏折叠状态在应用重启后保持
15. 导航 active 状态正确
16. 键盘 Tab 可见 focus
17. 临时在一个路由组件中抛出 Error 时显示 RouteErrorPage
18. 删除临时错误代码后 `git diff` 中无测试残留
19. AppErrorBoundary 提供重新加载按钮
20. 控制台无 React、Router、Zustand、TanStack Query 或 Tauri 异常
21. `apps/desktop/src/bindings.ts` 未修改
22. 没有修改 Rust 文件

### 提交信息

```
feat(frontend): 建立 Hash Router、主题系统和双层错误边界
```

### 跨任务备注

#### Task 9 测试需要更新

Task 8 完成后：
- `App.tsx` → 根布局和主题同步
- `HomePage.tsx` → AppInfo 加载、错误、重试和成功状态

因此 Task 9 中旧的"直接渲染 App 并断言'加载中...'"已经不再准确。

**Task 9 审核时需要将前端测试更新为 Router/HomePage 测试，本次 Task 8 不提前修改 Task 9。**

#### Command Palette 尚未安排

Phase 0 目标中仍包含 Command Palette 框架，但当前 Task 8 明确不实现，Task 9 和 Task 10 也没有覆盖。

**需要后续新增独立 Command Palette 基础任务，或明确将其延期到后续阶段。本次 Task 8 不实现。**

---

## Task 9：工程质量和 CI

**依赖**：Task 2, Task 3, Task 7, Task 8

**目标**：
- 建立单一来源的本地质量检查脚本；
- 引入 ESLint、Vitest、React Testing Library；
- 建立确定性的 Rust 与 Node/pnpm 工具链；
- 在 GitHub Actions 中运行与本地相同的质量检查；
- 在 Windows Runner 上验证 Tauri Release 编译，但不生成安装包。

**包含**：
- Rust fmt
- Rust Clippy
- Rust tests
- Rust check
- bindings 重新生成与差异检查
- ESLint
- TypeScript typecheck
- frontend tests
- frontend build
- Git diff whitespace check
- GitHub Actions
- Tauri Release 编译检查

**不包含**：
- Windows 安装包冒烟
- MSI/NSIS 安装验证
- 发布 GitHub Release
- 代码签名
- Command Palette
- Git Commit Hooks 的最终方案
- Task 10

### 精确文件

| 文件 | 职责 |
|------|------|
| `.github/workflows/ci.yml` | GitHub Actions CI 配置 |
| `scripts/check.ps1` | 本地一键验证脚本 |
| `rust-toolchain.toml` | 统一 Rust 工具链版本 |
| `package.json` | 根 package.json，增加 check 脚本 |
| `apps/desktop/package.json` | 增加 lint 和 test 脚本，增加测试依赖 |
| `pnpm-lock.yaml` | pnpm install 更新 |
| `apps/desktop/eslint.config.js` | ESLint Flat Config |
| `apps/desktop/vitest.config.ts` | Vitest 配置 |
| `apps/desktop/src/test/setup.ts` | 测试 setup（jest-dom matchers + cleanup） |
| `apps/desktop/src/__tests__/HomePage.test.tsx` | HomePage 集成测试 |
| `apps/desktop/src/__tests__/AppRouting.test.tsx` | Router 集成测试 |

**已从精确文件中删除**：
- `apps/desktop/src-tauri/rustfmt.toml` — 当前没有自定义 rustfmt 规则需求，不创建投机性配置；Rust 工具链通过仓库根目录 `rust-toolchain.toml` 统一。
- `apps/desktop/src/__tests__/App.test.tsx` — Task 8 后旧的 App 测试结构已失效；测试应覆盖实际拥有行为的 HomePage，以及 Router/AppLayout 集成。

**不得修改**：

```text
apps/desktop/src/bindings.ts
apps/desktop/src/App.tsx
apps/desktop/src/router.tsx
apps/desktop/src/pages/**
apps/desktop/src/layouts/**
apps/desktop/src/hooks/**
apps/desktop/src/stores/**
apps/desktop/src/components/**
apps/desktop/src-tauri/**
crates/**
Cargo.toml
Cargo.lock
```

测试文件除外。

### rust-toolchain.toml

新增根目录 `rust-toolchain.toml`：

```toml
[toolchain]
channel = "1.96.0"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

**要求**：
- 与 Workspace `rust-version = "1.96"` 对齐；
- 本地与 CI 使用同一 Rust 版本；
- 不升级 Workspace MSRV；
- 不增加额外 target；
- 不创建 crate 内部工具链文件；
- CI 不使用漂移的 `stable` 工具链。

### 前端质量依赖

从仓库根目录执行：

```powershell
pnpm --filter @devforge/desktop add -D `
  vitest `
  jsdom `
  @testing-library/react `
  @testing-library/jest-dom `
  eslint `
  @eslint/js `
  typescript-eslint `
  eslint-plugin-react-hooks `
  eslint-plugin-react-refresh `
  globals
```

**要求**：
- 必须包含 `vitest`；
- 同时更新 `apps/desktop/package.json` 和 `pnpm-lock.yaml`；
- 不添加 Jest；
- 不添加 Babel 测试配置；
- 不添加 Cypress、Playwright 或浏览器 E2E；
- 不升级现有 React、React Router、Vite、TypeScript 或 Tauri；
- Lockfile 只能包含本次预期依赖变化。

### apps/desktop/package.json

增加 scripts：

```json
{
  "scripts": {
    "lint": "eslint . --max-warnings 0",
    "test": "vitest run"
  }
}
```

保留已有：

```text
dev
build
typecheck
tauri
```

不得添加 watch 脚本或 coverage 阈值。

### 根 package.json

增加：

```json
{
  "scripts": {
    "check": "pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1"
  }
}
```

保留现有 workspace scripts。

最终本地完整检查命令为：

```powershell
pnpm check
```

### apps/desktop/eslint.config.js

新增 ESLint Flat Config，组合：
- `@eslint/js`
- `typescript-eslint`
- `eslint-plugin-react-hooks`
- `eslint-plugin-react-refresh`
- browser globals

至少忽略：

```text
dist/**
src-tauri/**
src/bindings.ts
```

**要求**：
- 生成的 bindings 不参与 ESLint；
- 不启用要求大规模重构的风格规则；
- 不与 TypeScript 编译器重复报告无价值规则；
- React Hooks 规则必须启用；
- React Refresh 规则仅应用于 React 源文件；
- `pnpm --filter @devforge/desktop lint` 必须零 warning；
- 不使用 legacy `.eslintrc`；
- 不添加 Prettier。

### apps/desktop/vitest.config.ts

```typescript
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    clearMocks: true,
    restoreMocks: true,
  },
});
```

**要求**：
- 使用 jsdom；
- 不启用 Vitest globals；
- 测试文件显式导入 `describe`、`it`、`expect`、`vi`；
- 不访问真实 Tauri；
- 不访问真实外部网络；
- 不添加任意 sleep；
- 不配置 coverage 阈值；
- 不修改生产 Vite 配置。

### apps/desktop/src/test/setup.ts

```typescript
import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

afterEach(() => {
  cleanup();
});
```

**原因**：
- 注册 jest-dom 的 Vitest matchers；
- 在未启用 Vitest globals 时显式执行 Testing Library cleanup；
- 避免测试之间残留 DOM。

**不得在 setup 中**：
- mock bindings；
- mock localStorage；
- 创建 QueryClient；
- 修改生产 Store；
- 吞掉 console error。

### apps/desktop/src/__tests__/HomePage.test.tsx

使用真实：
- `HomePage`
- `useAppInfo`
- TanStack Query

只 mock：

```text
commands.getAppInfo
```

每个测试创建独立 `QueryClient`，测试默认配置至少设置：

```typescript
defaultOptions: {
  queries: {
    retry: false,
  },
}
```

#### 测试一：加载并显示真实状态

验证：
1. 初始显示"加载中..."；
2. Promise resolve 后显示 `DevForge`；
3. 显示版本 `0.1.0`；
4. 显示测试数据目录；
5. 显示 `就绪（migration v1）`。

Mock 返回：

```typescript
{
  version: "0.1.0",
  data_dir: "C:\\Users\\test\\AppData\\Local\\DevForge",
  db_status: {
    type: "Ready",
    migration_version: 1,
  },
}
```

#### 测试二：错误和重试

验证：
1. 第一次调用 reject；
2. 页面显示 `role="alert"`；
3. 显示"加载应用信息失败"；
4. 点击"重试"；
5. 第二次调用 resolve；
6. 页面最终显示成功状态；
7. 调用次数为 2。

允许为交互增加 `@testing-library/user-event`，但仅在计划中明确加入依赖和精确文件后使用。也可以使用 Testing Library 原生事件能力而不新增依赖。

**不得**：
- mock `HomePage`；
- mock `useAppInfo`；
- 使用真实 Tauri invoke；
- 断言私有实现细节；
- 访问真实用户目录；
- 使用 sleep。

### apps/desktop/src/__tests__/AppRouting.test.tsx

使用 Memory Router 或 `createMemoryRouter` 构造测试路由环境。

验证外部行为：
1. `/settings` 显示设置页面和三个主题选项；
2. 未知路径显示"页面不存在"；
3. 首页入口能够渲染 HomePage；
4. App 能在 Router 上下文中渲染，而不是直接裸渲染。

**不得**：
- 修改生产 Router 仅为了测试；
- 直接依赖全局 hash 单例；
- 复制大量生产路由配置；
- 测试 CSS 像素值；
- 使用快照替代语义断言。

### scripts/check.ps1

脚本必须：

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
```

并自动切换到仓库根目录：

```powershell
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Push-Location $RepoRoot

try {
    # checks
}
finally {
    Pop-Location
}
```

不得只依赖 `$ErrorActionPreference` 判断 Cargo、pnpm、git 等原生命令是否失败。

增加统一辅助函数，执行原生命令后立即检查 `$LASTEXITCODE`，非零时抛出错误并停止后续检查。

检查顺序：

```text
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace
pnpm bindings:generate
git diff --exit-code -- apps/desktop/src/bindings.ts
pnpm --filter @devforge/desktop lint
pnpm --filter @devforge/desktop typecheck
pnpm --filter @devforge/desktop test
pnpm --filter @devforge/desktop build
git diff --check
```

**要求**：
- 每个步骤显示清晰标题；
- 任一步骤失败立即停止；
- 保留原始命令退出码；
- 不使用 `Invoke-Expression`；
- 不吞掉 stderr；
- bindings 过期时给出明确提示；
- 不自动提交生成文件；
- 不自动修改格式；
- 不执行 Tauri bundle；
- 成功后显示统一完成信息。

### .github/workflows/ci.yml

使用两个 Job：`quality` 和 `tauri-build`。

#### 通用配置

```yaml
name: CI

on:
  workflow_dispatch:
  push:
    branches: [main]
  pull_request:
    branches: [main]

permissions:
  contents: read

concurrency:
  group: ci-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always
```

#### quality Job

```yaml
quality:
  runs-on: windows-latest
  steps:
    - uses: actions/checkout@v7

    - uses: pnpm/action-setup@v6
      with:
        version: 10.33.2

    - uses: actions/setup-node@v6
      with:
        node-version: "22"
        cache: pnpm
        cache-dependency-path: pnpm-lock.yaml

    - uses: dtolnay/rust-toolchain@1.96.0
      with:
        components: rustfmt, clippy

    - uses: Swatinem/rust-cache@v2

    - run: pnpm install --frozen-lockfile

    - name: 完整质量检查
      shell: pwsh
      run: ./scripts/check.ps1
```

质量 Job 不重新手写每条检查，直接运行 `scripts/check.ps1`，保证 CI 和本地使用同一来源。

#### tauri-build Job

```yaml
tauri-build:
  needs: quality
  runs-on: windows-latest
  steps:
    - uses: actions/checkout@v7

    - uses: pnpm/action-setup@v6
      with:
        version: 10.33.2

    - uses: actions/setup-node@v6
      with:
        node-version: "22"
        cache: pnpm
        cache-dependency-path: pnpm-lock.yaml

    - uses: dtolnay/rust-toolchain@1.96.0
      with:
        components: rustfmt, clippy

    - uses: Swatinem/rust-cache@v2

    - run: pnpm install --frozen-lockfile

    - name: 构建 Tauri Release
      run: pnpm --filter @devforge/desktop tauri build --no-bundle --ci
```

**要求**：
- Task 9 只验证 Release 编译；
- 不生成 MSI 或 NSIS；
- 不上传安装包；
- 不创建 Release；
- 不使用签名密钥；
- 不使用 `tauri-action` 发布；
- 安装包与安装启动验证保留到 Task 10。

**CI Rust 工具链**：

```yaml
- uses: dtolnay/rust-toolchain@1.96.0
  with:
    components: rustfmt, clippy
```

与 `rust-toolchain.toml` 保持一致，不使用 `@stable`。

### bindings 检查

本地和 CI 都通过 `scripts/check.ps1` 执行：

```powershell
git diff --exit-code -- apps/desktop/src/bindings.ts
```

并检查 `$LASTEXITCODE`。

**原因**：
- Windows Runner 默认使用 PowerShell Core；
- 本地与 CI 应采用同一 PowerShell 行为；
- 避免维护两份不同的 bindings 校验逻辑。

### 验证命令

计划完成实施后，应从仓库根目录执行：

```powershell
pnpm install
pnpm check
pnpm --filter @devforge/desktop tauri build --no-bundle --ci
git diff --check
git status --short
```

并在推送后确认 GitHub Actions：

```text
quality       success
tauri-build   success
```

### 提交信息

```
chore: 建立 CI 和本地质量检查脚本
```

### 跨任务备注

#### Git Commit Hooks

Phase 0 文档要求 Commit Hooks，但当前 Task 9 不应在没有工具选择评审的情况下临时引入 Husky、Lefthook 或 simple-git-hooks。

需要：**在 Task 10 前新增独立 Commit Hooks 任务，或明确通过计划修订延期。**

#### Command Palette

继续保留已有备注：**Command Palette 尚未安排，需要新增独立任务或明确延期。**

---

## Task 10：Windows Release 冒烟验证

**依赖**：Task 9

**目标**：Release 构建可安装、可启动、可展示正确信息。

### 验证步骤

```powershell
# 1. Release 构建
cd apps/desktop
pnpm tauri build

# 2. 安装包位置
# target/release/bundle/msi/DevForge_0.1.0_x64_en-US.msi
# 或 target/release/bundle/nsis/DevForge_0.1.0_x64-setup.exe

# 3. 安装并启动

# 4. 验证：
# - 窗口标题为 "DevForge"
# - 显示版本号 0.1.0
# - 显示有效数据目录路径
# - 显示数据库状态 "就绪 (migration v1)"
# - 无 panic、无 crash

# 5. 验证数据目录（%LOCALAPPDATA%\DevForge，与 dirs::data_local_dir() 一致）
ls "$env:LOCALAPPDATA\DevForge"
# 应包含 devforge.db, devforge.db-wal, devforge.db-shm
```

**预期结果**：安装成功，启动正常，数据正确展示。

### 提交信息

```
chore: release 冒烟验证通过
```

---

## 依赖关系总览

```text
Task 1 (pnpm workspace)
   ↓
Task 2 (Rust workspace)
   ↓
Task 3 (Tauri + React 最小应用)
   ↓
Task 4 (IPC get_app_info)
   ↓                    ↓
Task 5 (React 展示)    Task 6 (SQLite migration)
   ↓                    ↓
   └─────── Task 7 ─────┘
            ↓
   Task 8 (Hash Router/Theme/ErrorBoundary)
            ↓
      Task 9 (CI + 质量检查)
            ↓
      Task 10 (Release 冒烟)
```

## 阶段退出条件检查清单

- [ ] Windows 开发环境可以一条命令启动（`pnpm tauri dev`）
- [ ] React 通过类型安全 IPC 调用 Rust 获取 AppInfo
- [ ] 应用展示版本号、数据目录、数据库状态
- [ ] SQLite Migration 能在空数据库运行并返回正确版本
- [ ] Rust Core 不直接依赖 React
- [ ] Application 层不依赖 Tauri（Phase 0 无 Domain 层）
- [ ] CI 可以运行 Rust 测试、Clippy、fmt、前端类型检查和测试
- [ ] Release 构建可以安装和启动
