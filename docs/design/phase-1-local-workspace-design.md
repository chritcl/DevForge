# Phase 1：Local Workspace 设计规格

## 1. 概述

Phase 1 实现完整的本地工作区闭环：创建 → 添加数据源 → 扫描文件 → 文件树展示 → 打开源码 → 重启恢复。

## 2. 用户流程

```
启动 DevForge
  ↓
首页显示工作区列表（最近打开排序）
  ↓
点击"创建工作区"
  ↓
输入名称和可选描述
  ↓
进入工作区 Explorer
  ↓
点击"添加数据源"
  ↓
选择系统目录（Tauri dialog）
  ↓
自动识别 Git 仓库或普通目录
  ↓
后台扫描目录
  ↓
文件树懒加载展示
  ↓
点击文件打开标签
  ↓
关闭应用
  ↓
再次启动恢复工作区和标签
```

## 3. 领域模型

### 3.1 核心实体

```rust
// crates/devforge-domain/src/workspace.rs

pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_opened_at: Option<DateTime<Utc>>,
    pub status: WorkspaceStatus,
}

pub enum WorkspaceStatus {
    Active,
    Archived,
}

pub struct WorkspaceId(Uuid);
```

### 3.2 数据源

```rust
// crates/devforge-domain/src/source.rs

pub struct Source {
    pub id: SourceId,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub root_path: PathBuf,
    pub kind: SourceKind,
    pub created_at: DateTime<Utc>,
}

pub enum SourceKind {
    Git,
    Directory,
}

pub struct SourceId(Uuid);
```

### 3.3 文档

```rust
// crates/devforge-domain/src/document.rs

pub struct Document {
    pub id: DocumentId,
    pub source_id: SourceId,
    pub relative_path: PathBuf,
    pub kind: DocumentKind,
    pub size: u64,
    pub modified_at: DateTime<Utc>,
    pub sensitivity: Sensitivity,
    pub content_readable: bool,
}

pub enum DocumentKind {
    Text,
    Markdown,
    Image,
    Binary,
    Unknown,
}

pub enum Sensitivity {
    Normal,
    Sensitive,
}

pub struct DocumentId(Uuid);
```

### 3.4 打开标签

```rust
// crates/devforge-domain/src/opentab.rs

pub struct OpenTab {
    pub id: Uuid,
    pub workspace_id: WorkspaceId,
    pub document_id: DocumentId,
    pub position: i32,
    pub is_active: bool,
    pub opened_at: DateTime<Utc>,
}
```

### 3.5 工作区布局

```rust
// crates/devforge-domain/src/layout.rs

pub struct WorkspaceLayout {
    pub workspace_id: WorkspaceId,
    pub sidebar_width: Option<f64>,
    pub explorer_expanded: Vec<SourceId>,
    pub active_tab_id: Option<Uuid>,
}
```

## 4. 状态机

### 4.1 Workspace 状态

```
Active ←→ Archived
```

- Active → Archived：归档
- Archived → Active：恢复
- 删除：硬删除 Workspace 元数据，不删除源目录

### 4.2 Source 状态

```
存在 → 已删除
```

- 删除 Source 只删除元数据，不删除源目录

## 5. SQLite Schema

### 5.1 workspaces 表

```sql
CREATE TABLE workspaces (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_opened_at TEXT
);
```

### 5.2 workspace_settings 表

