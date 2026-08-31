---
title: Editor366 No-op Preview Projection Sync
category: zircon_editor
report_id: Editor366-noop-preview-projection-sync-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor366 No-op Preview Projection Sync

UI asset preview mutations that report a boolean change now synchronize the editor instance
projection only when state actually changed. Reapplying a preview preset, mock selection, nested
value, suggestion, or clear action no longer rebuilds the same retained projection.

All 10 boolean preview operations preserve their session call, error mapping, lock release, and
boolean return value while routing through a preview-local guard. The preview-index selection API
returns `()` and its session API does not publish a changed result, so its direct synchronization is
intentionally retained.

The ignored Windows Release benchmark emits `EDITOR366_NOOP_PREVIEW_PROJECTION_SYNC_BENCH_V1`
over 17 alternating paired samples. Each sample models 64 unchanged preview actions against a
1,024-row projection: the legacy path performs 64 projection copies and the optimized path performs
zero. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor366 is prepared with Runtime438 under request
`runtime438-editor366-performance-batch-20260831ed-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
