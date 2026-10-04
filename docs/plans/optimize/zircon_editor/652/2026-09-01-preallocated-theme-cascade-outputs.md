---
title: Editor652 Preallocated Theme Cascade Outputs
category: zircon_editor
report_id: Editor652-preallocated-theme-cascade-outputs-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor652 Preallocated Theme Cascade Outputs

Theme cascade inspection now reserves the layer vector for imported layers plus the optional local
layer, token output from the total token-definition count, and rule output from a strict two-items
per-rule upper bound. Layer, token, active/shadowed, and rule ordering remain unchanged.

The ignored Windows Release benchmark emits `EDITOR652_CASCADE_OUTPUT_CAPACITY_BENCH_V1` over 21
alternating sample pairs, 4,096 cascades per sample, eight layers, 256 tokens per layer, and 128
rules per layer. The gate requires reserved P95 to be at most 80% of unreserved P95.

No direct Cargo validation was run. The coordinator owns combined Runtime652/Editor652 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor652 is prepared with Runtime652 under the shared `optimization_batch_jm_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.

The follow-up [Editor822 theme-cascade test-wiring repair](2026-09-19-theme-cascade-test-wiring-repair.md)
now declares the existing lower regression from `theme_cascade_inspection.rs`, so the Editor652
capacity tests and ignored Release marker are reachable by Rust test discovery.
