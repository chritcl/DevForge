import { render, screen } from "@testing-library/react";
import {
  QueryClient,
  QueryClientProvider,
} from "@tanstack/react-query";
import { createMemoryRouter, RouterProvider } from "react-router";
import { describe, it, expect, vi, afterEach } from "vitest";

import App from "../App";
import { HomePage } from "../pages/HomePage";
import { NotFoundPage } from "../pages/NotFoundPage";
import { SettingsPage } from "../pages/SettingsPage";

vi.mock("../bindings", () => ({
  commands: {
    getAppInfo: vi.fn(),
  },
}));

import { commands } from "../bindings";
import type { AppInfo } from "../bindings";

const SUCCESS_DATA: AppInfo = {
  version: "0.1.0",
  data_dir: "C:\\Users\\test\\AppData\\Local\\DevForge",
  db_status: {
    type: "Ready",
    migration_version: 1,
  },
};

function renderRoute(initialEntry: string) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  });

  const router = createMemoryRouter(
    [
      {
        path: "/",
        Component: App,
        children: [
          {
            index: true,
            Component: HomePage,
          },
          {
            path: "settings",
            Component: SettingsPage,
          },
          {
            path: "*",
            Component: NotFoundPage,
          },
        ],
      },
    ],
    {
      initialEntries: [initialEntry],
    },
  );

  const result = render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );

  return {
    ...result,
    queryClient,
    router,
  };
}

afterEach(() => {
  vi.mocked(commands.getAppInfo).mockReset();
});

describe("AppRouting", () => {
  it("设置页显示三个主题选项", async () => {
    renderRoute("/settings");

    expect(
      await screen.findByRole("heading", { name: "设置" }),
    ).toBeInTheDocument();
    expect(screen.getByText("浅色")).toBeInTheDocument();
    expect(screen.getByText("深色")).toBeInTheDocument();
    expect(screen.getByText("跟随系统")).toBeInTheDocument();

    // 设置页不应调用 getAppInfo
    expect(commands.getAppInfo).not.toHaveBeenCalled();
  });

  it("未知路径显示 404 页面", async () => {
    renderRoute("/missing");

    expect(
      await screen.findByText("页面不存在"),
    ).toBeInTheDocument();
    expect(screen.getByText("返回首页")).toBeInTheDocument();

    // 404 页不应调用 getAppInfo
    expect(commands.getAppInfo).not.toHaveBeenCalled();
  });

  it("首页通过 Router 上下文渲染并显示 AppInfo", async () => {
    vi.mocked(commands.getAppInfo).mockResolvedValueOnce(
      SUCCESS_DATA,
    );

    renderRoute("/");

    // 首页调用了 getAppInfo
    expect(commands.getAppInfo).toHaveBeenCalled();

    // 最终显示成功状态
    expect(await screen.findByText("DevForge")).toBeInTheDocument();
    expect(screen.getByText("0.1.0")).toBeInTheDocument();
    expect(
      screen.getByText("就绪（migration v1）"),
    ).toBeInTheDocument();
  });
});
