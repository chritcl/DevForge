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
- 各 crate 定义自己的错误类型，不使用统一的 AppError

### 精确文件

| 文件 | 职责 |
|------|------|
| `Cargo.toml` | workspace 根，定义 members、workspace.dependencies |
| `crates/devforge-application/Cargo.toml` | 应用服务 crate |
| `crates/devforge-application/src/lib.rs` | crate 入口 |
| `crates/devforge-application/src/app_info.rs` | AppInfo、DbStatus 类型定义 |
| `crates/devforge-application/src/ports.rs` | Port trait 定义 |
| `crates/devforge-application/src/error.rs` | ApplicationError |
| `crates/devforge-application/src/get_app_info.rs` | get_app_info 用例 |
| `crates/devforge-storage/Cargo.toml` | 存储 crate |
| `crates/devforge-storage/src/lib.rs` | crate 入口 |
| `crates/devforge-storage/src/error.rs` | StorageError |
| `crates/devforge-platform/Cargo.toml` | 平台适配 crate |
| `crates/devforge-platform/src/lib.rs` | crate 入口 |
| `crates/devforge-platform/src/error.rs` | PlatformError |

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
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
async-trait = "0.1"
specta = { version = "=2.0.0-rc.25", features = ["derive"] }
specta-typescript = "=0.0.12"
tauri-specta = { version = "=2.0.0-rc.25", features = ["typescript"] }

# 内部 crate
devforge-application = { path = "crates/devforge-application" }
devforge-storage = { path = "crates/devforge-storage" }
devforge-platform = { path = "crates/devforge-platform" }
```

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
thiserror = { workspace = true }
tokio = { workspace = true }
```

### devforge-application/src/lib.rs

```rust
pub mod app_info;
pub mod error;
pub mod get_app_info;
pub mod ports;
```

### devforge-application/src/app_info.rs

```rust
use serde::Serialize;
use specta::Type;

/// 应用基础信息（诊断 DTO）
///
/// 用于向 UI 展示应用状态，不是领域实体。
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

### devforge-application/src/error.rs

```rust
use thiserror::Error;

/// 应用层错误
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("端口错误: {0}")]
    Port(String),

    #[error("用例执行失败: {0}")]
    UseCase(String),
}
```

### devforge-application/src/ports.rs

```rust
use async_trait::async_trait;
use crate::app_info::{AppInfo, DbStatus};

/// 应用信息查询端口
pub trait AppInfoProvider: Send + Sync {
    fn get_app_info(&self) -> AppInfo;
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
use crate::ports::{AppInfoProvider, DatabaseStatusProvider};

/// 获取应用信息用例
pub struct GetAppInfo<P: AppInfoProvider, D: DatabaseStatusProvider> {
    app_info: P,
    db_status: D,
}

impl<P: AppInfoProvider, D: DatabaseStatusProvider> GetAppInfo<P, D> {
    pub fn new(app_info: P, db_status: D) -> Self {
        Self { app_info, db_status }
    }

