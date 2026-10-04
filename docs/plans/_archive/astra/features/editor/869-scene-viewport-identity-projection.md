---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-09-21-scene-viewport-identity-projection.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
tests:
  - zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs
  - tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py
---

# Editor869 - Scene viewport identity projection

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Render-time viewport retirement | Project only matching `editor.scene` identities; keep no-match capacity zero and reserve once on first match. | RED→GREEN `7/7`; lower order/empty regression and ignored `EDITOR869_SCENE_VIEW_IDENTITY_PROJECTION_BENCH_V1`; focused batch `43/43`; widened explicit batch `4441/4441` across `1127` files; exact Rustfmt. Managed gates pending. | implemented_pending_validation |

## Managed gate

Keep pending until Windows Release, allocator, marker, and Scene viewport
retirement/render p50/p95/p99 evidence pass.
