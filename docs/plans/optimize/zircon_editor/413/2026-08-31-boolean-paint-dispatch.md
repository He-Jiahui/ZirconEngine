---
title: Editor413 Boolean Paint Dispatch
category: zircon_editor
report_id: Editor413-boolean-paint-dispatch-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor413 Boolean Paint Dispatch

Editor inspector shadow check/select painting now passes the authored value directly to the shared
boolean parser instead of trimming at both the caller and parser. The parser dispatches supported
tokens by byte length, replacing a linear candidate scan while preserving whitespace,
case-insensitive, numeric, and false-value semantics.

Regression coverage verifies both paint callers use the parser as the sole trim boundary and keeps
the existing complete supported-token matrix. The ignored Windows Release benchmark emits
`EDITOR413_BOOLEAN_PAINT_DISPATCH_BENCH_V1` over 17 alternating paired samples and 1,048,576
parses of the deepest supported token (`checked`) per sample. The legacy path performs two trim
calls and up to five candidate comparisons; the optimized path performs one of each. The gate
requires `optimized_p95_ns <= legacy_p95_ns * 0.65` (at least 35% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor413 is prepared with Runtime483 under request
`runtime483-editor413-performance-batch-20260831ga-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
