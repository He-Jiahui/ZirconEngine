---
title: Runtime59 Allocation-free Completed Expiry Sweep
category: zircon_runtime
report_id: Runtime59-allocation-free-completed-expiry-sweep-2026-08-26
date: 2026-08-26
session_id: root-runtime59-two-task-asset-expiry-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime59 Allocation-free Completed Expiry Sweep

## Scope

This slice removes temporary key storage and the second hash-table lookup phase from asset-worker
completed-result expiry maintenance. It preserves completion-deadline checks, retained entry/byte
budget release, terminal notification, expiry reporting, and all public runtime contracts.

## Implementation

`expire_entries` previously scanned `completed`, cloned every expired `AssetRequest` into a temporary
`Vec`, then hashed each cloned key again to remove the completion, subtract its retained bytes, and
publish the terminal state. The optimized path uses `HashMap::retain` to account, terminate, and
remove expired completions in one traversal, then applies the accumulated byte reduction once.

The regression keeps one expired and one live completion, verifies exact entry and byte reporting,
confirms the live payload remains retained, and proves the expired entry reaches its terminal state.

## Performance Contract

| Evidence for 4,096 entries / 2,048 expirations | Retired path | Optimized gate |
| --- | ---: | ---: |
| Expired request-key clones | 2,048 | 0 |
| Temporary key vectors | 1 | 0 |
| Second-phase hash removals | 2,048 | 0 |
| Retired completion bytes | 6,144 | 6,144 |
| Alternating release benchmark | 11 samples x 64 sweeps | optimized P95 <= 60% of retired P95 |

The benchmark emits `RUNTIME59_ALLOCATION_FREE_COMPLETED_EXPIRY_SWEEP_BENCH_V1` with both P95
timings, reduction basis points, sample/iteration/entry/key-byte/expired-byte counts, clone/vector
counts, and second-phase hash removals.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, source-structure gates, and the focused expiry
regression are required before submission. The managed `runtime59_asset_expiry_` release gate covers
both expiry optimizations in one Cargo invocation: 2 source contracts, 4 Rust tests, and 2 performance
rows. Dynamic P95 evidence, integration SHA, automatic commit, and automatic WeCom performance
delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime59 still owns execution-runtime lifecycle, task scopes, typed results, cancellation,
dependency validation, thread budgets, timer convergence, shutdown, and product diagnostics. This
micro-optimization does not claim those milestones complete.
