---
title: Runtime416 SDF Failure Projection Reuse
category: zircon_runtime
report_id: Runtime416-sdf-failure-projection-reuse-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime416 SDF Failure Projection Reuse

SDF atlas generation-failure publication now clears and extends each run's existing failure vector
instead of replacing it with a newly collected vector. The projection counts failures while it
maps glyph slot indices, so capacity reuse and failure accounting happen in one pass.

The previous path allocated a new `Vec<Option<SdfGlyphGenerationError>>` for every run whenever a
new failure generation arrived, then traversed the complete result a second time to count failures.
The new path preserves the vector allocation established by atlas planning and visits each glyph
once while retaining missing-slot and out-of-range semantics.

The ignored Windows Release benchmark emits
`RUNTIME416_SDF_FAILURE_PROJECTION_REUSE_BENCH_V1` over 17 alternating paired samples, each
projecting 4,096 glyphs 64 times against 1,024 atlas slots, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime416 is prepared with Editor344 under request
`editor344-runtime416-performance-batch-20260830dh-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
