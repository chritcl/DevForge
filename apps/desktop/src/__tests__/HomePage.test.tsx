import { render, screen, fireEvent } from "@testing-library/react";
import {
  QueryClient,
  QueryClientProvider,
} from "@tanstack/react-query";
import { describe, it, expect, vi } from "vitest";

import { commands } from "../bindings";
import type { AppInfo } from "../bindings";
import { HomePage } from "../pages/HomePage";

vi.mock("../bindings", () => ({
  commands: {
    getAppInfo: vi.fn(),
  },
}));

const SUCCESS_DATA: AppInfo = {
  version: "0.1.0",
  data_dir: "C:\\Users\\test\\AppData\\Local\\DevForge",
  db_status: {
    type: "Ready",
    migration_version: 1,
  },
};

function renderHomePage() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  });

  const result = render(
    <QueryClientProvider client={queryClient}>
      <HomePage />
    </QueryClientProvider>,
  );

  return {
    ...result,
    queryClient,
  };
}

describe("HomePage", () => {
  it("加载并显示真实状态", async () => {
    let resolvePromise!: (value: AppInfo) => void;
    const pendingPromise = new Promise<AppInfo>((resolve) => {
      resolvePromise = resolve;
    });

    vi.mocked(commands.getAppInfo).mockReturnValueOnce(
      pendingPromise as ReturnType<typeof commands.getAppInfo>,
    );

    renderHomePage();

    // 初始显示加载中
    expect(screen.getByText("加载中...")).toBeInTheDocument();

    // 解析 Promise
    resolvePromise(SUCCESS_DATA);

    // 等待成功状态
    expect(await screen.findByText("DevForge")).toBeInTheDocument();
    expect(screen.getByText("0.1.0")).toBeInTheDocument();
    expect(
      screen.getByText(
        "C:\\Users\\test\\AppData\\Local\\DevForge",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText("就绪（migration v1）"),
    ).toBeInTheDocument();

    expect(commands.getAppInfo).toHaveBeenCalledTimes(1);
  });

  it("错误和重试", async () => {
    const mockFn = vi.mocked(commands.getAppInfo);

    // 第一次调用失败
    mockFn.mockRejectedValueOnce(new Error("连接失败"));

    renderHomePage();

    // 等待错误状态
    const alert = await screen.findByRole("alert");
    expect(alert).toBeInTheDocument();
    expect(
      screen.getByText(/加载应用信息失败/),
    ).toBeInTheDocument();

    // 第二次调用成功
    mockFn.mockResolvedValueOnce(SUCCESS_DATA);

    // 点击重试
    const retryButton = screen.getByText("重试");
    fireEvent.click(retryButton);

    // 等待成功状态
    expect(await screen.findByText("DevForge")).toBeInTheDocument();
    expect(screen.getByText("0.1.0")).toBeInTheDocument();
    expect(
      screen.getByText("就绪（migration v1）"),
    ).toBeInTheDocument();

    // 总调用次数为 2
    expect(mockFn).toHaveBeenCalledTimes(2);
  });
});
