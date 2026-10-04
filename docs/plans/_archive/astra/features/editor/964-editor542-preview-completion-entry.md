---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-editor542-preview-completion-entry.md
implementation_files:
  - zircon_editor/src/ui/host/editor_asset_manager/preview.rs
tests:
  - zircon_editor/src/ui/host/editor_asset_manager/preview.rs
---

# Editor964 Editor542 preview completion entry probe

Preview completion now uses one occupied `HashMap::entry` probe to validate the
generation and remove the matching job. Missing assets and stale tokens retain
their prior admission behavior. Marker
`EDITOR542_PREVIEW_COMPLETION_ENTRY_BENCH_V1` models 65,536 completions and the
131,072-to-65,536 probe reduction. Managed Editor Release validation remains
pending.
