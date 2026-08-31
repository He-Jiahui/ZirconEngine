---
title: Runtime485 Advanced Render ID Dispatch
category: zircon_runtime
report_id: Runtime485-advanced-render-id-dispatch-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime485 Advanced Render ID Dispatch

Runtime built-in catalog classification now dispatches advanced-render package IDs with a static
`match` instead of scanning a three-item slice. The accepted `virtual_geometry`, `hybrid_gi`, and
`solari` set and all descriptor classification behavior remain unchanged.

Regression coverage verifies every supported package and representative near-miss IDs. The
ignored Windows Release benchmark emits `RUNTIME485_ADVANCED_RENDER_ID_DISPATCH_BENCH_V1` over 17
alternating paired samples and 1,048,576 `solari` lookups per sample. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime485 is prepared with Editor415 under request
`runtime485-editor415-performance-batch-20260831gc-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
