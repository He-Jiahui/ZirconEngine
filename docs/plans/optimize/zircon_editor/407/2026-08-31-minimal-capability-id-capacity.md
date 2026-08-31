---
title: Editor407 Minimal Capability ID Capacity
category: zircon_editor
report_id: Editor407-minimal-capability-id-capacity-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor407 Minimal Capability ID Capacity

Editor minimal-host capability projection now reserves the exact six-entry result capacity before
converting static capability IDs to owned strings. Capability order, returned values, extension
blacklist checks, and the public `Vec<String>` API are unchanged.

Regression coverage requires the exact capacity reservation. The ignored Windows Release benchmark
emits `EDITOR407_MINIMAL_CAPABILITY_IDS_BENCH_V1` over 17 alternating paired samples and 65,536
projections per sample. The common path removes two vector growth allocations per projection. The
gate requires `optimized_p95_ns <= legacy_p95_ns * 0.80` (at least 20% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor407 is prepared with Runtime477 under request
`runtime477-editor407-performance-batch-20260831fu-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
