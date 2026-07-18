import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useAddLocalSource, useScanSource } from "../hooks/useSources";

interface AddSourceDialogProps {
  workspaceId: string;
  onClose: () => void;
}

export function AddSourceDialog({ workspaceId, onClose }: AddSourceDialogProps) {
  const [isAdding, setIsAdding] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const addLocalSource = useAddLocalSource();
  const scanSource = useScanSource();

  const handleSelectDirectory = async () => {
    try {
      setError(null);
      const selected = await open({
        directory: true,
        multiple: false,
        title: "选择数据源目录",
      });

      if (!selected) {
        return;
      }

      setIsAdding(true);
      const path = selected as string;

      // 添加数据源（后端会自动检测是 Git 仓库还是普通目录）
      const source = await addLocalSource.mutateAsync({
        workspace_id: workspaceId,
        path,
      });

      // 添加成功后自动触发扫描
      await scanSource.mutateAsync({
        source_id: source.id,
        root_path: source.root_path,
      });

      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsAdding(false);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>添加数据源</h2>
          <button className="dialog-close" onClick={onClose}>
            ×
          </button>
        </div>
        <div className="dialog-content">
          <p>选择一个本地目录或 Git 仓库作为数据源。</p>
          <p>系统会自动检测目录类型，并扫描其中的文件。</p>

          {error && <div className="dialog-error">{error}</div>}
        </div>
        <div className="dialog-footer">
          <button className="btn btn-secondary" onClick={onClose} disabled={isAdding}>
            取消
          </button>
          <button
            className="btn btn-primary"
            onClick={handleSelectDirectory}
            disabled={isAdding}
          >
            {isAdding ? "添加中..." : "选择目录"}
          </button>
        </div>
      </div>
    </div>
  );
}
