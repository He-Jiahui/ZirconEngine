---
title: Runtime455 Direct Viewport Resource Key
category: zircon_runtime
report_id: Runtime455-direct-viewport-resource-key-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime455 Direct Viewport Resource Key

Every published GPU viewport product now constructs its `viewport:{id}:{generation}` resource key
in one exact-capacity string. The former per-frame publication path invoked generic formatting for
both decimal fields before the key entered the bounded generation registry.

Prefix bytes, separators, zero values, and the full `u64` viewport/generation range remain
unchanged. Regression coverage compares representative and boundary pairs with the former format
expression.

The ignored Windows Release benchmark emits `RUNTIME455_DIRECT_VIEWPORT_RESOURCE_KEY_BENCH_V1`
over 17 alternating paired samples, each constructing 262,144 keys with mixed viewport IDs and
generations. The optimized path performs direct decimal writes into one exact-capacity allocation.
The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime455 is prepared with Editor385 under request
`runtime455-editor385-performance-batch-20260831ew-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
