---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-position-map-reuse.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-first-group-candidate.md
related_records:
  - docs/plans/astra/features/runtime/666-navigation-group-candidate-single-pass.md
  - docs/plans/astra/features/runtime/732-navigation-group-key-reuse.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/surface/navigation_index.rs
tests:
  - zircon_runtime/src/ui/surface/navigation_index/tests.rs
  - tools/tests/test_runtime_ui_navigation_index_performance_contract.py
---

# Runtime navigation first-group candidate streaming

The retained navigation rebuild now derives each group's first directional
candidate directly while streaming retained nodes. It no longer creates a
temporary per-group candidate vector, clones a group key for every candidate,
or sorts that temporary vector before publishing one node ID. Existing
deterministic tab ordering and manual group-target semantics remain unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A navigation rebuild | Compare each focus candidate with the retained group winner in the primary node stream. | TDD RED/GREEN source contracts, one-stream Rust regression, Rust parse check, and batched UI contracts `772/772`; managed Cargo/Release allocation and latency evidence remains pending. | implemented_pending_validation |

## Acceptance boundary

This record remains `implemented_pending_validation` until the existing
owner-attributed Windows Release batch verifies compilation, deterministic
candidate order, allocation behavior, and navigation p50/p95/p99. Tooling and
hard-cutover debts remain out of scope; no coordinator status was queried.
