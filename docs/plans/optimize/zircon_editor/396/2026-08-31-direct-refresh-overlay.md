---
title: Editor396 Direct Refresh Overlay
category: zircon_editor
report_id: Editor396-direct-refresh-overlay-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor396 Direct Refresh Overlay

Retained-host refresh diagnostics now allocate one output buffer, preserve the existing one-decimal
FPS formatter, and append all seven integer counters directly. Startup text and the exact
`FPS ... | present ... | ... | paint-only ...` contract remain unchanged while repeated generic
integer formatting is removed from the refresh overlay path.

Regression coverage compares representative FPS values and counters with the former full formatter
and retains the zero-present startup overlay. The ignored Windows Release benchmark emits
`EDITOR396_DIRECT_REFRESH_OVERLAY_BENCH_V1` over 17 alternating paired samples, each building
131,072 overlays. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower
P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor396 is prepared with Runtime466 under request
`runtime466-editor396-performance-batch-20260831fj-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
