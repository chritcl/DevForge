import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { Workspace } from "../types";

// 工作区相关 Hook

export function useWorkspaces() {
  return useQuery({
    queryKey: ["workspaces"],
    queryFn: () => invoke<Workspace[]>("list_workspaces"),
  });
}

export function useWorkspace(id: string) {
  return useQuery({
    queryKey: ["workspace", id],
    queryFn: () => invoke<Workspace>("get_workspace", { id }),
    enabled: !!id,
  });
}

export function useCreateWorkspace() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { name: string; description?: string }) =>
      invoke<Workspace>("create_workspace", params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
    },
  });
}

export function useUpdateWorkspace() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: {
      id: string;
      name?: string;
      description?: string | null;
    }) => invoke<Workspace>("update_workspace", params),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
      queryClient.invalidateQueries({ queryKey: ["workspace", variables.id] });
    },
  });
}

export function useDeleteWorkspace() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => invoke<void>("delete_workspace", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
    },
  });
}

export function useArchiveWorkspace() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => invoke<void>("archive_workspace", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
    },
  });
}

export function useMarkWorkspaceOpened() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => invoke<void>("mark_workspace_opened", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
    },
  });
}
