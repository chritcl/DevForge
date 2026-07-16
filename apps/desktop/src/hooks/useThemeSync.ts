import { useLayoutEffect } from "react";

import { useUIStore } from "../stores/ui";

type ResolvedTheme = "light" | "dark";

export function useThemeSync(): void {
  const theme = useUIStore((state) => state.theme);

  useLayoutEffect(() => {
    const mediaQuery = window.matchMedia(
      "(prefers-color-scheme: dark)",
    );

    const applyTheme = (): void => {
      const resolvedTheme: ResolvedTheme =
        theme === "system"
          ? mediaQuery.matches
            ? "dark"
            : "light"
          : theme;

      document.documentElement.dataset.theme = resolvedTheme;
      document.documentElement.style.colorScheme = resolvedTheme;
    };

    applyTheme();

    if (theme !== "system") {
      return undefined;
    }

    mediaQuery.addEventListener("change", applyTheme);

    return () => {
      mediaQuery.removeEventListener("change", applyTheme);
    };
  }, [theme]);
}
