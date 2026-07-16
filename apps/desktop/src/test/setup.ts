import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";

/**
 * 安装确定性的 window.matchMedia 实现
 *
 * useThemeSync 依赖 matchMedia 监听系统主题变化，
 * jsdom 不提供该 API，需要在测试环境中模拟。
 * 默认 matches = false，即模拟浅色系统主题。
 */
Object.defineProperty(window, "matchMedia", {
  writable: true,
  configurable: true,
  value: vi.fn((query: string) => {
    const listeners = new Set<(ev: MediaQueryListEvent) => void>();

    const mql: MediaQueryList = {
      matches: false,
      media: query,
      onchange: null,
      addListener: (callback: (ev: MediaQueryListEvent) => void) => {
        listeners.add(callback);
      },
      removeListener: (callback: (ev: MediaQueryListEvent) => void) => {
        listeners.delete(callback);
      },
      addEventListener: (
        _type: string,
        callback: (ev: Event) => void,
      ) => {
        listeners.add(
          callback as (ev: MediaQueryListEvent) => void,
        );
      },
      removeEventListener: (
        _type: string,
        callback: (ev: Event) => void,
      ) => {
        listeners.delete(
          callback as (ev: MediaQueryListEvent) => void,
        );
      },
      dispatchEvent: (event: Event) => {
        listeners.forEach((cb) =>
          cb(event as MediaQueryListEvent),
        );
        return true;
      },
    };

    return mql;
  }),
});

afterEach(() => {
  cleanup();
  localStorage.clear();
  document.documentElement.removeAttribute("data-theme");
  document.documentElement.style.colorScheme = "";
});
