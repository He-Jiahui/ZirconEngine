---
title: Runtime470 Short Config Load Lock
category: zircon_runtime
report_id: Runtime470-short-config-load-lock-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime470 Short Config Load Lock

Owned JSON config reads now clone the stored `Arc<Value>` while holding the config-store mutex and
perform the required deep `Value` clone after releasing the lock. Typed deserialization reuses the
same short lock step. The public API and owned-value semantics are unchanged, while concurrent
readers no longer serialize large JSON tree copies behind the global store lock.

Regression coverage verifies independent owned results and proves that the locked lookup shares
the stored allocation. The ignored Windows Release benchmark emits
`RUNTIME470_SHORT_CONFIG_LOAD_LOCK_BENCH_V1` over 17 alternating paired samples with four threads,
512 loads per thread, and 128 strings per JSON value. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95) and records the reduction from
2,048 deep JSON clones under the lock per sample to zero, replacing them with 2,048 short `Arc`
clones.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime470 is prepared with Editor400 under request
`runtime470-editor400-performance-batch-20260831fn-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
