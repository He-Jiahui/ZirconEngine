---
title: Editor10 Progress Snapshot Unique-ID Borrowed Batch
category: zircon_editor
report_id: Editor10-progress-snapshot-unique-id-batch-2026-09-09
date: 2026-09-09
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor10 Progress Snapshot Unique-ID Borrowed Batch

## Scope

This slice removes redundant intermediate collections from the bounded progress notification
snapshot path. It preserves the public `snapshot_for_ids` contract, notification-ID ordering,
one-notification-per-job admission, terminal-job pruning, and reused-ID protection. It does not
claim the parent plan's immutable generation pages, typed deltas, or product-wide progress
latency budget.

## Implementation

`ProgressNotificationCenter::snapshot` now captures notification IDs and their authoritative
`JobId` bindings in one capacity-sized `Vec` under the center lock. The job source receives a
borrowed iterator over that buffer through a crate-private unique-ID lookup, so it no longer
allocates a second ID vector or a `BTreeSet` for a caller that already owns the one-binding-per-job
invariant. The lookup preallocates its result from the trusted iterator lower bound and preserves caller
order. The existing public lookup remains deduplicating and JobId-sorted for general consumers.

`synchronize_captured` consumes the captured slice directly. Its identity check still prevents a
stale capture from removing a notification that has since been rebound to another job, while the
final presentation remains in the existing notification-ID order.

## Deterministic work model

For 64 retained bindings and 100,000 stable polls, the retired path performs four modeled linear
passes per poll (center materialization, center values pass, JobId deduplication, and lookup), or
25,600,000 binding visits. The new path performs one captured binding pass and one lookup pass, or
12,800,000 modeled visits: a 5,000 basis-point structural reduction. The model excludes tree
comparison cost, allocator internals, snapshot cloning, CPU time, RSS, and input-to-present
latency; it is not product performance evidence.

The ignored Rust release gate emits
`EDITOR_PROGRESS_SNAPSHOT_UNIQUE_ID_BENCH_V1` with the same workload and an elapsed-time ceiling.
Coordinator output is required before accepting a timing or allocation claim.

## Validation

- Focused Editor17 decision-notification contract batch: `8/8` passed; Python compilation passed.
- The merged Runtime/Editor performance-contract batch passed `1723/1723` (`Runtime 1143/1143`,
  `Editor 580/580`).
- Rustfmt (`--edition 2021 --check --config skip_children=true`) passed for both touched owners.
- Scoped `git diff --check` passed; only normal CRLF conversion warnings were emitted.
- The local Rust unit and ignored release gates are prepared but were not run through Cargo because
  the external `E:/Git/zr_vm` checkout is currently dirty. No coordinator status was queried or
  waited on.

## Remaining parent-plan work

Progress snapshots still clone visible rows, acquire the source lock, and have no generation or
delta cursor. Immutable pages, terminal outcome retention, typed progress validation, and managed
Windows release p50/p95/p99 CPU/allocation/RSS evidence remain open.

## Current source fingerprint

- `zircon_editor/src/core/jobs/progress.rs`: `C60EA8691E03EA20317D8D8A37AC538004F3E36DBAA9AC0B73743412348150DC`
- `zircon_editor/src/core/notifications/progress/center.rs`: `7D44659292948B3E072FF65C0705D3F294CD5D7A0278D751D7B4DF0C958D1FD2`
