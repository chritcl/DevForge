//! SQLite Repository 实现

use std::path::PathBuf;

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
impl WorkspaceRepository for SqliteWorkspaceRepository {
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

/// Workspace Repository Trait
#[async_trait]
pub trait WorkspaceRepository: Send + Sync {
    async fn create(&self, workspace: &Workspace) -> Result<(), DomainError>;
    async fn get(&self, id: &WorkspaceId) -> Result<Option<Workspace>, DomainError>;
    async fn list(&self) -> Result<Vec<Workspace>, DomainError>;
    async fn update(&self, workspace: &Workspace) -> Result<(), DomainError>;
    async fn delete(&self, id: &WorkspaceId) -> Result<(), DomainError>;
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
impl SourceRepository for SqliteSourceRepository {
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

/// Source Repository Trait
#[async_trait]
pub trait SourceRepository: Send + Sync {
    async fn create(&self, source: &Source) -> Result<(), DomainError>;
    async fn get(&self, id: &SourceId) -> Result<Option<Source>, DomainError>;
    async fn list_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<Source>, DomainError>;
    async fn delete(&self, id: &SourceId) -> Result<(), DomainError>;
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
impl DocumentRepository for SqliteDocumentRepository {
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
}

/// Document Repository Trait
#[async_trait]
pub trait DocumentRepository: Send + Sync {
    async fn create(&self, document: &Document) -> Result<(), DomainError>;
    async fn get(&self, id: &DocumentId) -> Result<Option<Document>, DomainError>;
    async fn list_by_source(&self, source_id: &SourceId) -> Result<Vec<Document>, DomainError>;
    async fn upsert(&self, document: &Document) -> Result<(), DomainError>;
    async fn delete_by_source(&self, source_id: &SourceId) -> Result<(), DomainError>;
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
impl OpenTabRepository for SqliteOpenTabRepository {
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

/// OpenTab Repository Trait
#[async_trait]
pub trait OpenTabRepository: Send + Sync {
    async fn create(&self, tab: &OpenTab) -> Result<(), DomainError>;
    async fn list_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<OpenTab>, DomainError>;
    async fn set_active(&self, workspace_id: &WorkspaceId, tab_id: &str)
        -> Result<(), DomainError>;
    async fn delete(&self, id: &str) -> Result<(), DomainError>;
    async fn delete_by_workspace(&self, workspace_id: &WorkspaceId) -> Result<(), DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
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
