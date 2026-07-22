import { useState } from "react";
import { useNavigate } from "react-router";
import {
  useWorkspaces,
  useArchivedWorkspaces,
  useArchiveWorkspace,
  useRestoreWorkspace,
  useDeleteWorkspace,
} from "../hooks/useWorkspaces";
import { WorkspaceFormDialog } from "../components/WorkspaceFormDialog";
import type { WorkspaceDto } from "../bindings";

type ViewMode = "active" | "archived";

export function WorkspaceListPage() {
  const navigate = useNavigate();
  const { data: activeWorkspaces, isLoading: activeLoading } = useWorkspaces();
  const { data: archivedWorkspaces, isLoading: archivedLoading } = useArchivedWorkspaces();
  const archiveWorkspace = useArchiveWorkspace();
  const restoreWorkspace = useRestoreWorkspace();
  const deleteWorkspace = useDeleteWorkspace();

  const [viewMode, setViewMode] = useState<ViewMode>("active");
  const [showCreateDialog, setShowCreateDialog] = useState(false);

  // 操作菜单状态
  const [openMenuId, setOpenMenuId] = useState<string | null>(null);

  // 删除确认状态
  const [deleteConfirm, setDeleteConfirm] = useState<WorkspaceDto | null>(null);
  const [deleteConfirmName, setDeleteConfirmName] = useState("");
  const [deleteError, setDeleteError] = useState<string | null>(null);

  const isLoading = viewMode === "active" ? activeLoading : archivedLoading;
  const workspaces = viewMode === "active" ? activeWorkspaces : archivedWorkspaces;

  const handleArchive = async (id: string) => {
    try {
      await archiveWorkspace.mutateAsync(id);
      setOpenMenuId(null);
    } catch (err) {
      console.error("归档失败:", err);
    }
  };

  const handleRestore = async (id: string) => {
    try {
      await restoreWorkspace.mutateAsync(id);
    } catch (err) {
      console.error("恢复失败:", err);
    }
  };

  const handleDelete = async () => {
    if (!deleteConfirm) return;

    try {
      setDeleteError(null);
      await deleteWorkspace.mutateAsync(deleteConfirm.id);
      setDeleteConfirm(null);
      setDeleteConfirmName("");
    } catch (err) {
      setDeleteError(formatError(err));
    }
  };

  const handleCardClick = (workspace: WorkspaceDto) => {
    if (viewMode === "active") {
      navigate(`/workspace/${workspace.id}`);
    }
  };

  const handleMenuButtonClick = (e: React.MouseEvent, workspaceId: string) => {
    e.stopPropagation();
    setOpenMenuId(openMenuId === workspaceId ? null : workspaceId);
  };

  const handleAction = (e: React.MouseEvent, action: () => void) => {
    e.stopPropagation();
    action();
    setOpenMenuId(null);
  };

  return (
    <div className="workspace-list-page">
      <div className="workspace-list-header">
        <h1>DevForge</h1>
        <p>本地知识库与 AI 编程工作台</p>
      </div>

      <div className="workspace-list-tabs">
        <button
          className={`workspace-tab ${viewMode === "active" ? "workspace-tab-active" : ""}`}
          onClick={() => setViewMode("active")}
        >
          活跃工作区（{activeWorkspaces?.length ?? 0}）
        </button>
        <button
          className={`workspace-tab ${viewMode === "archived" ? "workspace-tab-active" : ""}`}
          onClick={() => setViewMode("archived")}
        >
          已归档（{archivedWorkspaces?.length ?? 0}）
        </button>
      </div>

      {viewMode === "active" && (
        <div className="workspace-list-actions">
          <button
            className="btn btn-primary"
            onClick={() => setShowCreateDialog(true)}
          >
            创建工作区
          </button>
        </div>
      )}

      {isLoading ? (
        <div className="workspace-list-loading">加载中...</div>
      ) : (
        <div className="workspace-list">
          {workspaces?.map((workspace) => (
            <div
              key={workspace.id}
              className={`workspace-card ${viewMode === "active" ? "workspace-card-clickable" : ""}`}
              onClick={() => handleCardClick(workspace)}
            >
              <div className="workspace-card-header">
                <h2>{workspace.name}</h2>
                <div className="workspace-card-actions">
                  <button
                    className="workspace-card-menu-btn"
                    onClick={(e) => handleMenuButtonClick(e, workspace.id)}
                    title="操作"
                  >
                    ⋯
                  </button>
                  {openMenuId === workspace.id && (
                    <div className="workspace-card-menu">
                      {viewMode === "active" ? (
                        <>
                          <button
                            className="workspace-card-menu-item"
                            onClick={(e) =>
                              handleAction(e, () =>
                                navigate(`/workspace/${workspace.id}?settings=true`)
                              )
                            }
                          >
                            编辑
                          </button>
                          <button
                            className="workspace-card-menu-item"
                            onClick={(e) => handleAction(e, () => handleArchive(workspace.id))}
                          >
                            归档
                          </button>
                          <button
                            className="workspace-card-menu-item workspace-card-menu-item-danger"
                            onClick={(e) =>
                              handleAction(e, () => {
                                setDeleteConfirm(workspace);
                                setDeleteConfirmName("");
                                setDeleteError(null);
                              })
                            }
                          >
                            删除
                          </button>
                        </>
                      ) : (
                        <>
                          <button
                            className="workspace-card-menu-item"
                            onClick={(e) => handleAction(e, () => handleRestore(workspace.id))}
                          >
                            恢复
                          </button>
                          <button
                            className="workspace-card-menu-item workspace-card-menu-item-danger"
                            onClick={(e) =>
                              handleAction(e, () => {
                                setDeleteConfirm(workspace);
                                setDeleteConfirmName("");
                                setDeleteError(null);
                              })
                            }
                          >
                            删除
                          </button>
                        </>
                      )}
                    </div>
                  )}
                </div>
              </div>
              {workspace.description && (
                <p className="workspace-card-description">{workspace.description}</p>
              )}
              <div className="workspace-card-meta">
                <span>
                  创建于{" "}
                  {new Date(workspace.created_at).toLocaleDateString("zh-CN")}
                </span>
                {workspace.last_opened_at && (
                  <span>
                    最近打开{" "}
                    {new Date(workspace.last_opened_at).toLocaleDateString("zh-CN")}
                  </span>
                )}
              </div>
            </div>
          ))}

          {(!workspaces || workspaces.length === 0) && (
            <div className="workspace-list-empty">
              <div className="workspace-list-empty-icon">📁</div>
              <div className="workspace-list-empty-title">
                {viewMode === "active" ? "暂无活跃工作区" : "暂无已归档工作区"}
              </div>
              <div className="workspace-list-empty-message">
                {viewMode === "active"
                  ? "创建一个工作区开始使用 DevForge。"
                  : "归档的工作区会显示在这里。"}
              </div>
            </div>
          )}
        </div>
      )}

      {/* 创建工作区对话框 */}
      {showCreateDialog && (
        <WorkspaceFormDialog
          mode="create"
          onClose={() => setShowCreateDialog(false)}
          onSuccess={(workspace) => navigate(`/workspace/${workspace.id}`)}
        />
      )}

      {/* 删除确认对话框 */}
      {deleteConfirm && (
        <div
          className="dialog-overlay"
          onClick={() => setDeleteConfirm(null)}
        >
          <div className="dialog" onClick={(e) => e.stopPropagation()}>
            <div className="dialog-header">
              <h2>永久删除工作区</h2>
              <button
                className="dialog-close"
                onClick={() => setDeleteConfirm(null)}
              >
                ×
              </button>
            </div>
            <div className="dialog-content">
              <p>
                此操作将永久删除工作区 <strong>{deleteConfirm.name}</strong> 的所有
                DevForge 元数据。
              </p>
              <p className="dialog-warning">
                本地目录和文件不会被删除。数据源、文档索引和标签记录将被移除。
              </p>
              <div className="form-group">
                <label htmlFor="delete-confirm-name">
                  输入工作区名称 <strong>{deleteConfirm.name}</strong> 以确认删除
                </label>
                <input
                  id="delete-confirm-name"
                  type="text"
                  value={deleteConfirmName}
                  onChange={(e) => setDeleteConfirmName(e.target.value)}
                  placeholder={deleteConfirm.name}
                />
              </div>
              {deleteError && <div className="dialog-error">{deleteError}</div>}
            </div>
            <div className="dialog-footer">
              <button
                className="btn btn-secondary"
                onClick={() => setDeleteConfirm(null)}
              >
                取消
              </button>
              <button
                className="btn btn-danger"
                onClick={handleDelete}
                disabled={deleteConfirmName !== deleteConfirm.name || deleteWorkspace.isPending}
              >
                {deleteWorkspace.isPending ? "删除中..." : "永久删除"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

/// 格式化错误为用户可见的中文消息
function formatError(err: unknown): string {
  if (typeof err === "string") {
    return formatAppError(err);
  }
  if (err && typeof err === "object") {
    // 处理 Specta 生成的错误格式
    if ("WorkspaceNotFound" in err) return "工作区不存在";
    if ("DuplicateName" in err) return "工作区名称已存在";
    if ("Domain" in err) return `领域错误: ${(err as Record<string, unknown>).Domain}`;
    if ("SourceNotFound" in err) return "数据源不存在";
    if (err instanceof Error) return err.message;
  }
  return "未知错误";
}

function formatAppError(err: string): string {
  switch (err) {
    case "WorkspaceNotFound":
      return "工作区不存在";
    case "DuplicateName":
      return "工作区名称已存在";
    default:
      return err;
  }
}
