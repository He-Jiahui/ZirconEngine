---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/638/2026-09-01-preallocated-control-id-index.md
related_code:
  - zircon_editor/src/ui/asset_editor/preview/preview_projection.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_projection/optimization_batch_iz_editor638_tests.rs
tests:
  - zircon_editor/src/ui/asset_editor/preview/preview_projection/optimization_batch_iz_editor638_tests.rs
---

# Preview Control-ID Index Capacity

Preview control-ID indexing now reserves its borrowed map from the node iterator bound. First
duplicate IDs retain the prior first-node semantics and node ordering remains unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor638 | Reserve the bounded preview control-ID index | implemented_pending_validation | Source/behavior regressions and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Editor Cargo and Release p50/p95/p99 remain pending. |
