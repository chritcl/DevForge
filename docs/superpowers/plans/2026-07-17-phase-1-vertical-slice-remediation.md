# Phase 1 Vertical Slice Remediation Plan


## Goal

Fix all blocking defects in Phase 1 so users can complete the full end-to-end loop: create workspace -> add source -> scan files -> lazy-load file tree -> open code/Markdown/text -> open/switch/close tabs -> close app -> restart restore.

## Non-goals

- No Tantivy full-text index
- No Tree-sitter code parsing
- No AI Q&A
- No Git operations (clone/pull/push)
- No remote repository connectors
- No vector index
- No Code Graph
- No Agent modifications
- No plugin system
- No unrelated refactoring
- No early implementation of later phases

## Global Constraints

- All code comments in Chinese
- All text files UTF-8 without BOM
- Rust formatting: cargo fmt
- Rust lint: cargo clippy --workspace --all-targets -- -D warnings
- Frontend typecheck: pnpm typecheck
- Frontend test: pnpm test
- Domain code must not depend on Tauri, React, SQLite
- Application Services depend on Traits, not concrete implementations
- Tauri Commands expose use cases, not CRUD
- Frontend must not call invoke() directly, must use Specta-generated bindings

---

## Current Problem Evidence

### P0 - Data Corruption

#### Bug 1: modified_at calculation logic error

File: crates/devforge-application/src/discovery.rs:195-201

The code does Utc::now() - chrono::Duration::from_std(duration) which is wrong. The duration from t.duration_since(UNIX_EPOCH) represents time since epoch, not a recent offset. For a file modified in 2024, duration is ~54 years, and now - 54 years yields 1972. Correct approach: use chrono::DateTime::from_timestamp(secs, nanos) directly.

#### Bug 2: removed count does not update database

File: crates/devforge-application/src/discovery.rs:227-234

The comment says mark as unreadable but the code only increments a counter without any database operation. Deleted files remain in the documents table forever.

### P1 - Main Flow Unusable

#### Bug 3: Specta bindings incomplete

File: apps/desktop/src-tauri/src/lib.rs:16

collect_commands! only includes get_app_info. The generated bindings.ts only exports getAppInfo. The other 17 commands have no Specta-generated type bindings. All frontend hooks use raw invoke() calls.

#### Bug 4: No Add Source UI

File: apps/desktop/src/pages/WorkspacePage.tsx

The page shows no sources message but has no button or dialog to add sources. Hooks useAddGitSource, useAddDirectorySource, useScanSource are ready but no UI calls them.

#### Bug 5: File tree not lazy-loading

File: apps/desktop/src/components/FileTree.tsx:19

useDocuments(sourceId) calls list_documents without parent_path, returning ALL documents. For large projects this causes full table scan, large IPC transfer, and frontend memory bloat.

#### Bug 6: No startup restore logic

File: apps/desktop/src/pages/WorkspacePage.tsx

After app restart, entering a workspace does not query open_tabs to restore previous tab state. The useTabs hook exists but is not used.

#### Bug 7: Tab active status switching is a no-op

File: crates/devforge-application/src/tab.rs:98-104

The for loop body is empty. After opening a new tab, old tabs are never deactivated. OpenTab::execute calls create but never calls set_active afterwards.

#### Bug 8: WorkspacePage has no tab bar

File: apps/desktop/src/pages/WorkspacePage.tsx

The page only has sidebar file tree + single file viewer, no tab bar. Cannot open multiple files, switch tabs, or close tabs.

#### Bug 9: WorkspacePage does not record last_opened_at

Entering a workspace does not call mark_opened() to update last_opened_at, making recent workspace sorting inaccurate.

### P2 - Testing and Maintainability

#### Bug 10: Scan has no transaction protection

File: crates/devforge-application/src/discovery.rs:149-242

ScanSource::execute makes multiple document_repo calls without transaction protection. If interrupted midway, the database is left in an inconsistent state.

