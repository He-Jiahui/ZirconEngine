---
title: Runtime493 Root Relative Prefix
category: zircon_runtime
report_id: Runtime493-root-relative-prefix-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime493 Root Relative Prefix

Runtime plugin package root validation now checks the first byte once for Unix or Windows absolute
path separators. The prior implementation ran two prefix scans; relative and absolute path
semantics remain unchanged.

Regression coverage exercises ordinary relative roots, dot-relative roots, and both separator
forms. The ignored Windows Release benchmark emits `RUNTIME493_ROOT_RELATIVE_PREFIX_BENCH_V1` over
100,000 repeated relative-root lookups. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.90` (at least 10% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime493 is prepared with Editor423 under request
`runtime493-editor423-performance-batch-20260831gk-v1`. Receipt, validation ticket, measured
p95, pushed SHA, and notification result are recorded only after coordinator completion.
