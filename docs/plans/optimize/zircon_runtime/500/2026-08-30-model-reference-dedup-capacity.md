---
title: Runtime500 Model Reference Dedup Capacity
category: zircon_runtime
report_id: Runtime500-model-reference-dedup-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_materialization_failed
---

# Runtime500 Model Reference Dedup Capacity

Model reference collection already used a hash membership set to preserve first-seen unique output,
but both the set and result vector still grew from zero. The collector now reads the iterator size
hint and reserves its upper bound, falling back to the lower bound when no upper bound exists.
Reference equality, first-seen order, cloning ownership, and all call sites remain unchanged.

The source regression requires both reservations. The ignored Windows Release benchmark emits
`RUNTIME500_MODEL_REFERENCE_DEDUP_CAPACITY_BENCH_V1` over 17 alternating sample pairs with 16,384
unique references and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime500 is submitted with Editor500 under request
`runtime500-model-reference-editor500-capability-diagnostic-20260830cm-v1`. Receipt, ticket, source
manifest, and terminal performance data are recorded after coordinator acceptance.

## Managed validation terminal status (2026-08-30)

Ticket `238843353fc94c179b696d3601c7b58d` failed before Cargo in
`closure_planning/materialization`: foreign test
`zircon_runtime/src/asset/tests/migration/project_commandlet/crash_windows.rs` references missing
`zircon_runtime/src/core/resource/io/transaction/journal/intent.rs`. No Runtime500 test or
benchmark executed, so no performance result, commit, push, or WeCom notification is claimed.
