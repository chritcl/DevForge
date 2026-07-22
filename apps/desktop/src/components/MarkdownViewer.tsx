import Markdown from "react-markdown";
import rehypeSanitize, { defaultSchema } from "rehype-sanitize";

interface MarkdownViewerProps {
  /** Markdown 原始文本 */
  content: string;
}

/**
 * 自定义安全 schema，在默认 schema 基础上收紧协议白名单。
 *
 * 默认 schema 已阻止 <script>、<iframe>、on* 事件等。
 * 此处额外限制：
 * - img/link 的 href/src 只允许 http:、https: 和相对路径
 * - 禁止 data:、file:、javascript: 等协议
 */
const secureSchema = {
  ...defaultSchema,
  protocols: {
    ...defaultSchema.protocols,
    href: ["http", "https"],
    src: ["http", "https"],
  },
};

/** 协议白名单正则：只允许 http:、https: 和相对路径 */
const SAFE_URL_PATTERN = /^(https?:\/\/|\/|\.\/|\.\.\/|#)/i;

/** 检查 URL 是否安全（只允许 http/https 和相对路径） */
function isSafeUrl(url: string | undefined): boolean {
  if (!url) return true;
  // 空字符串、锚点、相对路径都安全
  if (url.startsWith("#") || url.startsWith("/") || url.startsWith("./") || url.startsWith("../")) {
    return true;
  }
  return SAFE_URL_PATTERN.test(url);
}

/**
 * 安全的 Markdown 查看器
 *
 * 使用 react-markdown 渲染 Markdown，rehype-sanitize 过滤危险 HTML。
 * 安全措施：
 * - 自定义 schema 限制协议白名单（只允许 http/https 和相对路径）
 * - 组件层过滤 img/link 的 URL（defense-in-depth）
 * - 禁止：<script>、<iframe>、事件处理器、javascript:/data:/file: 协议等
 */
export function MarkdownViewer({ content }: MarkdownViewerProps) {
  return (
    <div className="file-viewer-markdown-content">
      <Markdown
        rehypePlugins={[[rehypeSanitize, secureSchema]]}
        components={{
          // 链接：新窗口打开 + URL 安全检查
          a: ({ href, children, ...props }) => {
            if (!isSafeUrl(href)) {
              return <span>{children}</span>;
            }
            return (
              <a {...props} href={href} target="_blank" rel="noopener noreferrer">
                {children}
              </a>
            );
          },
          // 图片：URL 安全检查，阻止 file:/data:/javascript: 等协议
          img: ({ src, alt, ...props }) => {
            if (!isSafeUrl(src)) {
              return <span>[不安全的图片: {alt ?? "未知"}]</span>;
            }
            return <img {...props} src={src} alt={alt} />;
          },
        }}
      >
        {content}
      </Markdown>
    </div>
  );
}
