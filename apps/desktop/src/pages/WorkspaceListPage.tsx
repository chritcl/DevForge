import { useState } from "react";
import { useNavigate } from "react-router";
import { useWorkspaces, useCreateWorkspace } from "../hooks/useWorkspaces";

export function WorkspaceListPage() {
  const navigate = useNavigate();
  const { data: workspaces, isLoading, error } = useWorkspaces();
  const createWorkspace = useCreateWorkspace();
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [newName, setNewName] = useState("");
  const [newDescription, setNewDescription] = useState("");

  if (isLoading) {
    return <div className="workspace-list-loading">加载中...</div>;
  }

  if (error) {
    return (
      <div className="workspace-list-error">加载失败: {String(error)}</div>
    );
  }

  const handleCreate = async () => {
    if (!newName.trim()) return;

    try {
      const workspace = await createWorkspace.mutateAsync({
        name: newName.trim(),
        description: newDescription.trim() || undefined,
      });
      setShowCreateDialog(false);
      setNewName("");
      setNewDescription("");
      navigate(`/workspace/${workspace.id}`);
    } catch (err) {
      console.error("创建工作区失败:", err);
    }
  };

  return (
    <div className="workspace-list-page">
      <div className="workspace-list-header">
        <h1>DevForge</h1>
        <p>本地知识库与 AI 编程工作台</p>
      </div>

      <div className="workspace-list-actions">
        <button
          className="btn btn-primary"
          onClick={() => setShowCreateDialog(true)}
        >
          创建工作区
        </button>
      </div>

      <div className="workspace-list">
        {workspaces?.map((workspace) => (
          <div
            key={workspace.id}
            className="workspace-card"
            onClick={() => navigate(`/workspace/${workspace.id}`)}
          >
            <div className="workspace-card-header">
              <h2>{workspace.name}</h2>
              {workspace.status === "archived" && (
                <span className="workspace-archived-badge">已归档</span>
              )}
            </div>
            {workspace.description && (
              <p className="workspace-card-description">
                {workspace.description}
              </p>
            )}
            <div className="workspace-card-meta">
              <span>
                创建于{" "}
                {new Date(workspace.created_at).toLocaleDateString("zh-CN")}
              </span>
              {workspace.last_opened_at && (
                <span>
                  最近打开{" "}
                  {new Date(workspace.last_opened_at).toLocaleDateString(
                    "zh-CN"
                  )}
                </span>
              )}
            </div>
          </div>
        ))}

        {(!workspaces || workspaces.length === 0) && (
          <div className="workspace-list-empty">
            <div className="workspace-list-empty-icon">📁</div>
            <div className="workspace-list-empty-title">暂无工作区</div>
            <div className="workspace-list-empty-message">
              创建一个工作区开始使用 DevForge。
            </div>
          </div>
        )}
      </div>

      {/* 创建工作区对话框 */}
      {showCreateDialog && (
        <div className="dialog-overlay">
          <div className="dialog">
            <div className="dialog-header">
              <h2>创建工作区</h2>
              <button
                className="dialog-close"
                onClick={() => setShowCreateDialog(false)}
              >
                ×
              </button>
            </div>
            <div className="dialog-content">
              <div className="form-group">
                <label htmlFor="workspace-name">名称</label>
                <input
                  id="workspace-name"
                  type="text"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  placeholder="输入工作区名称"
                  autoFocus
                />
              </div>
              <div className="form-group">
                <label htmlFor="workspace-description">描述（可选）</label>
                <textarea
                  id="workspace-description"
                  value={newDescription}
                  onChange={(e) => setNewDescription(e.target.value)}
                  placeholder="输入工作区描述"
                  rows={3}
                />
              </div>
            </div>
            <div className="dialog-footer">
              <button
                className="btn btn-secondary"
                onClick={() => setShowCreateDialog(false)}
              >
                取消
              </button>
              <button
                className="btn btn-primary"
                onClick={handleCreate}
                disabled={!newName.trim() || createWorkspace.isPending}
              >
                {createWorkspace.isPending ? "创建中..." : "创建"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
