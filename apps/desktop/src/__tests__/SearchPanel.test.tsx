import { render, screen, fireEvent } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { SearchPanel } from "../components/SearchPanel";
import { vi, describe, it, expect } from "vitest";

// 模拟 useSearch hook
vi.mock("../hooks/useSearch", () => ({
  useSearch: vi.fn(),
}));

import { useSearch } from "../hooks/useSearch";
const mockUseSearch = vi.mocked(useSearch);

function renderWithQuery(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>,
  );
}

describe("SearchPanel", () => {
  it("显示搜索输入框", () => {
    mockUseSearch.mockReturnValue({
      data: undefined,
      isLoading: false,
      error: null,
    } as ReturnType<typeof useSearch>);

    renderWithQuery(
      <SearchPanel workspaceId="ws-1" onResultClick={() => {}} />,
    );

    expect(
      screen.getByPlaceholderText("搜索文件名和内容..."),
    ).toBeDefined();
  });

  it("空查询时显示提示", () => {
    mockUseSearch.mockReturnValue({
      data: undefined,
      isLoading: false,
      error: null,
    } as ReturnType<typeof useSearch>);

    renderWithQuery(
      <SearchPanel workspaceId="ws-1" onResultClick={() => {}} />,
    );

    expect(screen.getByText("输入关键词搜索工作区中的文件")).toBeDefined();
  });

  it("显示搜索结果", () => {
    mockUseSearch.mockReturnValue({
      data: [
        {
          document_id: "doc-1",
          path: "src/main.rs",
          file_name: "main.rs",
          score: 1.5,
        },
      ],
      isLoading: false,
      error: null,
    } as ReturnType<typeof useSearch>);

    renderWithQuery(
      <SearchPanel workspaceId="ws-1" onResultClick={() => {}} />,
    );

    // 输入触发搜索
    const input = screen.getByPlaceholderText("搜索文件名和内容...");
    fireEvent.change(input, { target: { value: "main" } });

    // 检查结果是否渲染（由于 useSearch 是 mock 的，结果直接显示）
    expect(screen.getByText("main.rs")).toBeDefined();
    expect(screen.getByText("src/main.rs")).toBeDefined();
  });

  it("点击结果触发回调", () => {
    const handleClick = vi.fn();
    mockUseSearch.mockReturnValue({
      data: [
        {
          document_id: "doc-1",
          path: "src/main.rs",
          file_name: "main.rs",
          score: 1.5,
        },
      ],
      isLoading: false,
      error: null,
    } as ReturnType<typeof useSearch>);

    renderWithQuery(
      <SearchPanel workspaceId="ws-1" onResultClick={handleClick} />,
    );

    const resultButton = screen.getByText("main.rs").closest("button");
    expect(resultButton).toBeDefined();
    fireEvent.click(resultButton!);

    expect(handleClick).toHaveBeenCalledWith("doc-1");
  });

  it("搜索中显示加载状态", () => {
    mockUseSearch.mockReturnValue({
      data: undefined,
      isLoading: true,
      error: null,
    } as ReturnType<typeof useSearch>);

    renderWithQuery(
      <SearchPanel workspaceId="ws-1" onResultClick={() => {}} />,
    );

    // 输入触发搜索
    const input = screen.getByPlaceholderText("搜索文件名和内容...");
    fireEvent.change(input, { target: { value: "test" } });

    expect(screen.getByText("搜索中...")).toBeDefined();
  });

  it("无结果时显示提示", () => {
    mockUseSearch.mockReturnValue({
      data: [],
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useSearch>);

    renderWithQuery(
      <SearchPanel workspaceId="ws-1" onResultClick={() => {}} />,
    );

    const input = screen.getByPlaceholderText("搜索文件名和内容...");
    fireEvent.change(input, { target: { value: "nonexistent" } });

    expect(screen.getByText("未找到匹配结果")).toBeDefined();
  });
});
