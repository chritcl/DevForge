import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { Source, ScanResult } from "../types";

// 数据源相关 Hook

export function useSources(workspaceId: string) {
  return useQuery({
    queryKey: ["sources", workspaceId],
    queryFn: () => invoke<Source[]>("list_sources", { workspace_id: workspaceId }),
    enabled: !!workspaceId,
  });
}

export function useAddGitSource() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { workspace_id: string; path: string }) =>
      invoke<Source>("add_git_source", params),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["sources", variables.workspace_id] });
    },
  });
}

export function useAddDirectorySource() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { workspace_id: string; path: string }) =>
      invoke<Source>("add_directory_source", params),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["sources", variables.workspace_id] });
    },
  });
}

export function useRemoveSource() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { id: string; workspace_id: string }) =>
      invoke<void>("remove_source", { id: params.id }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["sources", variables.workspace_id] });
    },
  });
}

export function useScanSource() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { source_id: string; root_path: string }) =>
      invoke<ScanResult>("scan_source", params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["documents"] });
    },
  });
}
