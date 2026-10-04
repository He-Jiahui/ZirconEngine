---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-09-21-layout-command-floating-existence.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/host/layout_commands.rs
tests:
  - tools/tests/test_editor867_871_workspace_identity_projection_performance_contract.py
---

# Editor867 - layout-command floating-window existence query

## Completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| View attach commands | Reuse the borrowed authoritative floating-window existence query in both attach paths instead of cloning the complete layout. | Combined RED→GREEN contract `7/7`; focused Editor859–871/Workbench batch `43/43`; widened explicit batch `4441/4441` across `1127` files; Editor865 exact-identity lower regression and marker reused; exact Rustfmt. Managed gates pending. | implemented_pending_validation |

## Managed gate

Keep pending until current-source Windows Release, allocator, marker, and attach
interaction p50/p95/p99 evidence pass.
