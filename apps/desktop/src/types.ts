// 纯前端 UI 类型（非 IPC 类型）
// IPC 类型从 bindings.ts 导入

// 标签页类型（前端 UI 使用）
export type TabDto = {
  id: string;
  workspace_id: string;
  document_id: string;
  position: number;
  is_active: boolean;
  opened_at: string;
};
