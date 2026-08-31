---
title: Editor323 Preview Projection Capacity
category: zircon_editor
report_id: Editor323-preview-projection-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor323 Preview Projection Capacity

Asset preview presentation now reserves exact lengths for palette slot targets, palette chooser
candidates, and canvas-node output while mapping the existing values explicitly. Preview ordering,
text formatting, selection fields, and UI performance counter behavior are unchanged.

Regression coverage checks all three output capacities and their slot-to-candidate-to-canvas order.
The ignored Windows Release benchmark emits `EDITOR323_PREVIEW_PROJECTION_CAPACITY_BENCH_V1` over
17 paired samples with 256 items per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor323 is submitted in the eight-task batch under request
`runtime375-378-editor321-324-performance-batch-20260830-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
