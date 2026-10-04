---
title: Runtime60 Event Queue High-Water Reserve
category: zircon_runtime
report_id: Runtime60-event-queue-high-water-reserve-2026-09-26
date: 2026-09-26
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_work_reduction_met_dynamic_pending
---

# Runtime60 event queue high-water reserve

The double-buffered ECS event queue reserves the empty next-frame buffer from the observed
high-water mark after swapping buffers. `Vec::reserve_exact(additional)` counts `additional` from
the current length, but the prior code subtracted the buffer capacity. After two growing bursts,
the old current buffer could have capacity 65,536 while the new high-water mark was 131,072;
`reserve_exact(65,536)` then did nothing because the buffer length was zero. The following frame
grew the vector during event sends.

The reserve request now subtracts the next buffer's length. This preserves event ordering,
generation, retention, and shrink-debounce behavior while moving the required growth to the
frame update where the high-water mark is observed.

| Evidence | Before | After / acceptance target |
| --- | ---: | ---: |
| Next-buffer capacity after 64- then 128-event frames | Can remain 64 | At least 128 |
| Capacity growth during a repeated 128-event third frame | 1 | 0 |
| Release P95 of a repeated growing-burst send phase | pending | At most 95% of legacy |

The regression sends two increasing frames and checks that a third matching burst keeps the
preallocated next-buffer capacity. The ignored
`RUNTIME60_EVENT_QUEUE_HIGH_WATER_RESERVE_BENCH_V1` benchmark compares the old and corrected
reserve formulas outside the timed phase, then times 131,072 `Events::send` calls from a
65,536-slot previous buffer over 17 alternating sample pairs. It checks that the legacy send
phase grows the vector and the corrected phase does not. The benchmark isolates send-phase
latency; it does not claim that total allocation work across update plus send disappears, or
that whole-world frame time improves. Grouped managed tests and Release timing remain pending.
This slice does not close Runtime60's broader event budgets or scene ECS review.
