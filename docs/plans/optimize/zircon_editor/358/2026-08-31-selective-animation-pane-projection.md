---
title: Editor358 Selective Animation Pane Projection
category: zircon_editor
report_id: Editor358-selective-animation-pane-projection-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor358 Selective Animation Pane Projection

Animation sequence and graph payload builders now project directly from the borrowed editor
presentation. Each builder copies only the strings and item lists present in its output contract,
instead of cloning the complete presentation and immediately dropping fields owned by the other
pane mode.

The pane payload schema and missing-presentation defaults remain unchanged. Regression tests cover
both sequence and graph field mappings, their empty defaults, and the absence of the full
presentation clone in production source.

The ignored Windows Release benchmark emits
`EDITOR358_SELECTIVE_ANIMATION_PANE_PROJECTION_BENCH_V1` over 17 alternating paired samples, each
building 128 sequence/graph payload pairs from five 256-item mode lists, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gates.

## Current batched validation handoff (2026-08-31)

Editor358 is prepared with Runtime429, Runtime430, and Editor357 under request
`runtime429-430-editor357-358-performance-batch-20260831dv-v1`. This recovery batch retains the
`optimization_batch_du_` filter because the earlier request stopped at artifact governance before
compilation. Receipt, validation ticket, measured p95, pushed SHA, and notification result are
recorded only after coordinator completion.
