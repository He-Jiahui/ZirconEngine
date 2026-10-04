---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-09-21-extension-retirement-identity-projection.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
tests:
  - zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs
  - tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py
---

# Editor871 - extension-retirement identity projection

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Extension view retirement | Project only IDs matching retiring descriptors, retaining ordered all-or-error close preflight and zero-capacity empty/no-match paths. | RED→GREEN `7/7`; lower set/order/empty regression and ignored `EDITOR871_EXTENSION_RETIRE_IDENTITY_PROJECTION_BENCH_V1`; focused batch `43/43`; widened explicit batch `4441/4441` across `1127` files; exact Rustfmt. Managed gates pending. | implemented_pending_validation |

## Managed gate

Keep pending until Windows Release, allocator, marker, and extension-retirement
p50/p95/p99 evidence pass. Tooling production remains deferred.
