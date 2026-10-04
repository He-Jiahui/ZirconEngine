---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-09-21-drawer-toggle-direct-query.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/layout/drawer_toggle.rs
tests:
  - zircon_editor/src/ui/host/workspace_state_direct_query_batch_tests.rs
  - tools/tests/test_editor863_866_workspace_direct_query_performance_contract.py
---

# Editor863 - drawer-toggle direct state query

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Retained drawer toggle | Borrow the active drawer map and return only mode/active/expanded state; remove the complete layout clone while preserving errors and region-reuse behavior. | Combined RED→GREEN contract `7/7`; lower semantics and ignored `EDITOR863_DRAWER_TOGGLE_DIRECT_QUERY_BENCH_V1`; focused Editor859–866 batch `26/26`; widened non-tooling batch `4183/4183` across `986` files; exact Rustfmt. Managed Release/allocator/product percentiles pending. | implemented_pending_validation |

## Scope boundary

Layout commands and shell reuse policy remain authoritative. Tooling production
is deferred.

## Managed gate

The combined Runtime→Editor→App batch is asynchronous. Do not close this entry
until current-source Release, allocator, marker, and drawer p50/p95/p99 receipts
pass.