    /// 执行用例，返回应用信息
    pub async fn execute(&self) -> AppInfo {
        let mut info = self.app_info.get_app_info();
        info.db_status = self.db_status.status().await;
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_info::DbStatus;
    use async_trait::async_trait;

    struct MockAppInfo;
    impl AppInfoProvider for MockAppInfo {
        fn get_app_info(&self) -> AppInfo {
            AppInfo {
                version: "0.1.0".into(),
                data_dir: "/tmp/test".into(),
                db_status: DbStatus::NotInitialized,
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
    async fn get_app_info_merges_db_status() {
        let use_case = GetAppInfo::new(MockAppInfo, MockDbReady);
        let info = use_case.execute().await;
        assert_eq!(info.version, "0.1.0");
        assert!(matches!(info.db_status, DbStatus::Ready { .. }));
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
thiserror = { workspace = true }
tracing = { workspace = true }
```

### devforge-storage/src/lib.rs

```rust
pub mod error;
```

### devforge-storage/src/error.rs

```rust
use thiserror::Error;

/// 存储层错误
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("数据库连接失败: {0}")]
    Connection(String),

    #[error("迁移失败: {0}")]
    Migration(String),

    #[error("查询失败: {0}")]
    Query(String),

    #[error("配置错误: {0}")]
    Config(String),
}
```

### devforge-platform/Cargo.toml

```toml
[package]
name = "devforge-platform"
version.workspace = true
edition.workspace = true

[dependencies]
thiserror = { workspace = true }
```

### devforge-platform/src/lib.rs

```rust
pub mod error;
```

### devforge-platform/src/error.rs

```rust
use thiserror::Error;

/// 平台层错误
#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("系统路径错误: {0}")]
    Path(String),

    #[error("系统调用失败: {0}")]
    SystemCall(String),

    #[error("权限错误: {0}")]
    Permission(String),
}
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
| `apps/desktop/package.json` | 前端包定义，name: @devforge/desktop |
| `apps/desktop/tsconfig.json` | TypeScript 配置 |
| `apps/desktop/tsconfig.node.json` | Node 侧 TS 配置 |
| `apps/desktop/vite.config.ts` | Vite 配置 |
| `apps/desktop/index.html` | HTML 入口 |
| `apps/desktop/src/main.tsx` | React 入口 |
| `apps/desktop/src/App.tsx` | 根组件（占位） |
| `apps/desktop/src/vite-env.d.ts` | Vite 类型声明 |
| `apps/desktop/src-tauri/Cargo.toml` | Tauri 宿主 crate |
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
    "test": "vitest run",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "react": "^19.1.0",
    "react-dom": "^19.1.0",
    "react-router": "^7"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@types/react": "^19",
    "@types/react-dom": "^19",
    "@vitejs/plugin-react": "^4",
    "typescript": "~5.8",
    "vite": "^7",
    "vitest": "^3"
  }
}
```

### apps/desktop/src-tauri/Cargo.toml

```toml
[package]
name = "devforge-desktop"
version.workspace = true
edition.workspace = true

[lib]
name = "devforge_desktop_lib"
crate-type = ["lib", "cdylib", "staticlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
serde = { workspace = true }
serde_json = { workspace = true }
devforge-application = { workspace = true }
devforge-platform = { workspace = true }
```

### apps/desktop/src-tauri/tauri.conf.json

```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-config-schema/schema.json",
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
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    devforge_desktop_lib::run();
}
```

### 验证命令

```powershell
cd apps/desktop
pnpm install
pnpm tauri dev
```

**预期结果**：窗口弹出，显示 React 占位内容。Ctrl+C 退出无 panic。

### 提交信息

```
feat(desktop): 创建最小 Tauri + React 应用
```

---

## Task 4：建立 IPC 层 — get_app_info 命令（specta 类型生成）

**依赖**：Task 2, Task 3

**目标**：React 能通过 Tauri Command 调用 Rust 并获取 AppInfo，TypeScript 类型由 specta 从 Rust 自动推导。

**架构决策**：
- Tauri Command 只调用 Application Use Case，不自行构造业务逻辑
- TypeScript 类型由 specta 从 Rust `#[derive(Type)]` 自动生成，不是手写 DTO
- 前端通过 specta 生成的绑定调用命令，类型在编译期保证一致

### 调用链

```text
Tauri get_app_info Command（#[specta::specta]）
        ↓
GetAppInfo Application Service
        ↓
PlatformInfoProvider + DatabaseStatusProvider
        ↓
Platform / Storage Adapter
        ↓
specta 自动生成 TypeScript 类型和命令绑定
```

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
| `crates/devforge-platform/src/app_info.rs` | PlatformInfoProvider 实现 |
| `apps/desktop/src-tauri/src/commands.rs` | Tauri Command 定义（带 specta 注解） |
| `apps/desktop/src-tauri/src/state.rs` | 管理 Application Service |
| `apps/desktop/src-tauri/src/lib.rs` | 注册 command、state、specta 导出 |
| `apps/desktop/src-tauri/Cargo.toml` | 添加 specta 依赖 |
| `apps/desktop/src/bindings.ts` | specta 生成的类型和命令绑定（提交到 git） |
| `apps/desktop/src-tauri/src/bin/export_bindings.rs` | 独立绑定生成入口 |

### crates/devforge-platform/src/app_info.rs

```rust
use devforge_application::ports::AppInfoProvider;
use devforge_application::app_info::AppInfo;

/// 平台信息提供者
///
/// data_dir 由调用方传入，确保与数据库初始化使用同一个路径。
pub struct PlatformInfo {
    data_dir: String,
}

impl PlatformInfo {
    pub fn new(data_dir: String) -> Self {
        Self { data_dir }
    }
}

impl AppInfoProvider for PlatformInfo {
    fn get_app_info(&self) -> AppInfo {
        AppInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            data_dir: self.data_dir.clone(),
            db_status: Default::default(), // 由 DatabaseStatusProvider 覆盖
        }
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
pub mod app_info;
pub mod error;
```

### apps/desktop/src-tauri/src/state.rs

```rust
use std::path::PathBuf;
use devforge_application::get_app_info::GetAppInfo;
use devforge_platform::app_info::PlatformInfo;
use devforge_application::ports::NotInitializedDbStatus;

/// 应用全局状态
///
/// 持有 Application Service，Tauri Command 通过此状态调用业务逻辑。
pub struct AppState {
    pub get_app_info: GetAppInfo<PlatformInfo, NotInitializedDbStatus>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        let data_dir_str = data_dir.to_string_lossy().to_string();
        let platform_info = PlatformInfo::new(data_dir_str);
        let db_status = NotInitializedDbStatus;
        Self {
            get_app_info: GetAppInfo::new(platform_info, db_status),
            data_dir,
        }
    }
}
```

### apps/desktop/src-tauri/src/commands.rs

```rust
use tauri::State;
use devforge_application::app_info::AppInfo;
use crate::state::AppState;

/// 获取应用信息
///
/// specta 从此函数签名自动推导 TypeScript 类型。
/// State 参数由 Tauri 注入，specta 自动排除，不出现在 TS 签名中。
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
///
/// 输出路径基于 CARGO_MANIFEST_DIR 构造，不依赖进程当前工作目录。
/// 输出文件：apps/desktop/src/bindings.ts
/// 该文件提交到 git，前端直接 import 使用。
pub fn export_bindings() {
    let builder = create_builder();
    let out_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("src")
        .join("bindings.ts");
    builder
        .export(Typescript::default(), &out_path)
        .expect("导出 TypeScript 绑定失败");
}

pub fn run() {
    let builder = create_builder();
    let data_dir = dirs::data_local_dir()
        .expect("无法获取数据目录")
        .join("DevForge");
    let app_state = AppState::new(data_dir);

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
```

### apps/desktop/src-tauri/src/main.rs

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    devforge_desktop_lib::run();
}
```

### apps/desktop/src-tauri/src/bin/export_bindings.rs（新增）

独立的绑定生成入口，供 CI 和本地开发使用。输出路径基于 `CARGO_MANIFEST_DIR`，不依赖 cwd。

```rust
/// 独立绑定生成入口
///
/// 运行方式：cargo run -p devforge-desktop --bin export_bindings
/// 输出：apps/desktop/src/bindings.ts（基于 CARGO_MANIFEST_DIR 构造）
fn main() {
    devforge_desktop_lib::export_bindings();
    println!("绑定已导出");
}
```

### apps/desktop/src-tauri/Cargo.toml（补充依赖）

```toml
[dependencies]
dirs = "6"
devforge-application = { workspace = true }
devforge-platform = { workspace = true }
specta = { workspace = true }
specta-typescript = { workspace = true }
tauri-specta = { workspace = true }
```

### apps/desktop/src/bindings.ts（specta 生成）

此文件由 specta 自动生成，提交到 git 作为类型契约。前端直接 import 使用。

```typescript
/** specta 自动生成，请勿手动编辑 */

/** 数据库状态 */
export type DbStatus =
  | { type: "NotInitialized" }
  | { type: "Ready"; migration_version: number }
  | { type: "Error"; message: string };

/** 应用基础信息 */
export interface AppInfo {
  version: string;
  data_dir: string;
  db_status: DbStatus;
}

/** 获取应用信息 */
export const commands = {
  async getAppInfo(): Promise<AppInfo> {
    return await window.__TAURI__.core.invoke("get_app_info");
  },
};
```

**重要**：以上内容仅为示意，实际文件由 `pnpm bindings:generate`（即 `cargo run -p devforge-desktop --bin export_bindings`）显式生成。如果生成结果与示意不同，以 specta 输出为准。

### 验证命令

```powershell
cargo test -p devforge-application
cargo test -p devforge-platform
cargo clippy --workspace --all-targets -- -D warnings
cd apps/desktop
pnpm install
pnpm typecheck
pnpm tauri dev
```

**验证步骤**：
1. `pnpm bindings:generate` 后检查 `apps/desktop/src/bindings.ts` 是否生成
2. 前端 `pnpm typecheck` 通过
3. `commands.getAppInfo()` 返回正确的 AppInfo
4. 任意修改 Rust AppInfo 字段后重新生成绑定，`pnpm typecheck` 应报错（类型一致性验证）

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
| `crates/devforge-storage/Cargo.toml` | 更新依赖 |
| `crates/devforge-storage/src/lib.rs` | 导出模块 |
| `crates/devforge-storage/src/pool.rs` | SQLite 连接池管理 |
| `crates/devforge-storage/src/migrator.rs` | Migration 运行器 |
| `crates/devforge-storage/migrations/20240101000001_init.sql` | 第一条 migration |
| `crates/devforge-storage/src/status.rs` | DbStatus 查询实现 |

### Cargo.toml 补充依赖

```toml
[dependencies]
sqlx = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
```

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
PlatformInfoProvider + DatabaseStatusProvider（async）
        ↓
SqlitePool 查询 _sqlx_migrations
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
use devforge_platform::app_info::PlatformInfo;
use devforge_storage::pool::Database;
use devforge_storage::status::SqliteDatabaseStatus;

/// 应用全局状态
///
/// 持有 Application Use Case，Tauri Command 通过此状态调用业务逻辑。
pub struct AppState {
    pub get_app_info: GetAppInfo<PlatformInfo, SqliteDatabaseStatus>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        let data_dir_str = data_dir.to_string_lossy().to_string();
        let platform_info = PlatformInfo::new(data_dir_str);
        let db_status = SqliteDatabaseStatus::new(None);
        Self {
            get_app_info: GetAppInfo::new(platform_info, db_status),
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
        let data_dir_str = self.data_dir.to_string_lossy().to_string();
        let platform_info = PlatformInfo::new(data_dir_str);
        let db_status = SqliteDatabaseStatus::new(Some(db.pool().clone()));
        self.get_app_info = GetAppInfo::new(platform_info, db_status);

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
