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

### 类型来源

所有 TypeScript 类型从 `src/bindings.ts` 导入，由 specta 从 Rust 自动生成。
不使用手写 DTO，不使用 `invokeCommand<T>()` 类型断言。

### 依赖安装

```powershell
cd apps/desktop
pnpm add @tanstack/react-query
```

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

### apps/desktop/src/hooks/useAppInfo.ts

```typescript
import { useQuery } from "@tanstack/react-query";
import { commands, type AppInfo } from "../bindings";
import { appKeys } from "../queryKeys";

export function useAppInfo() {
  return useQuery<AppInfo>({
    queryKey: appKeys.info(),
    queryFn: () => commands.getAppInfo(),
    // 本地 IPC 调用，非网络请求；配置错误不应盲目重试
    retry: false,
    staleTime: 30_000,
  });
}
```

### apps/desktop/src/components/HealthStatus.tsx

```typescript
import type { DbStatus } from "../bindings";

interface HealthStatusProps {
  dbStatus: DbStatus;
}

export function HealthStatus({ dbStatus }: HealthStatusProps) {
  const label =
    dbStatus.type === "Ready"
      ? `就绪 (migration v${dbStatus.migration_version})`
      : dbStatus.type === "Error"
        ? `错误: ${dbStatus.message}`
        : "未初始化";

  return (
    <div data-testid="health-status">
      <strong>数据库：</strong>
      {label}
    </div>
  );
}
```

### apps/desktop/src/App.tsx

```typescript
import { useAppInfo } from "./hooks/useAppInfo";
import { HealthStatus } from "./components/HealthStatus";

export default function App() {
  const { data, isLoading, error } = useAppInfo();

  if (isLoading) return <div>加载中...</div>;
  if (error) return <div>加载失败: {String(error)}</div>;
  if (!data) return null;

  return (
    <div style={{ padding: 24, fontFamily: "sans-serif" }}>
      <h1>DevForge</h1>
      <dl>
        <dt>版本</dt>
        <dd>{data.version}</dd>
        <dt>数据目录</dt>
        <dd>{data.data_dir}</dd>
        <dt>健康状态</dt>
        <dd><HealthStatus dbStatus={data.db_status} /></dd>
      </dl>
    </div>
  );
}
```

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

### 验证命令

```powershell
cd apps/desktop
pnpm typecheck
pnpm tauri dev
```

**验证步骤**：
1. 窗口显示 "DevForge"、版本号、数据目录、数据库状态（"未初始化"）
2. 修改 Rust `AppInfo` 任意字段后重新 `cargo tauri dev`，前端 `pnpm typecheck` 应报错

### 提交信息

```
feat(ui): React 使用 specta 生成的绑定展示 AppInfo
```

---

## Task 6：SQLite Bootstrap 和第一条 Migration

**依赖**：Task 2

**目标**：`devforge-storage` 能打开 SQLite 数据库、运行迁移、返回 DbStatus。

**架构决策**：使用 SQLx + SqlitePool（符合架构文档规定），而非 rusqlite 单连接模型。SqlitePool 可克隆、支持异步、天然适合 Tauri State 持有。

### 精确文件

| 文件 | 职责 |
|------|------|
| `crates/devforge-storage/Cargo.toml` | 更新依赖（sqlx, tokio, tracing, thiserror, async-trait） |
| `crates/devforge-storage/src/lib.rs` | 导出模块 |
| `crates/devforge-storage/src/error.rs` | StorageError（首次出现真实 SQLx 失败时创建） |
| `crates/devforge-storage/src/pool.rs` | SQLite 连接池管理 |
| `crates/devforge-storage/src/migrator.rs` | Migration 运行器 |
| `crates/devforge-storage/migrations/20240101000001_init.sql` | 第一条 migration |
| `crates/devforge-storage/src/status.rs` | DbStatus 查询实现 |

### Cargo.toml 补充依赖

```toml
[dependencies]
async-trait = { workspace = true }
devforge-application = { workspace = true }
sqlx = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
thiserror = { workspace = true }
```

同时将 `thiserror` 和 `tracing` 加回 workspace.dependencies。

### migrations/20240101000001_init.sql

