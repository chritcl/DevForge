import { useState } from "react";
import { useNavigate } from "react-router";
import { useWorkspace, useArchiveWorkspace, useDeleteWorkspace } from "../hooks/useWorkspaces";
import { useSources, useRemoveSource } from "../hooks/useSources";
import { WorkspaceFormDialog } from "./WorkspaceFormDialog";
import type { SourceDto } from "../bindings";

interface WorkspaceSettingsDialogProps {
  workspaceId: string;
  onClose: () => void;
}

type SettingsTab = "general" | "sources" | "danger";

export function WorkspaceSettingsDialog({
  workspaceId,
  onClose,
}: WorkspaceSettingsDialogProps) {
  const navigate = useNavigate();
  const { data: workspace } = useWorkspace(workspaceId);
  const { data: sources } = useSources(workspaceId);
  const archiveWorkspace = useArchiveWorkspace();
  const deleteWorkspace = useDeleteWorkspace();
  const removeSource = useRemoveSource();

  const [activeTab, setActiveTab] = useState<SettingsTab>("general");
  const [showEditDialog, setShowEditDialog] = useState(false);

  // 归档确认
  const [showArchiveConfirm, setShowArchiveConfirm] = useState(false);

  // 删除确认
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [deleteConfirmName, setDeleteConfirmName] = useState("");
  const [deleteError, setDeleteError] = useState<string | null>(null);

  // 移除数据源确认
  const [sourceToRemove, setSourceToRemove] = useState<SourceDto | null>(null);
  const [removeSourceError, setRemoveSourceError] = useState<string | null>(null);

  const handleArchive = async () => {
    try {
      await archiveWorkspace.mutateAsync(workspaceId);
      setShowArchiveConfirm(false);
      onClose();
    } catch (err) {
      console.error("归档失败:", err);
    }
  };

  const handleDelete = async () => {
    try {
      setDeleteError(null);
      await deleteWorkspace.mutateAsync(workspaceId);
      setShowDeleteConfirm(false);
      onClose();
      navigate("/");
    } catch (err) {
      setDeleteError(formatError(err));
    }
  };

  const handleRemoveSource = async () => {
    if (!sourceToRemove) return;

    try {
      setRemoveSourceError(null);
      await removeSource.mutateAsync({
        id: sourceToRemove.id,
        workspace_id: workspaceId,
      });
      setSourceToRemove(null);
    } catch (err) {
      setRemoveSourceError(formatError(err));
    }
  };

  if (!workspace) return null;

  return (
    <>
      <div className="dialog-overlay" onClick={onClose}>
        <div
          className="dialog dialog-wide"
          onClick={(e) => e.stopPropagation()}
        >
          <div className="dialog-header">
            <h2>工作区设置</h2>
            <button className="dialog-close" onClick={onClose}>
              ×
            </button>
          </div>

          <div className="settings-tabs">
            <button
              className={`settings-tab ${activeTab === "general" ? "settings-tab-active" : ""}`}
              onClick={() => setActiveTab("general")}
            >
              常规
            </button>
            <button
              className={`settings-tab ${activeTab === "sources" ? "settings-tab-active" : ""}`}
              onClick={() => setActiveTab("sources")}
            >
              数据源（{sources?.length ?? 0}）
            </button>
            <button
              className={`settings-tab ${activeTab === "danger" ? "settings-tab-active" : ""}`}
              onClick={() => setActiveTab("danger")}
            >
              危险操作
            </button>
          </div>

          <div className="settings-content">
            {activeTab === "general" && (
              <div className="settings-section">
                <div className="settings-field">
                  <label>名称</label>
                  <div className="settings-value">{workspace.name}</div>
                </div>
                <div className="settings-field">
                  <label>描述</label>
                  <div className="settings-value">
                    {workspace.description || <span className="text-muted">无描述</span>}
                  </div>
                </div>
                <div className="settings-field">
                  <label>工作区 ID</label>
                  <div className="settings-value settings-value-mono">{workspace.id}</div>
                </div>
                <div className="settings-field">
                  <label>创建时间</label>
                  <div className="settings-value">
                    {new Date(workspace.created_at).toLocaleString("zh-CN")}
                  </div>
                </div>
                <div className="settings-field">
                  <label>最近打开</label>
                  <div className="settings-value">
                    {workspace.last_opened_at
                      ? new Date(workspace.last_opened_at).toLocaleString("zh-CN")
                      : <span className="text-muted">未打开过</span>}
                  </div>
                </div>
                <button
                  className="btn btn-secondary"
                  onClick={() => setShowEditDialog(true)}
                >
                  编辑名称和描述
                </button>
              </div>
            )}

            {activeTab === "sources" && (
              <div className="settings-section">
                {sources?.map((source) => (
                  <div key={source.id} className="settings-source-item">
                    <div className="settings-source-info">
                      <div className="settings-source-name">{source.name}</div>
                      <div className="settings-source-meta">
                        <span className="settings-source-kind">
                          {source.kind === "Git" ? "Git 仓库" : "目录"}
                        </span>
                        <span className="settings-source-path">{source.root_path}</span>
                      </div>
                      <div className="settings-source-date">
                        添加于 {new Date(source.created_at).toLocaleDateString("zh-CN")}
                      </div>
                    </div>
                    <button
                      className="btn btn-small btn-danger-outline"
                      onClick={() => {
                        setSourceToRemove(source);
                        setRemoveSourceError(null);
                      }}
                    >
                      移除
                    </button>
                  </div>
                ))}
                {(!sources || sources.length === 0) && (
                  <div className="settings-empty">暂无数据源</div>
                )}
              </div>
            )}

            {activeTab === "danger" && (
              <div className="settings-section">
                <div className="settings-danger-item">
                  <div>
                    <h3>归档工作区</h3>
                    <p>归档后不会删除任何本地文件，可以稍后恢复。</p>
                  </div>
                  <button
                    className="btn btn-secondary"
                    onClick={() => setShowArchiveConfirm(true)}
                  >
                    归档
                  </button>
                </div>
                <div className="settings-danger-item">
                  <div>
                    <h3>永久删除工作区</h3>
                    <p>删除所有 DevForge 元数据。本地目录和文件不会被删除。</p>
                  </div>
                  <button
                    className="btn btn-danger"
                    onClick={() => {
                      setShowDeleteConfirm(true);
                      setDeleteConfirmName("");
                      setDeleteError(null);
                    }}
                  >
                    永久删除
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* 编辑对话框 */}
      {showEditDialog && workspace && (
        <WorkspaceFormDialog
          mode="edit"
          workspace={workspace}
          onClose={() => setShowEditDialog(false)}
        />
      )}

      {/* 归档确认 */}
      {showArchiveConfirm && (
        <div className="dialog-overlay" onClick={() => setShowArchiveConfirm(false)}>
          <div className="dialog" onClick={(e) => e.stopPropagation()}>
            <div className="dialog-header">
              <h2>确认归档</h2>
              <button
                className="dialog-close"
                onClick={() => setShowArchiveConfirm(false)}
              >
                ×
              </button>
            </div>
            <div className="dialog-content">
              <p>
                确定要归档工作区 <strong>{workspace.name}</strong> 吗？
              </p>
              <p className="dialog-info">
                归档后不会删除任何本地文件，可以稍后恢复。
              </p>
            </div>
            <div className="dialog-footer">
              <button
                className="btn btn-secondary"
                onClick={() => setShowArchiveConfirm(false)}
              >
                取消
              </button>
              <button
                className="btn btn-primary"
                onClick={handleArchive}
                disabled={archiveWorkspace.isPending}
              >
                {archiveWorkspace.isPending ? "归档中..." : "确认归档"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 删除确认 */}
      {showDeleteConfirm && (
        <div className="dialog-overlay" onClick={() => setShowDeleteConfirm(false)}>
          <div className="dialog" onClick={(e) => e.stopPropagation()}>
            <div className="dialog-header">
              <h2>永久删除工作区</h2>
              <button
                className="dialog-close"
                onClick={() => setShowDeleteConfirm(false)}
              >
                ×
              </button>
            </div>
            <div className="dialog-content">
              <p>
                此操作将永久删除工作区 <strong>{workspace.name}</strong> 的所有
                DevForge 元数据。
              </p>
              <p className="dialog-warning">
                本地目录和文件不会被删除。数据源、文档索引和标签记录将被移除。
              </p>
              <div className="form-group">
                <label htmlFor="settings-delete-confirm">
                  输入工作区名称 <strong>{workspace.name}</strong> 以确认删除
                </label>
                <input
                  id="settings-delete-confirm"
                  type="text"
                  value={deleteConfirmName}
                  onChange={(e) => setDeleteConfirmName(e.target.value)}
                  placeholder={workspace.name}
                />
              </div>
              {deleteError && <div className="dialog-error">{deleteError}</div>}
            </div>
            <div className="dialog-footer">
              <button
                className="btn btn-secondary"
                onClick={() => setShowDeleteConfirm(false)}
              >
                取消
              </button>
              <button
                className="btn btn-danger"
                onClick={handleDelete}
                disabled={deleteConfirmName !== workspace.name || deleteWorkspace.isPending}
              >
                {deleteWorkspace.isPending ? "删除中..." : "永久删除"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 移除数据源确认 */}
      {sourceToRemove && (
        <div className="dialog-overlay" onClick={() => setSourceToRemove(null)}>
          <div className="dialog" onClick={(e) => e.stopPropagation()}>
            <div className="dialog-header">
              <h2>移除数据源</h2>
              <button
                className="dialog-close"
                onClick={() => setSourceToRemove(null)}
              >
                ×
              </button>
            </div>
            <div className="dialog-content">
              <p>
                将从 DevForge 中移除数据源 <strong>{sourceToRemove.name}</strong> 及其文档索引和标签记录。
              </p>
              <div className="settings-source-detail">
                <div>类型：{sourceToRemove.kind === "Git" ? "Git 仓库" : "目录"}</div>
                <div>路径：{sourceToRemove.root_path}</div>
              </div>
              <p className="dialog-info">本地目录和文件不会被删除。</p>
              {removeSourceError && <div className="dialog-error">{removeSourceError}</div>}
            </div>
            <div className="dialog-footer">
              <button
                className="btn btn-secondary"
                onClick={() => setSourceToRemove(null)}
              >
                取消
              </button>
              <button
                className="btn btn-danger"
                onClick={handleRemoveSource}
                disabled={removeSource.isPending}
              >
                {removeSource.isPending ? "移除中..." : "确认移除"}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

/// 格式化错误为用户可见的中文消息
function formatError(err: unknown): string {
  if (typeof err === "string") {
    switch (err) {
      case "WorkspaceNotFound":
        return "工作区不存在";
      case "SourceNotFound":
        return "数据源不存在";
      default:
        return err;
    }
  }
  if (err && typeof err === "object") {
    if ("WorkspaceNotFound" in err) return "工作区不存在";
    if ("SourceNotFound" in err) return "数据源不存在";
    if ("Domain" in err) return `领域错误: ${(err as Record<string, unknown>).Domain}`;
    if (err instanceof Error) return err.message;
  }
  return "未知错误";
}
