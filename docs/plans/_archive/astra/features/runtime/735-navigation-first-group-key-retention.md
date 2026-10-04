---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-first-group-key-retention.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-navigation-first-group-candidate.md
related_records:
  - docs/plans/astra/features/runtime/733-navigation-first-group-candidate.md
  - docs/plans/astra/features/runtime/734-navigation-candidate-bucket-reuse.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/surface/navigation_index.rs
  - zircon_runtime/src/ui/surface/navigation_index/candidate_buckets.rs
tests:
  - zircon_runtime/src/ui/surface/navigation_index/tests.rs
  - tools/tests/test_runtime_ui_navigation_index_performance_contract.py
---

# Runtime navigation first-group key retention

The first directional candidate map now retains stable group keys across
navigation-index rebuilds. Existing entries are reset with a `seen` marker and
updated through `get_mut`; only new groups clone a key. Unseen groups are
pruned before publication, preserving manual group-target behavior while
avoiding recurring `String` and map-entry allocation.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A P1-14 | Retain first-group candidate keys and capacity across rebuilds without changing candidate ordering. | RED/GREEN source contracts `12/12`, lower Rust `seen` reset/prune, key-identity/capacity, finish-stream borrow, and helper-module compile shape checks, Rustfmt parse, and batched Runtime/Editor contracts `105/105`; managed Cargo/Release allocation and p50/p95/p99 evidence remains pending. | implemented_pending_validation |

## Acceptance boundary

This record remains `implemented_pending_validation` until the existing
owner-attributed Windows Release batch verifies compilation, deterministic
candidate order, allocation behavior, and navigation p50/p95/p99. Tooling and
hard-cutover debts remain out of scope; no coordinator status was queried.
