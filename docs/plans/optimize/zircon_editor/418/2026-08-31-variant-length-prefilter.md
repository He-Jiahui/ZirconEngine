---
title: Editor418 Variant Length Prefilter
category: zircon_editor
report_id: Editor418-variant-length-prefilter-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor418 Variant Length Prefilter

Retained-host dialog variant matching now rejects expected values with a different byte length
before performing case-insensitive comparison. The five variant sources, whitespace tokenization,
case-insensitive matches, and priority order are unchanged.

Regression coverage verifies case-insensitive matches and length-mismatched non-matches. The
ignored Windows Release benchmark emits `EDITOR418_VARIANT_LENGTH_PREFILTER_BENCH_V1` over
100,000 repeated negative lookups. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.90` (at least 10% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor418 is prepared with Runtime488 under request
`runtime488-editor418-performance-batch-20260831gf-v1`. Receipt, validation ticket, measured
p95, pushed SHA, and notification result are recorded only after coordinator completion.
