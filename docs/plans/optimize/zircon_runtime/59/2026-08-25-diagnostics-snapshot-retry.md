---
title: Runtime59 Diagnostics Snapshot Retry
category: zircon_runtime
report_id: Runtime59-diagnostics-snapshot-retry-2026-08-25
date: 2026-08-25
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime59 Diagnostics Snapshot Retry

## Scope

This slice repairs the bounded aggregate retry path used by `JobSchedulerDiagnosticsState` when a
report races a shard-local writer. It does not close Runtime59's execution ownership, task scope,
shutdown, deadline, or product diagnostics gaps.

## Implementation

The previous aggregate loop used `?` while reading each shard. A single contended shard therefore
returned the cached report immediately and bypassed all remaining aggregate attempts, despite the
declared 16-attempt retry budget. The implementation now separates one complete aggregate attempt
from the bounded retry loop:

- a contended or generation-invalid attempt consumes one retry and restarts the full aggregate;
- a successful attempt still rechecks every shard epoch before publishing the merged snapshot;
- the uncontended path still performs exactly one complete aggregate attempt;
- shard writers, atomics, cache locking, public report shape, and the 16-attempt bound are unchanged.

## Performance Contract

| Evidence | Retired single-attempt behavior | Optimized gate |
| --- | ---: | ---: |
| Deterministic transient contention | 0 fresh snapshots from 101 attempts | 101 fresh snapshots from 101 two-attempt samples |
| Aggregate retry depth | cached report after the first contended shard | fresh snapshot on attempt 2 |
| Release benchmark | retired and optimized P95 emitted in nanoseconds | optimized two-attempt P95 <= 2 ms |

The ignored release benchmark emits `TASK_DIAGNOSTICS_SNAPSHOT_RETRY` with sample count, measured
retired and optimized fresh-snapshot counts, attempt depth, and both P95 timings. Actual timings are
accepted only from the coordinator's terminal Windows-native release evidence and remain pending in
this record.

## Validation

The managed batch covers the new transient-contention regression, the existing diagnostics unit
suite, concurrent scheduler diagnostics regressions, and the ignored release benchmark in one Cargo
invocation. Exact Rust 1.94.1 `rustfmt --check` and scoped `git diff --check` passed before
submission (apart from the repository's existing CRLF notice). Test execution, measured P95,
integration SHA, and automatic WeCom performance delivery remain coordinator-owned and pending.
