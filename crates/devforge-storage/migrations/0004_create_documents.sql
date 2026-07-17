-- 文档表
CREATE TABLE IF NOT EXISTS documents (
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
