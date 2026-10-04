---
title: Runtime649 Preallocated Duplicate Diagnostics
category: zircon_runtime
report_id: Runtime649-preallocated-duplicate-diagnostics-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime649 Preallocated Duplicate Diagnostics

Incremental asset-registry rebuilds now reserve the duplicate-GUID diagnostic vector from the
loaded metadata count. Each metadata document can contribute at most one duplicate diagnostic, so
the scan size is a strict upper bound and diagnostic order, remint behavior, persistence, and error
propagation remain unchanged.

The ignored Windows Release benchmark emits
`RUNTIME649_DUPLICATE_DIAGNOSTIC_CAPACITY_BENCH_V1` over 21 alternating sample pairs, 64 batches per
sample, and 4,096 diagnostic entries per batch. The gate requires reserved P95 to be at most 80% of
unreserved P95.

No direct Cargo validation was run. The coordinator owns combined Runtime649/Editor649 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime649 is prepared with Editor649 under the shared `optimization_batch_jj_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
