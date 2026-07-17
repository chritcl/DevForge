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
  sourceRoot,
  onFileSelect,
}: FileTreeProps) {
  const [expanded, setExpanded] = useState(false);
  const { data: documents, isLoading, error } = useDocuments(sourceId);

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

  const docs = documents ?? [];

  // 按目录分组
  const rootDocs = docs.filter(
    (d) => !d.relative_path.includes("/") && !d.relative_path.includes("\\")
  );
  const dirDocs = docs.filter(
    (d) => d.relative_path.includes("/") || d.relative_path.includes("\\")
  );

  // 提取唯一的目录
  const dirs = [
    ...new Set(
      dirDocs.map((d) => {
        const parts = d.relative_path.split(/[/\\]/);
        return parts[0];
      })
    ),
  ];

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
          {/* 目录 */}
          {dirs.map((dir) => (
            <FileTreeDir
              key={dir}
              name={dir}
              sourceId={sourceId}
              sourceRoot={sourceRoot}
              parentPath={dir}
              documents={dirDocs}
              onFileSelect={onFileSelect}
            />
          ))}

          {/* 根目录文件 */}
          {rootDocs.map((doc) => (
            <FileTreeFile
              key={doc.id}
              doc={doc}
              onClick={() => onFileSelect(doc)}
            />
          ))}

          {docs.length === 0 && (
            <div className="file-tree-empty">暂无文件</div>
          )}
        </div>
      )}
    </div>
  );
}

interface FileTreeDirProps {
  name: string;
  sourceId: string;
  sourceRoot: string;
  parentPath: string;
  documents: DocumentDto[];
  onFileSelect: (doc: DocumentDto) => void;
}

function FileTreeDir({
  name,
  sourceId,
  sourceRoot,
  parentPath,
  documents,
  onFileSelect,
}: FileTreeDirProps) {
  const [expanded, setExpanded] = useState(false);

  // 获取当前目录下的文件
  const dirFiles = documents.filter((d) => {
    const parts = d.relative_path.split(/[/\\]/);
    return parts[0] === name && parts.length === 2;
  });

  // 获取子目录
  const subDirs = [
    ...new Set(
      documents
        .filter((d) => {
          const parts = d.relative_path.split(/[/\\]/);
          return parts[0] === name && parts.length > 2;
        })
        .map((d) => {
          const parts = d.relative_path.split(/[/\\]/);
          return parts[1];
        })
    ),
  ];

  return (
    <div className="file-tree-dir">
      <div
        className="file-tree-dir-header"
        onClick={() => setExpanded(!expanded)}
      >
        <span className="file-tree-icon">{expanded ? "📂" : "📁"}</span>
        <span className="file-tree-name">{name}</span>
      </div>

      {expanded && (
        <div className="file-tree-dir-content">
          {/* 子目录 */}
          {subDirs.map((subDir) => (
            <FileTreeDir
              key={subDir}
              name={subDir}
              sourceId={sourceId}
              sourceRoot={sourceRoot}
              parentPath={`${parentPath}/${subDir}`}
              documents={documents.filter((d) => {
                const parts = d.relative_path.split(/[/\\]/);
                return parts[0] === name && parts.length > 2;
              })}
              onFileSelect={onFileSelect}
            />
          ))}

          {/* 文件 */}
          {dirFiles.map((doc) => (
            <FileTreeFile
              key={doc.id}
              doc={doc}
              onClick={() => onFileSelect(doc)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

interface FileTreeFileProps {
  doc: DocumentDto;
  onClick: () => void;
}

function FileTreeFile({ doc, onClick }: FileTreeFileProps) {
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
      onClick={onClick}
    >
      <span className="file-tree-icon">{getFileIcon(doc.kind)}</span>
      <span className="file-tree-name">{fileName}</span>
      {isSensitive && <span className="file-tree-sensitive-badge">🔒</span>}
    </div>
  );
}
