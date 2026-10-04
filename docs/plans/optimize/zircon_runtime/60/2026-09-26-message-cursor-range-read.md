---
title: Runtime60 Message Cursor Range Read
category: zircon_runtime
report_id: Runtime60-message-cursor-range-read-2026-09-26
date: 2026-09-26
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: release_p95_target_pending_validation
---

# Runtime60 message cursor range read

`MessageCursor::read` previously constructed an iterator over the entire retained queue and
called `next()` once for every already-read entry. A cursor at the tail of a retained 65,536-entry
queue therefore traversed all 65,536 entries for each empty poll. The cursor now constructs a
`VecDeque::range(start..)` iterator at the computed unread index. Its acknowledgement, generation,
and retention-gap accounting remain unchanged.

| Evidence | Before | After / acceptance target |
| --- | ---: | ---: |
| Iterator skip operations for 32 empty reads of 65,536 retained entries | 2,097,152 | 0; direct range construction |
| Partial read, wrapped retained tail with nonzero start, and drop count | Existing behavior | Exact regression expectations |
| Empty-poll time P95, 31 alternating Release pairs | Pending | At most half of legacy P95 |
| Empty-poll time P50/P99 | Pending | Reported with raw samples |

The focused tests cover a partially consumed iterator, a genuinely wrapped `VecDeque` whose next
read starts at index one, the remaining IDs and order, an empty tail, and a retention gap counted
once. The ignored
`RUNTIME60_MESSAGE_CURSOR_RANGE_READ_BENCH_V1` Release comparator runs 32 empty reads per sample
over one 65,536-entry queue and reports P50/P95/P99 from 31 alternating legacy and optimized
pairs. The P95 gate is implemented but has not yet passed managed validation. This local slice
does not close Runtime60's broader message retention, identity, or lifecycle work.
