---
title: Runtime60 Linear Read-only Query Projection
category: zircon_runtime
report_id: Runtime60-linear-read-only-query-projection-2026-08-28
date: 2026-08-28
session_id: root-runtime60-single-write-conflict-probe-20260828
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime60 Linear Read-only Query Projection

## Scope

This slice advances the sorted access-set construction path behind RECS-P1-39. It replaces the
per-read binary membership probe used when a `QueryAccess` is projected into `SystemParamAccess`.
It does not claim the wider pairwise conflict graph, bitset migration, or incremental schedule
cache is complete.

## Implementation

`SystemParamAccess::add_query_access` still admits writes first and still routes every projected ID
through the existing `QueryAccess::add_write` and `add_read` checks. The read projection now walks
the sorted read and write slices with one monotonic write cursor. Because `QueryAccess` mirrors every
write into reads, the cursor can identify read-only IDs in `O(R + W)` without allocation instead of
performing `R` binary searches over writes.

Three Rust regressions cover mixed sorted read/write projection, fully interleaved writes, and an
existing system write conflicting with a projected read. The Python source contract locks the
monotonic cursor and preserves the four existing admission calls.

## Performance Evidence

The release model projects 65,536 sorted reads with 32,768 interleaved writes for 32 rounds per
sample. It uses 31 alternating legacy/optimized sample pairs after five warmups and checks equal
results. The acceptance threshold is at least 80% fewer membership comparisons, at least 50% lower
P50/P95, identical checksum, and no allocation.

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| Membership comparisons | 30,409,280 | 2,097,120 | -93.104% |
| P50 per 32 projections | 121,955,800 ns | 8,340,700 ns | -93.161% |
| P95 per 32 projections | 148,416,300 ns | 13,021,900 ns | -91.226% |
| Projection heap allocations | 0 | 0 | unchanged |

The preceding run independently measured P50 `131,874,800 -> 8,998,100 ns` (-93.177%) and P95
`192,223,800 -> 12,137,900 ns` (-93.686%). Both runs retained checksum `34359738368` for each
implementation. This model qualifies read-only access projection only; it is not a whole-schedule
or Unreal comparison.

## Validation

- Source contract: 3/3 passed after a confirmed 1/3 initial state with two expected failures.
- Exact Rust formatting and scoped `git diff --check`: passed.
- This task is queued in one Runtime60 five-task asynchronous validation batch. The batch runs 15
  source contracts, 15 `runtime60_batch_` Rust regressions, and six release models for five exact
  performance rows.
- No local Cargo lane was launched and no Cargo process was terminated.

## Remaining Parent-plan Work

RECS-P1-39 still requires bounded component/resource/event access representation, incremental
conflict-graph compilation, and product-scale diagnostics. Runtime60 P0/P1/P2 and G01-G40 remain
governed by the parent plan and are not hidden by this projection optimization.
