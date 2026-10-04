---
title: Runtime620 Preallocated Plugin Event Validation
category: zircon_runtime
report_id: Runtime620-preallocated-plugin-event-validation-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime620 Preallocated Plugin Event Validation

Plugin event catalog validation now reserves its borrowed event-ID membership index from the event
count. The previous hash set grew incrementally even though catalog length was known before the
validation loop.

Event field and namespace validation order, borrowed `&str` identities, payload checks, and the
first duplicate diagnostic remain unchanged. Existing behavior coverage locks the first duplicate
event error, while focused source coverage rejects an unreserved production index.

The ignored Windows Release benchmark emits `RUNTIME620_PREALLOCATED_PLUGIN_EVENT_BENCH_V1` over
17 alternating sample pairs with 32,768 unique long event IDs. The gate requires preallocated
membership P95 to be at most 85% of unreserved membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime620 is prepared with Editor620 under request
`runtime620-editor620-event-batch-claims-performance-20260901ij-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
