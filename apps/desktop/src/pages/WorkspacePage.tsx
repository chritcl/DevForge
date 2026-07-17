import { useState, useMemo, useCallback } from "react";
import { useParams } from "react-router";
import { useWorkspace } from "../hooks/useWorkspaces";
import { useSources } from "../hooks/useSources";
import { useTabs, useOpenTab, useCloseTab, useSetActiveTab } from "../hooks/useTabs";
import { useDocuments } from "../hooks/useDocuments";
import { FileTree } from "../components/FileTree";
import { FileViewer } from "../components/FileViewer";
import { TabBar } from "../components/TabBar";
import { AddSourceDialog } from "../components/AddSourceDialog";
import type { DocumentDto } from "../types";

export function WorkspacePage() {
  const { id } = useParams<{ id: string }>();
  const workspaceId = id ?? "";

  const { data: workspace, isLoading, error } = useWorkspace(workspaceId);
  const { data: sources } = useSources(workspaceId);
  const { data: tabs } = useTabs(workspaceId);

  const openTab = useOpenTab();
  const closeTab = useCloseTab();
  const setActiveTab = useSetActiveTab();

  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [showAddSource, setShowAddSource] = useState(false);

  // 收集所有需要查询文档的 source ID
  const sourceIds = useMemo(
    () => sources?.map((s) => s.id) ?? [],
    [sources]
  );

  // 查询所有 source 的文档以获取标签对应的文档信息
  const { data: allDocs1 } = useDocuments(sourceIds[0] ?? "", undefined);
  const { data: allDocs2 } = useDocuments(sourceIds[1] ?? "", undefined);
  const { data: allDocs3 } = useDocuments(sourceIds[2] ?? "", undefined);

  // 构建文档映射
  const documentMap = useMemo(() => {
    const map = new Map<string, DocumentDto>();
    [allDocs1, allDocs2, allDocs3].forEach((docs) => {
      docs?.forEach((doc) => map.set(doc.id, doc));
    });
    return map;
  }, [allDocs1, allDocs2, allDocs3]);

  // 找到当前活动标签对应的文档
  const activeDocument = useMemo(() => {
    if (!activeTabId || !tabs) return null;
    const activeTab = tabs.find((t) => t.id === activeTabId);
    if (!activeTab) return null;
    return documentMap.get(activeTab.document_id) ?? null;
  }, [activeTabId, tabs, documentMap]);

  // 找到活动文档对应的 source root
  const activeSourceRoot = useMemo(() => {
    if (!activeDocument) return "";
    const source = sources?.find((s) => s.id === activeDocument.source_id);
    return source?.root_path ?? "";
  }, [activeDocument, sources]);

  // 处理文件选择
  const handleFileSelect = useCallback(
    async (doc: DocumentDto) => {
      if (!workspaceId) return;

      try {
        const tab = await openTab.mutateAsync({
          workspace_id: workspaceId,
          document_id: doc.id,
        });
        setActiveTabId(tab.id);
      } catch (err) {
        console.error("打开标签失败:", err);
      }
    },
    [workspaceId, openTab]
  );

  // 处理标签点击
  const handleTabClick = useCallback(
    async (tabId: string) => {
      if (!workspaceId) return;

      setActiveTabId(tabId);
      try {
        await setActiveTab.mutateAsync({
          workspace_id: workspaceId,
          tab_id: tabId,
        });
      } catch (err) {
        console.error("设置活动标签失败:", err);
      }
    },
    [workspaceId, setActiveTab]
  );

  // 处理标签关闭
  const handleTabClose = useCallback(
    async (tabId: string) => {
      if (!workspaceId) return;

      try {
        await closeTab.mutateAsync({
          id: tabId,
          workspace_id: workspaceId,
        });

        // 如果关闭的是活动标签，切换到相邻标签
        if (tabId === activeTabId && tabs) {
          const currentIndex = tabs.findIndex((t) => t.id === tabId);
          const remainingTabs = tabs.filter((t) => t.id !== tabId);
          if (remainingTabs.length > 0) {
            const nextIndex = Math.min(currentIndex, remainingTabs.length - 1);
            setActiveTabId(remainingTabs[nextIndex].id);
          } else {
            setActiveTabId(null);
          }
        }
      } catch (err) {
        console.error("关闭标签失败:", err);
      }
    },
    [workspaceId, closeTab, activeTabId, tabs]
  );

  if (isLoading) {
    return <div className="workspace-loading">加载中...</div>;
  }

  if (error) {
    return <div className="workspace-error">加载失败: {String(error)}</div>;
  }

  if (!workspace) {
    return <div className="workspace-not-found">工作区不存在</div>;
  }

  return (
    <div className="workspace-page">
      <div className="workspace-header">
        <h1>{workspace.name}</h1>
        {workspace.description && (
          <p className="workspace-description">{workspace.description}</p>
        )}
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
                sourceRoot={source.root_path}
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
        </div>

        <div className="workspace-main">
          <TabBar
            tabs={tabs ?? []}
            documents={documentMap}
            activeTabId={activeTabId}
            onTabClick={handleTabClick}
            onTabClose={handleTabClose}
          />

          {activeDocument ? (
            <FileViewer
              document={activeDocument}
              sourceRoot={activeSourceRoot}
            />
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
    </div>
  );
}
