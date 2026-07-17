// 手动定义的类型（不经过 Specta 自动生成）

// 工作区类型
export type Workspace = {
  id: string;
  name: string;
  description: string | null;
  status: WorkspaceStatus;
  created_at: string;
  updated_at: string;
  last_opened_at: string | null;
};

export type WorkspaceStatus = "Active" | "Archived";

// 数据源类型
export type Source = {
  id: string;
  workspace_id: string;
  name: string;
  root_path: string;
  kind: SourceKind;
  created_at: string;
};

export type SourceKind = "Git" | "Directory";

// 文档类型
export type DocumentDto = {
  id: string;
  source_id: string;
  relative_path: string;
  kind: string;
  size: number;
  sensitivity: string;
  content_readable: boolean;
};

// 扫描结果
export type ScanResult = {
  added: number;
  updated: number;
  removed: number;
  skipped: number;
};
