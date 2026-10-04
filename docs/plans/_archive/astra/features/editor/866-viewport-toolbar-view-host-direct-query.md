---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-09-21-viewport-toolbar-view-host-direct-query.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/size.rs
tests:
  - zircon_editor/src/ui/host/workspace_state_direct_query_batch_tests.rs
  - tools/tests/test_editor863_866_workspace_direct_query_performance_contract.py
---

# Editor866 - viewport toolbar view-host direct query

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Viewport toolbar sizing | Borrow the matching instance and clone only its `ViewHost`, removing all-open-instance cloning while preserving floating/document/drawer/exclusive sizing. | Combined RED→GREEN contract `7/7`; exact-key lower regression and ignored `EDITOR866_VIEW_HOST_DIRECT_QUERY_BENCH_V1`; focused batch `26/26`; widened non-tooling batch `4183/4183` across `986` files; exact Rustfmt. Managed gates pending. | implemented_pending_validation |

## Managed gate

Keep pending until current-source Windows Release, allocator, lower marker, and
viewport toolbar p50/p95/p99 evidence pass. Tooling production remains deferred.
