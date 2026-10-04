---
title: Runtime640 Preallocated Animation Event Batches
category: zircon_runtime
report_id: Runtime640-preallocated-animation-event-batches-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime640 Preallocated Animation Event Batches

Bounded animation clip-event sampling now constructs the batch event vector with the request's
`max_events` limit. The existing budget loop still caps output and preserves cursor ordering,
oversized-event handling, and resumable playback semantics.

The regression exercises a real two-event sample and verifies the bounded capacity. The ignored
Windows Release benchmark emits `RUNTIME640_PREALLOCATED_ANIMATION_EVENT_BATCHES_BENCH_V1` over 17
alternating sample pairs and 65,536 event projections. The gate requires preallocated P95 to be at
most 85% of unreserved P95.

No direct Cargo validation was run. The coordinator owns aggregate Runtime/Editor regression and
performance validation. Measured P95, commit, push, and WeCom outcome are recorded only after
coordinator completion.
