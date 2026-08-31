---
title: Runtime499 V1 NavMesh Link Id Preflight
category: zircon_runtime
report_id: Runtime499-v1-link-id-preflight-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime499 V1 NavMesh Link Id Preflight

Version-one NavMesh migration now validates the complete off-mesh-link count once before entering
the conversion loop. Dense one-based `u32` link identifiers are then assigned without repeating a
checked integer conversion for every link. Empty input, the maximum representable link count, and
the prior oversized-input error payload remain unchanged.

Regression coverage verifies dense numbering and both sides of the `u32` boundary. The ignored
Windows Release benchmark emits `RUNTIME499_V1_LINK_ID_PREFLIGHT_BENCH_V1` across 31 samples of
100,000 identifiers. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.90` (at least 10% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-08-31)

Runtime499 is prepared with Editor429 under request
`runtime499-editor429-performance-batch-20260831gq-v1`. Receipt, validation ticket, measured
P95, pushed SHA, and notification result are recorded only after coordinator completion.
