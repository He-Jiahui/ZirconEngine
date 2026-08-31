---
title: Runtime464 Direct Window Lifecycle Diagnostics
category: zircon_runtime
report_id: Runtime464-direct-window-lifecycle-diagnostics-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime464 Direct Window Lifecycle Diagnostics

Window lifecycle diagnostics now map the closed enum and boolean domains to static text and append
each value into an exact-capacity output. The public two-line snapshot keeps the previous debug and
boolean bytes while avoiding generic formatting on application and platform diagnostic collection.

Regression coverage compares every exit-condition spelling and both close-request states with the
former formatter. The ignored Windows Release benchmark emits
`RUNTIME464_DIRECT_LIFECYCLE_DIAGNOSTICS_BENCH_V1` over 17 alternating paired samples, each building
262,144 two-line snapshots. The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.80` (at least
20% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime464 is prepared with Editor394 under request
`runtime464-editor394-performance-batch-20260831fh-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
