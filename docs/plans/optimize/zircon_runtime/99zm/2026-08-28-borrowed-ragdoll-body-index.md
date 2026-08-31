---
title: Runtime99ZM Borrowed Ragdoll Body Index
category: zircon_runtime
report_id: Runtime99ZM-borrowed-ragdoll-body-index-2026-08-28
date: 2026-08-28
session_id: root-runtime99zm-borrowed-ragdoll-body-index-20260828
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime99ZM Borrowed Ragdoll Body Index

## Scope

`write_simulated_pose_feed` resolves a physics body for every skeleton-bound joint twice: once
while building the parent-bone world map and again while publishing pose rows. The previous helper
linearly scanned `PhysicsWorldSyncState::bodies` for the joint entity and, when needed, scanned it
again for the connected-entity fallback. This made pose publication O(joints x bodies) in both
passes.

The implementation now builds one borrowed entity index and reuses it for both passes. Index slots
are created only for direct and connected entities referenced by joints with a skeleton binding;
the body array is then scanned once to fill those slots. `entry(...).or_insert(None)` and
`slot.get_or_insert(body)` retain the first body for duplicate entity IDs, matching the old
`Iterator::find` behavior. Direct joint entity lookup still precedes connected-entity fallback.
An input with no skeleton-bound joint returns `HashMap::new()` before scanning bodies, so the common
inactive-ragdoll path adds no allocation.

## Performance Evidence

The isolated Rust model uses 4,096 unique bodies, 128 trailing duplicate entity records, 4,096
skeleton-bound joints, and a connected-entity fallback for every seventh joint. It compares the
two old linear-resolution passes with the final selective borrowed index. Each variant uses 21
samples after three warmups and was compiled with
`rustc +1.94.1 --edition 2021 -O -C target-cpu=native` on Windows.

| Metric | Linear body scans | Selective borrowed index | Change |
|---|---:|---:|---:|
| Body resolution probes | 21,731,840 | 13,588 | -99.937% |
| P50 | 17,742,400 ns | 329,600 ns | -98.142% |
| P95 | 18,824,400 ns | 359,700 ns | -98.089% |
| Allocator calls | 0 | 1 | +1 bounded index allocation |
| Requested bytes | 0 | 139,280 | +139,280 bytes at 4,096 joints |

The baseline and optimized checksums both remained `285,233,152`. The acceptance gates were body
resolution probes at or below 1% of baseline and P50/P95 at or below 25% of baseline; all three
passed. Index capacity is bounded by skeleton-joint direct/connected references rather than total
physics body count, and the no-skeleton-joint path remains zero-allocation.

Model source:

- `.codex/state/session-coordinator/physics-ragdoll-borrowed-body-index-model.rs`

The model isolates lookup work and its added temporary index allocation. It does not replace
managed Cargo behavior tests, product-scale physics profiling, or allocator measurements of the
entire pose publication pipeline.

## Contracts And Validation

- `tools/tests/test_runtime99zm_borrowed_ragdoll_body_index_performance_contract.py` locks the
  joint-bounded borrowed index, zero-allocation empty path, first-duplicate preservation, one-index
  reuse, direct-entity precedence, connected fallback, and absence of linear body scans.
- Initial TDD RED failed 4/4 tests against the two linear scans. The first all-body index passed the
  original contract, then the stricter joint-bounded contract failed 4/4 before the final selective
  implementation passed 4/4.
- Scoped `rustfmt +1.94.1 --edition 2021 --check`, Python contract execution, and
  `git diff --check` pass.
- Cargo type checking and focused ragdoll behavior remain pending in a managed asynchronous
  coordinator batch; no direct Cargo command was run.

## Remaining Parent-Plan Work

Runtime99ZM still requires the M0 truth gate, provider reachability, fail-closed readiness, one
fixed-clock authority, native Jolt query/contact/constraint ownership, incremental world sync,
typed writeback disposition, stable skeleton identity, artifact-backed ragdoll lifecycle, and
product-scale qualification. This slice only removes repeated linear body resolution from the
existing pose-feed bridge; it does not close those parent gates or claim competitive engine parity.
