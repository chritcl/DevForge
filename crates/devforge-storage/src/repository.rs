//! SQLite Repository 实现

use std::path::{Component, PathBuf};

use async_trait::async_trait;
use sqlx::SqlitePool;

use devforge_domain::document::{Document, DocumentId, DocumentKind, Sensitivity};
use devforge_domain::error::DomainError;
use devforge_domain::opentab::OpenTab;
use devforge_domain::source::{Source, SourceId, SourceKind};
use devforge_domain::workspace::{Workspace, WorkspaceId, WorkspaceStatus};

/// Workspace Repository
pub struct SqliteWorkspaceRepository {
    pool: SqlitePool,
}

impl SqliteWorkspaceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl devforge_application::workspace::WorkspaceRepository for SqliteWorkspaceRepository {
    async fn create(&self, workspace: &Workspace) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO workspaces (id, name, description, status, created_at, updated_at, last_opened_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&workspace.id.0)
        .bind(&workspace.name)
        .bind(&workspace.description)
        .bind(workspace.status.to_string())
        .bind(workspace.created_at.to_rfc3339())
        .bind(workspace.updated_at.to_rfc3339())
        .bind(workspace.last_opened_at.map(|t| t.to_rfc3339()))
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;
        Ok(())
    }

    async fn get(&self, id: &WorkspaceId) -> Result<Option<Workspace>, DomainError> {
        let row: Option<(String, String, Option<String>, String, String, String, Option<String>)> =
            sqlx::query_as(
                "SELECT id, name, description, status, created_at, updated_at, last_opened_at FROM workspaces WHERE id = ?"
            )
            .bind(&id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;

        match row {
            Some((id, name, description, status, created_at, updated_at, last_opened_at)) => {
                Ok(Some(Workspace {
                    id: WorkspaceId(id),
                    name,
                    description,
                    status: WorkspaceStatus::parse_str(&status).ok_or_else(|| {
                        DomainError::InvalidInput(format!("无效的工作区状态: {status}"))
                    })?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .map_err(|e| DomainError::InvalidInput(format!("无效的创建时间: {e}")))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                        .map_err(|e| DomainError::InvalidInput(format!("无效的更新时间: {e}")))?
                        .with_timezone(&chrono::Utc),
                    last_opened_at: last_opened_at
                        .map(|t| chrono::DateTime::parse_from_rfc3339(&t))
                        .transpose()
                        .map_err(|e| DomainError::InvalidInput(format!("无效的最后打开时间: {e}")))?
                        .map(|t| t.with_timezone(&chrono::Utc)),
                }))
            }
            None => Ok(None),
        }
    }

    async fn list(&self) -> Result<Vec<Workspace>, DomainError> {
        let rows: Vec<(String, String, Option<String>, String, String, String, Option<String>)> =
            sqlx::query_as(
                "SELECT id, name, description, status, created_at, updated_at, last_opened_at FROM workspaces WHERE status = 'active' ORDER BY last_opened_at DESC NULLS LAST, updated_at DESC"
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;

        let mut workspaces = Vec::new();
        for (id, name, description, status, created_at, updated_at, last_opened_at) in rows {
            workspaces.push(Workspace {
                id: WorkspaceId(id),
                name,
                description,
                status: WorkspaceStatus::parse_str(&status).ok_or_else(|| {
                    DomainError::InvalidInput(format!("无效的工作区状态: {status}"))
                })?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| DomainError::InvalidInput(format!("无效的创建时间: {e}")))?
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                    .map_err(|e| DomainError::InvalidInput(format!("无效的更新时间: {e}")))?
                    .with_timezone(&chrono::Utc),
                last_opened_at: last_opened_at
                    .map(|t| chrono::DateTime::parse_from_rfc3339(&t))
                    .transpose()
                    .map_err(|e| DomainError::InvalidInput(format!("无效的最后打开时间: {e}")))?
                    .map(|t| t.with_timezone(&chrono::Utc)),
            });
        }
        Ok(workspaces)
    }

    async fn update(&self, workspace: &Workspace) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE workspaces SET name = ?, description = ?, status = ?, updated_at = ?, last_opened_at = ? WHERE id = ?"
        )
        .bind(&workspace.name)
        .bind(&workspace.description)
        .bind(workspace.status.to_string())
        .bind(workspace.updated_at.to_rfc3339())
        .bind(workspace.last_opened_at.map(|t| t.to_rfc3339()))
        .bind(&workspace.id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;
        Ok(())
    }

    async fn delete(&self, id: &WorkspaceId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM workspaces WHERE id = ?")
            .bind(&id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;
        Ok(())
    }
}

