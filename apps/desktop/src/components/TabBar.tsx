import type { TabDto, DocumentLookupDto } from "../bindings";

interface TabBarProps {
  tabs: TabDto[];
  documentLookups: Map<string, DocumentLookupDto>;
  activeTabId: string | null;
  onTabClick: (tabId: string) => void;
  onTabClose: (tabId: string) => void;
}

export function TabBar({
  tabs,
  documentLookups,
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
        const lookup = documentLookups.get(tab.document_id);
        const doc = lookup?.status === "found" ? lookup.document : null;
        const isAvailable = lookup?.status === "found";
        const fileName = doc
          ? doc.relative_path.split(/[/\\]/).pop() ?? doc.relative_path
          : isAvailable
            ? "未知文件"
            : "文件不可用";
        const isActive = tab.id === activeTabId;
        const isSensitive = doc?.sensitivity === "sensitive";

        return (
          <div
            key={tab.id}
            className={`tab-item ${isActive ? "tab-item-active" : ""} ${!isAvailable ? "tab-item-unavailable" : ""}`}
            onClick={() => onTabClick(tab.id)}
          >
            <span className="tab-item-icon">
              {isAvailable ? getFileIcon(doc?.kind ?? "unknown") : "❓"}
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
