import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { commands } from "../bindings";

// 工作区相关 Hook

export function useWorkspaces() {
  return useQuery({
    queryKey: ["workspaces", "active"],
    queryFn: async () => {
      const result = await commands.listWorkspaces();
      if (result.status === "error") throw result.error;
      return result.data;
    },
  });
}

export function useArchivedWorkspaces() {
  return useQuery({
    queryKey: ["workspaces", "archived"],
    queryFn: async () => {
      const result = await commands.listArchivedWorkspaces();
      if (result.status === "error") throw result.error;
      return result.data;
    },
  });
}

export function useWorkspace(id: string) {
  return useQuery({
    queryKey: ["workspace", id],
    queryFn: async () => {
      const result = await commands.getWorkspace(id);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    enabled: !!id,
  });
}

export function useCreateWorkspace() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (params: { name: string; description?: string | null }) => {
      const result = await commands.createWorkspace(params.name, params.description ?? null);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["workspaces", "active"] });
    },
  });
}

export function useUpdateWorkspace() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (params: { id: string; name: string; description?: string | null }) => {
      const result = await commands.updateWorkspace(params.id, params.name, params.description ?? null);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
      queryClient.invalidateQueries({ queryKey: ["workspace", variables.id] });
    },
  });
}

export function useDeleteWorkspace() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      const result = await commands.deleteWorkspace(id);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
    },
  });
}

export function useArchiveWorkspace() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      const result = await commands.archiveWorkspace(id);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
      queryClient.invalidateQueries({ queryKey: ["workspace", id] });
    },
  });
}

export function useRestoreWorkspace() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      const result = await commands.restoreWorkspace(id);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
      queryClient.invalidateQueries({ queryKey: ["workspace", id] });
    },
  });
}

export function useMarkWorkspaceOpened() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      const result = await commands.markWorkspaceOpened(id);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["workspaces"] });
    },
  });
}
