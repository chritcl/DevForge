import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { commands } from "../bindings";

// 数据源相关 Hook

export function useSources(workspaceId: string) {
  return useQuery({
    queryKey: ["sources", workspaceId],
    queryFn: async () => {
      const result = await commands.listSources(workspaceId);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    enabled: !!workspaceId,
  });
}

export function useAddLocalSource() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (params: { workspace_id: string; path: string }) => {
      const result = await commands.addLocalSource(params.workspace_id, params.path);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["sources", variables.workspace_id] });
    },
  });
}

export function useRemoveSource() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (params: { id: string; workspace_id: string }) => {
      const result = await commands.removeSource(params.id);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["sources", variables.workspace_id] });
      // 移除数据源后，级联清理关联的文档、文件树和标签缓存
      queryClient.invalidateQueries({ queryKey: ["documents"] });
      queryClient.invalidateQueries({ queryKey: ["file-tree"] });
      queryClient.invalidateQueries({ queryKey: ["documents-by-ids"] });
      queryClient.invalidateQueries({ queryKey: ["tabs", variables.workspace_id] });
    },
  });
}

export function useScanSource() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (source_id: string) => {
      const result = await commands.scanSource(source_id);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["documents"] });
    },
  });
}