#### Bug 11: list_documents backend filter logic is flawed

File: crates/devforge-application/src/document.rs:70-103

When parent_path is None, the code returns ALL documents instead of only root-level files and first-level directories.

---

## File Ownership

| Crate / Directory | Owner | Responsibility |
|---|---|---|
| crates/devforge-domain/ | domain | Domain models, no external deps |
| crates/devforge-application/ | application | Use cases, depends on domain |
| crates/devforge-storage/ | storage | SQLite implementation |
| apps/desktop/src-tauri/ | desktop | Tauri commands and state |
| apps/desktop/src/ | frontend | React frontend |

---

## Task 1: Fix Specta Binding Generation

**Goal**: Register all Tauri commands in Specta collect_commands!, regenerate bindings.ts, make frontend use type-safe Specta bindings instead of raw invoke().

**Prerequisites**: None

**Files to modify**:- apps/desktop/src-tauri/src/lib.rs (collect_commands! macro)- apps/desktop/src-tauri/src/commands/workspace.rs (add specta annotation)- apps/desktop/src-tauri/src/commands/source.rs (add specta annotation)- apps/desktop/src-tauri/src/commands/discovery.rs (add specta annotation)- apps/desktop/src-tauri/src/commands/document.rs (add specta annotation)- apps/desktop/src-tauri/src/commands/tab.rs (add specta annotation)- apps/desktop/src/bindings.ts (regenerate)- apps/desktop/src/hooks/useWorkspaces.ts (use Specta bindings)- apps/desktop/src/hooks/useSources.ts (use Specta bindings)- apps/desktop/src/hooks/useDocuments.ts (use Specta bindings)- apps/desktop/src/hooks/useTabs.ts (use Specta bindings)- apps/desktop/src/types.ts (remove duplicate types)

**File ownership**: desktop + frontend

**Parallelizable**: No (subsequent tasks depend on Specta bindings)

**Input interface**: Existing Tauri command signatures

**Output interface**: bindings.ts containing all 18 commands and types

**Failure test**: cargo run -p devforge-desktop --bin export_bindings must succeed, bindings.ts must contain all 18 commands

**Implementation steps**:
1. Add specta annotation to each tauri::command function
2. List all commands in collect_commands!
3. Ensure all DTO types derive specta::Type
4. Ensure all error types derive specta::Type or use wrapper
5. Run export_bindings to regenerate bindings.ts
6. Change frontend hooks to use commands.xxx() instead of invoke()
7. Remove duplicate types from types.ts

**Verification commands**:
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd apps/desktop && pnpm typecheck && pnpm test

**Commit boundary**: Single commit, message: fix(ipc): register all Tauri commands with Specta bindings

**Rollback**: git revert the commit

---

## Task 2: Fix modified_at Calculation and removed Update

**Goal**: Fix two data corruption bugs in the scan process.

**Prerequisites**: None

**Files to modify**:
- crates/devforge-application/src/discovery.rs (modified_at calculation + removed logic)

**File ownership**: application

**Parallelizable**: Yes (parallel with Task 1)

**Input interface**: ScanSource::execute method

**Output interface**: ScanResult struct

**Failure tests**:
1. New test: create file, scan, verify modified_at matches actual file modification time (within 1 second tolerance)
2. New test: scan, delete file, re-scan, verify deleted file is marked content_readable = false

**Implementation steps**:
1. Fix modified_at calculation (lines 195-201): use chrono::DateTime::from_timestamp(secs, nanos)
2. Fix removed logic (lines 227-234): for documents not in current_paths, call document_repo.upsert() with content_readable = false
3. Add unit tests

**Verification commands**:
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

**Commit boundary**: Single commit, message: fix(scan): fix modified_at calculation and removed document update

**Rollback**: git revert the commit

---

## Task 3: Fix list_documents Lazy-Load Query

**Goal**: Make list_documents support true directory-level lazy loading instead of returning all documents.

