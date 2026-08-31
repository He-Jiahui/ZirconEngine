---
title: Runtime60 Single-write Conflict Probe Optimization
category: zircon_runtime
report_id: Runtime60-single-write-conflict-probe-2026-08-28
date: 2026-08-28
session_id: root-runtime60-single-write-conflict-probe-20260828
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime60 Single-write Conflict Probe Optimization

## Scope

This slice advances the sorted-access-set construction cost behind RECS-P1-39. It removes redundant
binary searches when a query or system declares a new component write. It does not claim that the
wider pairwise conflict graph, bitset migration, or incremental schedule cache is complete.

## Implementation

`QueryAccess::add_write` now performs one binary search in the mirrored read set and reuses the
returned vacant index to insert the read entry. Because all fields are private and both supported
mutation paths preserve `writes` as a subset of `reads`, an ID absent from reads cannot be present
in writes. The write vector keeps its existing sorted insertion and conflict diagnostics are
unchanged.

Three Rust regressions cover sorted direct writes, ParamSet merges, and repeated-write diagnostics.
The Python source contract locks the single conflict probe, insertion-index reuse, and both
write-set mutation paths.

## Performance Evidence

The release model builds eight access sets of 65,536 distinct ascending component IDs per sample,
uses 31 alternating legacy/optimized sample pairs after five warmups, and checks equal retained
results. The acceptance threshold is at least 30% lower P50, at least 25% lower P95, a 50% binary
search reduction, and no allocation regression.

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| Binary searches per new write | 4 | 2 | -50.00% |
| P50 per eight builds | 81,605,800 ns | 51,788,100 ns | -36.54% |
| P95 per eight builds | 112,620,800 ns | 67,443,600 ns | -40.11% |
| Heap allocations | 240 | 240 | unchanged |
| Allocated bytes | 16,776,704 | 16,776,704 | unchanged |

The preceding expanded run independently measured P50 `73,188,400 -> 44,298,300 ns` (-39.47%)
and P95 `175,001,300 -> 115,555,900 ns` (-33.97%). Both runs retained checksum `524288` for each
implementation. This focused model qualifies the access-set construction change; it is not a
whole-engine or Unreal comparison.

## Validation

- Source contract: 3/3 passed after a confirmed 1/3 initial state with two expected failures.
- Exact Rust formatting, Python bytecode compilation, and scoped `git diff --check`: passed.
- This task is queued in one Runtime60 five-task asynchronous validation batch. The batch runs 15
  source contracts, 15 `runtime60_batch_` Rust regressions, and six release models for five exact
  performance rows, following the asynchronous validation policy.
- No local Cargo lane was launched and no Cargo process was terminated.

## Remaining Parent-plan Work

RECS-P1-39 still requires a bounded component/resource/event access representation, incremental
conflict-graph compilation, and product-scale diagnostics. Runtime60 P0/P1/P2 and G01-G40 remain
governed by the parent plan and are not hidden by this admission-path optimization.
