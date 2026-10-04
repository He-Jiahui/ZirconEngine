---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/60/2026-09-26-event-queue-high-water-reserve.md
implementation_files:
  - zircon_runtime/src/scene/ecs/events/queue.rs
tests:
  - zircon_runtime/src/scene/ecs/events/queue.rs
---

# Runtime60 event queue high-water reserve completion list

| Completed slice | Evidence | Remaining acceptance |
| --- | --- | --- |
| Next-frame event buffer reservation now requests the high-water mark relative to vector length. | Two growing frames followed by a matching third frame verify capacity and no send-path growth. | Grouped managed Runtime filter `runtime60_event_queue_reserves_next_buffer_after_two_growing_bursts`; ignored Release marker `RUNTIME60_EVENT_QUEUE_HIGH_WATER_RESERVE_BENCH_V1` must meet the 95% P95 send-phase target. |

The source and benchmark are ready for the batch validation lane. Dynamic timing and product
acceptance stay open until terminal results are available.
