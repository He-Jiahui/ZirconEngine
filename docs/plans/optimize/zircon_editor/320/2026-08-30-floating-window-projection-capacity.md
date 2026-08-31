---
title: Editor320 Floating Window Projection Capacity
category: zircon_editor
report_id: Editor320-floating-window-projection-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor320 Floating Window Projection Capacity

`collect_floating_windows_with_template_v2_data` now reserves the floating-window output length,
and `floating_window_data` reserves the per-window tab output length. Window projection data,
active-pane behavior, tab conversion, and source ordering are unchanged.

Regression coverage is included in the combined Runtime/Editor batch. The ignored Windows Release
benchmark emits `EDITOR320_FLOATING_WINDOW_PROJECTION_CAPACITY_BENCH_V1` over 17 paired samples
with 64 windows and 16 tabs per window, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor320 is submitted in the six-task batch under request
`runtime372-374-editor318-320-performance-batch-20260830-v4`, ticket
`4eab46c0a22440dcbb177cd77dcb2b88` (superseded by the corrected v4 manifest), with source
manifest details are recorded in the session submission log after acceptance. Cargo, performance,
review, commit, push, and WeCom remain coordinator-owned.

## Validation attempt (2026-08-30)

The corresponding batch produced no valid Cargo, performance, or commit evidence; no successful
WeCom notification was sent.
