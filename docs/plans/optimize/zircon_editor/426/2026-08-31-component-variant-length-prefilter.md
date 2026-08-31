---
title: Editor426 Component Variant Length Prefilter
category: zircon_editor
report_id: Editor426-component-variant-length-prefilter-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor426 Component Variant Length Prefilter

Shared retained-host component variant matching now rejects tokens with a different byte length
before case-insensitive comparison. Existing ASCII whitespace and punctuation tokenization,
case-insensitive matching, and token order remain unchanged for all callers.

Regression coverage verifies exact and case-insensitive matches plus a length-mismatched negative.
The ignored Windows Release benchmark emits
`EDITOR426_COMPONENT_VARIANT_LENGTH_PREFILTER_BENCH_V1` over 100,000 repeated negative lookups.
The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.90` (at least 10% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor426 is prepared with Runtime496 under request
`runtime496-editor426-performance-batch-20260831gn-v1`. Receipt, validation ticket, measured
p95, pushed SHA, and notification result are recorded only after coordinator completion.
