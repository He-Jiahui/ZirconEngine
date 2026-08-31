# Runtime08C Graph Mask Target Dedup Optimization Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: `docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md`, P1-12
- Status: implementation and release performance gate complete; batched managed validation pending

## Problem

`evaluate_graph` materializes the unique masked target list on every graph
evaluation. The previous implementation checked every incoming target against
the growing output vector. With T distinct targets this performed T * (T - 1)
/ 2 string comparisons and repeatedly grew the output allocation, making the
dedup step O(T^2).

## Change

- Count the bounded input target references before materializing the result.
- Reserve the membership set and ordered output once.
- Use borrowed `&str` hash membership so duplicate checks do not clone target
  strings.
- Clone only the first occurrence into the public ordered result, preserving
  the previous first-seen order and output type.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| 2,048 distinct mask targets | 2,096,128 growing-vector string comparisons | 2,048 expected O(1) hash insertions | removes the quadratic comparison term |
| Output allocation | geometric vector growth | one upper-bound reservation | repeated output growth removed |
| Duplicate target cloning | first occurrence only | first occurrence only | unchanged |

The ignored release gate runs 17 alternating legacy/optimized sample pairs on
128 clips with 16 distinct target paths each. Acceptance requires optimized
nearest-rank P95 to be at most 35% of legacy P95, a minimum 65% reduction.
Exact Windows timing values remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826_runtime08c_graph_mask_target_dedup_preserves_first_seen_order`
  compares the
  optimized output byte-for-byte with the legacy algorithm and locks duplicate
  ordering.
- `optimization_batch_20260826_runtime08c_graph_mask_target_dedup_uses_reserved_hash_membership`
  locks the
  linear membership structure and one-time reservations.
- `optimization_batch_20260826_runtime08c_graph_mask_target_dedup_performance_evidence`
  emits
  `RUNTIME08C_GRAPH_MASK_TARGET_DEDUP_BENCH_V1`, all raw samples, and the 65%
  P95 reduction threshold.
- Exact-file Rust 1.94.1 rustfmt, source contract, and scoped diff checks are
  required before the multi-task validation request.

## Remaining Plan Work

This slice does not close Runtime08C P1-12. The immutable compiled program,
dense parameter and pose slots, bounded generation cache, and removal of
per-frame parameter-map cloning remain open.
