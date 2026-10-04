---
title: Runtime652 Preallocated Disconnected Subscriber IDs
category: zircon_runtime
report_id: Runtime652-preallocated-disconnected-subscriber-ids-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime652 Preallocated Disconnected Subscriber IDs

Event publication now reserves disconnected-subscriber ID storage from the immutable subscriber
snapshot length. The existing inline first-ID representation remains intact, so only additional IDs
reserve `subscriber_count - 1`; delivery order, last-subscriber event ownership, pruning, and topic
cleanup semantics remain unchanged.

The ignored Windows Release benchmark emits `RUNTIME652_DISCONNECTED_ID_CAPACITY_BENCH_V1` over 21
alternating sample pairs, 8,192 publishes per sample, and 256 disconnected subscribers per publish.
The gate requires reserved P95 to be at most 80% of unreserved P95.

No direct Cargo validation was run. The coordinator owns combined Runtime652/Editor652 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime652 is prepared with Editor652 under the shared `optimization_batch_jm_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
