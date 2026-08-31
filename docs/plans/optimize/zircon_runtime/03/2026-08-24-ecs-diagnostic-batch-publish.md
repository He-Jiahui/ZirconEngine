---
title: Runtime03 ECS Diagnostic Batch-publish Optimization
category: zircon_runtime
report_id: Runtime03-ecs-diagnostic-batch-publish-2026-08-24
date: 2026-08-24
session_id: root-runtime03-ecs-diagnostic-batch-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime03 ECS Diagnostic Batch-publish Optimization

## Scope

This slice removes the Runtime03 lock amplification in the ECS frame-diagnostics producer. It does
not claim the parent plan's metric registry, bounded cardinality, immutable snapshot, profiler,
configuration, or artifact milestones are complete.

## Implementation

`EcsFramePerformanceDiagnostics::publish` previously routed each of its 58 fixed metrics through
`CoreHandle::record_diagnostic`, acquiring and releasing the same diagnostics mutex once per
series. It now acquires the store once through `update_diagnostic_store` and reuses the existing
`record_diagnostics` projection to write the entire frame batch.

The projection order, path, frame index, value, unit, and subsystem tags remain unchanged. A
behavior test compares the complete 58-series published snapshot with direct store recording. A
source contract requires exactly one store update and rejects the former per-series publish paths.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 25,000 ECS frame-diagnostics publications, 58 series each | 1,450,000 diagnostics mutex acquisitions | 25,000 acquisitions; <= 3 s | 98.2759% lock-acquisition reduction |

The ignored Windows-native release evidence prints `RUNTIME_DIAGNOSTICS_BENCH_V1` with exact
elapsed nanoseconds and publications per second. The lock counts are source-deterministic; elapsed
time is accepted only from coordinator terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, and the source batch contract: passed.
- ECS behavior regressions plus release performance evidence: pending one coordinator-managed
  Runtime03+Runtime44 batch ticket using the `optimization_wave_20260824p_` filter and
  `--include-ignored`.
- No local Cargo lane is launched; the batch is queued behind other managed Runtime/Editor work.

## Remaining Parent-plan Work

Other producers can still take repeated diagnostics locks, and full snapshots still clone every
series and its retained history. Those separate Runtime03 P1/P2 items remain open.
