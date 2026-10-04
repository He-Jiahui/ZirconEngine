---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/640/2026-09-01-preallocated-animation-event-batches.md
related_code:
  - zircon_runtime/src/animation/clip_event.rs
tests:
  - zircon_runtime/src/animation/clip_event.rs
---

# Animation Event Batch Capacity

Bounded animation sampling now reserves its event vector from `max_events`. Budget, cursor,
ordering, oversized-event handling, and resumable playback semantics remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime640 | Reserve the bounded animation event batch before sampling | implemented_pending_validation | Existing behavior regression and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 remain pending. |
