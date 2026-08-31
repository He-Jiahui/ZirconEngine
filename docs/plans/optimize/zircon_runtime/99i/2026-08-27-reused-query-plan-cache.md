---
title: Runtime99i Reused Query Plan Cache
category: zircon_runtime
report_id: Runtime99i-reused-query-plan-cache-2026-08-27
date: 2026-08-27
session_id: root-runtime99i-contiguous-transition-validation-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime99i Reused Query Plan Cache

## Scope

`QueryState::update_cache` previously compiled incremental matches into a temporary
`Vec<CachedArchetypePlan>` and then extended its retained cache. A full cache rebuild similarly
collected a new vector and replaced the retained vector, discarding reusable capacity.

The incremental path now reserves once in the retained cache and pushes each compiled plan
directly. The full path clears the retained vector, reserves only the missing capacity, and pushes
the same plans in matching-archetype order. Query access matching, plan compilation, component
binding order, membership refresh, cache hit/miss/rebuild counters, candidate counts, and stable
query iteration are unchanged.

## Behavior Evidence

- `cached_query_compiles_only_new_archetypes_that_match_its_access` covers an unmatched archetype
  generation followed by one newly matching archetype and exact query output.
- `cached_query_ignores_membership_changes_in_unmatched_existing_archetypes` covers the no-new-plan
  cache-hit path.
- Every test constructs its `QueryState` through the full initial rebuild before exercising the
  incremental path.
- `test_runtime99i_reused_query_plan_cache_performance_contract.py` requires direct retained-cache
  pushes, full-rebuild capacity reuse, and one compilation counter increment per matching
  archetype; it rejects both temporary plan-vector names and whole-vector replacement.

## Deterministic Performance Model

The isolated release model uses 131,072 plans. Its outer plan layout is 40 bytes on x64 and matches
the production ownership shape: archetype id, membership generation, and a binding `Vec` owner.
Binding vectors are empty so the model measures only the outer cache storage changed by this slice;
it does not attribute binding compilation or world matching work to the optimization.

| Metric | Temporary outer vector | Retained-cache push | Reduction |
|---|---:|---:|---:|
| full-rebuild outer allocations | 1 | 0 | 100.000% |
| full-rebuild outer allocated bytes | 5,242,880 | 0 | 100.000% |
| incremental outer allocations | 1 | 0 | 100.000% |
| incremental outer allocated bytes | 5,242,880 | 0 | 100.000% |

Each run uses five warmups and 31 alternating full-rebuild sample pairs. Each timed sample batches
eight rebuilds to amortize Windows scheduling noise:

| Run | Legacy P50 ns | Reused P50 ns | Reduction | Legacy P95 ns | Reused P95 ns | Reduction |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 15,560,800 | 3,901,800 | 74.920% | 17,556,400 | 8,370,300 | 52.320% |
| 2 | 16,911,700 | 5,866,400 | 65.310% | 20,358,600 | 8,521,100 | 58.150% |
| 3 | 19,299,000 | 7,494,800 | 61.160% | 61,307,000 | 30,518,700 | 50.220% |
| 4 | 35,810,800 | 11,725,000 | 67.260% | 75,300,500 | 30,852,100 | 59.030% |

The four-run worst case reduces P50 by 61.160% and P95 by 50.220%. Result checksum
`11692943496689493728` and timing checksum `2450731473214532829` are identical between paths. The
managed gate requires zero outer allocations/bytes in both reused paths, at least 50% lower P50,
at least 40% lower P95, and exact nonzero checksum parity.

This model proves the removed outer storage projection only. It is not an end-to-end ECS schedule,
entity iteration, component fetch, frame-time, power, or external-engine comparison.

## Validation

Passed locally without Cargo:

- 3/3 Python source/performance contracts;
- Rust formatting and scoped diff checks;
- four independent release-model runs with exact plan equality, nonzero checksums, and every gate
  met.

Managed validation must run the focused `cached_query_` Rust batch, all three Python contracts,
formatting, scoped diff, and a fresh release model in one coordinator ticket. Cargo validation is
not claimed until that asynchronous ticket reaches a passing terminal state.

## Remaining Parent-Plan Work

Runtime99i still owns archetype transition/storage coverage, query scheduling and conflict policy,
parallel execution, event and command throughput, change-detection wraparound, product-scale ECS
workloads, and external-engine comparison. This slice only removes transient outer query-plan
storage during cache rebuilds.
