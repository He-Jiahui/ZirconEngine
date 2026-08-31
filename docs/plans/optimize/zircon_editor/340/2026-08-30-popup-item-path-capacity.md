---
title: Editor340 Popup Item Path Capacity
category: zircon_editor
report_id: Editor340-popup-item-path-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor340 Popup Item Path Capacity

Retained menu pointer routing now reserves the open submenu depth plus the terminal hit before
walking the popup hierarchy. Child-first hit testing, route-index lookup, returned item paths,
disabled-item handling, submenu routing, and popup-surface fallback remain unchanged while nested
pointer events avoid repeated path-vector growth.

The ignored Windows Release benchmark emits `EDITOR340_POPUP_ITEM_PATH_CAPACITY_BENCH_V1` over 17
alternating paired samples at depth 24, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor340 is submitted with Editor341 under request
`editor340-editor341-performance-batch-20260830cx-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
