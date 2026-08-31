---
title: Runtime58 Borrowed Registration Manifest Filter
category: zircon_runtime
report_id: Runtime58-borrowed-registration-manifest-filter-2026-08-26
date: 2026-08-26
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime58 Borrowed Registration Manifest Filter

## Scope

This slice removes whole-package cloning from the native runtime registration candidate filter. It
preserves package ordering, FeatureExtension exclusion, runtime-module admission, shader source
resolution, diagnostic ordering, and the public registration report shape.

## Implementation

`NativePluginLoadProjection::runtime_plugin_registration_reports` now filters the borrowed manifest
slice before entering the mapping stage. A retained manifest is cloned only when the registration
report needs an owned runtime-only manifest; non-runtime and feature-extension packages never pay
that clone cost.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 1,024 package candidates, 512 retained | 1,024 manifest clones | 512 retained-manifest clones | 50% clone reduction |
| Candidate filtering | clone then filter | borrow then clone retained | filter precedes ownership |
| Windows-native release p95 | dynamic evidence pending | <= 80% of legacy p95 | coordinator gate |

The ignored release benchmark alternates 17 samples over 512 iterations and prints
`RUNTIME58_BORROWED_REGISTRATION_MANIFEST_FILTER_BENCH_V1` with both p95 timings, package and
retained counts, and clone counts. Exact elapsed-time evidence is accepted only from the coordinator
terminal receipt.

## Validation

- Source contracts assert borrowed filtering precedes cloning.
- Functional coverage retains the existing runtime-module and package-kind admission rules.
- Exact Rustfmt and scoped diff checks are required before submission.
- One Windows-native release Cargo invocation batches this task with the lazy feature manifest task;
  no per-task Cargo lane is launched.
- Commit integration, benchmark values, record finalization, and automatic WeCom delivery remain
  coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime58 still owns the typed bridge contract, call lease, generation retirement, registration
replay product wiring, and native/VM lifecycle transaction described by its parent plan.
