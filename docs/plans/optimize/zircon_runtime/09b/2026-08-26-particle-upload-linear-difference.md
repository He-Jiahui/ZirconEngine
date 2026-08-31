---
title: Runtime09B Particle Upload Linear Difference
category: zircon_runtime
report_id: Runtime09B-particle-upload-linear-difference-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09B Particle Upload Linear Difference

## Scope

This slice replaces ordered-set reconstruction in particle upload delta planning with a linear
difference over the visibility history contract's ascending unique emitter vectors. It does not
remove history snapshot reconstruction or implement incremental particle ownership.

## Change

- Compare current and previous emitter vectors with a two-pointer sorted-difference helper.
- Remove both temporary `BTreeSet<EntityId>` indexes and their membership probes.
- Preserve current emitter order, ascending dirty emitters, ascending removed emitters, and the
  existing full-rebuild behavior.

## Deterministic Performance Evidence

| 100,000 current and 100,000 previous emitters | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-index admissions | 200,000 | 0 | 100% removed |
| Tree membership probes | 200,000 | 0 | 100% removed |
| Sequential input visits | not used | at most 400,000 | linear, cache-local |
| Delta order | ascending | ascending | unchanged |

The ignored release gate alternates 17 tree-index and linear-difference samples with one eighth of
each generation changed. It emits `RUNTIME09B_PARTICLE_UPLOAD_LINEAR_DIFFERENCE_BENCH_V1`;
acceptance requires linear-difference P95 to be at most 60% of tree-index P95. Exact Windows
timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826q_runtime09b_linear_particle_difference_preserves_plan_order`
  covers emitter, dirty, and removed output contracts.
- `optimization_batch_20260826q_runtime09b_particle_upload_uses_linear_difference` requires one
  helper and two product calls while rejecting tree sets and membership lookup.
- `optimization_batch_20260826q_runtime09b_particle_upload_linear_difference_performance_evidence`
  emits workload counts and both P95 values, then enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime09B still rebuilds particle history and clones the complete current emitter vector per
frame. Persistent particle generations, dirty/remove journals, bounded upload ranges, GPU-driven
visibility, stable-frame zero work, and complete scale qualification remain open.
