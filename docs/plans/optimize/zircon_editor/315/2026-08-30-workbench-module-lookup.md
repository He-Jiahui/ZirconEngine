---
title: Editor315 Workbench Module Lookup
category: zircon_editor
report_id: Editor315-workbench-module-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor315 Workbench Module Lookup

## Scope

Workbench module feedback selection now performs one control-node lookup per candidate and reads
both `selected` and `checked` flags from the same metadata record. The previous path performed a
second indexed lookup whenever `selected` was false. Module priority, default Effect fallback,
and feedback output contracts remain unchanged.

## Tests And Performance Gate

The source file owns two non-ignored behavior/source-contract tests and one ignored Release
benchmark under the `optimization_batch_20260830bq_` prefix. The benchmark emits
`EDITOR315_WORKBENCH_MODULE_LOOKUP_CAPACITY_BENCH_V1`, probes the 11 module controls 100,000
times across 17 interleaved samples, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

The baseline performs separate map lookups for the two flags; the optimized path performs one
lookup and two metadata reads. The benchmark reports exact raw samples and P95 values for this
selection stage only, not complete Workbench dispatch latency.

No direct Cargo command was run. The coordinator owns the combined Runtime/Editor Release compile,
batched behavior tests, ignored benchmarks, exact P95 evidence, record finalization, manifest-only
commit, push to `origin/main`, and one-shot WeCom publication with measured data.

## Current batched validation handoff (2026-08-30)

Editor315 is included in ticket `bb793f894807473ea8c78a90c6fc2d35` for request
`runtime-editor-369-371-315-317-20260830-v2`, with source manifest hash
`391c0060104af61c0806431d76bebbaf6f1d74c41c216b63aa899577269baf4c`. The batch also binds
`external_image_copy.rs` at `a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.
Cargo, performance, review, commit, push, and WeCom remain coordinator-owned and pending.

## Validation attempt (2026-08-30)

Corrected batch ticket `bb793f894807473ea8c78a90c6fc2d35` ended `failed`. The coordinator
provided no valid Cargo, performance, or commit evidence; the external validation resource was
left unchanged and no successful WeCom notification was sent.
