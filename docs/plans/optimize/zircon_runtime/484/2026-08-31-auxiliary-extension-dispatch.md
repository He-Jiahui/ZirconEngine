---
title: Runtime484 Auxiliary Extension Dispatch
category: zircon_runtime
report_id: Runtime484-auxiliary-extension-dispatch-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime484 Auxiliary Extension Dispatch

Runtime project asset discovery now dispatches auxiliary source extensions by byte length before
performing case-insensitive comparison. The supported `bin`, `ttf`, `otf`, `woff`, and `woff2`
set is unchanged, while the longest extension no longer compares against four impossible lengths.

Regression coverage verifies mixed-case supported extensions, unsupported values, and full path
classification. The ignored Windows Release benchmark emits
`RUNTIME484_AUXILIARY_EXTENSION_DISPATCH_BENCH_V1` over 17 alternating paired samples and
1,048,576 checks of `woff2` per sample. The comparison count falls from five to one, and the gate
requires `optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime484 is prepared with Editor414 under request
`runtime484-editor414-performance-batch-20260831gb-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
