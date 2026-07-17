import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { DocumentDto } from "../types";

// 文档相关 Hook

export function useDocuments(sourceId: string, parentPath?: string) {
  return useQuery({
    queryKey: ["documents", sourceId, parentPath],
    queryFn: () =>
      invoke<DocumentDto[]>("list_documents", {
        source_id: sourceId,
        parent_path: parentPath ?? null,
      }),
    enabled: !!sourceId,
  });
}

export function useDocumentContent(documentId: string, sourceRoot: string) {
  return useQuery({
    queryKey: ["document-content", documentId],
    queryFn: () =>
      invoke<string>("read_document_content", {
        document_id: documentId,
        source_root: sourceRoot,
      }),
    enabled: !!documentId && !!sourceRoot,
  });
}
