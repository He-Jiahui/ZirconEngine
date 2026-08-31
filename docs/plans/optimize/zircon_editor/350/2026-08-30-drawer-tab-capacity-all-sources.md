---
title: Editor350 Drawer Tab Capacity All Sources
category: zircon_editor
report_id: Editor350-drawer-tab-capacity-all-sources-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor350 Drawer Tab Capacity All Sources

Drawer-tab profiling now includes tab rows from every floating window in its initial capacity,
alongside the left, right, and bottom docks. The total remains saturating and preserves the
existing row traversal and visible-frame filtering.

The previous capacity covered only the three fixed docks. Floating-window tabs were appended after
the vector was created and could grow it repeatedly; counting their model rows before collection
keeps the normal drawer projection within one allocation.

The ignored Windows Release benchmark emits
`EDITOR350_DRAWER_TABS_ALL_SOURCES_CAPACITY_BENCH_V1` over 17 alternating paired samples, each
building 2,048 batches with 384 fixed-dock and 128 floating-window tabs, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor350 is prepared with Runtime422 under request
`runtime422-editor350-performance-batch-20260830dn-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
