---
title: Runtime495 Event Catalog Owner Byte Boundary
category: zircon_runtime
report_id: Runtime495-event-catalog-owner-byte-boundary-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime495 Event Catalog Owner Byte Boundary

Runtime plugin event-catalog ownership validation now compares the owner prefix and its dot
boundary directly in bytes. The prior implementation created a suffix view and ran a second prefix
check; owner and non-owner namespace semantics remain unchanged, including empty-owner behavior.

Regression coverage exercises exact owner boundaries, missing suffixes, and owner-prefix
collisions. The ignored Windows Release benchmark emits
`RUNTIME495_EVENT_CATALOG_OWNER_BYTE_BOUNDARY_BENCH_V1` over 100,000 nested namespaces. The gate
requires `optimized_p95_ns <= legacy_p95_ns * 0.90` (at least 10% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime495 is prepared with Editor425 under request
`runtime495-editor425-performance-batch-20260831gm-v1`. Receipt, validation ticket, measured
p95, pushed SHA, and notification result are recorded only after coordinator completion.
