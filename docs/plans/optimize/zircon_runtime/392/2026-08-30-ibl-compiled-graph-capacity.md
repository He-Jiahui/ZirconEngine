---
title: Runtime392 IBL Compiled Graph Capacity
category: zircon_runtime
report_id: Runtime392-ibl-compiled-graph-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime392 IBL Compiled Graph Capacity

Realtime IBL compiled graph variants now reserve graph pass and resource-lifetime counts before
materializing recording passes and required resource names. Unknown-pass diagnostics, authored
pass ordering, resource-name sorting, and graph cache behavior remain unchanged while known-size
projection avoids repeated vector growth.

The ignored Windows Release benchmark emits `RUNTIME392_IBL_COMPILED_GRAPH_CAPACITY_BENCH_V1` over
17 alternating paired samples with 256 passes per sample, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime392 is submitted with Editor336 under request
`runtime392-editor336-performance-batch-20260830cq-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
