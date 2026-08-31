---
title: Runtime58 Lazy Runtime Feature Manifests
category: zircon_runtime
report_id: Runtime58-lazy-runtime-feature-manifests-2026-08-26
date: 2026-08-26
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime58 Lazy Runtime Feature Manifests

## Scope

This slice replaces the temporary concatenated feature vector in native runtime registration with a
borrowed iterator over optional features followed by feature extensions. Runtime feature filtering
and report ownership stay unchanged.

## Implementation

`runtime_feature_manifests` now streams references in the existing optional-then-extension order.
The caller filters those references for runtime modules first and clones only the admitted feature
manifests into the reports it must own. Empty and non-runtime feature sets still produce no reports,
and feature source/diagnostic fan-out remains deterministic.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 1,024 optional/extension features | temporary feature vector `1` | `0` | allocation removed |
| Feature manifest clones | 2,048 | 1,024 runtime-admitted clones | filter before clone |
| Windows-native release p95 | dynamic evidence pending | <= 80% of legacy p95 | coordinator gate |

The ignored release benchmark alternates 17 samples over 512 iterations and prints
`RUNTIME58_LAZY_RUNTIME_FEATURE_MANIFEST_BENCH_V1` with both p95 timings, feature count,
temporary-vector count, and feature clone counts. Exact elapsed-time evidence is accepted only from
the coordinator terminal receipt.

## Validation

- Functional regressions lock optional-before-extension order and runtime-module filtering.
- Source contracts prevent reintroduction of the eager concatenating vector.
- Exact Rustfmt and scoped diff checks are required before submission.
- One Windows-native release Cargo invocation batches this task with the borrowed registration
  filter task; no per-task Cargo lane is launched.
- Commit integration, benchmark values, record finalization, and automatic WeCom delivery remain
  coordinator-owned and pending.

## Remaining Parent-plan Work

The parent Runtime58 plan still requires unified bridge generations, tracked call leases, safe native
retirement, replay activation in App/Editor, World binding replacement, and typed diagnostics.
