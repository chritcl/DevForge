# 子计划 01：Domain 和 Storage

## 目标

建立领域模型和 SQLite Schema，为后续功能提供数据基础。

## 精确修改文件

### 新增文件

| 文件 | 职责 |
|------|------|
| crates/devforge-domain/Cargo.toml | Domain crate 配置 |
| crates/devforge-domain/src/lib.rs | Domain 入口 |
| crates/devforge-domain/src/workspace.rs | Workspace 领域模型 |
| crates/devforge-domain/src/source.rs | Source 领域模型 |
| crates/devforge-domain/src/document.rs | Document 领域模型 |
| crates/devforge-domain/src/opentab.rs | OpenTab 领域模型 |
| crates/devforge-domain/src/error.rs | Domain 错误类型 |
| crates/devforge-storage/migrations/0002_create_workspaces.sql | Workspace 表 |
| crates/devforge-storage/migrations/0003_create_sources.sql | Source 表 |
| crates/devforge-storage/migrations/0004_create_documents.sql | Document 表 |
| crates/devforge-storage/migrations/0005_create_open_tabs.sql | OpenTab 表 |

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| Cargo.toml | 添加 devforge-domain 到 workspace |
| crates/devforge-storage/Cargo.toml | 添加 devforge-domain 依赖 |
| crates/devforge-storage/src/lib.rs | 添加 repository 模块 |

## 公共接口签名

### Domain 类型

```rust
// crates/devforge-domain/src/workspace.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub description: Option<String>,
    pub status: WorkspaceStatus,
    pub created_at: String,
    pub updated_at: String,
    pub last_opened_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceStatus {
    Active,
    Archived,
}
```

### Repository Trait

```rust
// crates/devforge-domain/src/repository.rs

#[async_trait]
pub trait WorkspaceRepository: Send + Sync {
    async fn create(&self, workspace: &Workspace) -> Result<(), DomainError>;
    async fn get(&self, id: &WorkspaceId) -> Result<Option<Workspace>, DomainError>;
    async fn list(&self) -> Result<Vec<Workspace>, DomainError>;
    async fn update(&self, workspace: &Workspace) -> Result<(), DomainError>;
    async fn delete(&self, id: &WorkspaceId) -> Result<(), DomainError>;
}
```

## 依赖关系

- 无外部依赖（除 serde、uuid、chrono）

## 不能并行的任务

- 无

## 失败测试

1. Workspace 创建后可以查询
2. Workspace 列表按 last_opened_at 排序
3. Workspace 更新后 updated_at 变化
4. Workspace 删除后不可查询
5. Migration 在空数据库运行成功
6. Migration 在已有数据库幂等运行

## 最小实现步骤

1. 创建 devforge-domain crate
2. 定义 Workspace、Source、Document、OpenTab 类型
3. 定义 Repository Trait
4. 创建 Migration 文件
5. 实现 SQLite Repository
6. 编写测试

## 精确验证命令

```bash
cargo test -p devforge-domain
cargo test -p devforge-storage
cargo clippy --workspace --all-targets -- -D warnings
```

## 独立提交信息

```
feat(domain): 添加 Phase 1 领域模型和 SQLite Schema

- 创建 devforge-domain crate
- 定义 Workspace、Source、Document、OpenTab 领域模型
- 定义 Repository Trait
- 添加 0002-0005 Migration
- 实现 SQLite WorkspaceRepository
```

## 回滚和兼容性风险

- 新增 Migration 不影响现有 0001
- 删除 domain crate 需同步修改 Cargo.toml
