---
title: Runtime475 Endpoint Module Name Validation
category: zircon_runtime
report_id: Runtime475-endpoint-module-name-validation-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime475 Endpoint Module Name Validation

Runtime module registration now validates canonical names by decoding only the first and last
Unicode scalar values and rejecting either endpoint when it is whitespace. Empty names, leading
or trailing Unicode whitespace, and internal whitespace preserve the prior `trim() == name`
semantics. Descriptor validation and the registration transaction are unchanged.

This removes a length-dependent scan when a malformed name has a long whitespace prefix or
suffix. Regression coverage checks empty, ASCII, tab, and Unicode whitespace cases. The ignored
Windows Release benchmark emits `RUNTIME475_ENDPOINT_MODULE_NAME_REJECTION_BENCH_V1` over 17
alternating paired samples and 32,768 checks per sample. Its 2,048-byte rejection input has 2,047
leading whitespace bytes: the legacy path examines the prefix while the optimized path rejects
after one endpoint character. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.25` (at
least 75% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime475 is prepared with Editor405 under request
`runtime475-editor405-performance-batch-20260831fs-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
