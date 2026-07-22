import { useQuery } from "@tanstack/react-query";
import { commands } from "../bindings";
import type { DocumentDto } from "../bindings";

// 文档相关 Hook

/// 判断文档是否可读取正文
///
/// 仅用于减少无效 IPC 请求，Rust 后端仍执行最终安全拒绝。
function canReadDocument(document: DocumentDto | null | undefined): boolean {
  if (!document) return false;
  return (
    document.sensitivity === "normal" &&
    document.content_readable &&
    document.kind !== "binary"
  );
}

export function useDocuments(sourceId: string, parentPath?: string) {
  return useQuery({
    queryKey: ["documents", sourceId, parentPath],
    queryFn: async () => {
      const result = await commands.listDocuments(sourceId, parentPath ?? null);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    enabled: !!sourceId,
  });
}

export function useFileTree(sourceId: string, parentPath?: string) {
  return useQuery({
    queryKey: ["file-tree", sourceId, parentPath],
    queryFn: async () => {
      const result = await commands.listFileTree(sourceId, parentPath ?? null);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    enabled: !!sourceId,
  });
}

export function useDocumentContent(documentId: string, document: DocumentDto | null | undefined) {
  const canRead = canReadDocument(document);

  return useQuery({
    queryKey: ["document-content", documentId],
    queryFn: async () => {
      const result = await commands.readDocumentContent(documentId);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    enabled: canRead,
  });
}

export function useDocumentsByIds(documentIds: string[]) {
  return useQuery({
    queryKey: ["documents-by-ids", ...documentIds.sort()],
    queryFn: async () => {
      const result = await commands.getDocumentsByIds(documentIds);
      if (result.status === "error") throw result.error;
      return result.data;
    },
    enabled: documentIds.length > 0,
  });
}
