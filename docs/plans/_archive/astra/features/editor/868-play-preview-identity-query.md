---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-09-21-play-preview-identity-query.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/editor_manager_workspace.rs
  - zircon_editor/src/ui/workbench/shell_state.rs
tests:
  - zircon_editor/src/ui/host/workspace_state_identity_projection_tests.rs
  - tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py
---

# Editor868 - Play Preview identity query

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Play Preview focus | Borrow ordered workspace state and clone only the first matching identity instead of every open view record. | RED→GREEN `7/7`; lower first/missing semantics and ignored `EDITOR868_PLAY_PREVIEW_IDENTITY_QUERY_BENCH_V1`; focused batch `43/43`; widened explicit batch `4441/4441` across `1127` files; exact Rustfmt. Managed gates pending. | implemented_pending_validation |

## Managed gate

Keep pending until Windows Release, allocator, marker, and Play Preview product
p50/p95/p99 evidence pass.
