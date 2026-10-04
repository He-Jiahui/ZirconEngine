# Cargo failed-cleanup SQL overlap

Plan: `docs/plans/optimize/zircon_tooling/06-session-coordinator-codex-app-control-plane-current-source-review.md`

Milestone: Tooling06 performance batch candidate 62

Status: implemented; focused validation passed; queued for batched coordinator validation

Files:
- `tools/session_coordinator/cargo_jobs.py`
- `tools/tests/test_tooling06_cargo_cleanup_failed_sql_overlap_performance_contract.py`

## Problem

Allocating one Cargo target loaded every job with failed cleanup, materialized
the rows, and searched their normalized target trees in Python. Failed cleanup
history grows independently of the requested target, so target allocation paid
O(N) row transfer and Python path comparisons even when only one tree overlaps.

## Implementation

The lookup now filters exact, ancestor, and descendant `target_key` overlap in
SQLite, preserves the existing newest-first ordering, and returns only the
first blocking row. The cleanup error, target identity, and rejection behavior
are unchanged.

## Performance

Representative in-memory SQLite workload: 10,000 failed-cleanup rows with one
descendant overlap. Measured samples used the same schema and query ordering.

| path | examined/returned rows | p50 | p95 |
| --- | ---: | ---: | ---: |
| full scan plus Python overlap | 10,000 | 25,553,600 ns | 35,958,700 ns |
| SQL overlap filter with limit | 1 | 5,571,100 ns | 11,211,100 ns |

- p50 reduction: 78.198%, 4.59x speedup
- p95 reduction: 68.822%, 3.21x speedup

## Validation

- Static performance contract: passed.
- Focused regression
  `CargoJobTests.test_failed_cleanup_blocks_overlapping_target_reacquire`:
  1/1 passed in 9.375s.
- Exact/ancestor/descendant/non-prefix SQL behavior probe: passed.
- Full `CargoJobTests` batch reached the 300s local timeout without emitting a
  failure; it remains queued for the accumulated asynchronous validation batch
  and is not counted as passed here.