**Prerequisites**: None

**Files to modify**:
- crates/devforge-application/src/document.rs (ListDocuments::execute filter logic)
- crates/devforge-application/src/discovery.rs (DocumentRepository trait add method)
- crates/devforge-storage/src/repository.rs (add list_by_source_and_parent)

**File ownership**: application + storage

**Parallelizable**: Yes (parallel with Task 1, 2)

**Input interface**: list_documents(source_id, parent_path: Option<String>)

**Output interface**: Vec<DocumentDto> - only direct children (files and directory entries)

**Failure tests**:
1. New test: source contains src/main.rs, src/lib.rs, README.md; list_documents(sid, None) returns README.md + one src/ directory entry
2. New test: list_documents(sid, Some(src)) returns main.rs and lib.rs

**Implementation steps**:
1. Add to DocumentRepository trait: async fn list_by_source_and_parent(source_id, parent_path: Option<&str>) -> Result<Vec<Document>, DomainError>
2. Implement in SqliteDocumentRepository using SQL LIKE or path prefix matching
3. Modify ListDocuments::execute to use new method
4. Return results including directory entries (representative entries for subdirectories)
5. Add unit tests

**Verification commands**:
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

**Commit boundary**: Single commit, message: feat(document): implement directory-level lazy-load query

**Rollback**: git revert the commit

---

## Task 4: Complete Add Source and Scan UI

**Goal**: Add Add Source button, directory picker dialog, scan trigger, and source list to WorkspacePage.

**Prerequisites**: Task 1 (Specta bindings)

**Files to modify**:
- apps/desktop/src/pages/WorkspacePage.tsx (add source UI)
- apps/desktop/src/components/AddSourceDialog.tsx (new dialog component)
- apps/desktop/src/components/SourceList.tsx (new source list component)

**File ownership**: frontend

**Parallelizable**: No

**Input interface**: useAddGitSource, useAddDirectorySource, useScanSource hooks

**Output interface**: Source list UI + scan trigger

**Failure tests**:
1. New test: WorkspacePage renders Add Source button
2. New test: clicking button opens dialog
3. New test: dialog has directory selection capability

**Implementation steps**:
1. Create AddSourceDialog component with Tauri dialog.open() for directory selection
2. Auto-detect .git directory to choose source type
3. Call useAddGitSource or useAddDirectorySource, then auto-trigger useScanSource
4. Create SourceList component showing name, path, type with remove and re-scan support
5. Integrate in WorkspacePage sidebar

**Verification commands**:
cd apps/desktop && pnpm typecheck && pnpm test

**Commit boundary**: Single commit, message: feat(ui): complete add source and scan UI

**Rollback**: git revert the commit

---

## Task 5: Implement True Lazy-Load File Tree

**Goal**: Rewrite FileTree component to use directory-level lazy loading.

**Prerequisites**: Task 1 (Specta bindings), Task 3 (lazy-load query)

**Files to modify**:
- apps/desktop/src/components/FileTree.tsx (rewrite for lazy loading)
- apps/desktop/src/hooks/useDocuments.ts (use parentPath parameter)
- apps/desktop/src/components/TreeNode.tsx (new tree node component, optional)

**File ownership**: frontend

**Parallelizable**: No

**Input interface**: list_documents(source_id, parent_path) IPC command

**Output interface**: Expandable/collapsible file tree, each directory loads children on expand

**Failure tests**:
1. New test: initial load only shows root-level items
2. New test: expanding a directory loads its children
3. New test: collapsing a directory unloads children

**Implementation steps**:
1. Rewrite FileTree: initial call list_documents(sourceId, None) for root only
2. On directory expand: list_documents(sourceId, dirPath) for children
3. Use React Query cache to avoid duplicate requests
4. Create TreeNode component distinguishing file vs directory nodes
5. Handle path separators (Windows backslash vs Unix forward slash)

