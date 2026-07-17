import { useDocumentContent } from "../hooks/useDocuments";
import type { DocumentDto } from "../types";

interface FileViewerProps {
  document: DocumentDto;
  sourceRoot: string;
}

export function FileViewer({ document, sourceRoot }: FileViewerProps) {
  const {
    data: content,
    isLoading,
    error,
  } = useDocumentContent(document.id, sourceRoot);

  if (isLoading) {
    return (
      <div className="file-viewer-loading">
        <span>加载中...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="file-viewer-error">
        <span>加载失败: {String(error)}</span>
      </div>
    );
  }

  // 敏感文件
  if (document.sensitivity === "Sensitive") {
    return (
      <div className="file-viewer-sensitive">
        <div className="file-viewer-sensitive-icon">🔒</div>
        <div className="file-viewer-sensitive-title">敏感文件</div>
        <div className="file-viewer-sensitive-message">
          此文件包含敏感内容（如密钥、密码等），默认不显示内容。
        </div>
      </div>
    );
  }

  // 不可读文件
  if (!document.content_readable) {
    return (
      <div className="file-viewer-binary">
        <div className="file-viewer-binary-icon">⚙️</div>
        <div className="file-viewer-binary-title">二进制文件</div>
        <div className="file-viewer-binary-message">
          此文件是二进制文件，无法以文本形式显示。
        </div>
      </div>
    );
  }

  // Markdown 渲染
  if (document.kind === "markdown") {
    return (
      <div className="file-viewer-markdown">
        <div className="file-viewer-markdown-header">
          <span className="file-viewer-icon">📝</span>
          <span className="file-viewer-name">
            {document.relative_path.split(/[/\\]/).pop()}
          </span>
        </div>
        <div className="file-viewer-markdown-content">
          <pre>{content}</pre>
        </div>
      </div>
    );
  }

  // 文本文件
  return (
    <div className="file-viewer-text">
      <div className="file-viewer-text-header">
        <span className="file-viewer-icon">📄</span>
        <span className="file-viewer-name">
          {document.relative_path.split(/[/\\]/).pop()}
        </span>
      </div>
      <div className="file-viewer-text-content">
        <pre>{content}</pre>
      </div>
    </div>
  );
}
