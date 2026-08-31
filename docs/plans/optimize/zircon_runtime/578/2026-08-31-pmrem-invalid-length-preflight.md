---
title: Runtime578 PMREM Invalid-Length Preflight
category: zircon_runtime
report_id: Runtime578-pmrem-invalid-length-preflight-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime578 PMREM Invalid-Length Preflight

PMREM texture construction now validates the selected RGBA16F mip-chain length before copying
the artifact slice into an owned `Vec`. The same canonical length helper remains the decoder
authority, so valid payloads and all existing errors are unchanged while malformed payloads avoid
allocating and copying the full section.

Regression coverage checks exact and truncated lengths. The ignored Windows Release benchmark
emits `RUNTIME578_PMREM_INVALID_LENGTH_PREFLIGHT_BENCH_V1` over 31 alternating sample pairs of
250,000 invalid payload checks with a 128-face-size, eight-mip payload. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.90`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both batch gates pass.

## Current Batched Validation Handoff (2026-08-31)

Runtime578 is prepared with Editor578 under request
`runtime578-editor578-pmrem-overlay-performance-20260831gw-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
