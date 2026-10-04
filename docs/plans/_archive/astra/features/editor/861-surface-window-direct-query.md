---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-09-21-surface-window-direct-query.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/source_window/focus.rs
tests:
  - zircon_editor/src/ui/host/workspace_state_surface_window_query_tests.rs
  - tools/tests/test_editor861_surface_window_direct_query_performance_contract.py
---

# Editor861 - focused surface-window direct query

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Native focus surface mapping | Resolve the focused surface key directly against the authoritative floating-window layout, cloning only the matched ID and deleting the obsolete snapshot helper. | Intentional RED→GREEN contract `4/4`; lower exact/case-mismatch/missing regressions and ignored `EDITOR861_SURFACE_WINDOW_DIRECT_QUERY_BENCH_V1` marker are wired. Structural full-snapshot builds change `1→0`; the combined related batch passes `43/43`, and the current-tree non-tooling loader passes `4121/4121` across `974` files. Managed Cargo/Release, allocator, and product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Scope boundary

The caller retains special `main` handling and missing keys still clear the
focused floating-window identity. Layout mutation and native focus dispatch are
unchanged; tooling production remains deferred.

## Managed gate

No local Cargo command or coordinator poll was started. Keep this completion
entry pending until the owner-attributed Windows Release lane supplies current-
source compilation, lower marker, allocator, and native-focus percentile
evidence.
