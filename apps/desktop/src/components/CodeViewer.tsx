import { useMemo, useState, useEffect } from "react";
import Editor from "@monaco-editor/react";

/** 大文件阈值（字符数），超过此值显示警告而非直接渲染 */
const LARGE_FILE_THRESHOLD = 500_000;

/** 文件扩展名到 Monaco 语言 ID 的映射 */
const EXTENSION_TO_LANGUAGE: Record<string, string> = {
  // Web
  ".js": "javascript",
  ".jsx": "javascript",
  ".mjs": "javascript",
  ".cjs": "javascript",
  ".ts": "typescript",
  ".tsx": "typescript",
  ".mts": "typescript",
  ".cts": "typescript",
  ".html": "html",
  ".htm": "html",
  ".css": "css",
  ".scss": "scss",
  ".sass": "scss",
  ".less": "less",
  ".json": "json",
  ".jsonc": "json",
  ".json5": "json5",
  ".xml": "xml",
  ".svg": "xml",
  // Rust
  ".rs": "rust",
  // Python
  ".py": "python",
  ".pyi": "python",
  // Go
  ".go": "go",
  // Java
  ".java": "java",
  // C/C++
  ".c": "c",
  ".h": "c",
  ".cpp": "cpp",
  ".cxx": "cpp",
  ".cc": "cpp",
  ".hpp": "cpp",
  ".hxx": "cpp",
  // Shell
  ".sh": "shell",
  ".bash": "shell",
  ".zsh": "shell",
  ".ps1": "powershell",
  ".psm1": "powershell",
  ".psd1": "powershell",
  // 配置
  ".yaml": "yaml",
  ".yml": "yaml",
  ".toml": "ini",
  ".ini": "ini",
  ".env": "ini",
  ".properties": "ini",
  // 文档
  ".md": "markdown",
  ".mdx": "markdown",
  ".rst": "plaintext",
  // 数据
  ".sql": "sql",
  ".graphql": "graphql",
  ".gql": "graphql",
  // 其他
  ".rb": "ruby",
  ".php": "php",
  ".swift": "swift",
  ".kt": "kotlin",
  ".kts": "kotlin",
  ".scala": "scala",
  ".dart": "dart",
  ".lua": "lua",
  ".r": "r",
  ".R": "r",
  ".vue": "html",
  ".svelte": "html",
  ".astro": "html",
  ".dockerfile": "dockerfile",
  ".makefile": "makefile",
  ".gradle": "groovy",
  ".groovy": "groovy",
};

/** 根据文件路径推断 Monaco 语言 ID */
function detectLanguage(filePath: string): string {
  const name = filePath.split(/[/\\]/).pop() ?? filePath;
  const lower = name.toLowerCase();

  // 特殊文件名
  if (lower === "dockerfile") return "dockerfile";
  if (lower === "makefile" || lower === "gnumakefile") return "makefile";
  if (lower === "cmakelists.txt") return "cmake";
  if (lower === "cargo.toml" || lower === "rust-toolchain.toml") return "ini";
  if (lower === "package.json") return "json";
  if (lower === "tsconfig.json") return "jsonc";

  // 按扩展名匹配
  const dotIndex = name.lastIndexOf(".");
  if (dotIndex === -1) return "plaintext";

  const ext = name.slice(dotIndex);
  return EXTENSION_TO_LANGUAGE[ext] ?? "plaintext";
}

interface CodeViewerProps {
  /** 文件路径（用于推断语言） */
  filePath: string;
  /** 文件内容 */
  content: string;
}

/**
 * 只读代码查看器
 *
 * 使用 Monaco Editor 提供语法高亮和只读查看。
 * 大文件（超过 500k 字符）显示警告，用户确认后才渲染。
 */
export function CodeViewer({ filePath, content }: CodeViewerProps) {
  const language = useMemo(() => detectLanguage(filePath), [filePath]);
  const isLarge = content.length > LARGE_FILE_THRESHOLD;
  const [forceRender, setForceRender] = useState(false);

  // 监听主题变化
  const [theme, setTheme] = useState<"vs" | "vs-dark">(
    document.documentElement.dataset.theme === "dark" ? "vs-dark" : "vs",
  );

  useEffect(() => {
    const observer = new MutationObserver(() => {
      setTheme(
        document.documentElement.dataset.theme === "dark" ? "vs-dark" : "vs",
      );
    });
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    return () => observer.disconnect();
  }, []);

  if (isLarge && !forceRender) {
    return (
      <div className="file-viewer-large-warning">
        <div className="file-viewer-large-icon">📄</div>
        <div className="file-viewer-large-title">文件较大</div>
        <div className="file-viewer-large-message">
          此文件包含 {content.length.toLocaleString()} 个字符，可能影响性能。
        </div>
        <button
          className="btn btn-secondary"
          onClick={() => setForceRender(true)}
        >
          仍然查看
        </button>
      </div>
    );
  }

  return (
    <div className="file-viewer-code">
      <Editor
        height="100%"
        language={language}
        value={content}
        theme={theme}
        options={{
          readOnly: true,
          domReadOnly: true,
          fontSize: 14,
          lineNumbers: "on",
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          wordWrap: "on",
          automaticLayout: true,
          renderWhitespace: "selection",
          bracketPairColorization: { enabled: true },
          guides: { bracketPairs: true },
          folding: true,
          contextmenu: false,
          // 禁用编辑相关功能
          quickSuggestions: false,
          suggestOnTriggerCharacters: false,
          parameterHints: { enabled: false },
          hover: { enabled: "on" as const },
        }}
      />
    </div>
  );
}
