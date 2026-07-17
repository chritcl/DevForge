import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

// 标签页类型
export type TabDto = {
  id: string;
  workspace_id: string;
  document_id: string;
  position: number;
  is_active: boolean;
  opened_at: string;
};

// 标签页相关 Hook

export function useTabs(workspaceId: string) {
  return useQuery({
    queryKey: ["tabs", workspaceId],
    queryFn: () => invoke<TabDto[]>("list_tabs", { workspace_id: workspaceId }),
    enabled: !!workspaceId,
  });
}

export function useOpenTab() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { workspace_id: string; document_id: string }) =>
      invoke<TabDto>("open_tab", params),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["tabs", variables.workspace_id] });
    },
  });
}

export function useCloseTab() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { id: string; workspace_id: string }) =>
      invoke<void>("close_tab", { id: params.id }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["tabs", variables.workspace_id] });
    },
  });
}

export function useSetActiveTab() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { workspace_id: string; tab_id: string }) =>
      invoke<void>("set_active_tab", params),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["tabs", variables.workspace_id] });
    },
  });
}
