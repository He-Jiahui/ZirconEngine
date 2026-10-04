---
title: Runtime602 Sprite Phase Stream
category: zircon_runtime
report_id: Runtime602-sprite-phase-stream-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime602 Sprite Phase Stream

The default sprite extract path now streams phase inputs directly from its owned sprite snapshot
array into `build_sprite_phase_queue`. The previous path first allocated and filled a complete
`Vec<SpritePhaseExtractInput>`, traversed it once, and immediately discarded it. Entity, source
index, alpha-mode queue resolution, z order, depth, and every default ordering field are preserved.
The explicit phase-input entry point remains unchanged for callers that supply queue overrides.

A focused equivalence test compares the default path with the explicit phase-input path across
multiple sprites and ordering values. A source guard requires direct queue construction without the
intermediate input collection. The ignored Windows Release benchmark emits
`RUNTIME602_SPRITE_PHASE_STREAM_BENCH_V1` over 17 alternating sample pairs and 65,536 sprite inputs.
The gate requires streamed P95 to be at most 80% of the legacy buffered path.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime602 is prepared with Editor602 under request
`runtime602-editor602-sprite-command-performance-20260901ht-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
