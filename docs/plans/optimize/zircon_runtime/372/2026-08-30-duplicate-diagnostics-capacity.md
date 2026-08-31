---
title: Runtime372 Duplicate Diagnostics Capacity
category: zircon_runtime
report_id: Runtime372-duplicate-diagnostics-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime372 Duplicate Diagnostics Capacity

`project_duplicate_selection_diagnostics` now reserves one diagnostic-vector capacity per
manifest selection. Selection diagnostics still precede feature diagnostics, duplicate severity
classification is unchanged, and the original scan order is preserved.

Regression coverage checks both capacity reservations and selection-then-feature ordering. The
ignored Windows Release benchmark emits `RUNTIME372_DUPLICATE_DIAGNOSTICS_CAPACITY_BENCH_V1` over
17 paired samples with 1,024 selections per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime372 is submitted in the six-task batch under request
`runtime372-374-editor318-320-performance-batch-20260830-v4`, ticket
`4eab46c0a22440dcbb177cd77dcb2b88` (superseded by the corrected v4 manifest), with source
manifest details are recorded in the session submission log after acceptance. Cargo, performance,
review, commit, push, and WeCom remain coordinator-owned.

## Validation attempt (2026-08-30)

Ticket `50337ad7bd9b458a97f0447f09ed19fd` ended `failed`. The coordinator provided no valid
Cargo, performance, or commit evidence; no successful WeCom notification was sent.
