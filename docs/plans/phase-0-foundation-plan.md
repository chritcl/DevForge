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
    "build:desktop": "pnpm --filter @devforge/desktop build"
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

**设计决策**：Phase 0 只创建 4 个 crate。`devforge-runtime` 等出现 TaskSupervisor 再创建；`devforge-shared` 容易变成垃圾桶，暂缓。错误类型暂放 `devforge-domain`，跨 crate 共享时再拆。

### 精确文件

| 文件 | 职责 |
|------|------|
| `Cargo.toml` | workspace 根，定义 members、workspace.dependencies |
| `crates/devforge-domain/Cargo.toml` | 领域模型 crate |
| `crates/devforge-domain/src/lib.rs` | crate 入口，导出 app_info 和 error 模块 |
| `crates/devforge-domain/src/app_info.rs` | AppInfo 领域类型 |
| `crates/devforge-domain/src/error.rs` | 统一错误枚举 |
| `crates/devforge-application/Cargo.toml` | 应用服务 crate |
| `crates/devforge-application/src/lib.rs` | crate 入口 |
| `crates/devforge-application/src/ports.rs` | Port trait 定义 |
| `crates/devforge-application/src/get_app_info.rs` | get_app_info 用例 |
| `crates/devforge-storage/Cargo.toml` | 存储 crate |
| `crates/devforge-storage/src/lib.rs` | crate 入口 |
| `crates/devforge-platform/Cargo.toml` | 平台适配 crate |
| `crates/devforge-platform/src/lib.rs` | crate 入口 |

### 根 Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/devforge-domain",
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

# 内部 crate
devforge-domain = { path = "crates/devforge-domain" }
devforge-application = { path = "crates/devforge-application" }
devforge-storage = { path = "crates/devforge-storage" }
devforge-platform = { path = "crates/devforge-platform" }
```

### devforge-domain/src/lib.rs

```rust
pub mod app_info;
pub mod error;
```

### devforge-domain/src/error.rs

```rust
use thiserror::Error;

/// 统一应用错误
///
/// Phase 0 暂放 domain 层。当出现跨 crate 共享需求时，
/// 拆出 devforge-shared。
#[derive(Debug, Error)]
pub enum AppError {
    #[error("存储错误: {0}")]
    Storage(String),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("平台错误: {0}")]
    Platform(String),

    #[error("未找到: {0}")]
    NotFound(String),
}
```

### devforge-domain/src/app_info.rs

```rust
use serde::{Deserialize, Serialize};

/// 应用基础信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub version: &'static str,
    pub data_dir: String,
    pub db_status: DbStatus,
}

/// 数据库状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DbStatus {
    NotInitialized,
    Ready { migration_version: u32 },
    Error { message: String },
}
```

### devforge-application/src/ports.rs

```rust
use devforge_domain::app_info::{AppInfo, DbStatus};

/// 应用信息查询端口
pub trait AppInfoProvider: Send + Sync {
    fn get_app_info(&self) -> AppInfo;
}

/// 数据库状态查询端口
pub trait DatabaseStatusProvider: Send + Sync {
    fn status(&self) -> DbStatus;
}
```

### devforge-application/src/get_app_info.rs

```rust
use devforge_domain::app_info::AppInfo;
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

    pub fn execute(&self) -> AppInfo {
        let mut info = self.app_info.get_app_info();
        info.db_status = self.db_status.status();
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use devforge_domain::app_info::DbStatus;

    struct MockAppInfo;
    impl AppInfoProvider for MockAppInfo {
        fn get_app_info(&self) -> AppInfo {
            AppInfo {
                version: "0.1.0",
                data_dir: "/tmp/test".into(),
                db_status: DbStatus::NotInitialized,
            }
        }
    }

    struct MockDbReady;
    impl DatabaseStatusProvider for MockDbReady {
        fn status(&self) -> DbStatus {
            DbStatus::Ready { migration_version: 1 }
        }
    }

    #[test]
    fn get_app_info_merges_db_status() {
        let use_case = GetAppInfo::new(MockAppInfo, MockDbReady);
        let info = use_case.execute();
        assert_eq!(info.version, "0.1.0");
        assert!(matches!(info.db_status, DbStatus::Ready { .. }));
    }
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
feat(rust): 创建 Rust workspace 和最小 crate 骨架
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
    "@tauri-apps/plugin-shell": "^2",
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
tauri-plugin-shell = "2"
serde = { workspace = true }
serde_json = { workspace = true }
devforge-domain = { workspace = true }
devforge-application = { workspace = true }
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
    "security": {
      "csp": null
    }
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
    "core:default",
    "shell:allow-open"
  ]
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

## Task 4：建立 IPC 层 — get_app_info 命令

**依赖**：Task 2, Task 3

**目标**：React 能通过 Tauri Command 调用 Rust 并获取 AppInfo。

### 精确文件

| 文件 | 职责 |
|------|------|
| `apps/desktop/src-tauri/src/commands.rs` | Tauri Command 定义 |
| `apps/desktop/src/lib.rs` | 注册 command |
| `apps/desktop/src/types/generated.ts` | 从 Rust 生成的 TS 类型（手动维护阶段） |
| `apps/desktop/src/api/client.ts` | IPC 客户端封装 |
| `apps/desktop/src/api/app.ts` | app 相关 API |

