---
title: Runtime388 Vertical SDF Advance Capacity
category: zircon_runtime
report_id: Runtime388-vertical-sdf-advance-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime388 Vertical SDF Advance Capacity

Unshaped vertical SDF text now reserves its natural-advance vector from the glyph count before
zipping text characters with glyphs. Character/glyph truncation, mixed vertical advance
calculation, invalid-advance handling, atlas lookup, clipping, cursor movement, and emitted vertex
order remain unchanged.

The ignored Windows Release benchmark emits `RUNTIME388_VERTICAL_SDF_ADVANCE_CAPACITY_BENCH_V1`
over 17 paired samples with 512 common ASCII glyphs per sample, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime388 is submitted with Editor334 under request
`runtime388-editor334-performance-batch-20260830cl-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
