---
title: Runtime439 Final Widget Token Map Move
category: zircon_runtime
report_id: Runtime439-final-widget-token-map-move-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime439 Final Widget Token Map Move

Widget style collection still composes one resolved token map per widget asset, but now moves that
map into the final resolved stylesheet. The old loop cloned the complete `BTreeMap` for every
stylesheet and then dropped the original map. The new `split_last` path clones only the preceding
stylesheets and transfers ownership to the last one.

Stylesheet order, stylesheet values, resolved token values, duplicate-widget suppression, and empty
stylesheet behavior remain unchanged. A widget with one stylesheet now performs zero token-map
clones instead of one; a widget with N stylesheets performs N-1 clones instead of N. Regression
coverage checks order, shared values, empty input, and the ownership-transfer implementation.

The ignored Windows Release benchmark emits
`RUNTIME439_FINAL_WIDGET_TOKEN_MAP_MOVE_BENCH_V1` over 17 alternating paired samples. Each sample
builds 32 single-stylesheet projections with 512 resolved tokens. The legacy path deep-clones
16,384 token entries per sample after the common fixture copy; the optimized path clones none. The
gate requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime439 is prepared with Editor367 under request
`runtime439-editor367-performance-batch-20260831ee-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
