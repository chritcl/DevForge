import { useQuery } from "@tanstack/react-query";

import { commands } from "../bindings";
import { appKeys } from "../queryKeys";

export function useAppInfo() {
  return useQuery({
    queryKey: appKeys.info(),
    queryFn: commands.getAppInfo,
    // 本地 Tauri IPC，不依赖互联网
    networkMode: "always",
    // 配置或 IPC 错误不应盲目重试
    retry: false,
    staleTime: 30_000,
  });
}
