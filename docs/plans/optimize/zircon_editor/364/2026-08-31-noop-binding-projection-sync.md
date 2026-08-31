---
title: Editor364 No-op Binding Projection Sync
category: zircon_editor
report_id: Editor364-noop-binding-projection-sync-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor364 No-op Binding Projection Sync

UI asset binding mutations now synchronize the editor instance projection only when the session
reports an actual state change. Reapplying the selected binding, route, action, or payload value no
longer rebuilds the same retained projection.

All 16 binding mutation entry points preserve their existing session operation, error mapping,
lock release, and boolean return value. A binding-local guard owns the projection-sync decision, so
no-op edits return without entering the expensive synchronization path. Initial session restore
continues to publish its projection in the lifecycle owner before these methods return.

The ignored Windows Release benchmark emits `EDITOR364_NOOP_BINDING_PROJECTION_SYNC_BENCH_V1`
over 17 alternating paired samples. Each sample models 64 unchanged binding actions against a
1,024-row projection: the legacy path performs 64 projection copies and the optimized path performs
zero. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor364 is prepared with Runtime436 under request
`runtime436-editor364-performance-batch-20260831eb-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
