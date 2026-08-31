---
title: Runtime490 Underscore Scan
category: zircon_runtime
report_id: Runtime490-underscore-scan-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime490 Underscore Scan

Runtime plugin package-id underscore validation now detects trailing and repeated underscores in
one byte-state-machine pass. The prior implementation split the dotted value and scanned each
segment with two string operations; the accepted and rejected segment rules remain unchanged.

Regression coverage exercises valid dotted identifiers and trailing/repeated underscore cases.
The ignored Windows Release benchmark emits `RUNTIME490_UNDERSCORE_SCAN_BENCH_V1` over 100,000
repeated valid identifiers. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.90` (at least 10% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime490 is prepared with Editor420 under request
`runtime490-editor420-performance-batch-20260831gh-v1`. Receipt, validation ticket, measured
p95, pushed SHA, and notification result are recorded only after coordinator completion.
