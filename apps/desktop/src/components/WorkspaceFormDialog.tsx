import { useState } from "react";
import { useCreateWorkspace, useUpdateWorkspace } from "../hooks/useWorkspaces";
import type { WorkspaceDto } from "../bindings";

interface WorkspaceFormDialogProps {
  mode: "create" | "edit";
  workspace?: WorkspaceDto;
  onClose: () => void;
  onSuccess?: (workspace: WorkspaceDto) => void;
}

export function WorkspaceFormDialog({
  mode,
  workspace,
  onClose,
  onSuccess,
}: WorkspaceFormDialogProps) {
  const createWorkspace = useCreateWorkspace();
  const updateWorkspace = useUpdateWorkspace();

  // 编辑模式使用 workspace 的初始值，创建模式使用空值
  const [name, setName] = useState(
    mode === "edit" && workspace ? workspace.name : ""
  );
  const [description, setDescription] = useState(
    mode === "edit" && workspace ? (workspace.description ?? "") : ""
  );
  const [error, setError] = useState<string | null>(null);

  const isPending = createWorkspace.isPending || updateWorkspace.isPending;
  const hasChanges =
    mode === "create"
      ? name.trim() !== "" || description.trim() !== ""
      : name.trim() !== (workspace?.name ?? "") ||
        description.trim() !== (workspace?.description ?? "");

  const handleSubmit = async () => {
    if (!name.trim()) {
      setError("工作区名称不能为空");
      return;
    }

    try {
      setError(null);

      if (mode === "create") {
        const result = await createWorkspace.mutateAsync({
          name: name.trim(),
          description: description.trim() || null,
        });
        onSuccess?.(result);
      } else if (workspace) {
        const result = await updateWorkspace.mutateAsync({
          id: workspace.id,
          name: name.trim(),
          description: description.trim() || null,
        });
        onSuccess?.(result);
      }

      onClose();
    } catch (err) {
      setError(formatError(err));
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape" && !isPending) {
      onClose();
    }
    if (e.key === "Enter" && e.ctrlKey && hasChanges && !isPending) {
      handleSubmit();
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()} onKeyDown={handleKeyDown}>
        <div className="dialog-header">
          <h2>{mode === "create" ? "创建工作区" : "编辑工作区"}</h2>
          <button className="dialog-close" onClick={onClose} disabled={isPending}>
            ×
          </button>
        </div>
        <div className="dialog-content">
          <div className="form-group">
            <label htmlFor="workspace-form-name">名称</label>
            <input
              id="workspace-form-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="输入工作区名称"
              autoFocus
              disabled={isPending}
            />
          </div>
          <div className="form-group">
            <label htmlFor="workspace-form-description">描述（可选）</label>
            <textarea
              id="workspace-form-description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="输入工作区描述"
              rows={3}
              disabled={isPending}
            />
          </div>
          {error && <div className="dialog-error">{error}</div>}
        </div>
        <div className="dialog-footer">
          <button className="btn btn-secondary" onClick={onClose} disabled={isPending}>
            取消
          </button>
          <button
            className="btn btn-primary"
            onClick={handleSubmit}
            disabled={!name.trim() || (mode === "edit" && !hasChanges) || isPending}
          >
            {isPending
              ? mode === "create"
                ? "创建中..."
                : "保存中..."
              : mode === "create"
                ? "创建"
                : "保存"}
          </button>
        </div>
      </div>
    </div>
  );
}

/// 格式化错误为用户可见的中文消息
function formatError(err: unknown): string {
  if (typeof err === "string") {
    switch (err) {
      case "WorkspaceNotFound":
        return "工作区不存在";
      case "DuplicateName":
        return "工作区名称已存在";
      default:
        return err;
    }
  }
  if (err && typeof err === "object") {
    if ("WorkspaceNotFound" in err) return "工作区不存在";
    if ("DuplicateName" in err) return "工作区名称已存在";
    if ("Domain" in err) return `领域错误: ${(err as Record<string, unknown>).Domain}`;
    if (err instanceof Error) return err.message;
  }
  return "未知错误";
}