/// Source Repository
pub struct SqliteSourceRepository {
    pool: SqlitePool,
}

impl SqliteSourceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl devforge_application::source::SourceRepository for SqliteSourceRepository {
    async fn create(&self, source: &Source) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO sources (id, workspace_id, name, root_path, kind, created_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&source.id.0)
        .bind(&source.workspace_id.0)
        .bind(&source.name)
        .bind(source.root_path.to_string_lossy().as_ref())
        .bind(source.kind.to_string())
        .bind(source.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;
        Ok(())
    }

    async fn get(&self, id: &SourceId) -> Result<Option<Source>, DomainError> {
        let row: Option<(String, String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, workspace_id, name, root_path, kind, created_at FROM sources WHERE id = ?",
        )
        .bind(&id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;

        match row {
            Some((id, workspace_id, name, root_path, kind, created_at)) => Ok(Some(Source {
                id: SourceId(id),
                workspace_id: WorkspaceId(workspace_id),
                name,
                root_path: PathBuf::from(root_path),
                kind: SourceKind::parse_str(&kind).ok_or_else(|| {
                    DomainError::InvalidInput(format!("无效的数据源类型: {kind}"))
                })?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| DomainError::InvalidInput(format!("无效的创建时间: {e}")))?
                    .with_timezone(&chrono::Utc),
            })),
            None => Ok(None),
        }
    }

    async fn list_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<Source>, DomainError> {
        let rows: Vec<(String, String, String, String, String, String)> =
            sqlx::query_as(
                "SELECT id, workspace_id, name, root_path, kind, created_at FROM sources WHERE workspace_id = ? ORDER BY name"
            )
            .bind(&workspace_id.0)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;

        let mut sources = Vec::new();
        for (id, workspace_id, name, root_path, kind, created_at) in rows {
            sources.push(Source {
                id: SourceId(id),
                workspace_id: WorkspaceId(workspace_id),
                name,
                root_path: PathBuf::from(root_path),
                kind: SourceKind::parse_str(&kind).ok_or_else(|| {
                    DomainError::InvalidInput(format!("无效的数据源类型: {kind}"))
                })?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| DomainError::InvalidInput(format!("无效的创建时间: {e}")))?
                    .with_timezone(&chrono::Utc),
            });
        }
        Ok(sources)
    }

    async fn delete(&self, id: &SourceId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM sources WHERE id = ?")
            .bind(&id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;
        Ok(())
    }
}

/// Document Repository
pub struct SqliteDocumentRepository {
    pool: SqlitePool,
}

impl SqliteDocumentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl devforge_application::discovery::DocumentRepository for SqliteDocumentRepository {
    async fn create(&self, document: &Document) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO documents (id, source_id, relative_path, kind, size, modified_at, sensitivity, content_readable, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&document.id.0)
        .bind(&document.source_id.0)
        .bind(document.relative_path.to_string_lossy().as_ref())
        .bind(document.kind.to_string())
        .bind(document.size as i64)
        .bind(document.modified_at.to_rfc3339())
        .bind(document.sensitivity.to_string())
        .bind(document.content_readable as i32)
        .bind(document.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;
        Ok(())
    }