```sql
-- 应用元数据表
CREATE TABLE IF NOT EXISTS app_meta (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 插入 schema 版本
INSERT OR IGNORE INTO app_meta (key, value) VALUES ('schema_version', '1');

-- 工作区表（预留，验证 migration 链）
CREATE TABLE IF NOT EXISTS workspace (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

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
    pub async fn open(db_path: &Path) -> Result<Self, StorageError> {
        let opts = SqliteConnectOptions::new()
            .filename(db_path)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5))
            .foreign_keys(true)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await
            .map_err(|e| StorageError::Connection(format!("打开数据库失败: {e}")))?;

        Ok(Self { pool })
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
```

### crates/devforge-storage/src/migrator.rs

```rust
use sqlx::SqlitePool;
use crate::error::StorageError;

/// 运行所有待执行迁移
///
/// 使用 sqlx::migrate! 宏加载 migrations/ 目录下的 SQL 文件。
/// SQLx 自动创建 _sqlx_migrations 表记录已执行的迁移。
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), StorageError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| StorageError::Migration(format!("迁移失败: {e}")))
}

/// 获取当前 schema 版本
///
/// 从 _sqlx_migrations 表读取最大版本号。
pub async fn schema_version(pool: &SqlitePool) -> Result<u32, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query(format!("读取 schema 版本失败: {e}")))?;

    Ok(row.0 as u32)
}
```

### crates/devforge-storage/src/status.rs

```rust
use sqlx::SqlitePool;
use async_trait::async_trait;
use devforge_application::ports::DatabaseStatusProvider;
use devforge_application::app_info::DbStatus;
use crate::migrator;

/// 基于 SQLx 的数据库状态提供者
pub struct SqliteDatabaseStatus {
    pool: Option<SqlitePool>,
}

impl SqliteDatabaseStatus {
    pub fn new(pool: Option<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DatabaseStatusProvider for SqliteDatabaseStatus {
    async fn status(&self) -> DbStatus {
        match &self.pool {
            Some(pool) => match migrator::schema_version(pool).await {
                Ok(v) => DbStatus::Ready { migration_version: v },
                Err(e) => DbStatus::Error { message: e.to_string() },
            },
            None => DbStatus::NotInitialized,
        }
    }
}
```

### crates/devforge-storage/Cargo.toml（更新依赖）

```toml
[dependencies]
async-trait = { workspace = true }
devforge-application = { workspace = true }
sqlx = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
thiserror = { workspace = true }
```

### 验证命令

```powershell
cargo test -p devforge-storage
cargo clippy --workspace --all-targets -- -D warnings
```

**预期结果**：测试通过，migration 在临时数据库中执行成功。

### 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn migration_runs_on_empty_db() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).await.unwrap();
        migrator::run_migrations(db.pool()).await.unwrap();
        let version = migrator::schema_version(db.pool()).await.unwrap();
        assert_eq!(version, 1);
    }

    #[tokio::test]
    async fn pool_is_cloneable() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).await.unwrap();
        let pool_clone = db.pool().clone();
        // 克隆的池应能执行查询
        let version = migrator::schema_version(&pool_clone).await.unwrap();
        assert_eq!(version, 0); // 未运行迁移时版本为 0
    }
}
```

**Cargo.toml 补充**：

```toml
[dev-dependencies]
tempfile = "3"
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
```

### 提交信息

```
feat(storage): 使用 SQLx + SqlitePool 实现 SQLite bootstrap
```

---

## Task 7：串联 Storage 到 Tauri Command

**依赖**：Task 4, Task 6

**目标**：`get_app_info` 返回真实数据库状态。

**架构决策**：
- Tauri Command 仍通过 `GetAppInfo` Use Case 获取数据
- `DatabaseStatusProvider` trait 改为异步，支持 SQLx 异步查询
- Tauri State 持有 `GetAppInfo` 实例，不直接暴露 SqlitePool

### 调用链

```text
Tauri get_app_info Command（async）
        ↓
GetAppInfo.execute().await
        ↓
AppMetadataProvider + DatabaseStatusProvider（async）
        ↓
PlatformMetadata    + SqlitePool 查询 _sqlx_migrations
        ↓
