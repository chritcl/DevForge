import { useQuery } from "@tanstack/react-query";
import { commands } from "../bindings";

/**
 * 工作区全文搜索 hook
 *
 * 空查询不执行搜索（enabled: false）。
 */
export function useSearch(workspaceId: string, query: string) {
  const trimmed = query.trim();
  return useQuery({
    queryKey: ["search", workspaceId, trimmed],
    queryFn: async () => {
      const result = await commands.searchWorkspace(workspaceId, trimmed);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    enabled: !!workspaceId && trimmed.length > 0,
  });
}
