---
title: Runtime604 Pointer ID Index Union
category: zircon_runtime
report_id: Runtime604-pointer-id-index-union-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime604 Pointer ID Index Union

`PickingPipelineReport` now derives its canonical pointer-ID set from the keys of the ray-count and
output-count `BTreeMap` indexes that the report has already built. The previous path scanned every
raw ray and backend output again and inserted repeated pointer IDs into another tree. Pointer order,
per-pointer counts, empty-hit outputs, report totals, blocking state, and hit summaries are
unchanged.

A focused behavior test verifies the sorted union across overlapping count indexes. A source guard
requires both index key streams and rejects access to raw `RayMap` and `PointerHits` collections in
the union helper. The ignored Windows Release benchmark emits
`RUNTIME604_POINTER_ID_INDEX_UNION_BENCH_V1` over 17 alternating sample pairs with 64 unique
pointers represented by 65,536 rays and 65,536 outputs. The modeled pointer-ID visits fall from
131,072 to 128, and the gate requires indexed-union P95 to be at most 10% of the legacy raw-scan
P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime604 is prepared with Editor604 under request
`runtime604-editor604-pointer-keymap-performance-20260901hv-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
