---
title: Runtime394 Bindless View Capacity
category: zircon_runtime
report_id: Runtime394-bindless-view-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime394 Bindless View Capacity

Bindless material bind-group reconstruction now reserves slot capacity before collecting texture
views and samplers. Binding array order, fallback slots, sampler/texture pairing, and GPU bind-group
descriptors remain unchanged while repeated rebuilds avoid vector growth reallocations.

The ignored Windows Release benchmark emits `RUNTIME394_BINDLESS_VIEW_CAPACITY_BENCH_V1` over 17
alternating paired samples with 256 slots per sample, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime394 is submitted with Runtime395 under request
`runtime394-runtime395-performance-batch-20260830ct-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
