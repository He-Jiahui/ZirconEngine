---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-09-21-floating-window-instance-ids-direct-query.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/workbench/layout/document_node.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs
tests:
  - zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close/floating_window.rs
  - tools/tests/test_editor872_floating_window_instance_ids_direct_query_performance_contract.py
---

# Editor872 - floating-window instance-ID direct query

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Native floating close | Borrow the authoritative layout and clone only target-window tab identities through reusable depth-first `DocumentNode` projection. | RED→GREEN `5/5`; lower order/empty/missing semantics; existing Editor314 capacity marker plus ignored `EDITOR872_FLOATING_WINDOW_INSTANCE_IDS_DIRECT_QUERY_BENCH_V1`; focused batch `52/52`; scoped non-tooling batch `4195/4195` across `988` files; exact Rustfmt. Managed gates pending. | implemented_pending_validation |

## Managed gate

Keep pending until current-source Windows Release, allocator, both markers, and
native floating-close p50/p95/p99 evidence pass.
