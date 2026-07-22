import { useDocumentContent } from "../hooks/useDocuments";
import { CodeViewer } from "./CodeViewer";
import { MarkdownViewer } from "./MarkdownViewer";
import type { DocumentDto } from "../bindings";

interface FileViewerProps {
  document: DocumentDto;
}

export function FileViewer({ document }: FileViewerProps) {
  const {
    data: content,
    isLoading,
    error,
  } = useDocumentContent(document.id, document);

  // 敏感文件（UI 判断，后端仍会拒绝）
  if (document.sensitivity === "sensitive") {
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

  // 二进制文件
  if (document.kind === "binary") {
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

  // 不可读文件
  if (!document.content_readable) {
    return (
      <div className="file-viewer-binary">
        <div className="file-viewer-binary-icon">⚙️</div>
        <div className="file-viewer-binary-title">不可读文件</div>
        <div className="file-viewer-binary-message">
          此文件无法以文本形式显示。
        </div>
      </div>
    );
  }

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

  if (content === undefined || content === null) {
    return (
      <div className="file-viewer-error">
        <span>文件内容为空</span>
      </div>
    );
  }

  // Markdown 渲染（安全的 react-markdown + rehype-sanitize）
  if (document.kind === "markdown") {
    return (
      <div className="file-viewer-markdown">
        <div className="file-viewer-markdown-header">
          <span className="file-viewer-icon">📝</span>
          <span className="file-viewer-name">
            {document.relative_path.split(/[/\\]/).pop()}
          </span>
        </div>
        <MarkdownViewer content={content} />
      </div>
    );
  }

  // 代码和文本文件（Monaco Editor 只读查看器）
  return (
    <div className="file-viewer-text">
      <div className="file-viewer-text-header">
        <span className="file-viewer-icon">📄</span>
        <span className="file-viewer-name">
          {document.relative_path.split(/[/\\]/).pop()}
        </span>
      </div>
      <CodeViewer filePath={document.relative_path} content={content} />
    </div>
  );
}
