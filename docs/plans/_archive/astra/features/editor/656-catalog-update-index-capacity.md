---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/656/2026-09-02-preallocated-catalog-update-index.md
related_code:
tests:
---

# Catalog Update Index Capacity

Batch catalog publication now materializes the update iterator once and reserves its UUID-indexed
map from the iterator lower bound. Unknown upper bounds remain conservative and one-publication
COW semantics are unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor656 | Reserve the bounded catalog update index before filtering and sorting | implemented_pending_validation | Source regression and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Editor Cargo and Release p50/p95/p99 remain pending. |
