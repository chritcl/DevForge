import { useState, useMemo, useCallback, useEffect, useRef } from "react";
import { useParams, useNavigate, useSearchParams } from "react-router";
import {
  useWorkspace,
  useMarkWorkspaceOpened,
  useRestoreWorkspace,
  useDeleteWorkspace,
} from "../hooks/useWorkspaces";
import { useSources } from "../hooks/useSources";
import { useTabs, useOpenTab, useCloseTab, useSetActiveTab } from "../hooks/useTabs";
import { useDocumentsByIds } from "../hooks/useDocuments";
import { FileTree } from "../components/FileTree";
import { FileViewer } from "../components/FileViewer";
import { TabBar } from "../components/TabBar";
import { AddSourceDialog } from "../components/AddSourceDialog";
import { SearchPanel } from "../components/SearchPanel";
import { WorkspaceSettingsDialog } from "../components/WorkspaceSettingsDialog";
import type { DocumentDto, DocumentLookupDto } from "../bindings";

export function WorkspacePage() {
  const { id } = useParams<{ id: string }>();
  const workspaceId = id ?? "";
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();

  const { data: workspace, isLoading, error } = useWorkspace(workspaceId);
  const { data: sources } = useSources(workspaceId);
  const { data: tabs } = useTabs(workspaceId);
  const markOpened = useMarkWorkspaceOpened();
  const restoreWorkspace = useRestoreWorkspace();
  const deleteWorkspace = useDeleteWorkspace();

  const openTab = useOpenTab();
  const closeTab = useCloseTab();
  const setActiveTab = useSetActiveTab();

  // 用户手动选择的标签 ID
  const [userSelectedTabId, setUserSelectedTabId] = useState<string | null>(null);
  const [showAddSource, setShowAddSource] = useState(false);
  // 从 URL 参数初始化设置对话框状态
  const [showSettings, setShowSettings] = useState(searchParams.get("settings") === "true");

  // 删除确认状态
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [deleteConfirmName, setDeleteConfirmName] = useState("");
  const [deleteError, setDeleteError] = useState<string | null>(null);

  const isArchived = workspace?.status === "Archived";

  // 关闭设置时清除 URL 参数
  const handleCloseSettings = useCallback(() => {
    setShowSettings(false);
    if (searchParams.has("settings")) {
      setSearchParams({}, { replace: true });
    }
  }, [searchParams, setSearchParams]);

  // 标记工作区已打开（仅活跃工作区）
  const hasMarkedOpened = useRef(false);
  useEffect(() => {
    if (workspaceId && !hasMarkedOpened.current && !isArchived) {
      hasMarkedOpened.current = true;
      markOpened.mutate(workspaceId);
    }
  }, [workspaceId, markOpened, isArchived]);

  // 批量获取标签对应的文档信息（一次 IPC）
  const documentIds = useMemo(
    () => tabs?.map((t) => t.document_id) ?? [],
    [tabs]
  );
  const { data: lookups } = useDocumentsByIds(documentIds);

  // 构建查找结果 Map
  const documentLookupMap = useMemo(() => {
    const map = new Map<string, DocumentLookupDto>();
    lookups?.forEach((lookup) => map.set(lookup.document_id, lookup));
    return map;
  }, [lookups]);

  // 派生活动标签 ID：用户选择 > 后端保存的活动标签 > 第一个标签
  const activeTabId = useMemo(() => {
    if (userSelectedTabId && tabs?.some((t) => t.id === userSelectedTabId)) {
      return userSelectedTabId;
    }
    if (tabs && tabs.length > 0) {
      return (tabs.find((t) => t.is_active) ?? tabs[0]).id;
    }
    return null;
  }, [userSelectedTabId, tabs]);

  // 找到当前活动标签对应的文档
  const activeDocument = useMemo(() => {
    if (!activeTabId || !tabs) return null;
    const activeTab = tabs.find((t) => t.id === activeTabId);
    if (!activeTab) return null;
    const lookup = documentLookupMap.get(activeTab.document_id);
    return lookup?.status === "found" ? lookup.document : null;
  }, [activeTabId, tabs, documentLookupMap]);

  // 处理文件选择
  const handleFileSelect = useCallback(
    async (doc: DocumentDto) => {
      if (!workspaceId || isArchived) return;

      try {
        const tab = await openTab.mutateAsync({
          workspace_id: workspaceId,
          document_id: doc.id,
        });
        setUserSelectedTabId(tab.id);
      } catch (err) {
        console.error("打开标签失败:", err);
      }
    },
    [workspaceId, openTab, isArchived]
  );

  // 处理搜索结果点击（通过 document_id 打开标签，并记录行号）
  const handleSearchResultClick = useCallback(
    async (documentId: string, lineNumber: number) => {
      if (!workspaceId || isArchived) return;

      try {
        const tab = await openTab.mutateAsync({
          workspace_id: workspaceId,
          document_id: documentId,
        });
        setUserSelectedTabId(tab.id);
        // 将行号存储在 URL 参数中，供 FileViewer 使用
        setSearchParams({ line: String(lineNumber) }, { replace: true });
      } catch (err) {
        console.error("打开标签失败:", err);
      }
    },
    [workspaceId, openTab, isArchived, setSearchParams],
  );

  // 处理标签点击
  const handleTabClick = useCallback(
    async (tabId: string) => {
      if (!workspaceId || isArchived) return;

      setUserSelectedTabId(tabId);
      try {
        await setActiveTab.mutateAsync({
          workspace_id: workspaceId,
          tab_id: tabId,
        });
      } catch (err) {
        console.error("设置活动标签失败:", err);
      }
    },
    [workspaceId, setActiveTab, isArchived]
  );

  // 处理标签关闭
  // 后端已自动选择下一个活动标签，前端清除用户选择让 activeTabId 派生逻辑接管
  const handleTabClose = useCallback(
    async (tabId: string) => {
      if (!workspaceId || isArchived) return;

      try {
        await closeTab.mutateAsync({
          id: tabId,
          workspace_id: workspaceId,
        });
        // 清除用户手动选择，让 useMemo 回退到后端 is_active 字段
        setUserSelectedTabId(null);
      } catch (err) {
        console.error("关闭标签失败:", err);
      }
    },
    [workspaceId, closeTab, isArchived]
  );

  // 恢复工作区
  const handleRestore = async () => {
    try {
      await restoreWorkspace.mutateAsync(workspaceId);
    } catch (err) {
      console.error("恢复失败:", err);
    }
  };

  // 删除工作区
  const handleDelete = async () => {
    try {
      setDeleteError(null);
      await deleteWorkspace.mutateAsync(workspaceId);
      navigate("/");
    } catch (err) {
      setDeleteError(formatError(err));
    }
  };

  if (isLoading) {
    return <div className="workspace-loading">加载中...</div>;
  }

  if (error) {
    return <div className="workspace-error">加载失败: {String(error)}</div>;
  }

  if (!workspace) {
    return <div className="workspace-not-found">工作区不存在</div>;
  }

  // 归档工作区显示归档状态页面
  if (isArchived) {
    return (
      <div className="workspace-page">
        <div className="workspace-archived-page">
          <div className="workspace-archived-icon">📦</div>
          <h1 className="workspace-archived-title">工作区已归档</h1>
          <p className="workspace-archived-name">{workspace.name}</p>
          {workspace.description && (
            <p className="workspace-archived-description">{workspace.description}</p>
          )}
          <p className="workspace-archived-message">
            此工作区已归档，无法直接访问文件和数据源。恢复后可以正常使用。
          </p>
          <div className="workspace-archived-actions">
            <button className="btn btn-primary" onClick={handleRestore}>
              恢复工作区
            </button>
            <button className="btn btn-secondary" onClick={() => navigate("/")}>
              返回工作区列表
            </button>
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

        {/* 删除确认对话框 */}
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
                  <label htmlFor="delete-confirm-name">
                    输入工作区名称 <strong>{workspace.name}</strong> 以确认删除
                  </label>
                  <input
                    id="delete-confirm-name"
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
      </div>
    );
  }

  // 活跃工作区正常显示
  return (
    <div className="workspace-page">
      <div className="workspace-header">
        <h1>{workspace.name}</h1>
        {workspace.description && (
          <p className="workspace-description">{workspace.description}</p>
        )}
        <button
          className="workspace-settings-btn"
          onClick={() => setShowSettings(true)}
          title="工作区设置"
        >
          ⚙ 设置
        </button>
      </div>

      <div className="workspace-content">
        <div className="workspace-sidebar">
          <div className="workspace-explorer">
            <div className="workspace-explorer-header">
              <h2>资源管理器</h2>
              <button
                className="btn btn-small btn-primary"
                onClick={() => setShowAddSource(true)}
                title="添加数据源"
              >
                +
              </button>
            </div>
            {sources?.map((source) => (
              <FileTree
                key={source.id}
                sourceId={source.id}
                sourceName={source.name}
                onFileSelect={handleFileSelect}
              />
            ))}
            {(!sources || sources.length === 0) && (
              <div className="workspace-no-sources">
                <p>暂无数据源</p>
                <button
                  className="btn btn-primary"
                  onClick={() => setShowAddSource(true)}
                >
                  添加数据源
                </button>
              </div>
            )}
          </div>
          <SearchPanel
            workspaceId={workspaceId}
            onResultClick={handleSearchResultClick}
          />
        </div>

        <div className="workspace-main">
          <TabBar
            tabs={tabs ?? []}
            documentLookups={documentLookupMap}
            activeTabId={activeTabId}
            onTabClick={handleTabClick}
            onTabClose={handleTabClose}
          />

          {activeDocument ? (
            <FileViewer document={activeDocument} />
          ) : (
            <div className="workspace-welcome">
              <div className="workspace-welcome-icon">📁</div>
              <div className="workspace-welcome-title">
                欢迎使用 DevForge
              </div>
              <div className="workspace-welcome-message">
                在左侧资源管理器中选择文件以查看内容。
              </div>
            </div>
          )}
        </div>
      </div>

      {showAddSource && workspaceId && (
        <AddSourceDialog
          workspaceId={workspaceId}
          onClose={() => setShowAddSource(false)}
        />
      )}

      {showSettings && workspaceId && !isArchived && (
        <WorkspaceSettingsDialog
          workspaceId={workspaceId}
          onClose={handleCloseSettings}
        />
      )}
    </div>
  );
}

/// 格式化错误为用户可见的中文消息
function formatError(err: unknown): string {
  if (typeof err === "string") {
    switch (err) {
      case "WorkspaceNotFound":
        return "工作区不存在";
      default:
        return err;
    }
  }
  if (err && typeof err === "object") {
    if ("WorkspaceNotFound" in err) return "工作区不存在";
    if ("Domain" in err) return `领域错误: ${(err as Record<string, unknown>).Domain}`;
    if (err instanceof Error) return err.message;
  }
  return "未知错误";
}
