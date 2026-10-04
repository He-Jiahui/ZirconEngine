---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-09-21-floating-window-exists-direct-query.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs
tests:
  - zircon_editor/src/ui/host/workspace_state_direct_query_batch_tests.rs
  - tools/tests/test_editor863_866_workspace_direct_query_performance_contract.py
---

# Editor865 - floating-window existence direct query

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Native floating-window close | Check the requested window ID against borrowed authoritative layout state instead of cloning the whole layout. | Combined RED→GREEN contract `7/7`; exact/case-sensitive lower regression and ignored `EDITOR865_FLOATING_WINDOW_EXISTS_DIRECT_QUERY_BENCH_V1`; focused batch `26/26`; widened non-tooling batch `4183/4183` across `986` files; exact Rustfmt. Managed gates pending. | implemented_pending_validation |

## Managed gate

Keep pending until current-source Windows Release, allocator, lower marker, and
native-window close p50/p95/p99 evidence pass.
