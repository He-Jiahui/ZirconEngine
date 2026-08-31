---
title: Runtime408 RGBA8 Aligned Readback Copy
category: zircon_runtime
report_id: Runtime408-rgba8-aligned-readback-copy-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime408 RGBA8 Aligned Readback Copy

RGBA8 texture readback now recognizes rows whose payload already equals the GPU-aligned row stride
and copies the mapped output as one contiguous block. Readbacks that contain row padding retain the
existing row-by-row extraction path, so padding removal and output bytes remain unchanged.

The ignored Windows Release benchmark emits `RUNTIME408_RGBA8_ALIGNED_READBACK_COPY_BENCH_V1` over
17 alternating paired samples with an 8 MiB aligned payload spanning 32,768 rows, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime408 is submitted with Runtime409 under request
`runtime408-runtime409-performance-batch-20260830dc-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
