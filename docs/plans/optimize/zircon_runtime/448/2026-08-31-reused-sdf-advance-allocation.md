---
title: Runtime448 Reused SDF Advance Allocation
category: zircon_runtime
report_id: Runtime448-reused-sdf-advance-allocation-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime448 Reused SDF Advance Allocation

Runtime SDF text advance resolution now sanitizes each grapheme advance and records whether any
advance is nonzero while filling the final glyph vector. The mismatched grapheme/glyph path returns
that same allocation instead of consuming it into a second allocated vector for another sanitizing
pass.

Grapheme expansion, combining-character zero advances, invalid-number sanitization, exact glyph
count validation, and all-zero rejection remain unchanged. Regression coverage compares the new
path with the former two-pass implementation and requires direct return of the filled allocation.

The ignored Windows Release benchmark emits `RUNTIME448_REUSED_SDF_ADVANCE_ALLOCATION_BENCH_V1`
over 17 alternating paired samples. Each sample resolves 128 batches of 4,096 combining graphemes.
The legacy model allocates two glyph vectors per resolution and the optimized model allocates one.
The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.85`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime448 is prepared with Editor376 under request
`runtime448-editor376-performance-batch-20260831en-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
