---
title: Runtime641 Preallocated Cube LUT Samples
category: zircon_runtime
report_id: Runtime641-preallocated-cube-lut-samples-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime641 Preallocated Cube LUT Samples

The `.cube` LUT parser now derives the required RGBA byte count when it reads a valid
`LUT_3D_SIZE` declaration and reserves the sample buffer before parsing the sample rows. The target
is capped by source byte length, which still reaches the exact output size for a valid LUT while
avoiding a large eager allocation for a truncated file that only declares a maximum size. Sample
order, channel conversion, supported-size validation, and malformed-row diagnostics remain
unchanged.

The ignored Windows Release benchmark emits `RUNTIME641_PREALLOCATED_CUBE_LUT_SAMPLE_BENCH_V1`
over 17 alternating sample pairs with 1,048,576 RGBA samples. The gate requires preallocated P95 to
be at most 80% of the unreserved sample-buffer P95.

No direct Cargo validation was run. The coordinator owns combined Runtime641/Editor641 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime641 is prepared with Editor641 under the shared `optimization_batch_jb_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
