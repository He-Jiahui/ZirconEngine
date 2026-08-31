---
title: Runtime09H2 Sorted Volume Evaluation Fast Path
category: zircon_runtime
report_id: Runtime09H2-sorted-volume-fast-path-2026-08-26
date: 2026-08-26
session_id: root-runtime09h2-sorted-volume-fast-path-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09H2 sorted volume evaluation fast path

## Scope

- Parent scope: the Runtime09H2 Volume acceptance row, specifically `eval candidates` and CPU evaluation time.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: the framework volume evaluator, its source/performance contract, and this record.
- This slice removes candidate materialization for the priority-sorted scene path only. It does not close Volume persistence, unsupported shapes, overlay ownership, plugin resources, GPU effects, color correctness, or the remaining Runtime09H2 exit conditions.

## Change

- `VolumeEvaluator::evaluate` first checks the published extract slice for nondecreasing priority.
- The normal scene path applies active, mask-matching, positive-influence volumes directly from the borrowed slice, with no applicable-candidate `Vec`.
- Direct callers with unsorted input retain the existing indexed stable priority sort and therefore preserve input order for equal or unordered priorities.
- The existing priority test now evaluates the same volumes in sorted and unsorted order and requires identical resolved settings.
- The performance contract also guards the producer side: scene extraction must continue sorting before it projects the final extract slice.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime09h2_sorted_volume_fast_path_performance_contract -v` initially passed 2/4; the sorted branch was absent and the old path still materialized all applicable candidates.
- GREEN: the same command passes 4/4 after the direct sorted path is implemented.
- `rustfmt --edition 2021 --config skip_children=true` completed for the owned Rust file.
- The standalone model is compiled with `rustc -O`; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating legacy/optimized sample pairs, with 64 evaluations of 8,192 priority-sorted sphere volumes per sample. It retains active filtering, layer masks, distance/blend influence, three-field application, stable order, and a process-local allocation counter. Three local runs passed exact checksum parity and the acceptance thresholds; the table records the latest run.

| Metric | Materialized applicable `Vec` | Direct sorted slice | Change |
|---|---:|---:|---:|
| P50 | 4.9758 ms | 2.9539 ms | -40.635% |
| P95 | 7.5418 ms | 4.2221 ms | -44.017% |
| allocations / evaluation | 12 | 0 | -100.000% |

The other two runs produced P50 reductions of 38.577% and 49.134%, P95 reductions of 34.311% and 47.994%, and zero optimized allocations. These timings isolate sorted CPU Volume candidate evaluation; they do not claim GPU time, complete frame time, or visual correctness.

## Async validation

One coordinator batch must run the four Python contracts, all seven evaluator Rust tests in the real `zircon_runtime` crate, Rust formatting checks, scoped diff checks, exact model parity, and the same performance workload.

Acceptance requires 4/4 source contracts and 7/7 Rust tests to pass, exact checksum parity, zero optimized allocations, allocation reduction of at least 99.9%, and P50/P95 reductions of at least 20%. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 and allocation reductions and label them as 8,192-volume sorted CPU evaluator evidence.
