---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-09-21-tab-drop-drawer-mode-direct-query.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/layout/tab_drop.rs
tests:
  - zircon_editor/src/ui/host/workspace_state_direct_query_batch_tests.rs
  - tools/tests/test_editor863_866_workspace_direct_query_performance_contract.py
---

# Editor864 - tab-drop drawer-mode direct query

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Drawer tab drop | Query only the requested active drawer mode and remove the full layout clone while preserving collapsed reopening and effect order. | Combined RED→GREEN contract `7/7`; lower semantics and ignored `EDITOR864_TAB_DROP_DRAWER_MODE_DIRECT_QUERY_BENCH_V1`; focused batch `26/26`; widened non-tooling batch `4183/4183` across `986` files; exact Rustfmt. Managed gates pending. | implemented_pending_validation |

## Managed gate

Keep pending until the asynchronous combined lane supplies Windows Release,
allocator, marker, and tab-drop product p50/p95/p99 evidence.
