import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { MarkdownViewer } from "../components/MarkdownViewer";

describe("MarkdownViewer", () => {
  it("渲染基本 Markdown", () => {
    render(<MarkdownViewer content="# Hello World" />);
    expect(screen.getByText("Hello World")).toBeDefined();
  });

  it("渲染段落", () => {
    render(<MarkdownViewer content="这是一段文本" />);
    expect(screen.getByText("这是一段文本")).toBeDefined();
  });

  it("渲染列表", () => {
    const md = "- 项目一\n- 项目二\n- 项目三";
    render(<MarkdownViewer content={md} />);
    expect(screen.getByText("项目一")).toBeDefined();
    expect(screen.getByText("项目二")).toBeDefined();
    expect(screen.getByText("项目三")).toBeDefined();
  });

  it("渲染代码块", () => {
    const md = "```rust\nfn main() {}\n```";
    const { container } = render(<MarkdownViewer content={md} />);
    const code = container.querySelector("code");
    expect(code).toBeDefined();
    expect(code?.textContent).toContain("fn main() {}");
  });

  it("链接在新窗口打开", () => {
    const md = "[链接](https://example.com)";
    render(<MarkdownViewer content={md} />);
    const link = screen.getByText("链接");
    expect(link.getAttribute("target")).toBe("_blank");
    expect(link.getAttribute("rel")).toContain("noopener");
  });

  it("阻止不安全的 javascript: 链接", () => {
    // rehype-sanitize 会过滤 javascript: 协议，渲染不应崩溃
    const md = "[点击](javascript:alert(1))";
    const { container } = render(<MarkdownViewer content={md} />);
    // 验证没有 javascript: 链接存在
    const links = Array.from(container.querySelectorAll("a"));
    const hasUnsafeLink = links.some(
      (l) => l.getAttribute("href")?.includes("javascript:") ?? false,
    );
    expect(hasUnsafeLink).toBe(false);
  });

  it("渲染表格", () => {
    const md = "| 列1 | 列2 |\n| --- | --- |\n| 值1 | 值2 |";
    const { container } = render(<MarkdownViewer content={md} />);
    // 检查表格结构存在
    const table = container.querySelector("table");
    expect(table).toBeDefined();
  });
});
