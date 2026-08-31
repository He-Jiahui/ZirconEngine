---
title: Editor575 Model Projection Metadata
category: zircon_editor
report_id: Editor575-model-projection-metadata-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor575 Model Projection Metadata

Retained pane model projection now uses `ModelRc::map_preserving_metadata` directly. This fixes
loss of retained generation metadata and removes the temporary `Rc<VecModel<_>>` allocation used
by the generic layout constructor. Projection still borrows source rows and creates one contiguous
result allocation; wrapper allocations fall from two to one per projection.

Regression coverage verifies mapped values and pointer-identical metadata sharing. The ignored
Windows Release benchmark emits `EDITOR575_MODEL_PROJECTION_METADATA_BENCH_V1` across 31
alternating sample pairs of 50,000 eight-row projections. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.90` (at least 10% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both batch gates pass.

## Current Batched Validation Handoff (2026-08-31)

Editor575 is prepared with Runtime575 under request
`runtime575-editor575-performance-batch-20260831gt-v1`. Receipt, validation ticket, measured P95,
pushed SHA, and notification result are recorded only after coordinator completion.
