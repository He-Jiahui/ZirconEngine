---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-position-map-reuse.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-group-key-reuse.md
related_records:
  - docs/plans/astra/features/runtime/731-navigation-position-map-reuse.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/surface/navigation_index.rs
tests:
  - zircon_runtime/src/ui/surface/navigation_index/tests.rs
  - tools/tests/test_runtime_ui_navigation_index_performance_contract.py
---

# Runtime navigation group position-map key reuse

The navigation index now checks for an existing group position bucket before
cloning the owned `UiNavigationGroupId`. Stable groups rebuild their nested
lookup map in place; only new scopes pay the key clone and bucket allocation.
Candidate vectors, ordering, stale-scope pruning, and query semantics are
unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A navigation rebuild | Lookup existing group buckets before owned insertion. | TDD RED/GREEN source contract, existing capacity/pruning regression, Rust parse check, and batched UI contracts `772/772`; managed Cargo/Release allocation and latency evidence remains pending. | implemented_pending_validation |

## Acceptance boundary

This record remains `implemented_pending_validation` until the existing
owner-attributed Windows Release batch verifies compilation, deterministic
candidate order, allocation behavior, and navigation p50/p95/p99. Tooling and
hard-cutover debts remain out of scope; no coordinator status was queried.
