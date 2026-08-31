---
title: Runtime407 RGBA16Float Aligned Readback Copy
category: zircon_runtime
report_id: Runtime407-rgba16float-aligned-readback-copy-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime407 RGBA16Float Aligned Readback Copy

RGBA16Float 3D texture readback now recognizes rows whose payload already equals the GPU-aligned row
stride and copies the mapped output as one contiguous block. Readbacks that contain row padding keep
the existing slice-and-row copy path, so padding removal and output bytes are unchanged.

The ignored Windows Release benchmark emits
`RUNTIME407_RGBA16FLOAT_ALIGNED_READBACK_COPY_BENCH_V1` over 17 alternating paired samples with an
8 MiB aligned volume spanning 8,192 rows, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime407 is submitted with Runtime406 under request
`runtime406-runtime407-performance-batch-20260830db-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