### apps/desktop/src-tauri/src/commands.rs

```rust
use devforge_domain::app_info::{AppInfo, DbStatus};

/// 获取应用信息
#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION"),
        data_dir: dirs::data_dir()
            .map(|p| p.join("devforge").to_string_lossy().to_string())
            .unwrap_or_default(),
        db_status: DbStatus::NotInitialized,
    }
}
```

### apps/desktop/src-tauri/src/lib.rs（更新）

```rust
mod commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
        ])
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

### apps/desktop/src/types/generated.ts

```typescript
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
```

### apps/desktop/src/api/client.ts

```typescript
import { invoke } from "@tauri-apps/api/core";

/**
 * 类型安全的 IPC 调用封装
 * 所有 Tauri Command 调用必须通过此函数
 */
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  return invoke<T>(command, args);
}
```

### apps/desktop/src/api/app.ts

```typescript
import { invokeCommand } from "./client";
import type { AppInfo } from "../types/generated";

/** 获取应用信息 */
export function getAppInfo(): Promise<AppInfo> {
  return invokeCommand<AppInfo>("get_app_info");
}
```

### apps/desktop/src-tauri/Cargo.toml 补充依赖

```toml
[dependencies]
dirs = "6"
```

### 验证命令

```powershell
cd apps/desktop
pnpm typecheck
pnpm tauri dev
# 在 DevTools Console 中测试:
# await invoke("get_app_info")
```

**预期结果**：`invoke("get_app_info")` 返回包含 version、data_dir、db_status 的对象。

### 提交信息

```
feat(ipc): 建立 get_app_info Tauri Command 和 IPC 客户端层
```

---

## Task 5：React 展示 AppInfo 和健康状态

**依赖**：Task 4

**目标**：应用启动后自动获取并展示 AppInfo。

### 精确文件

| 文件 | 职责 |
|------|------|
| `apps/desktop/src/App.tsx` | 根组件，展示应用信息 |
| `apps/desktop/src/components/HealthStatus.tsx` | 健康状态组件 |
| `apps/desktop/src/hooks/useAppInfo.ts` | AppInfo 查询 hook |
| `apps/desktop/src/main.tsx` | 更新：挂载 QueryClientProvider |

### 依赖安装

```powershell
cd apps/desktop
pnpm add @tanstack/react-query
```

### apps/desktop/src/hooks/useAppInfo.ts

```typescript
import { useQuery } from "@tanstack/react-query";
import { getAppInfo } from "../api/app";
import type { AppInfo } from "../types/generated";

export function useAppInfo() {
  return useQuery<AppInfo>({
    queryKey: ["app-info"],
    queryFn: getAppInfo,
    staleTime: 30_000,
  });
}
```

### apps/desktop/src/components/HealthStatus.tsx

```typescript
import type { DbStatus } from "../types/generated";

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

**预期结果**：窗口显示 "DevForge"、版本号、数据目录、数据库状态（"未初始化"）。

### 提交信息

```
feat(ui): React 展示 AppInfo 和健康状态
```

---

## Task 6：SQLite Bootstrap 和第一条 Migration

**依赖**：Task 2

**目标**：`devforge-storage` 能打开 SQLite 数据库、运行迁移、返回 DbStatus。

### 精确文件

| 文件 | 职责 |
|------|------|
| `crates/devforge-storage/Cargo.toml` | 更新依赖 |
| `crates/devforge-storage/src/lib.rs` | 导出模块 |
| `crates/devforge-storage/src/pool.rs` | SQLite 连接池管理 |
| `crates/devforge-storage/src/migrator.rs` | Migration 运行器 |
| `crates/devforge-storage/migrations/001_init.sql` | 第一条 migration |
| `crates/devforge-storage/src/status.rs` | DbStatus 查询实现 |

### Cargo.toml 补充依赖

```toml
[dependencies]
devforge-domain = { workspace = true }
rusqlite = { version = "0.35", features = ["bundled"] }
rusqlite_migration = "1"
tracing = { workspace = true }
```

### migrations/001_init.sql

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
use rusqlite::Connection;
use devforge_domain::error::AppError;

/// SQLite 连接管理
pub struct Database {
    conn: Connection,
}

impl Database {
    /// 打开数据库并启用 WAL 模式
    pub fn open(db_path: &Path) -> Result<Self, AppError> {
        let conn = Connection::open(db_path)
            .map_err(|e| AppError::Storage(format!("打开数据库失败: {e}")))?;

        // 启用 WAL 模式
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| AppError::Storage(format!("设置 WAL 失败: {e}")))?;

        Ok(Self { conn })
    }

    /// 获取底层连接引用
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}
```

### crates/devforge-storage/src/migrator.rs

```rust
use rusqlite::Connection;
use rusqlite_migration::Migrations;
use devforge_domain::error::AppError;

/// 获取所有迁移
pub fn migrations() -> Migrations<'static> {
    Migrations::from_iter([
        rusqlite_migration::Migration::new(
            "001_init",
            include_str!("../migrations/001_init.sql"),
        ),
    ])
}