    async fn get(&self, id: &DocumentId) -> Result<Option<Document>, DomainError> {
        let row: Option<(String, String, String, String, i64, String, String, i32, String)> =
            sqlx::query_as(
                "SELECT id, source_id, relative_path, kind, size, modified_at, sensitivity, content_readable, created_at FROM documents WHERE id = ?"
            )
            .bind(&id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;

        match row {
            Some((
                id,
                source_id,
                relative_path,
                kind,
                size,
                modified_at,
                sensitivity,
                content_readable,
                created_at,
            )) => Ok(Some(Document {
                id: DocumentId(id),
                source_id: SourceId(source_id),
                relative_path: PathBuf::from(relative_path),
                kind: DocumentKind::parse_str(&kind)
                    .ok_or_else(|| DomainError::InvalidInput(format!("无效的文档类型: {kind}")))?,
                size: size as u64,
                modified_at: chrono::DateTime::parse_from_rfc3339(&modified_at)
                    .map_err(|e| DomainError::InvalidInput(format!("无效的修改时间: {e}")))?
                    .with_timezone(&chrono::Utc),
                sensitivity: Sensitivity::parse_str(&sensitivity).ok_or_else(|| {
                    DomainError::InvalidInput(format!("无效的敏感度: {sensitivity}"))
                })?,
                content_readable: content_readable != 0,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| DomainError::InvalidInput(format!("无效的创建时间: {e}")))?
                    .with_timezone(&chrono::Utc),
            })),
            None => Ok(None),
        }
    }

