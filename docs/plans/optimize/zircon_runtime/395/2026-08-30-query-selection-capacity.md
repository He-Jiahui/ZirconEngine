---
title: Runtime395 Query Selection Capacity
category: zircon_runtime
report_id: Runtime395-query-selection-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime395 Query Selection Capacity

Material-management query selection now reserves the filtered page record count before copying
material resource ids into the selection input. Query filtering, page order, selection semantics,
and empty-result behavior remain unchanged while large result pages avoid vector growth.

The ignored Windows Release benchmark emits `RUNTIME395_QUERY_SELECTION_CAPACITY_BENCH_V1` over 17
alternating paired samples with 512 records per sample, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime395 is submitted with Runtime394 under request
`runtime394-runtime395-performance-batch-20260830ct-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
