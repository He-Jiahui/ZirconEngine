---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-editor543-reconcile-carried-entry.md
implementation_files:
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/reconcile.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher/tests.rs
tests:
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/reconcile.rs
---

# Editor965 Editor543 reconcile carried map entry

Asset-session reconcile now carries the selected `BTreeMap` entry from
`get_key_value`, `range`, or `first_key_value` instead of probing with
`contains_key` and then `get`. Cursor progress, ordering, and allowance
accounting remain unchanged. Marker
`EDITOR543_RECONCILE_CARRIED_ENTRY_BENCH_V1` models 65,536 visits and the
131,072-to-65,536 tree-lookup reduction. Managed Editor Release validation
remains pending.