**Verification commands**:
cd apps/desktop && pnpm typecheck && pnpm test

**Commit boundary**: Single commit, message: feat(filetree): implement directory-level lazy-load file tree

**Rollback**: git revert the commit

---

## Task 6: Complete Tab Bar and Tab Management

**Goal**: Add tab bar to WorkspacePage, support open/switch/close tabs, integrate file viewer.

**Prerequisites**: Task 1 (Specta bindings), Task 5 (file tree)

**Files to modify**:
- apps/desktop/src/pages/WorkspacePage.tsx (add tab bar)
- apps/desktop/src/components/TabBar.tsx (new tab bar component)
- apps/desktop/src/components/FileViewer.tsx (may need adjustments)

**File ownership**: frontend

**Parallelizable**: No

**Input interface**: open_tab, close_tab, list_tabs, set_active_tab IPC commands

**Output interface**: Tab bar UI + multi-file viewing

**Failure tests**:
1. New test: clicking a file opens a new tab
2. New test: clicking a tab switches active file
3. New test: clicking close button closes tab
4. New test: clicking already-open file just switches (no duplicate)

**Implementation steps**:
1. Create TabBar component with tab list, click-to-switch, close button, file type icons, active highlight
2. Modify WorkspacePage: file tree onFileSelect calls openTab.mutate()
3. Show FileViewer below tab bar for active tab document
4. On closing active tab, auto-switch to adjacent tab
5. Handle tab-document association

**Verification commands**:
cd apps/desktop && pnpm typecheck && pnpm test

**Commit boundary**: Single commit, message: feat(tabs): complete tab bar and tab management

**Rollback**: git revert the commit

---

## Task 7: Fix Tab Active Status + Add Transaction Protection

**Goal**: Fix the no-op bug in OpenTab::execute, add transaction protection to scanning.

**Prerequisites**: None

**Files to modify**:
- crates/devforge-application/src/tab.rs (fix OpenTab::execute)
- crates/devforge-application/src/discovery.rs (add transaction support)
- crates/devforge-storage/src/repository.rs (implement transaction)

**File ownership**: application + storage

**Parallelizable**: Yes (parallel with Task 1-6)

**Input interface**: OpenTab::execute(workspace_id, document_id)

**Output interface**: TabDto (new or existing tab)

**Failure tests**:
1. New test: open two tabs, verify only one has is_active = true
2. New test: open existing tab, verify it becomes active

**Implementation steps**:
1. Fix OpenTab::execute (lines 96-107): after creating tab, call self.tab_repo.set_active(); remove the empty for loop
2. Add transaction protection to ScanSource::execute using SQLite BEGIN IMMEDIATE + COMMIT / ROLLBACK
3. Add corresponding unit tests

**Verification commands**:
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

**Commit boundary**: Single commit, message: fix(tab): fix tab active status switching + scan transaction protection

**Rollback**: git revert the commit

---

## Task 8: Implement Startup Restore and last_opened_at Update

**Goal**: Restore previous tab state when entering a workspace, update last_opened_at.

**Prerequisites**: Task 6 (tab bar)

**Files to modify**:
- apps/desktop/src/pages/WorkspacePage.tsx (load tabs on mount + update last_opened_at)
- apps/desktop/src-tauri/src/commands/workspace.rs (add mark_workspace_opened command)
- crates/devforge-application/src/workspace.rs (add MarkWorkspaceOpened use case)

**File ownership**: application + desktop + frontend

**Parallelizable**: No

**Input interface**: list_tabs(workspace_id) + mark_workspace_opened(id) IPC commands

**Output interface**: Tab bar restored + workspace last_opened_at updated

**Failure tests**:
1. New test: open tabs, close app, restart, verify tabs are restored
2. New test: enter workspace, verify last_opened_at is updated
3. New test: tabs for deleted documents are auto-cleaned

