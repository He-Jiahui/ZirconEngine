---
title: Runtime479 Crate Name Edge Scan
category: zircon_runtime
report_id: Runtime479-crate-name-edge-scan-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime479 Crate Name Edge Scan

Runtime plugin descriptor crate-name validation now checks only the first and last Unicode scalar
for outer whitespace instead of calling `trim()` twice. Empty input is still rejected explicitly,
and prefix, lowercase-token, trailing-underscore, and repeated-underscore diagnostics are
unchanged. This preserves the existing boundary rule while avoiding repeated full-string scans for
malformed names with long whitespace runs.

Regression coverage verifies empty, ASCII, and Unicode outer whitespace cases plus a valid crate
name. The ignored Windows Release benchmark emits
`RUNTIME479_DESCRIPTOR_CRATE_NAME_EDGE_SCAN_BENCH_V1` over 17 alternating paired samples, a
2,048-byte whitespace-heavy name, and 1,024 lookups per sample. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.25` (at least 75% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime479 is prepared with Editor409 under request
`runtime479-editor409-performance-batch-20260831fw-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