    async fn list_by_source(&self, source_id: &SourceId) -> Result<Vec<Document>, DomainError> {
        let rows: Vec<(String, String, String, String, i64, String, String, i32, String)> =
            sqlx::query_as(
                "SELECT id, source_id, relative_path, kind, size, modified_at, sensitivity, content_readable, created_at FROM documents WHERE source_id = ? ORDER BY relative_path"
            )
            .bind(&source_id.0)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;

        let mut documents = Vec::new();
        for (
            id,
            source_id,
            relative_path,
            kind,
            size,
            modified_at,
            sensitivity,
            content_readable,
            created_at,
        ) in rows
        {
            documents.push(Document {
                id: DocumentId(id),
                source_id: SourceId(source_id),
                relative_path: PathBuf::from(relative_path),
                kind: DocumentKind::parse_str(&kind)
                    .ok_or_else(|| DomainError::InvalidInput(format!("无效的文档类型: {kind}")))?,
                size: size as u64,
                modified_at: chrono::DateTime::parse_from_rfc3339(&modified_at)
                    .map_err(|e| DomainError::InvalidInput(format!("无效的修改时间: {e}")))?
                    .with_timezone(&chrono::Utc),
                sensitivity: Sensitivity::parse_str(&sensitivity).ok_or_else(|| {
                    DomainError::InvalidInput(format!("无效的敏感度: {sensitivity}"))
                })?,
                content_readable: content_readable != 0,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| DomainError::InvalidInput(format!("无效的创建时间: {e}")))?
                    .with_timezone(&chrono::Utc),
            });
        }
        Ok(documents)
    }

    async fn list_by_source_and_parent(
        &self,
        source_id: &SourceId,
        parent_path: Option<&str>,
    ) -> Result<Vec<Document>, DomainError> {
        let all_docs = self.list_by_source(source_id).await?;
        let mut result = Vec::new();
        let mut seen_dirs = std::collections::HashSet::new();

        for doc in all_docs {
            let path_str = doc.relative_path.to_string_lossy().to_string();
            let path = std::path::PathBuf::from(&path_str);

            match parent_path {
                Some(parent) => {
                    // 只返回直接子文件和子目录
                    if let Some(parent_of) = path.parent() {
                        if parent_of.to_string_lossy() == parent {
                            result.push(doc);
                        }
                    }
                }
                None => {
                    // 根目录：返回根目录下的文件和第一层目录
                    let components: Vec<_> = path.components().collect();
                    if components.len() == 1 {
                        // 根目录下的文件
                        result.push(doc);
                    } else if components.len() > 1 {
                        // 子目录：只返回目录条目
                        let dir_name = components[0].as_os_str().to_string_lossy().to_string();
                        if !seen_dirs.contains(&dir_name) {
                            seen_dirs.insert(dir_name.clone());
                            // 创建一个虚拟的目录文档
                            let mut dir_doc = doc;
                            dir_doc.relative_path = std::path::PathBuf::from(&dir_name);
                            dir_doc.kind = DocumentKind::Unknown;
                            dir_doc.size = 0;
                            dir_doc.content_readable = false;
                            result.push(dir_doc);
                        }
                    }
                }
            }
        }

        result.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        Ok(result)
    }

    async fn upsert(&self, document: &Document) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO documents (id, source_id, relative_path, kind, size, modified_at, sensitivity, content_readable, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(source_id, relative_path) DO UPDATE SET kind = ?, size = ?, modified_at = ?, sensitivity = ?, content_readable = ?"
        )
        .bind(&document.id.0)
        .bind(&document.source_id.0)
        .bind(document.relative_path.to_string_lossy().as_ref())
        .bind(document.kind.to_string())
        .bind(document.size as i64)
        .bind(document.modified_at.to_rfc3339())
        .bind(document.sensitivity.to_string())
        .bind(document.content_readable as i32)
        .bind(document.created_at.to_rfc3339())
        .bind(document.kind.to_string())
        .bind(document.size as i64)
        .bind(document.modified_at.to_rfc3339())
        .bind(document.sensitivity.to_string())
        .bind(document.content_readable as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;
        Ok(())
    }

    async fn delete_by_source(&self, source_id: &SourceId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM documents WHERE source_id = ?")
            .bind(&source_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;
        Ok(())
    }

    async fn list_file_tree(
        &self,
        source_id: &SourceId,
        parent_path: Option<&str>,
    ) -> Result<Vec<devforge_application::document::FileTreeEntry>, DomainError> {
        // 当前实现：读取整个 Source 后投影直接子项
        // 注意：对大型仓库存在性能缺口，后续将通过 SQL 前缀查询优化
        let all_docs = self.list_by_source(source_id).await?;
        let mut entries = Vec::new();
        let mut seen_dirs = std::collections::BTreeSet::new();

        for doc in &all_docs {
            let rel = &doc.relative_path;

            match parent_path {
                Some(parent) => {
                    // 验证 parent_path 是有效的相对路径
                    let parent_path = std::path::Path::new(parent);
                    // 检查是否为 parent 的直接子项
                    if let Ok(rest) = rel.strip_prefix(parent_path) {
                        let components: Vec<_> = rest.components().collect();
                        if components.len() == 1 {
                            // 直接子文件
                            if let Component::Normal(name) = components[0] {
                                let name = name.to_string_lossy().to_string();
                                entries.push(devforge_application::document::FileTreeEntry {
                                    key: devforge_application::document::FileTreeEntry::file_key(
                                        &doc.id,
                                    ),
                                    source_id: doc.source_id.clone(),
                                    relative_path: rel.clone(),
                                    name,
                                    entry_kind:
                                        devforge_application::document::FileTreeEntryKind::File,
                                    document: Some(doc.clone()),
                                });
                            }
                        } else if components.len() > 1 {
                            // 子目录
                            if let Component::Normal(dir_name) = components[0] {
                                let dir_name = dir_name.to_string_lossy().to_string();
                                let dir_rel = std::path::PathBuf::from(parent).join(&dir_name);
                                let dir_key =
                                    devforge_application::document::FileTreeEntry::directory_key(
                                        source_id, &dir_rel,
                                    );
                                if seen_dirs.insert(dir_key.clone()) {
                                    entries.push(devforge_application::document::FileTreeEntry {
                                        key: dir_key,
                                        source_id: source_id.clone(),
                                        relative_path: dir_rel,
                                        name: dir_name,
                                        entry_kind: devforge_application::document::FileTreeEntryKind::Directory,
                                        document: None,
                                    });
                                }
                            }
                        }
                    }
                }
                None => {
                    // 根目录
                    let components: Vec<_> = rel.components().collect();
                    if components.len() == 1 {
                        // 根目录文件
                        if let Component::Normal(name) = components[0] {
                            let name = name.to_string_lossy().to_string();
                            entries.push(devforge_application::document::FileTreeEntry {
                                key: devforge_application::document::FileTreeEntry::file_key(
                                    &doc.id,
                                ),
                                source_id: doc.source_id.clone(),
                                relative_path: rel.clone(),
                                name,
                                entry_kind: devforge_application::document::FileTreeEntryKind::File,
                                document: Some(doc.clone()),
                            });
                        }
                    } else if components.len() > 1 {
                        // 第一层目录
                        if let Component::Normal(dir_name) = components[0] {
                            let dir_name = dir_name.to_string_lossy().to_string();
                            let dir_rel = std::path::PathBuf::from(&dir_name);
                            let dir_key =
                                devforge_application::document::FileTreeEntry::directory_key(
                                    source_id, &dir_rel,
                                );
                            if seen_dirs.insert(dir_key.clone()) {
                                entries.push(devforge_application::document::FileTreeEntry {
                                    key: dir_key,
                                    source_id: source_id.clone(),
                                    relative_path: dir_rel,
                                    name: dir_name,
                                    entry_kind: devforge_application::document::FileTreeEntryKind::Directory,
                                    document: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        // 排序：目录在前，文件在后，同类按名称排序（Windows 大小写不敏感）
        entries.sort_by(|a, b| match (&a.entry_kind, &b.entry_kind) {
            (
                devforge_application::document::FileTreeEntryKind::Directory,
                devforge_application::document::FileTreeEntryKind::File,
            ) => std::cmp::Ordering::Less,
            (
                devforge_application::document::FileTreeEntryKind::File,
                devforge_application::document::FileTreeEntryKind::Directory,
            ) => std::cmp::Ordering::Greater,
            _ => a
                .name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then_with(|| a.name.cmp(&b.name)),
        });

        Ok(entries)
    }
}

/// OpenTab Repository
pub struct SqliteOpenTabRepository {
    pool: SqlitePool,
}

impl SqliteOpenTabRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl devforge_application::tab::TabRepository for SqliteOpenTabRepository {
    async fn create(&self, tab: &OpenTab) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO open_tabs (id, workspace_id, document_id, position, is_active, opened_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&tab.id)
        .bind(&tab.workspace_id.0)
        .bind(&tab.document_id.0)
        .bind(tab.position)
        .bind(tab.is_active as i32)
        .bind(tab.opened_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;
        Ok(())
    }

    async fn list_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<OpenTab>, DomainError> {
        let rows: Vec<(String, String, String, i32, i32, String)> =
            sqlx::query_as(
                "SELECT id, workspace_id, document_id, position, is_active, opened_at FROM open_tabs WHERE workspace_id = ? ORDER BY position"
            )
            .bind(&workspace_id.0)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;

        let mut tabs = Vec::new();
        for (id, workspace_id, document_id, position, is_active, opened_at) in rows {
            tabs.push(OpenTab {
                id,
                workspace_id: WorkspaceId(workspace_id),
                document_id: DocumentId(document_id),
                position,
                is_active: is_active != 0,
                opened_at: chrono::DateTime::parse_from_rfc3339(&opened_at)
                    .map_err(|e| DomainError::InvalidInput(format!("无效的打开时间: {e}")))?
                    .with_timezone(&chrono::Utc),
            });
        }
        Ok(tabs)
    }

    async fn set_active(
        &self,
        workspace_id: &WorkspaceId,
        tab_id: &str,
    ) -> Result<(), DomainError> {
        // 先将所有标签设为非活动
        sqlx::query("UPDATE open_tabs SET is_active = 0 WHERE workspace_id = ?")
            .bind(&workspace_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;

        // 将指定标签设为活动
        sqlx::query("UPDATE open_tabs SET is_active = 1 WHERE id = ? AND workspace_id = ?")
            .bind(tab_id)
            .bind(&workspace_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM open_tabs WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;
        Ok(())
    }

    async fn delete_by_workspace(&self, workspace_id: &WorkspaceId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM open_tabs WHERE workspace_id = ?")
            .bind(&workspace_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Io(std::io::Error::other(e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use devforge_application::discovery::DocumentRepository;
    use devforge_application::source::SourceRepository;
    use devforge_application::tab::TabRepository;
    use devforge_application::workspace::WorkspaceRepository;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(include_str!("../migrations/0001_create_app_meta.sql"))
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(include_str!("../migrations/0002_create_workspaces.sql"))
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(include_str!("../migrations/0003_create_sources.sql"))
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(include_str!("../migrations/0004_create_documents.sql"))
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(include_str!("../migrations/0005_create_open_tabs.sql"))
            .execute(&pool)
            .await
            .unwrap();
        pool
    }

    #[tokio::test]
    async fn workspace_crud() {
        let pool = setup_test_db().await;
        let repo = SqliteWorkspaceRepository::new(pool);

        // 创建
        let workspace = Workspace::new("测试工作区".to_owned(), Some("描述".to_owned())).unwrap();
        repo.create(&workspace).await.unwrap();

        // 查询
        let fetched = repo.get(&workspace.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "测试工作区");
        assert_eq!(fetched.description, Some("描述".to_owned()));

        // 列表
        let list = repo.list().await.unwrap();
        assert_eq!(list.len(), 1);

        // 更新
        let mut updated = fetched;
        updated.name = "新名称".to_owned();
        repo.update(&updated).await.unwrap();

        let fetched = repo.get(&workspace.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "新名称");

        // 删除
        repo.delete(&workspace.id).await.unwrap();
        let fetched = repo.get(&workspace.id).await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn source_crud() {
        let pool = setup_test_db().await;
        let workspace_repo = SqliteWorkspaceRepository::new(pool.clone());
        let source_repo = SqliteSourceRepository::new(pool);

        // 先创建工作区
        let workspace = Workspace::new("测试工作区".to_owned(), None).unwrap();
        workspace_repo.create(&workspace).await.unwrap();

        // 创建数据源
        let source = Source::new_git(
            workspace.id.clone(),
            "test-repo".to_owned(),
            PathBuf::from("/test/path"),
        );
        source_repo.create(&source).await.unwrap();

        // 查询
        let fetched = source_repo.get(&source.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "test-repo");
        assert_eq!(fetched.kind, SourceKind::Git);

        // 列表
        let list = source_repo.list_by_workspace(&workspace.id).await.unwrap();
        assert_eq!(list.len(), 1);

        // 删除
        source_repo.delete(&source.id).await.unwrap();
        let fetched = source_repo.get(&source.id).await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn document_crud() {
        let pool = setup_test_db().await;
        let workspace_repo = SqliteWorkspaceRepository::new(pool.clone());
        let source_repo = SqliteSourceRepository::new(pool.clone());
        let doc_repo = SqliteDocumentRepository::new(pool);

        // 先创建工作区和数据源
        let workspace = Workspace::new("测试工作区".to_owned(), None).unwrap();
        workspace_repo.create(&workspace).await.unwrap();

        let source = Source::new_git(
            workspace.id.clone(),
            "test-repo".to_owned(),
            PathBuf::from("/test/path"),
        );
        source_repo.create(&source).await.unwrap();

        // 创建文档
        let doc = Document::new(
            source.id.clone(),
            PathBuf::from("src/main.rs"),
            1024,
            Utc::now(),
        );
        doc_repo.create(&doc).await.unwrap();

        // 查询
        let fetched = doc_repo.get(&doc.id).await.unwrap().unwrap();
        assert_eq!(fetched.kind, DocumentKind::Text);

        // 列表
        let list = doc_repo.list_by_source(&source.id).await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn open_tab_crud() {
        let pool = setup_test_db().await;
        let workspace_repo = SqliteWorkspaceRepository::new(pool.clone());
        let source_repo = SqliteSourceRepository::new(pool.clone());
        let doc_repo = SqliteDocumentRepository::new(pool.clone());
        let tab_repo = SqliteOpenTabRepository::new(pool);

        // 先创建工作区、数据源和文档
        let workspace = Workspace::new("测试工作区".to_owned(), None).unwrap();
        workspace_repo.create(&workspace).await.unwrap();

        let source = Source::new_git(
            workspace.id.clone(),
            "test-repo".to_owned(),
            PathBuf::from("/test/path"),
        );
        source_repo.create(&source).await.unwrap();

        let doc = Document::new(
            source.id.clone(),
            PathBuf::from("src/main.rs"),
            1024,
            Utc::now(),
        );
        doc_repo.create(&doc).await.unwrap();

        // 创建标签
        let tab = OpenTab::new(workspace.id.clone(), doc.id.clone(), 0);
        tab_repo.create(&tab).await.unwrap();

        // 列表
        let list = tab_repo.list_by_workspace(&workspace.id).await.unwrap();
        assert_eq!(list.len(), 1);
        assert!(!list[0].is_active);

        // 设置活动标签
        tab_repo.set_active(&workspace.id, &tab.id).await.unwrap();
        let list = tab_repo.list_by_workspace(&workspace.id).await.unwrap();
        assert!(list[0].is_active);

        // 删除标签
        tab_repo.delete(&tab.id).await.unwrap();
        let list = tab_repo.list_by_workspace(&workspace.id).await.unwrap();
        assert_eq!(list.len(), 0);
    }
}
