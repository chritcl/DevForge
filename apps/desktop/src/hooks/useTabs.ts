import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { commands } from "../bindings";

// 标签页相关 Hook

export function useTabs(workspaceId: string) {
  return useQuery({
    queryKey: ["tabs", workspaceId],
    queryFn: async () => {
      const result = await commands.listTabs(workspaceId);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    enabled: !!workspaceId,
  });
}

export function useOpenTab() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (params: { workspace_id: string; document_id: string }) => {
      const result = await commands.openTab(params.workspace_id, params.document_id);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["tabs", variables.workspace_id] });
    },
  });
}

export function useCloseTab() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (params: { id: string; workspace_id: string }) => {
      const result = await commands.closeTab(params.id);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["tabs", variables.workspace_id] });
    },
  });
}

export function useSetActiveTab() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (params: { workspace_id: string; tab_id: string }) => {
      const result = await commands.setActiveTab(params.workspace_id, params.tab_id);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["tabs", variables.workspace_id] });
    },
  });
}
