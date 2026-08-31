---
title: Editor585 Grid Marker Suffix
category: zircon_editor
report_id: Editor585-grid-marker-suffix-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor585 Grid Marker Suffix

Viewport floor-grid painting now classifies major lines from the canonical two-byte suffix on their
control IDs. The four authored major lines end in `H2`, `H4`, `V2`, or `V5`; the former path ran up
to four full substring searches for every painted line. Glow colors, geometry, clip, order, surface
color, and opacity remain unchanged for the authored viewport grid.

Regression coverage verifies all four canonical major IDs, ordinary minor IDs, case sensitivity,
and rejection of a marker that appears only in the middle of a noncanonical ID. The ignored
Windows Release benchmark emits `EDITOR585_GRID_MARKER_SUFFIX_BENCH_V1` over 21 alternating sample
pairs and 32,768 classifications per sample. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.25`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both batch gates pass.

## Current Batched Validation Handoff (2026-08-31)

Editor585 is prepared with Runtime585 under request
`runtime585-editor585-shader-grid-performance-20260831hd-v1`. Receipt, validation ticket, measured
P95, pushed SHA, and notification result are recorded only after coordinator completion.
