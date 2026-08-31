---
title: Runtime415 Stable Morph Roll In Place
category: zircon_runtime
report_id: Runtime415-stable-morph-roll-in-place-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime415 Stable Morph Roll In Place

GPU Scene previous-morph-weight publication now updates its existing map in place. Stable keys
retain the same `Arc<[f32]>`, changed keys replace only their value, and removed keys are counted
while being retained out. A second insertion pass runs only when the retained map is smaller than
the current map, so stable frames need one map traversal.

The previous path first scanned all previous keys to count removals, then cleared every entry and
rebuilt the map from current state. Even an unchanged frame therefore performed two full hash-map
passes plus an atomic `Arc` decrement/increment pair and reinsertion for every weight state. The
new stable path performs no map rebuild and no `Arc` reference-count churn while preserving exact
removed/current/previous counts.

The ignored Windows Release benchmark emits
`RUNTIME415_STABLE_MORPH_ROLL_IN_PLACE_BENCH_V1` over 17 alternating paired samples, each rolling
2,048 stable weight states 64 times, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime415 is prepared with Runtime414 under request
`runtime414-runtime415-performance-batch-20260830dg-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
