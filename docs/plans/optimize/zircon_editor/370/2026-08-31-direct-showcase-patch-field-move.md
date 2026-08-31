---
title: Editor370 Direct Showcase Patch Field Move
category: zircon_editor
report_id: Editor370-direct-showcase-patch-field-move-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor370 Direct Showcase Patch Field Move

Showcase component event adaptation now keeps the resolved event intact while dispatching it to
state, then moves the control id and changed property into the projection patch. It also formats the
changed value text before moving the value into patch state. The former path cloned all three owned
fields to keep them available after their first use.

Event resolution, state dispatch, patch control id, state property/value, display text, and unchanged
event behavior remain unchanged. Regression coverage requires dispatch to precede field movement
and rejects restoration of any of the three production clones.

The ignored Windows Release benchmark emits `EDITOR370_DIRECT_SHOWCASE_PATCH_FIELD_MOVE_BENCH_V1`
over 17 alternating paired samples. Each sample models 8,192 projections with long control,
property, and value strings. The legacy model performs 24,576 extra field clones per sample; the
optimized model performs zero. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor370 is prepared with Runtime442 under request
`runtime442-editor370-performance-batch-20260831eh-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
