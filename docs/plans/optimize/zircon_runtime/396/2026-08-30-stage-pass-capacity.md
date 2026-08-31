---
title: Runtime396 Stage Pass Capacity
category: zircon_runtime
report_id: Runtime396-stage-pass-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime396 Stage Pass Capacity

Render-pipeline stage projection now sums the known pass counts across active feature descriptors
and reserves that upper bound before its single filtering traversal. Stage matching, descriptor
order, pass order, clone semantics, and empty-stage behavior remain unchanged while matching-heavy
pipeline compilation avoids repeated vector growth and relocation.

The ignored Windows Release benchmark emits `RUNTIME396_STAGE_PASS_CAPACITY_BENCH_V1` over 17 paired
samples with 32 descriptors and 256 passes per descriptor, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime396 is submitted with Runtime397 under request
`runtime396-runtime397-performance-batch-20260830cu-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
