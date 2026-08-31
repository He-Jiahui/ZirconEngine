---
title: Runtime460 Direct Script Callback ID
category: zircon_runtime
report_id: Runtime460-direct-script-callback-id-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime460 Direct Script Callback ID

Script behavior callback references now build their provider-qualified stable ID directly into one
exact-capacity string. The script bridge no longer invokes generic formatting for the fixed
`package::node` identity grammar used by diagnostics and cross-plugin callback resolution.

Separator placement and package/node bytes remain unchanged for short and long identifiers.
Existing validation continues to reject empty or untrimmed components before an ID can be built.

The ignored Windows Release benchmark emits `RUNTIME460_DIRECT_SCRIPT_CALLBACK_ID_BENCH_V1` over
17 alternating paired samples, each building 262,144 representative callback IDs. The gate
requires `optimized_p95_ns <= legacy_p95_ns * 0.70` (at least 30% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime460 is prepared with Editor390 under request
`runtime460-editor390-performance-batch-20260831fb-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
