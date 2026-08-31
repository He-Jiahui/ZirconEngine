---
title: Runtime471 Borrowed Blocked Unload Dependents
category: zircon_runtime
report_id: Runtime471-borrowed-blocked-unload-dependents-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime471 Borrowed Blocked Unload Dependents

Core service unload preflight now retains borrowed `RegistryName` references while it determines
the earliest blocked service. It materializes the public owned service and dependent names only
after the winning blocked index is final. The existing scan order, lowest-index selection,
dependent ordering, return type, and error contract are unchanged.

This removes transient String copies that were immediately discarded whenever a later service
scan selected a lower unload index. Regression coverage proves the retained rows borrow the
original registry names and the final result still owns independent strings. The ignored Windows
Release benchmark emits `RUNTIME471_BORROWED_BLOCKED_DEPENDENTS_BENCH_V1` over 17 alternating
paired samples, 2,048 scans per sample, and six descending groups of 16 dependents. The pressure
case reduces name copies per scan from 97 to 17. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.60` (at least 40% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime471 is prepared with Editor401 under request
`runtime471-editor401-performance-batch-20260831fo-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
