---
title: Runtime386 Material Validation Capacity
category: zircon_runtime
report_id: Runtime386-material-validation-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime386 Material Validation Capacity

Advanced PBR material validation previously grew its error vector from zero even though only 11
owned properties can emit errors. The implementation now reserves
`min(values.len(), ADVANCED_VALIDATED_PROPERTY_COUNT)`, preserving property order, predicates,
diagnostic source, path, name, and expected text while avoiding capacity growth for invalid dense
materials. Existing clearcoat-normal-scale work in the file is retained unchanged.

The source regression requires the bounded reservation. The ignored Windows Release benchmark
emits `RUNTIME386_MATERIAL_VALIDATION_CAPACITY_BENCH_V1` over 17 alternating sample pairs with 11
errors per iteration and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime386 is submitted with Editor332 under request
`runtime386-editor332-performance-batch-20260830cj-v1`. Receipt, validation ticket, source manifest,
and terminal performance data are recorded after coordinator acceptance.

Validation attempt: ticket `2a782c0f0aeb4ce6932faa73ee8f7722` failed during coordinator
materialization with `unmanaged_artifacts_detected` for
`D:\ZirconBuilds\mvp-test-fixtures-36724`. Cargo did not start; regression, P95, commit/push, and
WeCom success evidence remain pending.
