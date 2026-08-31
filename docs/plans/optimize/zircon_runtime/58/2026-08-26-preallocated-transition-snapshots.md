---
title: Runtime58 Preallocated Transition Snapshots
category: zircon_runtime
report_id: Runtime58-preallocated-transition-snapshots-2026-08-26
date: 2026-08-26
session_id: root-runtime58-three-task-bridge-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime58 Preallocated Transition Snapshots

## Scope

Bridge owner transition reports already know the exact number of affected slots before projecting
their snapshots. The report can reserve that many snapshot slots while preserving filtering for
invalid slots and the public affected-slot/snapshot order.

## Implementation

`owner_transition_report` now builds snapshots with `Vec::with_capacity(affected_slots.len())`
and pushes the same valid entries in the same order. Missing slots still produce no snapshot and
may leave spare capacity, which is bounded by the supplied affected-slot list.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Snapshot capacity reserve for 2,048 affected slots | 0 | 2,048 |
| Snapshot ordering and filtering | unchanged | unchanged |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `RUNTIME58_PREALLOCATED_TRANSITION_SNAPSHOTS_BENCH_V1` with both p95
durations, sample/iteration/affected-slot counts, and the snapshot reserve.

## Validation

Scoped rustfmt, diff checks, source contracts, and a full transition-report regression are prepared.
The managed `runtime58_batch_` release gate alternates legacy/optimized samples and covers all three
bridge optimizations in one Cargo invocation: 3 source contracts, 8 Rust tests, and 3 performance
rows. Commit integration, terminal P95 values, and WeCom delivery remain coordinator-owned.
