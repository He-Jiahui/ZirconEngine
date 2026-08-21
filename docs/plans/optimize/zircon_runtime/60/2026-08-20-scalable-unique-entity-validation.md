# Runtime60 Scalable Unique Entity Validation Optimization Record

- Date: 2026-08-20
- Owner: `optimize-runtime60-unique-entity-validation-r1-01a00797-20260820`
- Source plan: `docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md`, RECS-P1-18
- Status: implementation complete; combined managed validation pending

## Problem

`UniqueEntityArray::new` and the mutable many-query alias guard validated a
fixed entity array with nested scans. A duplicate-free request of N entities
therefore performed `N * (N - 1) / 2` comparisons before any query fetch could
start.

## Change

- Arrays of at most 16 entities retain the allocation-free inline scan, which
  avoids sort setup for the common small-query case.
- Larger arrays build a stack-resident `(EntityId, original_index)` array and
  use unstable sorting, reducing validation from O(N^2) to O(N log N) without a
  heap allocation.
- Duplicate groups retain their original indices. The validator selects the
  lowest second-occurrence index across all groups, preserving the previous
  `DuplicateEntity(id)` and `AliasedMutability(id)` result when multiple values
  are duplicated.

## Deterministic Performance Evidence

The managed release gate uses 4,096 unique deterministically shuffled IDs:

| Measure | Legacy | Optimized | Gate |
|---|---:|---:|---:|
| Validation complexity | O(N^2) | O(N log N) | one complexity class lower |
| Legacy candidate comparisons | 8,386,560 | n/a | exact |
| Sort comparisons | n/a | emitted by the production comparator | <= 2% of legacy |
| Production heap allocations | 0 | 0 | unchanged |
| Timing distribution | 21 samples | 21 samples | alternating order |
| Nearest-rank P95 | pending | pending | optimized <= 25% of legacy |

Exact Windows P50/P95 values remain pending the combined coordinator batch and
must be written here before integration acceptance.

## Acceptance

- Small-array regression preserves the existing first duplicate result.
- A 32-entity regression with two independent duplicate groups proves the
  sorted path still reports the earliest duplicate in request order.
- `unique_entity_validation_release_benchmark_evidence` emits the raw 21-pair
  distributions, recomputable nearest-rank P50/P95 values, exact legacy work,
  and measured sort comparisons.
- Exact-file Rustfmt, scoped diff checks, Cargo regressions, and release timing
  are required in one managed multi-task Windows validation copy. No per-task
  Cargo invocation is used.

## Remaining Scope

This slice removes the quadratic fixed-array validation path. RECS-P1-18 still
asks for a public duplicate-index diagnostic; the current public
`QueryEntityError` carries only the duplicate entity ID, so that API extension
remains open with the broader query-error contract work. RECS-P1-19 and later
query allocation findings are unchanged.
