# Control Cargo target narrow rank

Plan: `docs/plans/optimize/zircon_tooling/06-session-coordinator-codex-app-control-plane-current-source-review.md`

Milestone: Tooling06 performance batch candidate 60

Status: implemented; focused validation passed; queued for batched coordinator validation

Files:
- `tools/session_coordinator/control_plane/snapshot.py`
- `tools/tests/test_tooling06_control_cargo_target_narrow_rank_performance_contract.py`

## Problem

The control-plane current-Cargo-target projection ranked `SELECT *` from every
non-empty `cargo_jobs.target_dir`. The outer projection consumed only thirteen
scalar lifecycle fields, but SQLite still carried command, environment, process
tree, compatibility, and other JSON/text columns through the window sorter.

## Implementation

The `latest_target` CTE now selects only the thirteen fields consumed by the
outer projection plus `ROW_NUMBER()`. Partitioning, ordering, deleted-target
filtering, filesystem existence checks, and browser response fields are
unchanged.

## Performance

Representative in-memory SQLite workload: 5,000 Cargo jobs across 1,000 target
directories, five historical jobs per target, and four unused 8 KiB text/JSON
columns. Twenty-one measured samples followed three warmups.

| query | p50 | p95 |
| --- | ---: | ---: |
| full-row rank | 4,936,144,900 ns | 6,681,443,700 ns |
| narrow rank | 31,354,500 ns | 48,018,500 ns |

- p50 reduction: 99.365%, 157.43x speedup
- p95 reduction: 99.281%, 139.14x speedup

## Validation

- Static performance contract: passed.
- Direct current-target projection regression group: 3/3 passed in 19.575s.
- Full `ControlSnapshotTests`: 23/24 passed. The unrelated existing Codex spool
  projection test failed because `spoolOverflow` was absent; this candidate does
  not modify Codex spool projection code and the RED remains assigned to the
  batched asynchronous validation/fix workflow.
