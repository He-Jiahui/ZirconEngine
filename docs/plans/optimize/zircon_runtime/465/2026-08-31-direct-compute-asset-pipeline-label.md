---
title: Runtime465 Direct Compute Asset Pipeline Label
category: zircon_runtime
report_id: Runtime465-direct-compute-asset-pipeline-label-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime465 Direct Compute Asset Pipeline Label

Asset-backed compute pass lowering now writes the canonical resource scheme, path, and optional
label into one exact-capacity pipeline label. The `compute.asset:<locator>` identity remains byte
compatible across all five resource schemes while avoiding nested display formatting during render
feature lowering.

Regression coverage compares resource, library, package, builtin, and memory locators, including
fragment labels, with the former formatter. The ignored Windows Release benchmark emits
`RUNTIME465_DIRECT_COMPUTE_ASSET_LABEL_BENCH_V1` over 17 alternating paired samples, each building
262,144 labels. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower
P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime465 is prepared with Editor395 under request
`runtime465-editor395-performance-batch-20260831fi-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
