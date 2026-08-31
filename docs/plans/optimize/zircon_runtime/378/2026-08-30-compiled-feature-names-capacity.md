---
title: Runtime378 Compiled Feature Names Capacity
category: zircon_runtime
report_id: Runtime378-compiled-feature-names-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime378 Compiled Feature Names Capacity

`compiled_feature_names` now reserves the enabled-feature slice length before cloning feature names.
Feature-name conversion and source ordering are unchanged.

The ignored Windows Release benchmark emits `RUNTIME378_COMPILED_FEATURE_NAMES_CAPACITY_BENCH_V1`
over 17 paired samples with 256 features per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime378 is submitted in the eight-task batch under request
`runtime375-378-editor321-324-performance-batch-20260830-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
