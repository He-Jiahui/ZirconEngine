---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/636/2026-09-01-preallocated-material-selection-results.md
related_code:
  - zircon_runtime/src/core/framework/render/material/management/selection.rs
  - zircon_runtime/src/core/framework/render/material/management/selection/optimization_batch_iy_runtime636_tests.rs
tests:
  - zircon_runtime/src/core/framework/render/material/management/selection/optimization_batch_iy_runtime636_tests.rs
---

# Material Selection Result Capacity

Material selection now reserves selected and missing result vectors from the unique request count.
Request order, duplicate collapse, ownership, and missing-ID reporting remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime636 | Reserve bounded material-selection result partitions | implemented_pending_validation | Source/behavior regressions and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 remain pending. |
