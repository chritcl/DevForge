import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { DocumentDto, DocumentLookupDto, FileTreeEntryDto } from "../bindings";

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
    queryFn: () =>
      invoke<DocumentDto[]>("list_documents", {
        source_id: sourceId,
        parent_path: parentPath ?? null,
      }),
    enabled: !!sourceId,
  });
}

export function useFileTree(sourceId: string, parentPath?: string) {
  return useQuery({
    queryKey: ["file-tree", sourceId, parentPath],
    queryFn: () =>
      invoke<FileTreeEntryDto[]>("list_file_tree", {
        source_id: sourceId,
        parent_path: parentPath ?? null,
      }),
    enabled: !!sourceId,
  });
}

export function useDocumentContent(documentId: string, document: DocumentDto | null | undefined) {
  const canRead = canReadDocument(document);

  return useQuery({
    queryKey: ["document-content", documentId],
    queryFn: () =>
      invoke<string>("read_document_content", {
        document_id: documentId,
      }),
    enabled: canRead,
  });
}

export function useDocumentsByIds(documentIds: string[]) {
  return useQuery({
    queryKey: ["documents-by-ids", ...documentIds.sort()],
    queryFn: () =>
      invoke<DocumentLookupDto[]>("get_documents_by_ids", {
        document_ids: documentIds,
      }),
    enabled: documentIds.length > 0,
  });
}
