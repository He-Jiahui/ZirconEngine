---
title: Editor501 Selection Root Capacity
category: zircon_editor
report_id: Editor501-selection-root-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor501 Selection Root Capacity

Interactive-transform root selection previously grew both its deduplication set and authored-order
vector from zero. The selector now reads the input iterator lower bound and reserves both stores
before its existing single pass. Duplicate filtering, authored order, ancestor filtering, and
primary-root selection are unchanged; iterators without a useful lower bound retain zero-capacity
behavior.

The source regression requires both reservations. The ignored Windows Release benchmark emits
`EDITOR501_SELECTION_ROOT_CAPACITY_BENCH_V1` for 32,768 unique selections and requires zero
optimized set/vector growth events versus a positive legacy count, which is a 100% growth-event
reduction.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, focused tests, ignored performance evidence, manifest-only commit/push, and one-shot
WeCom publication after the declared gates pass.

## Current batched validation handoff (2026-08-30)

Editor501 is batched with Runtime501 under request
`runtime501-animation-graph-editor501-selection-root-capacity-20260830cn-v1`. Receipt, ticket,
source manifest, and terminal evidence are recorded after coordinator acceptance.
