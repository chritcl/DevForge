-- 数据源表
CREATE TABLE IF NOT EXISTS sources (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    root_path TEXT NOT NULL,
    kind TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(workspace_id, root_path)
);
