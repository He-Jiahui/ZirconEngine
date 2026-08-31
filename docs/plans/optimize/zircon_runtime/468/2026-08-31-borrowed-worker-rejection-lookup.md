---
title: Runtime468 Borrowed Worker Rejection Lookup
category: zircon_runtime
report_id: Runtime468-borrowed-worker-rejection-lookup-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime468 Borrowed Worker Rejection Lookup

Asset worker duplicate-request admission now borrows the in-flight completion entry while deciding
whether another observer can join. The successful join still performs its one required `Arc`
clone, while a full observer budget no longer performs an unnecessary atomic increment/decrement
pair and a terminal entry is removed directly from the registry.

Regression coverage holds an actual worker request in flight with `waiter_capacity=1`, verifies the
second observer retains the existing typed rejection, and requires the old cloned lookup to be
absent. The ignored Windows Release benchmark emits
`RUNTIME468_BORROWED_WORKER_REJECTION_LOOKUP_BENCH_V1` over 17 alternating paired samples, each
performing 262,144 lookups across 64 entries. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.85` (at least 15% lower P95) and records the reduction from
524,288 legacy `Arc` reference-count operations per sample to zero on the rejection lookup.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime468 is prepared with Editor398 under request
`runtime468-editor398-performance-batch-20260831fl-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
