---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/659/2026-09-02-single-index-map-fast-path.md
related_code:
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for/optimization_batch_jt_runtime659_tests.rs
tests:
  - zircon_runtime/src/core/runtime/tasks/parallel_for/optimization_batch_jt_runtime659_tests.rs
---

# Single-Index Map Fast Path

`parallel_map_indices` now executes a one-element map directly, avoiding a Rayon pool install and
parallel range construction. Empty and multi-element behavior remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime659 | Add the exact one-element direct-map fast path | implemented_pending_validation | Source regression and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 remain pending. |

The current package-wide Runtime development validator was relaunched with the
Runtime event/task batch after source stabilization and was intentionally left
unpolled. The matching Runtime02 Release admission returned
`request_overloaded` before a Cargo job existed, so the ignored benchmark and
its threshold remain pending.