```sql
CREATE TABLE workspace_settings (
    workspace_id TEXT PRIMARY KEY NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    sidebar_width REAL,
    explorer_expanded TEXT,  -- JSON array of source IDs
    active_tab_id TEXT,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### 5.3 sources 表

```sql
CREATE TABLE sources (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    root_path TEXT NOT NULL,
    kind TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(workspace_id, root_path)
);
```

### 5.4 documents 表

```sql
CREATE TABLE documents (
    id TEXT PRIMARY KEY NOT NULL,
    source_id TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    relative_path TEXT NOT NULL,
    kind TEXT NOT NULL,
    size INTEGER NOT NULL,
    modified_at TEXT NOT NULL,
    sensitivity TEXT NOT NULL DEFAULT 'normal',
    content_readable INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(source_id, relative_path)
);
```

### 5.5 open_tabs 表

```sql
CREATE TABLE open_tabs (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    position INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 0,
    opened_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## 6. 删除语义

- 删除 Workspace：级联删除 workspace_settings、sources、documents、open_tabs
- 删除 Source：级联删除 documents
- **绝不删除源目录或源文件**

## 7. Rust Crate 边界

```
devforge-domain          # 领域模型，无外部依赖
devforge-application     # 用例，依赖 domain
devforge-storage         # SQLite 实现，依赖 domain
devforge-desktop         # Tauri 命令，依赖 application
```

## 8. Tauri Command 契约

### 8.1 工作区命令

```rust
#[tauri::command]
#[specta::specta]
async fn create_workspace(name: String, description: Option<String>) -> Result<Workspace, AppError>;

#[tauri::command]
#[specta::specta]
async fn list_workspaces() -> Result<Vec<Workspace>, AppError>;

#[tauri::command]
#[specta::specta]
async fn get_workspace(id: String) -> Result<Workspace, AppError>;

#[tauri::command]
#[specta::specta]
async fn update_workspace(id: String, name: Option<String>, description: Option<String>) -> Result<Workspace, AppError>;

#[tauri::command]
#[specta::specta]
async fn archive_workspace(id: String) -> Result<(), AppError>;

#[tauri::command]
#[specta::specta]
async fn restore_workspace(id: String) -> Result<(), AppError>;

#[tauri::command]
#[specta::specta]
async fn delete_workspace(id: String) -> Result<(), AppError>;
```

### 8.2 数据源命令

```rust
#[tauri::command]
#[specta::specta]
async fn add_git_source(workspace_id: String, path: String) -> Result<Source, AppError>;

#[tauri::command]
#[specta::specta]
async fn add_directory_source(workspace_id: String, path: String) -> Result<Source, AppError>;

#[tauri::command]
#[specta::specta]
async fn list_sources(workspace_id: String) -> Result<Vec<Source>, AppError>;

#[tauri::command]
#[specta::specta]
async fn remove_source(id: String) -> Result<(), AppError>;
```

### 8.3 文档命令

```rust
#[tauri::command]
#[specta::specta]
async fn list_documents(source_id: String, parent_path: Option<String>) -> Result<Vec<Document>, AppError>;

#[tauri::command]
#[specta::specta]
async fn read_document_content(id: String) -> Result<String, AppError>;
```

### 8.4 标签命令

```rust
#[tauri::command]
#[specta::specta]
async fn open_tab(workspace_id: String, document_id: String) -> Result<OpenTab, AppError>;

#[tauri::command]
#[specta::specta]
async fn close_tab(id: String) -> Result<(), AppError>;

#[tauri::command]
#[specta::specta]
async fn list_tabs(workspace_id: String) -> Result<Vec<OpenTab>, AppError>;

#[tauri::command]
#[specta::specta]
async fn set_active_tab(id: String) -> Result<(), AppError>;
```

## 9. 文件路径安全（PathGuard）

所有路径操作必须验证：

1. 路径不包含 `..`
2. 路径是绝对路径
3. 路径在 Source 根目录内
4. Windows 大小写规范化比较
5. 解析 symlink 和 junction
6. 文件存在且可读

```rust
pub struct PathGuard {
    source_root: PathBuf,
}

impl PathGuard {
    pub fn new(source_root: PathBuf) -> Self;
    pub fn validate(&self, path: &Path) -> Result<PathBuf, PathError>;
}
```

## 10. 文件发现规则

### 10.1 扫描规则

- 递归遍历目录
- 遵守 `.gitignore`
- 遵守 `.devforgeignore`
- 忽略 `.git` 目录
- 识别二进制文件
- 文件大小限制：100MB
- 文件类型识别

### 10.2 敏感文件识别

```
.env
.env.*
*.pem
*.key
*.pfx
*.p12
id_rsa
id_ed25519
credentials
secrets
```

敏感文件在文件树中显示，但不返回正文。

## 11. 文件树懒加载协议

### 11.1 查询协议

```rust
#[tauri::command]
async fn list_documents(
    source_id: String,
    parent_path: Option<String>,  // None 表示根目录
) -> Result<Vec<Document>, AppError>;
```

### 11.2 React Query Key

```typescript
['documents', workspaceId, sourceId, parentPath]
```

## 12. 文件读取协议

1. 验证 PathGuard
2. 检查 sensitivity
3. 检查文件大小
4. 读取内容
5. 返回字符串

## 13. 标签恢复

1. 应用启动时查询 `open_tabs` 表
2. 恢复标签顺序和活动状态
3. 验证文档仍存在

## 14. 错误协议

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("工作区不存在")]
    WorkspaceNotFound,
    #[error("数据源不存在")]
    SourceNotFound,
    #[error("文档不存在")]
    DocumentNotFound,
    #[error("路径安全违规")]
    PathViolation,
    #[error("敏感文件不可读")]
    SensitiveFile,
    #[error("文件过大")]
    FileTooLarge,
    #[error("IO 错误")]
    Io(#[from] std::io::Error),
    #[error("数据库错误")]
    Database(#[from] sqlx::Error),
}
```

## 15. 测试策略

### 15.1 Rust 测试

- Domain 单元测试
- SQLite Repository 集成测试
- PathGuard 安全测试
- 文件发现测试

### 15.2 React 测试

- 工作区列表
- 文件树加载
- 标签管理

## 16. 性能边界

- 文件树懒加载：每次最多返回 100 个条目
- 文件内容读取：最大 10MB
- 扫描：异步后台执行

## 17. 非目标

- Tantivy 索引
- Tree-sitter 解析
- AI 功能
- Git 操作
- 远程仓库
