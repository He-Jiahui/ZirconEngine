---
title: Editor587 Editor Pane Single Pass
category: zircon_editor
report_id: Editor587-editor-pane-single-pass-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor587 Editor Pane Single Pass

Retained-host lifecycle recomputation now consumes its shared view-instance snapshot once while
collecting UI-asset and animation editor pane presentations. The former path scanned the same
snapshot separately for each visible pane family and cloned every matching instance ID into the
result maps. The combined collector routes both families in one pass and moves each owned ID.

Visibility gates remain intact: a pane family is queried only when that family is visible, failed
presentation lookups remain omitted, unrelated descriptors remain ignored, and both result maps
retain their existing key and value types.

The ignored Windows Release benchmark emits `EDITOR587_EDITOR_PANE_SINGLE_PASS_BENCH_V1` over 21
alternating sample pairs and 24,576 mixed view instances per sample. The legacy model performs two
snapshot passes and 16,384 ID clones; the optimized model performs one pass and zero ID clones. The
gate requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor587 is prepared with Runtime587 under request
`runtime587-editor587-shader-pane-performance-20260901hk-v1`. Receipt, validation ticket, measured
P95, pushed SHA, and notification result are recorded only after coordinator completion.
