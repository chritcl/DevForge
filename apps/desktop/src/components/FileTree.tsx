import { useState } from "react";
import { useFileTree } from "../hooks/useDocuments";
import type { DocumentDto, FileTreeEntryDto } from "../bindings";

interface FileTreeProps {
  sourceId: string;
  sourceName: string;
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
  const { data: items, isLoading, error } = useFileTree(sourceId, parentPath ?? undefined);

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

  return (
    <>
      {items.map((entry) =>
        entry.entry_kind === "directory" ? (
          <FileTreeDirItem
            key={entry.key}
            entry={entry}
            sourceId={sourceId}
            onFileSelect={onFileSelect}
          />
        ) : (
          <FileTreeFileItem
            key={entry.key}
            entry={entry}
            onFileSelect={onFileSelect}
          />
        )
      )}
    </>
  );
}

interface FileTreeDirItemProps {
  entry: FileTreeEntryDto;
  sourceId: string;
  onFileSelect: (doc: DocumentDto) => void;
}

function FileTreeDirItem({ entry, sourceId, onFileSelect }: FileTreeDirItemProps) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="file-tree-dir">
      <div
        className="file-tree-dir-header"
        onClick={() => setExpanded(!expanded)}
      >
        <span className="file-tree-icon">{expanded ? "📂" : "📁"}</span>
        <span className="file-tree-name">{entry.name}</span>
      </div>

      {expanded && (
        <div className="file-tree-dir-content">
          <FileTreeDirectory
            sourceId={sourceId}
            parentPath={entry.relative_path}
            onFileSelect={onFileSelect}
          />
        </div>
      )}
    </div>
  );
}

interface FileTreeFileItemProps {
  entry: FileTreeEntryDto;
  onFileSelect: (doc: DocumentDto) => void;
}

function FileTreeFileItem({ entry, onFileSelect }: FileTreeFileItemProps) {
  const doc = entry.document;
  if (!doc) return null;

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

  const isSensitive = doc.sensitivity === "sensitive";

  return (
    <div
      className={`file-tree-file ${isSensitive ? "file-tree-file-sensitive" : ""}`}
      onClick={() => onFileSelect(doc)}
    >
      <span className="file-tree-icon">{getFileIcon(doc.kind)}</span>
      <span className="file-tree-name">{entry.name}</span>
      {isSensitive && <span className="file-tree-sensitive-badge">🔒</span>}
    </div>
  );
}
