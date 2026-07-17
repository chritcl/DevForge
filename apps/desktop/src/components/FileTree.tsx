import { useState } from "react";
import { useDocuments } from "../hooks/useDocuments";
import type { DocumentDto } from "../types";

interface FileTreeProps {
  sourceId: string;
  sourceName: string;
  sourceRoot: string;
  onFileSelect: (doc: DocumentDto) => void;
}

export function FileTree({
  sourceId,
  sourceName,
  onFileSelect,
}: FileTreeProps) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="file-tree">
      <div
        className="file-tree-header"
        onClick={() => setExpanded(!expanded)}
      >
        <span className="file-tree-icon">{expanded ? "📂" : "📁"}</span>
        <span className="file-tree-name">{sourceName}</span>
      </div>

      {expanded && (
        <div className="file-tree-content">
          <FileTreeDirectory
            sourceId={sourceId}
            parentPath={null}
            onFileSelect={onFileSelect}
          />
        </div>
      )}
    </div>
  );
}

interface FileTreeDirectoryProps {
  sourceId: string;
  parentPath: string | null;
  onFileSelect: (doc: DocumentDto) => void;
}

function FileTreeDirectory({
  sourceId,
  parentPath,
  onFileSelect,
}: FileTreeDirectoryProps) {
  const { data: items, isLoading, error } = useDocuments(sourceId, parentPath ?? undefined);

  if (isLoading) {
    return (
      <div className="file-tree-loading">
        <span>加载中...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="file-tree-error">
        <span>加载失败</span>
      </div>
    );
  }

  if (!items || items.length === 0) {
    return (
      <div className="file-tree-empty">暂无文件</div>
    );
  }

  // 分离目录和文件
  // 目录条目的特征：kind 为 "unknown" 且 content_readable 为 false 且 size 为 0
  const dirs = items.filter(
    (item) => item.kind === "unknown" && !item.content_readable && item.size === 0
  );
  const files = items.filter(
    (item) => !(item.kind === "unknown" && !item.content_readable && item.size === 0)
  );

  return (
    <>
      {dirs.map((dir) => (
        <FileTreeDirItem
          key={dir.id}
          dir={dir}
          sourceId={sourceId}
          onFileSelect={onFileSelect}
        />
      ))}
      {files.map((file) => (
        <FileTreeFileItem
          key={file.id}
          doc={file}
          onFileSelect={onFileSelect}
        />
      ))}
    </>
  );
}

interface FileTreeDirItemProps {
  dir: DocumentDto;
  sourceId: string;
  onFileSelect: (doc: DocumentDto) => void;
}

function FileTreeDirItem({ dir, sourceId, onFileSelect }: FileTreeDirItemProps) {
  const [expanded, setExpanded] = useState(false);
  const dirName = dir.relative_path.split(/[/\\]/).pop() ?? dir.relative_path;

  return (
    <div className="file-tree-dir">
      <div
        className="file-tree-dir-header"
        onClick={() => setExpanded(!expanded)}
      >
        <span className="file-tree-icon">{expanded ? "📂" : "📁"}</span>
        <span className="file-tree-name">{dirName}</span>
      </div>

      {expanded && (
        <div className="file-tree-dir-content">
          <FileTreeDirectory
            sourceId={sourceId}
            parentPath={dir.relative_path}
            onFileSelect={onFileSelect}
          />
        </div>
      )}
    </div>
  );
}

interface FileTreeFileItemProps {
  doc: DocumentDto;
  onFileSelect: (doc: DocumentDto) => void;
}

function FileTreeFileItem({ doc, onFileSelect }: FileTreeFileItemProps) {
  const fileName =
    doc.relative_path.split(/[/\\]/).pop() ?? doc.relative_path;

  const getFileIcon = (kind: string) => {
    switch (kind) {
      case "text":
        return "📄";
      case "markdown":
        return "📝";
      case "image":
        return "🖼️";
      case "binary":
        return "⚙️";
      default:
        return "📄";
    }
  };

  const isSensitive = doc.sensitivity === "Sensitive";

  return (
    <div
      className={`file-tree-file ${isSensitive ? "file-tree-file-sensitive" : ""}`}
      onClick={() => onFileSelect(doc)}
    >
      <span className="file-tree-icon">{getFileIcon(doc.kind)}</span>
      <span className="file-tree-name">{fileName}</span>
      {isSensitive && <span className="file-tree-sensitive-badge">🔒</span>}
    </div>
  );
}
