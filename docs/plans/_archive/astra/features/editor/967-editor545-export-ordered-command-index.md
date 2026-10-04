---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-editor545-export-ordered-command-index.md
implementation_files:
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/plan.rs
tests:
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/plan.rs
---

# Editor967 Editor545 export ordered-command index

Ordered export-plan traversal now checks the command at the expected stage
position and falls back to the authoritative lookup for reordered or malformed
public vectors. Marker `EDITOR545_EXPORT_ORDERED_COMMAND_INDEX_BENCH_V1` models
65,536 eight-stage traversals and the 2,359,296-to-524,288 comparison
reduction. Managed Editor Release validation remains pending.
