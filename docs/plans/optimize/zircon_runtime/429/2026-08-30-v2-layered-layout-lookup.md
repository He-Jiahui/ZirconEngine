---
title: Runtime429 V2 Layered Layout Lookup
category: zircon_runtime
report_id: Runtime429-v2-layered-layout-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime429 V2 Layered Layout Lookup

V2 surface-tree layout inference now validates and reads self and slot layout tables through a
borrowed layered view. Slot precedence and horizontal/vertical parent-axis restoration retain the
existing contract.

The previous implementation cloned both TOML maps and every overlaid value before parsing a fixed
set of layout fields. The layered view removes that temporary map and all merge-time clones while
retaining the V2 asset/path validation boundary.

The ignored Windows Release benchmark emits `RUNTIME429_V2_LAYERED_LAYOUT_LOOKUP_BENCH_V1` over 17
alternating paired samples, each performing 4,096 lookups across tables with 96 additional
attributes, requiring `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime429 is prepared with Editor357 under request
`runtime429-editor357-performance-batch-20260830du-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
