---
title: Runtime419 Adopt First Keyboard Event Batch
category: zircon_runtime
report_id: Runtime419-adopt-first-keyboard-event-batch-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime419 Adopt First Keyboard Event Batch

Semantic keyboard dispatch now adopts the first non-empty component-event report vector as its
output. Later event-kind batches append in the established order, while empty batches remain
allocation-free and an all-empty action still returns the default empty vector.

The previous path always allocated an empty destination and moved the first report vector into it.
Most semantic actions map to one event kind, so the new path transfers that vector's ownership
directly and avoids a second allocation and element move for the common case.

The ignored Windows Release benchmark emits
`RUNTIME419_ADOPT_FIRST_KEYBOARD_EVENT_BATCH_BENCH_V1` over 17 alternating paired samples, each
adopting 32,768 non-empty batches of 32 reports, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime419 is prepared with Editor347 under request
`runtime419-editor347-performance-batch-20260830dk-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
