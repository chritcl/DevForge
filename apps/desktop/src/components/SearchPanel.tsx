import { useState, useCallback } from "react";
import { useSearch } from "../hooks/useSearch";
import type { SearchResultDto } from "../bindings";

interface SearchPanelProps {
  workspaceId: string;
  onResultClick: (documentId: string, lineNumber: number) => void;
}

export function SearchPanel({ workspaceId, onResultClick }: SearchPanelProps) {
  const [query, setQuery] = useState("");
  const { data: results, isLoading, error } = useSearch(workspaceId, query);

  const handleResultClick = useCallback(
    (result: SearchResultDto) => {
      onResultClick(result.document_id, result.line_number);
    },
    [onResultClick],
  );

  return (
    <div className="search-panel">
      <div className="search-panel-input-wrapper">
        <input
          type="text"
          className="search-panel-input"
          placeholder="搜索文件名和内容..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
      </div>

      <div className="search-panel-results">
        {isLoading && (
          <div className="search-panel-status">搜索中...</div>
        )}

        {error && (
          <div className="search-panel-error">
            搜索失败: {String(error)}
          </div>
        )}

        {!isLoading && !error && query.trim().length > 0 && results && results.length === 0 && (
          <div className="search-panel-status">未找到匹配结果</div>
        )}

        {results && results.length > 0 && (
          <div className="search-panel-result-list">
            {results.map((result) => (
              <button
                key={result.document_id}
                className="search-panel-result-item"
                onClick={() => handleResultClick(result)}
                title={`${result.path}:${result.line_number}`}
              >
                <div className="search-panel-result-header">
                  <span className="search-panel-result-name">
                    {result.file_name}
                  </span>
                  <span className="search-panel-result-line">
                    行 {result.line_number}
                  </span>
                </div>
                <div className="search-panel-result-path">
                  {result.path}
                </div>
                {result.snippet && (
                  <div
                    className="search-panel-result-snippet"
                    dangerouslySetInnerHTML={{ __html: result.snippet }}
                  />
                )}
              </button>
            ))}
          </div>
        )}

        {query.trim().length === 0 && (
          <div className="search-panel-hint">
            输入关键词搜索工作区中的文件
          </div>
        )}
      </div>
    </div>
  );
}