AppInfo（组合后的最终 DTO）
```

### 精确文件修改

| 文件 | 变更 |
|------|------|
| `crates/devforge-application/src/ports.rs` | DatabaseStatusProvider 改为 async trait |
| `crates/devforge-application/src/get_app_info.rs` | execute 改为 async |
| `crates/devforge-storage/src/status.rs` | 实现 async DatabaseStatusProvider |
| `apps/desktop/src-tauri/Cargo.toml` | 添加依赖 |
| `apps/desktop/src-tauri/src/state.rs` | 使用 GetAppInfo + 真实 DbStatus |
| `apps/desktop/src-tauri/src/commands.rs` | async command |
| `apps/desktop/src-tauri/src/lib.rs` | async 初始化 |

### crates/devforge-application/Cargo.toml（补充依赖）

```toml
[dependencies]
async-trait = { workspace = true }
```

### crates/devforge-application/src/ports.rs（已在 Task 2 中定义）

无需修改，Task 2 已定义 async trait。

### crates/devforge-application/src/get_app_info.rs（已在 Task 2 中定义）

无需修改，Task 2 已定义 async execute。

### crates/devforge-storage/src/status.rs（已在 Task 6 中定义）

无需修改，Task 6 已实现 `SqliteDatabaseStatus`。

### apps/desktop/src-tauri/Cargo.toml（补充依赖）

```toml
[dependencies]
devforge-application = { workspace = true }
devforge-platform = { workspace = true }
devforge-storage = { workspace = true }
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
```

### apps/desktop/src-tauri/src/state.rs（更新）

```rust
use std::path::PathBuf;
use devforge_application::get_app_info::GetAppInfo;
use devforge_platform::app_info::PlatformMetadata;
use devforge_storage::pool::Database;
use devforge_storage::status::SqliteDatabaseStatus;

/// 应用全局状态
///
/// 持有 Application Use Case，Tauri Command 通过此状态调用业务逻辑。
pub struct AppState {
    pub get_app_info: GetAppInfo<PlatformMetadata, SqliteDatabaseStatus>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        let platform_metadata = PlatformMetadata::new(data_dir.clone());
        let db_status = SqliteDatabaseStatus::new(None);
        Self {
            get_app_info: GetAppInfo::new(platform_metadata, db_status),
            data_dir,
        }
    }

    /// 初始化数据库（运行迁移）
    ///
    /// 初始化完成后，替换 DatabaseStatusProvider 为持有真实 SqlitePool 的版本。
    pub async fn init_db(&mut self) -> Result<(), String> {
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("创建数据目录失败: {e}"))?;

        let db_path = self.data_dir.join("devforge.db");
        let db = Database::open(&db_path).await
            .map_err(|e| format!("打开数据库失败: {e}"))?;

        devforge_storage::migrator::run_migrations(db.pool()).await
            .map_err(|e| format!("运行迁移失败: {e}"))?;

        // 用持有真实 Pool 的版本替换
        let platform_metadata = PlatformMetadata::new(self.data_dir.clone());
        let db_status = SqliteDatabaseStatus::new(Some(db.pool().clone()));
        self.get_app_info = GetAppInfo::new(platform_metadata, db_status);

        Ok(())
    }
}
```

### apps/desktop/src-tauri/src/commands.rs（更新）

```rust
use tauri::State;
use devforge_application::app_info::AppInfo;
use crate::state::AppState;

/// 获取应用信息
///
/// 通过 Application Use Case 获取，不在 Command 中构造业务逻辑。
#[specta::specta]
#[tauri::command]
pub async fn get_app_info(state: State<'_, AppState>) -> AppInfo {
    state.get_app_info.execute().await
}
```

### apps/desktop/src-tauri/src/lib.rs（更新）

```rust
mod commands;
mod state;

use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};
use state::AppState;

/// 创建 specta Builder（绑定生成和 run 共用同一个 Builder）
fn create_builder() -> Builder<tauri::Wry> {
    Builder::new()
        .commands(collect_commands![commands::get_app_info,])
}

/// 导出 specta 生成的 TypeScript 绑定
pub fn export_bindings() {
    let builder = create_builder();
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("导出 TypeScript 绑定失败");
}

/// 初始化应用状态（异步，需要 tokio runtime）
///
/// 使用 dirs::data_local_dir()（%LOCALAPPDATA%\DevForge）作为数据目录。
pub async fn init() -> Result<AppState, String> {
    let data_dir = dirs::data_local_dir()
        .expect("无法获取数据目录")
        .join("DevForge");

    let mut app_state = AppState::new(data_dir);
    app_state.init_db().await?;
    Ok(app_state)
}

