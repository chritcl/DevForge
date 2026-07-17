import { useState } from "react";
import { useParams } from "react-router";
import { useWorkspace } from "../hooks/useWorkspaces";
import { useSources } from "../hooks/useSources";
import { FileTree } from "../components/FileTree";
import { FileViewer } from "../components/FileViewer";
import { AddSourceDialog } from "../components/AddSourceDialog";
import type { DocumentDto } from "../types";

export function WorkspacePage() {
  const { id } = useParams<{ id: string }>();
  const { data: workspace, isLoading, error } = useWorkspace(id ?? "");
  const { data: sources } = useSources(id ?? "");
  const [selectedDoc, setSelectedDoc] = useState<DocumentDto | null>(null);
  const [selectedSourceRoot, setSelectedSourceRoot] = useState<string>("");
  const [showAddSource, setShowAddSource] = useState(false);

  if (isLoading) {
    return <div className="workspace-loading">加载中...</div>;
  }

  if (error) {
    return <div className="workspace-error">加载失败: {String(error)}</div>;
  }

  if (!workspace) {
    return <div className="workspace-not-found">工作区不存在</div>;
  }

  const handleFileSelect = (doc: DocumentDto) => {
    setSelectedDoc(doc);
    // 找到对应的 source root
    const source = sources?.find((s) => s.id === doc.source_id);
    if (source) {
      setSelectedSourceRoot(source.root_path);
    }
  };

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
          {selectedDoc ? (
            <FileViewer
              document={selectedDoc}
              sourceRoot={selectedSourceRoot}
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

      {showAddSource && id && (
        <AddSourceDialog
          workspaceId={id}
          onClose={() => setShowAddSource(false)}
        />
      )}
    </div>
  );
}
