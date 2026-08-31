---
title: Runtime85 Project Import Dependency Dedup Optimization
category: zircon_runtime
report_id: Runtime85-project-import-dependency-dedup-2026-08-24
date: 2026-08-24
session_id: root-runtime85-project-dependency-dedup-20260824
implementation_status: implementation_complete
validation_status: managed_validation_queued
---

# Runtime85 Project Import Dependency Dedup Optimization

## Scope

This slice removes quadratic dependency-ID admission from project import resolution. It advances
Runtime85's large dependency-graph import path without changing locator resolution, missing
dependency diagnostics, staging order, metadata merge behavior, or registry publication.

## Implementation

Each source dependency is still resolved exactly once and successful IDs still enter the output in
first-locator order. A request-local `HashSet<AssetId>` now decides whether an ID is new before it is
appended to the ordered result Vec. Multiple locators that resolve to the same resource therefore
retain the former first-occurrence semantics without rescanning the growing result for every input.

Unresolved dependency diagnostics remain outside the dedup set and retain source order. The set is
local to one imported record, so it does not create cross-record state or change publication
ownership.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| Dedup admission complexity | O(n x unique) | expected O(n) | request-local hash admission |
| 4,096 dependencies / 1,024 unique IDs | 2,098,176 linear comparisons | 4,096 hash admissions | 99.8048% fewer admission operations |
| Output ordering | first resolved locator | first resolved locator | unchanged |
| Release p95 | dynamic evidence pending | <= 50% of legacy p95 | coordinator release gate |

The ignored Windows-native release evidence alternates 21 legacy/optimized sample pairs and prints
`RUNTIME85_PROJECT_DEPENDENCY_DEDUP_BENCH_V1` with exact p95 nanoseconds, dependency cardinalities,
legacy comparison count, hash admission count, and deterministic operation reduction. Dynamic
elapsed time is accepted only from coordinator terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, first-occurrence ordering regression, and the
  production hash-admission source contract are performed before coordinator submission.
- The focused regression and ignored release performance evidence are queued with Editor02 in one
  shared Runtime/Editor coordinator batch; no per-task Cargo lane is launched.
- Terminal marker values, commit integration, optimization-record finalization, and automatic WeCom
  delivery remain pending.

### Recovery Batch 2026-08-31

- Ownership transfer apply: `6779c8224fc74a1aa211d8739d11592e`.
- The correctness and ignored release tests now share the
  `runtime85_project_dedup_recovery_batch_` filter with project-root deduplication.
- Managed batch script: `tools/zircon-validation-runtime85-project-dedup-recovery-batch.ps1`.
- Coordinator ticket: `pending_submission`; terminal timings and pass/fail remain authoritative in
  the managed log.

## Remaining Parent-plan Work

Runtime85 still owns importer recipe authority, discovery/import staging, subasset identity,
artifact/cook determinism, durable generation publication, worker qualification, and product-scale
fault and performance matrices. Those parent milestones remain separate and are not claimed
complete by this dependency-admission optimization.
