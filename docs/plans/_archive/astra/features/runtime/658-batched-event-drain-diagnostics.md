---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/658/2026-09-02-batched-event-drain-diagnostics.md
related_code:
  - zircon_runtime/src/core/runtime/events/diagnostics.rs
  - zircon_runtime/src/core/runtime/events/diagnostics/optimization_batch_js_runtime658_tests.rs
tests:
  - zircon_runtime/src/core/runtime/events/diagnostics/optimization_batch_js_runtime658_tests.rs
---

# Batched Event Drain Diagnostics

Subscriber deactivation now aggregates queue-depth and age diagnostics for one drained batch,
reducing per-event atomic and clock work while preserving counts, age totals, maximum age, and
disconnect accounting.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime658 | Publish one diagnostics aggregate for a detached event queue drain | implemented_pending_validation | Source regression and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 remain pending. |

The current package-wide Runtime development validator was relaunched with the
Runtime event/task batch after source stabilization and was intentionally left
unpolled. A Runtime02 Release admission in the same wave returned
`request_overloaded` before creating a Cargo job; no managed test or benchmark
result is inferred.
