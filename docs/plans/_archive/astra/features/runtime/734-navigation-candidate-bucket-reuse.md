---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-position-map-reuse.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-candidate-bucket-reuse.md
related_records:
  - docs/plans/astra/features/runtime/731-navigation-position-map-reuse.md
  - docs/plans/astra/features/runtime/732-navigation-group-key-reuse.md
  - docs/plans/astra/features/runtime/733-navigation-first-group-candidate.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/surface/navigation_index.rs
  - zircon_runtime/src/ui/surface/navigation_index/candidate_buckets.rs
tests:
  - zircon_runtime/src/ui/surface/navigation_index/tests.rs
  - tools/tests/test_runtime_ui_navigation_index_performance_contract.py
---

# Runtime navigation candidate bucket reuse

The navigation index now retains the candidate vectors owned by stable modal and
MUI scopes. Rebuilds clear each vector in place, prune empty stale scopes, and
append through a borrowed group-key lookup; only new scopes clone their key and
allocate a first vector. The lower regression covers both group and MUI-root
bucket capacity retention. Sorted candidate vectors remain the deterministic
ordering authority.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A navigation rebuild | Reuse group/MUI candidate buckets and avoid stable group-key clones. | TDD RED/GREEN source contracts, lower Rust capacity/pruning regression, Rust parse checks, and batched UI contracts `772/772`; managed Cargo/Release allocation and latency evidence remains pending. | implemented_pending_validation |

## Acceptance boundary

This record remains `implemented_pending_validation` until the existing
owner-attributed Windows Release batch verifies compilation, deterministic
candidate order, allocation behavior, and navigation p50/p95/p99. Tooling and
hard-cutover debts remain out of scope; no coordinator status was queried.
