---
title: Runtime428 Layered Layout Lookup
category: zircon_runtime
report_id: Runtime428-layered-layout-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime428 Layered Layout Lookup

Template layout inference now reads self and slot layout tables through a borrowed layered view.
Slot attributes retain normal precedence, while horizontal and vertical parent containers restore
the self width or height exactly as before.

The previous path cloned the complete self table, cloned and inserted every slot entry, and then
cloned the restored axis value even though inference reads only a fixed set of layout fields. The
layered view removes all table materialization and value cloning from this path.

The ignored Windows Release benchmark emits `RUNTIME428_LAYERED_LAYOUT_LOOKUP_BENCH_V1` over 17
alternating paired samples, each performing 4,096 layout lookups across tables with 96 additional
attributes, requiring `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime428 is prepared with Editor356 under request
`runtime428-editor356-performance-batch-20260830dt-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
