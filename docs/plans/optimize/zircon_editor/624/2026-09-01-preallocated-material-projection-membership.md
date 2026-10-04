---
title: Editor624 Preallocated Material Projection Membership
category: zircon_editor
report_id: Editor624-preallocated-material-projection-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor624 Preallocated Material Projection Membership

Material editor property and texture projection now reserve borrowed hash membership from the
loaded shader schema sizes. Both projection passes previously maintained ordered tree sets even
though the sets are only queried to suppress material overrides already represented by the shader.

Shader schema rows remain first and retain schema order. Unknown material overrides keep the
existing ordered-map iteration order, and names remain borrowed without per-entry string clones.
The no-shader path still allocates zero capacity.

The ignored Windows Release benchmark emits `EDITOR624_PREALLOCATED_MATERIAL_PROJECTION_BENCH_V1`
over 17 alternating sample pairs with 32,768 unique long schema names. The gate requires the
preallocated hash-membership P95 to be at most 40% of the ordered-tree P95.

No direct Cargo validation was run. The coordinator owns the combined Runtime624/Editor624 Windows
Release compile, focused regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both performance gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor624 is prepared with Runtime624 under request
`runtime624-editor624-volume-material-membership-performance-20260901in-v1`. Receipt, validation
ticket, measured P95, pushed SHA, and notification result are recorded only after coordinator
completion.
