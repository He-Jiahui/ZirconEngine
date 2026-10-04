---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/60/2026-09-26-message-cursor-range-read.md
implementation_files:
  - zircon_runtime/src/scene/ecs/messages/cursor.rs
tests:
  - zircon_runtime/src/scene/ecs/messages/cursor.rs
---

# Runtime60 message cursor range read completion list

| Completed slice | Evidence | Remaining acceptance |
| --- | --- | --- |
| Tracked reads start at the unread queue index without walking the retained prefix. | Focused regressions assert partial consumption, a physically wrapped queue with nonzero read start, retained ID/order, empty reads, and retention-gap accounting. | Grouped managed Runtime filter `runtime60_message_cursor_range_read`; ignored Release marker `RUNTIME60_MESSAGE_CURSOR_RANGE_READ_BENCH_V1` must meet optimized P95 at most half of legacy P95. |

The source and Release comparator are ready for the batch validation lane. Dynamic timings and
product acceptance remain pending terminal managed results.
