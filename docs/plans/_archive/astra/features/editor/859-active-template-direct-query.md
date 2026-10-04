---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-09-21-active-template-direct-query.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/workbench_snapshot_access.rs
tests:
  - zircon_editor/src/ui/host/workspace_state_active_template_query_tests.rs
  - tools/tests/test_editor859_active_template_direct_query_performance_contract.py
---

# Editor859 - active activity-window template direct query

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Retained Workbench template gate | Resolve the active Workbench/exclusive-page descriptor from authoritative session state and compare its capability-eligible template document directly, avoiding full Chrome/Workbench snapshot construction for a boolean gate. | Intentional RED→GREEN contract `4/4`; lower Workbench/exclusive/missing-authority regressions and ignored `EDITOR859_ACTIVE_TEMPLATE_DIRECT_QUERY_BENCH_V1` marker are wired. Structural full-snapshot builds change `1→0` per predicate; the combined related batch passes `35/35`, and the current-tree non-tooling loader passes `4121/4121` across `974` files in `221.095s`. Managed Cargo/Release, allocator, and product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Scope boundary

The test-only owned document-ID accessor and full presentation projections stay
unchanged. Missing authority and capability-ineligible descriptors still return
false; tooling production remains deferred.

## Managed gate

No local Cargo command or coordinator poll was started. Keep this completion
entry pending until the owner-attributed Windows Release lane supplies current-
source compilation, lower marker, allocator, and active-workbench interaction
percentile evidence.