/// 启动 Tauri 应用（同步，阻塞直到退出）
pub fn run(app_state: AppState) {
    let builder = create_builder();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
```

### apps/desktop/src-tauri/src/main.rs（更新）

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() {
    match devforge_desktop_lib::init().await {
        Ok(app_state) => devforge_desktop_lib::run(app_state),
        Err(e) => {
            eprintln!("应用初始化失败: {e}");
            std::process::exit(1);
        }
    }
}
```

### 验证命令

```powershell
cargo test --workspace
cd apps/desktop
pnpm typecheck
pnpm tauri dev
```

**预期结果**：窗口显示数据库状态为 "就绪 (migration v1)"。数据目录下生成 `devforge.db` 和 `devforge.db-wal`。

### 提交信息

```
feat(storage): 串联 SQLite 到 Tauri get_app_info 命令
```

---

## Task 8：前端基础设施 — Router、Theme、ErrorBoundary

**依赖**：Task 5

**目标**：建立前端路由、主题切换、错误边界基础框架。

### 精确文件

| 文件 | 职责 |
|------|------|
| `apps/desktop/src/main.tsx` | 更新：添加 Router |
| `apps/desktop/src/App.tsx` | 更新：使用 layout |
| `apps/desktop/src/router.tsx` | 路由配置 |
| `apps/desktop/src/layouts/AppLayout.tsx` | 基础布局骨架 |
| `apps/desktop/src/pages/HomePage.tsx` | 首页（原 App 内容） |
| `apps/desktop/src/pages/SettingsPage.tsx` | 设置页占位 |
| `apps/desktop/src/components/ErrorBoundary.tsx` | 错误边界 |
| `apps/desktop/src/stores/ui.ts` | Zustand UI 状态 |
| `apps/desktop/src/styles/global.css` | 全局样式 |

### 依赖安装

```powershell
cd apps/desktop
pnpm add zustand
```

### apps/desktop/src/router.tsx

```typescript
import { createBrowserRouter } from "react-router";
import { AppLayout } from "./layouts/AppLayout";
import { HomePage } from "./pages/HomePage";
import { SettingsPage } from "./pages/SettingsPage";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppLayout />,
    children: [
      { index: true, element: <HomePage /> },
      { path: "settings", element: <SettingsPage /> },
    ],
  },
]);
```

### apps/desktop/src/layouts/AppLayout.tsx

```typescript
import { Outlet } from "react-router";

export function AppLayout() {
  return (
    <div className="app-layout">
      <aside className="activity-bar">
        {/* Activity Bar 占位 */}
      </aside>
      <main className="main-content">
        <Outlet />
      </main>
    </div>
  );
}
```

### apps/desktop/src/components/ErrorBoundary.tsx

```typescript
import { Component, type ReactNode } from "react";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return (
        this.props.fallback ?? (
          <div style={{ padding: 24, color: "red" }}>
            <h2>发生错误</h2>
            <pre>{this.state.error?.message}</pre>
          </div>
        )
      );
    }
    return this.props.children;
  }
}
```

### apps/desktop/src/stores/ui.ts

```typescript
import { create } from "zustand";

interface UIState {
  theme: "light" | "dark" | "system";
  sidebarCollapsed: boolean;
  setTheme: (theme: UIState["theme"]) => void;
  toggleSidebar: () => void;
}

export const useUIStore = create<UIState>((set) => ({
  theme: "system",
  sidebarCollapsed: false,
  setTheme: (theme) => set({ theme }),
  toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
}));
```

### apps/desktop/src/main.tsx（更新）

```typescript
import React from "react";
import ReactDOM from "react-dom/client";
import { RouterProvider } from "react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { router } from "./router";
import "./styles/global.css";

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </ErrorBoundary>
  </React.StrictMode>,
);
```

### apps/desktop/src/styles/global.css

```css
*,
*::before,
*::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html, body, #root {
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

.app-layout {
  display: flex;
  height: 100%;
}

.activity-bar {
  width: 48px;
  background: var(--bg-secondary, #f5f5f5);
  border-right: 1px solid var(--border, #e0e0e0);
}

.main-content {
  flex: 1;
  overflow: auto;
}
```

### 验证命令

```powershell
cd apps/desktop
pnpm typecheck
pnpm tauri dev
# 访问 http://localhost:1420/settings 验证路由
```

**预期结果**：首页正常显示，`/settings` 路由可访问，ErrorBoundary 能捕获渲染错误。

### 提交信息

```
feat(frontend): 建立 Router、Theme、ErrorBoundary 基础设施
```

---

## Task 9：工程质量和 CI

**依赖**：Task 2, Task 3, Task 8

**目标**：本地一条命令验证全部质量检查，GitHub Actions 基础 CI 可用。

### 精确文件

| 文件 | 职责 |
|------|------|
| `.github/workflows/ci.yml` | GitHub Actions CI 配置 |
| `scripts/check.ps1` | 本地一键验证脚本 |
| `apps/desktop/src-tauri/rustfmt.toml` | Rust 格式配置（如需覆盖默认） |
| `apps/desktop/vitest.config.ts` | Vitest 配置 |
| `apps/desktop/src/__tests__/App.test.tsx` | 前端基础测试 |

### .github/workflows/ci.yml

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  rust:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: 格式检查
        run: cargo fmt --check
      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: 测试
        run: cargo test --workspace

  bindings:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: 重新生成 specta 绑定
        run: cargo run -p devforge-desktop --bin export_bindings
      - name: 检查绑定是否最新
        run: |
          git diff --exit-code -- apps/desktop/src/bindings.ts || (
            echo "::error::apps/desktop/src/bindings.ts 与 Rust 类型不同步，请运行 pnpm bindings:generate 并提交更新"
            exit 1
          )

  frontend:
    needs: [bindings]
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: pnpm
      - run: pnpm install
      - name: 类型检查
        run: pnpm --filter @devforge/desktop typecheck
      - name: 测试
        run: pnpm --filter @devforge/desktop test

  build:
    needs: [rust, frontend]
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: pnpm
      - run: pnpm install
      - name: 构建 Tauri 应用
        run: pnpm --filter @devforge/desktop tauri build
```

### scripts/check.ps1

```powershell
# 本地一键验证脚本
$ErrorActionPreference = "Stop"

Write-Host "=== Rust 格式检查 ===" -ForegroundColor Cyan
cargo fmt --check

Write-Host "`n=== Rust Clippy ===" -ForegroundColor Cyan
cargo clippy --workspace --all-targets -- -D warnings

Write-Host "`n=== Rust 测试 ===" -ForegroundColor Cyan
cargo test --workspace

Write-Host "`n=== 重新生成 specta 绑定 ===" -ForegroundColor Cyan
pnpm bindings:generate

Write-Host "`n=== 检查绑定是否最新 ===" -ForegroundColor Cyan
$diff = git diff --stat -- apps/desktop/src/bindings.ts
if ($diff) {
    Write-Host "apps/desktop/src/bindings.ts 与 Rust 类型不同步，请提交更新" -ForegroundColor Red
    exit 1
}

Write-Host "`n=== 前端类型检查 ===" -ForegroundColor Cyan
pnpm --filter @devforge/desktop typecheck

Write-Host "`n=== 前端测试 ===" -ForegroundColor Cyan
pnpm --filter @devforge/desktop test

Write-Host "`n=== 全部通过 ✓ ===" -ForegroundColor Green
```

**类型同步验证原理**：
- `src/bindings.ts` 由 specta 从 Rust 类型生成，提交到 git
- CI 重新生成绑定（`cargo run -p devforge-desktop --bin export_bindings`），再用 `git diff --exit-code -- apps/desktop/src/bindings.ts` 检查是否最新
- 本地脚本通过 `pnpm bindings:generate` 重新生成
- 前端 `pnpm typecheck` 验证所有 import 与 bindings 一致
- 仅运行 typecheck 不能发现 bindings 过期，必须先重新生成再 diff
- 这实现了"Rust 与 TypeScript 类型同步方案确定"的退出条件

### apps/desktop/vitest.config.ts

```typescript
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: "jsdom",
  },
});
```

### apps/desktop/src/__tests__/App.test.tsx

```typescript
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, it, expect, vi } from "vitest";
import App from "../App";

// Mock specta 生成的 bindings
vi.mock("../bindings", () => ({
  commands: {
    getAppInfo: vi.fn().mockResolvedValue({
      version: "0.1.0",
      data_dir: "/tmp/test",
      db_status: { type: "NotInitialized" },
    }),
  },
}));

function renderWithProviders(ui: React.ReactElement) {
  const client = new QueryClient();
  return render(
    <QueryClientProvider client={client}>{ui}</QueryClientProvider>,
  );
}

describe("App", () => {
  it("显示加载状态", () => {
    renderWithProviders(<App />);
    expect(screen.getByText("加载中...")).toBeTruthy();
  });
});
```

### 前端测试依赖

```powershell
cd apps/desktop
pnpm add -D @testing-library/react @testing-library/jest-dom jsdom
```

### 验证命令

```powershell
# 运行完整检查
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

**预期结果**：所有检查通过，0 error。

### 提交信息

```
chore: 建立 CI 和本地质量检查脚本
```

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
      Task 8 (Router/Theme/ErrorBoundary)
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
