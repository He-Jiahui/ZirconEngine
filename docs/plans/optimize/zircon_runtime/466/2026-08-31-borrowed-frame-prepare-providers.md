---
title: Runtime466 Borrowed Frame Prepare Providers
category: zircon_runtime
report_id: Runtime466-borrowed-frame-prepare-providers-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime466 Borrowed Frame Prepare Providers

Hybrid GI and virtual geometry frame preparation now borrow their registered runtime providers while
mutating only the disjoint viewport map. A map-scoped generation guard preserves the existing
unknown-viewport and stale-generation errors while removing two `Arc` strong-count increments and
decrements from every frame where both features are enabled.

Structural regression coverage requires both production provider accesses to use borrowed
registrations and keeps one mutable viewport lookup in the generation guard. The ignored Windows
Release benchmark emits `RUNTIME466_BORROWED_PREPARE_PROVIDERS_BENCH_V1` over 17 alternating paired
samples, each modeling 262,144 frames with two providers. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.20` (at least 80% lower P95 for provider access).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime466 is prepared with Editor396 under request
`runtime466-editor396-performance-batch-20260831fj-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
