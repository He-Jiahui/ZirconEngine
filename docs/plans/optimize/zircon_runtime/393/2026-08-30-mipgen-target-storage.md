---
title: Runtime393 Mipgen Target Storage
category: zircon_runtime
report_id: Runtime393-mipgen-target-storage-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime393 Mipgen Target Storage

Runtime mip generation now materializes its at-most-four target views in a fixed `Option` array
instead of allocating a vector for every dispatch. Generated-mip counts from zero through four,
fallback storage view binding, source/target mip levels, and dispatch ordering remain unchanged
while per-dispatch heap allocation is removed.

The ignored Windows Release benchmark emits `RUNTIME393_MIPGEN_TARGET_STORAGE_BENCH_V1` over 17
alternating paired samples with four target views per sample, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current validation handoff (2026-08-30)

Runtime393 is submitted with Editor337 under request
`runtime393-editor337-performance-batch-20260830cr-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
