---
title: Runtime482 Package ID Edge Scan
category: zircon_runtime
report_id: Runtime482-package-id-edge-scan-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime482 Package ID Edge Scan

Runtime plugin package-ID validation now checks only the first and last Unicode scalar for outer
whitespace instead of trimming the complete ID twice. Empty input, lowercase dotted-segment
validation, and the existing diagnostic remain unchanged.

Regression coverage verifies leading, trailing, and Unicode whitespace rejection together with a
valid dotted package ID. The ignored Windows Release benchmark emits
`RUNTIME482_PACKAGE_ID_EDGE_SCAN_BENCH_V1` over 17 alternating paired samples, a package ID with a
2 MiB trailing-whitespace suffix, and 1,024 checks per sample. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.25` (at least 75% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime482 is prepared with Editor412 under request
`runtime482-editor412-performance-batch-20260831fz-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
