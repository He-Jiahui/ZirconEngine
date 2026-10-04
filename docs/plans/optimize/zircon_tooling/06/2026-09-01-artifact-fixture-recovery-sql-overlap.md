# Artifact fixture recovery SQL overlap

Plan: `docs/plans/optimize/zircon_tooling/06-session-coordinator-codex-app-control-plane-current-source-review.md`

Milestone: Tooling06 performance batch candidate 61

Status: implemented; focused module validation passed; queued for batched coordinator validation

Files:
- `tools/session_coordinator/artifact_governance.py`
- `tools/tests/test_tooling06_artifact_fixture_recovery_sql_overlap_performance_contract.py`

## Problem

Completing one unmanaged-artifact cleanup selected every active fixture lease,
normalized every returned path, and tested exact/ancestor/descendant overlap in
Python. Fixture history grows independently of the cleanup target, so a single
completion incurred O(N) row materialization and Python path work.

## Implementation

The active fixture query now filters normalized `target_key` values for exact,
ancestor, or descendant overlap in SQLite. It returns only the matching
`lease_id` and `target_dir` rows needed for the existing recovery update and
audit event. Status transitions, compare-and-update protection, and event
payloads are unchanged.

## Performance

Representative in-memory SQLite workload: 10,000 active fixture leases with one
descendant overlap. Thirty-one measured samples followed three warmups.

| path | returned rows | p50 | p95 |
| --- | ---: | ---: | ---: |
| full scan plus Python overlap | 10,000 | 17,556,100 ns | 28,421,600 ns |
| SQL overlap filter | 1 | 5,481,900 ns | 9,576,900 ns |

- p50 reduction: 68.775%, 3.20x speedup
- p95 reduction: 66.304%, 2.97x speedup

## Validation

- Static performance contract: passed.
- `tools.session_coordinator.tests.test_artifact_governance`: 35/35 passed in
  76.101s.
