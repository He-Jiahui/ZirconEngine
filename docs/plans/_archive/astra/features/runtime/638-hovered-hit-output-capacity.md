---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/638/2026-09-01-preallocated-hovered-hit-output.md
related_code:
  - zircon_runtime/src/core/framework/picking/pointer_hits.rs
  - zircon_runtime/src/core/framework/picking/pointer_hits/optimization_batch_iz_runtime638_tests.rs
tests:
  - zircon_runtime/src/core/framework/picking/pointer_hits/optimization_batch_iz_runtime638_tests.rs
---

# Hovered Hit Output Capacity

Hovered-hit projection reserves its output from the sorted input hit count. Blocking behavior,
ordering, and returned ownership remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime638 | Reserve the bounded hovered-hit projection output | implemented_pending_validation | Source/behavior regressions and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 remain pending. |
