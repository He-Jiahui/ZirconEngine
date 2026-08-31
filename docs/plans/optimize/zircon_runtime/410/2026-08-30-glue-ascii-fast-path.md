---
title: Runtime410 Glue ASCII Fast Path
category: zircon_runtime
report_id: Runtime410-glue-ascii-fast-path-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime410 Glue ASCII Fast Path

Glyph fallback eligibility now accepts all-ASCII text through `str::is_ascii` before Unicode
decoding. Every glue character and variation selector in this contract is non-ASCII. Non-ASCII text
now checks both predicates in one character pass instead of decoding the string twice.

The ignored Windows Release benchmark emits `RUNTIME410_GLUE_ASCII_FAST_PATH_BENCH_V1` over 17
alternating paired samples with 65,536 ASCII bytes, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime410 is submitted with Runtime411 under request
`runtime410-runtime411-performance-batch-20260830dd-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
