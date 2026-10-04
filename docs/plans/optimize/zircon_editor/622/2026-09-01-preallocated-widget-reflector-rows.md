---
title: Editor622 Preallocated Widget Reflector Rows
category: zircon_editor
report_id: Editor622-preallocated-widget-reflector-rows-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor622 Preallocated Widget Reflector Rows

Workbench widget-reflector row projection now reserves both the output vector and visited-node hash
set from the snapshot node count. The previous containers grew independently during tree and
orphan traversal even though the maximum number of emitted rows was already known.

Root traversal, cycle suppression, child depth, and ordered orphan completion are unchanged.
Existing behavior coverage locks the exact tree/orphan row order, while focused source coverage
locks both production capacities.

The ignored Windows Release benchmark emits `EDITOR622_PREALLOCATED_WIDGET_REFLECTOR_BENCH_V1`
over 17 alternating sample pairs with 65,536 visits across 8,192 unique node IDs. The gate requires
the jointly preallocated row/membership path P95 to be at most 85% of the unreserved path P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor622 is prepared with Runtime622 under request
`runtime622-editor622-owner-reflector-performance-20260901il-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
