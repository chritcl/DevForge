-- 工作区表
CREATE TABLE IF NOT EXISTS workspaces (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_opened_at TEXT
);

-- 工作区设置表
CREATE TABLE IF NOT EXISTS workspace_settings (
    workspace_id TEXT PRIMARY KEY NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    sidebar_width REAL,
    explorer_expanded TEXT,
    active_tab_id TEXT,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
