---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/630/2026-09-01-preallocated-keyed-io-shutdown-notifications.md
related_code:
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/lane.rs
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/lane/optimization_batch_it_runtime630_tests.rs
tests:
  - zircon_runtime/src/core/runtime/tasks/bounded_keyed_io/lane/optimization_batch_it_runtime630_tests.rs
---

# Keyed I/O Shutdown Notification Capacity

Shutdown terminal notifications now reserve from the suspended and queued entry counts held under
the lane lock. At most one notification is emitted per input entry, so ordering and admission
semantics remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime630 | Reserve the bounded terminal-notification batch during keyed I/O shutdown | implemented_pending_validation | Source regression and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 remain pending. |
