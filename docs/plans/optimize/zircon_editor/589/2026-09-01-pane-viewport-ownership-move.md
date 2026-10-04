---
title: Editor589 Pane Viewport Ownership Move
category: zircon_editor
report_id: Editor589-pane-viewport-ownership-move-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor589 Pane Viewport Ownership Move

Retained-host viewport conversion now consumes `SceneViewportChromeData` and moves its ten
`SharedString` fields into the host contract. Both callers already owned the source value: pane
projection consumes `PaneData`, while toolbar synchronization creates a temporary viewport model.
The pane path also moves its owned project-overview model instead of cloning its `ModelRc` before
the original is discarded.

Viewport flags, snap values, labels, and empty toolbar frame remain unchanged. Both call sites are
covered by a source contract, and the new evidence fixture was split into a dedicated test module
so `apply_presentation.rs` remains an 839-line orchestration boundary.

The ignored Windows Release benchmark emits `EDITOR589_VIEWPORT_OWNERSHIP_MOVE_BENCH_V1` over 21
alternating sample pairs and 32,768 viewport conversions per sample. The legacy model performs ten
shared-string clones per viewport; the optimized model performs none. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor589 is prepared with Runtime589 under request
`runtime589-editor589-cache-viewport-performance-20260901hm-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
