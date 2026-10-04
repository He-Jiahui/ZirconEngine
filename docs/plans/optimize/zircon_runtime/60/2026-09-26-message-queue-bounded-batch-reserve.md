---
title: Runtime60 Message Queue Bounded Batch Reserve
category: zircon_runtime
report_id: Runtime60-message-queue-bounded-batch-reserve-2026-09-26
date: 2026-09-26
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_capacity_target_pending_release_validation
---

# Runtime60 message queue bounded batch reserve

`Messages::write_batch_at_frame` previously passed the iterator's full lower size hint to
`VecDeque::reserve`. The queue enforces a retained-entry limit after each write, but its storage
keeps the reserved capacity. A 65,536-message batch with a 1,024-entry retention limit therefore
could leave at least 65,536 queue slots allocated after the batch. This contradicts the Runtime60
bounded message-retention memory intent even though `len()` and drop metrics were correct.

The reserve request now stops at the greater of the current length and the retention entry limit,
plus one slot for the moment between push and eviction. Saturating arithmetic covers zero and
`usize::MAX` limits and a queue whose current length already exceeds the configured limit. The
write loop and its full ID output are unchanged. The ID vector still reflects the whole batch;
this slice bounds only the queue's retained storage reserve.

| Evidence | Before | After / acceptance target |
| --- | ---: | ---: |
| Queue reserve request for 65,536 messages, retention 1,024, initially empty | 65,536 | At most 1,025 |
| Retained queue capacity after the batch | pending Release comparison | At most 1/16 of legacy |
| Returned IDs, retained tail, budget-drop metrics | Existing behavior | Exact parity regression |
| Batch write time P50/P95/P99 | pending | Reported as diagnostic, no timing claim |

The ignored `RUNTIME60_MESSAGE_QUEUE_BOUNDED_BATCH_RESERVE_BENCH_V1` benchmark runs 31
alternating legacy and corrected batch-write pairs over 65,536 messages. It enforces the queue
capacity ratio and prints nearest-rank P50/P95/P99 timings without treating them as passing
latency evidence. The focused tests cover full ID output, retained order, drop metrics, capacity,
zero and unbounded limits, and an overfull arithmetic state. Grouped managed Runtime tests and
Release comparison remain pending. This slice does not close Runtime60's broader event/message
budget, identity, or lifecycle work.
