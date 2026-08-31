---
title: Runtime467 Borrowed Runtime Camera Key Lookup
category: zircon_runtime
report_id: Runtime467-borrowed-runtime-camera-key-lookup-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime467 Borrowed Runtime Camera Key Lookup

Viewport Hybrid GI and virtual geometry runtime lookup now probes each camera history map with the
borrowed key. The key is cloned only when a provider state is first created, removing repeated key
copies and shared render-layer `Arc` strong-count traffic from existing per-camera runtime access.

Regression coverage requires the old `entry(key.clone())` hit path to be absent and verifies that a
provider creates one state across repeated lookups. The ignored Windows Release benchmark emits
`RUNTIME467_BORROWED_RUNTIME_KEY_LOOKUP_BENCH_V1` over 17 alternating paired samples, each performing
262,144 existing-key lookups with two shared layer sets. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime467 is prepared with Editor397 under request
`runtime467-editor397-performance-batch-20260831fk-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
