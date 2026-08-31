---
title: Runtime07 Preallocated Host Capability Snapshot
category: zircon_runtime
report_id: Runtime07-preallocated-host-capability-snapshot-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Preallocated Host Capability Snapshot

## Scope

This slice removes geometric result-vector growth from `HostRegistry::capabilities()` and replaces
stable sorting with an equivalent unique-key unstable sort. It preserves cloned record ownership,
vacant-slot omission, generation-aware raw-handle ordering, poisoned-lock recovery, and the public
snapshot shape.

## Change

- Derive the exact live-record capacity in constant time from the registry invariant
  `slots.len() - free_slots.len()` while the state lock is held.
- Extend one preallocated vector with cloned live records instead of collecting from a `filter_map`
  whose lower size hint is zero.
- Use `sort_unstable_by_key` because one live record exists per slot and the raw handle combines the
  generation with the unique slot index, so sort keys cannot collide within a snapshot.
- Keep record cloning inside the lock and make the guard lifetime explicit; the legacy temporary
  guard already ended after the collection statement, so the measured concurrency benefit is the
  removal of lock-held vector growth rather than moving sorting out of the critical section.
- Add a Rust behavior regression for revoke, slot reuse, live-record projection, and raw-handle
  ordering, plus a Python performance structure contract.

## Deterministic Performance Evidence

The standalone optimized Rust model snapshots 65,536 slots with 57,344 live records, every eighth
slot vacant, varied generations to force nontrivial raw-handle sorting, and owned `String` labels.
It alternates legacy and optimized order across 31 samples and asserts exact vector equality for
every measured pair. Both paths produced checksum `15760848229908480`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 57,360 | 57,345 | 0.026% |
| Requested allocation bytes | 7,692,160 | 3,497,984 | 54.525% |
| Snapshot P50 | 15.0005 ms | 13.1026 ms | 12.652% |
| Snapshot P95 | 26.3770 ms | 16.6414 ms | 36.909% |

Evidence marker: `RUNTIME07_PREALLOCATED_HOST_CAPABILITY_SNAPSHOT_MODEL_V1`.

The 57,344 required label clones dominate allocation-call totals. The removed 15 calls are the
result vector's growth events, while exact capacity removes their cumulative requested bytes. A
second full run remained favorable: P50 improved 14.600% and P95 improved 32.288%.

## Validation

- `python tools/tests/test_runtime07_preallocated_host_capability_snapshot_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks; the unstable-sort extension then
  failed the expected 1 of 3 checks before the final implementation.
- The standalone Rust model compiled with Rust 1.94.1, asserts exact ordered output equality, and
  passed two complete 31-sample runs.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07
  batch; this candidate will be paired with another completed optimization.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
