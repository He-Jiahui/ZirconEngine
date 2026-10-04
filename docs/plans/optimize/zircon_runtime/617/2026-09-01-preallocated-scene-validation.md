---
title: Runtime617 Preallocated Scene Validation
category: zircon_runtime
report_id: Runtime617-preallocated-scene-validation-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime617 Preallocated Scene Validation

Dynamic-scene validation now reserves its entity-source and component-type membership indexes from
the corresponding input lengths. Both sets previously started empty and repeatedly reallocated
while validating a supported scene even though their maximum sizes were already known.

Entity traversal order, component descriptor validation order, borrowed component type IDs, and
first-duplicate diagnostics are unchanged. Existing behavior coverage still locks the first
duplicate source and borrowed identity contract; focused source coverage rejects unreserved
production sets.

The ignored Windows Release benchmark emits `RUNTIME617_PREALLOCATED_SCENE_VALIDATION_BENCH_V1`
over 17 alternating sample pairs with 32,768 unique entity sources and 32,768 long component type
IDs. The gate requires reserved validation P95 to be at most 85% of unreserved validation P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime617 is prepared with Editor617 under request
`runtime617-editor617-scene-plugin-admission-performance-20260901ig-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
