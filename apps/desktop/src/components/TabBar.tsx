import type { TabDto } from "../hooks/useTabs";
import type { DocumentDto } from "../types";

interface TabBarProps {
  tabs: TabDto[];
  documents: Map<string, DocumentDto>;
  activeTabId: string | null;
  onTabClick: (tabId: string) => void;
  onTabClose: (tabId: string) => void;
}

export function TabBar({
  tabs,
  documents,
  activeTabId,
  onTabClick,
  onTabClose,
}: TabBarProps) {
  if (tabs.length === 0) {
    return null;
  }

  return (
    <div className="tab-bar">
      {tabs.map((tab) => {
        const doc = documents.get(tab.document_id);
        const fileName = doc
          ? doc.relative_path.split(/[/\\]/).pop() ?? doc.relative_path
          : "未知文件";
        const isActive = tab.id === activeTabId;
        const isSensitive = doc?.sensitivity === "Sensitive";

        return (
          <div
            key={tab.id}
            className={`tab-item ${isActive ? "tab-item-active" : ""}`}
            onClick={() => onTabClick(tab.id)}
          >
            <span className="tab-item-icon">
              {getFileIcon(doc?.kind ?? "unknown")}
            </span>
            <span className="tab-item-name">
              {fileName}
              {isSensitive && " 🔒"}
            </span>
            <button
              className="tab-item-close"
              onClick={(e) => {
                e.stopPropagation();
                onTabClose(tab.id);
              }}
            >
              ×
            </button>
          </div>
        );
      })}
    </div>
  );
}

function getFileIcon(kind: string): string {
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
}
