---
title: Editor365 No-op Style Projection Sync
category: zircon_editor
report_id: Editor365-noop-style-projection-sync-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor365 No-op Style Projection Sync

UI asset style mutations now synchronize the editor instance projection only when the session
reports an actual state change. Reapplying a selected theme, class, token, rule, declaration, or
refactor value no longer rebuilds the same retained projection.

The 25 previously unconditional style synchronization sites preserve their session operation,
error mapping, lock release, and boolean result while routing through a style-local guard. The two
theme-source operations that already guarded hydration and synchronization remain structurally
unchanged, including their import refresh ordering.

The ignored Windows Release benchmark emits `EDITOR365_NOOP_STYLE_PROJECTION_SYNC_BENCH_V1` over
17 alternating paired samples. Each sample models 64 unchanged style actions against a 1,024-row
projection: the legacy path performs 64 projection copies and the optimized path performs zero.
The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor365 is prepared with Runtime437 under request
`runtime437-editor365-performance-batch-20260831ec-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
