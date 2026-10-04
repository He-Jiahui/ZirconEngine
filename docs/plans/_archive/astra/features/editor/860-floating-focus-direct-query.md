---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/334/2026-09-21-floating-focus-direct-query.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/layout/floating_window/dispatch.rs
tests:
  - zircon_editor/src/ui/host/workspace_state_floating_focus_query_tests.rs
  - tools/tests/test_editor860_floating_focus_direct_query_performance_contract.py
  - tools/tests/test_editor_retained_workbench_contribution_projection_contract.py
---

# Editor860 - floating-window focus direct query

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Floating callback-window focus | Resolve one requested window directly from authoritative layout and scan only its document tree once, preserving `focused → active → first` priority while eliminating Chrome, command-context, and Workbench-model construction. | Intentional RED→GREEN contract `4/4`; lower nested-priority/missing/empty regressions and ignored `EDITOR860_FLOATING_FOCUS_DIRECT_QUERY_BENCH_V1` marker are wired. Structural Chrome/model builds change `1→0` per dispatch; stale contribution-projection contract repaired `3/3`; combined related batch passes `35/35`, and the current-tree non-tooling loader passes `4121/4121` across `974` files in `221.095s`. Managed Cargo/Release, allocator, and product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Scope boundary

Complete retained presentation builds still project real contributions and
capabilities. Only the identity-only focus dispatch bypasses that projection;
layout mutation, focus commands, and tooling production are unchanged.

## Managed gate

No local Cargo command or coordinator poll was started. Keep this completion
entry pending until the owner-attributed Windows Release lane supplies current-
source compilation, lower marker, allocator, and floating-focus percentile
evidence.
