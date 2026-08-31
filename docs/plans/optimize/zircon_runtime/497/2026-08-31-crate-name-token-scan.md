---
title: Runtime497 Crate Name Token Scan
category: zircon_runtime
report_id: Runtime497-crate-name-token-scan-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime497 Crate Name Token Scan

Runtime module crate-name validation now checks the required `zircon_plugin_` prefix and the
remaining lowercase ASCII token bytes without rescanning the prefix through a second predicate.
The accepted and rejected crate-name set remains unchanged.

Regression coverage exercises valid names, missing prefixes, uppercase bytes, and punctuation. The
ignored Windows Release benchmark emits `RUNTIME497_CRATE_NAME_TOKEN_SCAN_BENCH_V1` over 100,000
valid crate-name lookups. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.90` (at least 10% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime497 is prepared with Editor427 under request
`runtime497-editor427-performance-batch-20260831go-v1`. Receipt, validation ticket, measured
p95, pushed SHA, and notification result are recorded only after coordinator completion.