/// 运行所有待执行迁移
pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    let mut migs = migrations();
    migs.to_latest(conn)
        .map_err(|e| AppError::Storage(format!("迁移失败: {e}")))
}

/// 获取当前 schema 版本
pub fn schema_version(conn: &Connection) -> Result<u32, AppError> {
    let version: String = conn
        .query_row(
            "SELECT value FROM app_meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Storage(format!("读取 schema 版本失败: {e}")))?;

    version
        .parse::<u32>()
        .map_err(|e| AppError::Storage(format!("schema 版本格式错误: {e}")))
}
```

### crates/devforge-storage/src/status.rs

```rust
use std::path::Path;
use devforge_domain::app_info::DbStatus;
use crate::pool::Database;
use crate::migrator;

/// 检查数据库状态
pub fn check_db_status(data_dir: &Path) -> DbStatus {
    let db_path = data_dir.join("devforge.db");

    if !db_path.exists() {
        return DbStatus::NotInitialized;
    }

    match Database::open(&db_path) {
        Ok(db) => match migrator::schema_version(db.connection()) {
            Ok(v) => DbStatus::Ready { migration_version: v },
            Err(e) => DbStatus::Error { message: e.to_string() },
        },
        Err(e) => DbStatus::Error { message: e.to_string() },
    }
}
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
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn migration_runs_on_empty_db() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrator::run_migrations(db.connection()).unwrap();
        let version = migrator::schema_version(db.connection()).unwrap();
        assert_eq!(version, 1);
    }
}
```

**Cargo.toml 补充**：

```toml
[dev-dependencies]
tempfile = "3"
```

### 提交信息

```
feat(storage): SQLite bootstrap 和第一条 migration
```

---

## Task 7：串联 Storage 到 Tauri Command

**依赖**：Task 4, Task 6

**目标**：`get_app_info` 返回真实数据库状态。

### 精确文件修改

| 文件 | 变更 |
|------|------|
| `apps/desktop/src-tauri/Cargo.toml` | 添加 devforge-storage 依赖 |
| `apps/desktop/src-tauri/src/lib.rs` | 启动时初始化数据库 |
| `apps/desktop/src-tauri/src/commands.rs` | 使用真实 DbStatus |
| `apps/desktop/src-tauri/src/state.rs` | 管理 AppState |

### apps/desktop/src-tauri/src/state.rs

```rust
use std::path::PathBuf;
use devforge_domain::app_info::DbStatus;
use devforge_storage::pool::Database;

/// 应用全局状态
pub struct AppState {
    pub data_dir: PathBuf,
    pub db: Option<Database>,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            db: None,
        }
    }

    /// 初始化数据库（运行迁移）
    pub fn init_db(&mut self) -> Result<(), String> {
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("创建数据目录失败: {e}"))?;

        let db_path = self.data_dir.join("devforge.db");
        let db = Database::open(&db_path)
            .map_err(|e| format!("打开数据库失败: {e}"))?;

        devforge_storage::migrator::run_migrations(db.connection())
            .map_err(|e| format!("运行迁移失败: {e}"))?;

        self.db = Some(db);
        Ok(())
    }

    pub fn db_status(&self) -> DbStatus {
        match &self.db {
            Some(db) => match devforge_storage::migrator::schema_version(db.connection()) {
                Ok(v) => DbStatus::Ready { migration_version: v },
                Err(e) => DbStatus::Error { message: e.to_string() },
            },
            None => DbStatus::NotInitialized,
        }
    }
}
```

### apps/desktop/src-tauri/src/commands.rs（更新）

```rust
use tauri::State;
use devforge_domain::app_info::AppInfo;
use crate::state::AppState;

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION"),
        data_dir: state.data_dir.to_string_lossy().to_string(),
        db_status: state.db_status(),
    }
}
```

### apps/desktop/src-tauri/src/lib.rs（更新）

```rust
mod commands;
mod state;

use state::AppState;

pub fn run() {
    let data_dir = dirs::data_dir()
        .expect("无法获取数据目录")
        .join("devforge");

    let mut app_state = AppState::new(data_dir);
    if let Err(e) = app_state.init_db() {
        eprintln!("数据库初始化失败: {e}");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
```

### 验证命令

```powershell
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

  frontend:
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

Write-Host "`n=== 前端类型检查 ===" -ForegroundColor Cyan
pnpm --filter @devforge/desktop typecheck

Write-Host "`n=== 前端测试 ===" -ForegroundColor Cyan
pnpm --filter @devforge/desktop test

Write-Host "`n=== 全部通过 ✓ ===" -ForegroundColor Green
```

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

// Mock IPC
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    version: "0.1.0",
    data_dir: "/tmp/test",
    db_status: { type: "NotInitialized" },
  }),
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

# 5. 验证数据目录
ls "$env:APPDATA\devforge"
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
- [ ] Domain 层不依赖 Tauri
- [ ] CI 可以运行 Rust 测试、Clippy、fmt、前端类型检查和测试
- [ ] Release 构建可以安装和启动
