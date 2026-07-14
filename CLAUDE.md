# DevForge Project Instructions

## Product

DevForge is a Windows-first, local-first developer knowledge and
AI task workbench.

The product specification is in `docs/product/vision.md`.

## Core architecture

- Desktop host: Tauri 2
- Frontend: React + TypeScript
- Backend: Rust workspace
- Business data source of truth: SQLite
- Lexical search: Tantivy
- Code parsing: Tree-sitter
- Semantic enhancement: optional LSP
- AI changes require explicit user approval

Read `docs/architecture/overview.md` before making architectural changes.

## Hard boundaries

- Domain crates must not depend on Tauri.
- React must not access SQLite or the filesystem directly.
- Tauri commands expose application use cases, not database CRUD.
- AI cannot write files or execute commands without Policy Engine approval.
- Search indexes are rebuildable; SQLite is the source of truth.
- Windows is the first supported platform, but core crates remain cross-platform.

## Development workflow

1. Read the current phase specification.
2. Explore the existing code before proposing changes.
3. Write or update an implementation plan.
4. Implement one independently testable task at a time.
5. Write failing tests before implementation.
6. Run formatting, linting and relevant tests.
7. Review the diff before committing.
8. Update the relevant design document when a decision changes.

## Commands

- Frontend install: `pnpm install`
- Frontend check: `pnpm typecheck`
- Frontend test: `pnpm test`
- Rust format: `cargo fmt --check`
- Rust lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Rust test: `cargo test --workspace`

## Encoding and tooling

- Use UTF-8 without BOM.
- Use Chinese comments only where comments add real value.
- Prefer `pnpm` over npm and yarn.
- Do not commit secrets, local paths or credentials.

## Current phase

Current implementation phase:
`docs/phases/phase-0-foundation.md`

Current approved plan:
`docs/plans/phase-0-foundation-plan.md`