**Implementation steps**:
1. Add mark_opened method to WorkspaceRepository (or use existing update)
2. Create MarkWorkspaceOpened use case
3. Add mark_workspace_opened Tauri command
4. In WorkspacePage: on mount call mark_workspace_opened, use useTabs to load existing tabs, verify tab documents still exist, set active tab

**Verification commands**:
cargo test --workspace
cd apps/desktop && pnpm typecheck && pnpm test

**Commit boundary**: Single commit, message: feat(persistence): implement startup restore and last_opened_at update

**Rollback**: git revert the commit

---

## Task 9: End-to-End Verification and Documentation Update

**Goal**: Run all verification, update documentation to mark Phase 1 status.

**Prerequisites**: All Task 1-8 complete

**Files to modify**:
- docs/phases/phase-1-workspaces.md (update exit condition status)
- docs/superpowers/plans/2026-07-17-devforge-phase-1-local-workspace-roadmap.md (update sub-plan status)

**File ownership**: No code changes

**Parallelizable**: No

**Failure tests**: N/A (verification task)

**Implementation steps**:
1. Run: cargo test --workspace
2. Run: cargo fmt --check
3. Run: cargo clippy --workspace --all-targets -- -D warnings
4. Run: cd apps/desktop && pnpm typecheck
5. Run: cd apps/desktop && pnpm test
6. Manual verification of full user flow (see Final Manual Acceptance Steps)
7. Update documentation status

**Verification commands**:
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd apps/desktop && pnpm typecheck && pnpm test

**Commit boundary**: Single commit, message: docs(phase1): update Phase 1 implementation status

**Rollback**: git revert the commit

---

## Task Dependency Graph

Task 1 (Specta bindings) --+--> Task 4 (Source UI) --> Task 5 (Lazy file tree) --> Task 6 (Tab bar) --> Task 8 (Startup restore) --> Task 9 (Verification)
                            |
Task 2 (modified_at) ------+
                            |
Task 3 (Lazy query) --------+
                            |
Task 7 (Tab active) --------+

- Task 1, 2, 3, 7 can run in parallel
- Task 4 depends on Task 1
- Task 5 depends on Task 1, 3
- Task 6 depends on Task 1, 5
- Task 8 depends on Task 6
- Task 9 depends on all

---

## Risk and Mitigation

### Risk 1: Specta Type Compatibility

Some Rust types (PathBuf, DateTime<Utc>) may not directly generate TypeScript types via Specta. Use DTO layer (existing DocumentDto, TabDto) instead of exposing domain types directly. Create DTOs for Workspace, Source if Specta cannot handle them.

### Risk 2: Lazy-Load Query Performance

list_by_source_and_parent using LIKE or string prefix matching may be slow on large datasets. The existing unique index on (source_id, relative_path) can support LIKE prefix queries. If insufficient, add a parent_path column.

### Risk 3: Tab Restore with Deleted Documents

On startup restore, verify document existence and auto-clean invalid tabs. This is expected behavior, not a failure.

### Risk 4: Tauri Dialog API Availability

Check package.json for @tauri-apps/plugin-dialog. If not available, fall back to text input for manual path entry.

---

## Final Manual Acceptance Steps

1. Start DevForge application
2. Click Create Workspace, enter name and description
3. Enter workspace, click Add Source
4. Select a directory containing code (e.g., the DevForge project itself)
5. Wait for scan to complete, confirm file count > 0
6. Expand src/ directory in file tree, confirm only root and first-level children loaded
7. Continue expanding subdirectories, confirm lazy loading works
8. Click a .rs file, confirm new tab opens in tab bar
9. Click another file, confirm second tab opens
10. Click first tab, confirm switching back to first file
11. Close a tab, confirm tab is removed
12. Close the application
13. Restart the application
14. Confirm workspace list shows previously created workspace
15. Enter workspace, confirm file tree and tabs restored
16. Return to home, delete workspace
17. Confirm original directory still exists (not deleted)

