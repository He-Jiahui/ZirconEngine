# Runtime55 path commit-gate reclaim

- Owner: `optimize-runtime55-path-gate-reclaim-r1-01a00797-20260821`
- Source plan: `55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md`
- Finding: `FND-P1-039`
- Status: implementation and deterministic cardinality evidence complete; combined managed Cargo validation pending

## Problem

The process-wide config path commit-gate registry stored a `PathBuf` and a dead `Weak` for every
path ever opened. Dropping the final manager released the gate but never removed its key. Long-lived
processes that create runtimes for many projects or sessions therefore retained path allocations and
grew the shared hash table without a bound.

## Change

`ConfigCommitFence::drop` now reclaims its registry entry when it owns the final strong reference to
the shared path gate. It checks the strong count both before and while holding the registry lock, and
compares the registered weak pointer with its gate before removal. A concurrent registration either
upgrades the old gate before the check or creates a new gate after removal; it cannot have its live
entry removed by an older fence.

The registration path does not sweep unrelated keys. Epoch allocation, same-path supersession,
commit serialization, cancellation, path normalization, and poison recovery are unchanged.

## Deterministic evidence

The release workload opens and closes 65,536 unique normalized paths.

| Metric | Legacy dead-Weak registry | Last-owner reclaim | Reduction |
| --- | ---: | ---: | ---: |
| Final stale entries | 65,536 | 0 | 100% |
| Peak entries | 65,536 | 1 | 99.998% |
| Retained path-string bytes | 1,966,080 | 0 | 100% |

The byte count includes only the 30 encoded bytes in each workload path. It deliberately excludes
`PathBuf`, hash-table, allocator, and `Weak` overhead, so it is a conservative retained-memory
floor rather than an RSS claim.

The release benchmark also records 21 alternating legacy/reclaimed timing pairs and independently
computable nearest-rank P50/P95. Timing is diagnostic only because this change targets retained
cardinality; the release gate is exact zero stale entries and a peak of one for sequential owners.

## Acceptance

- `path_commit_gate_registry_reclaims_only_after_the_last_fence_drops` proves an earlier same-path
  fence cannot remove a live shared gate and that the last fence reclaims the path immediately.
- `path_commit_gate_registry_reclaim_release_benchmark` emits raw samples and exact cardinality and
  retained-key fields for the 65,536-path workload.
- The managed Runtime Rust batch runs the existing config-manager regressions and this ignored
  release benchmark together. No per-task Cargo process is launched from this session.

Pinned validation artifacts:

- Runtime55 child: `zircon-validation-runtime55-path-gate-reclaim.ps1`, SHA-256
  `F6BD9BA29C57FC9C707E83608B90802905865FDCD8ACD83E1E4014EB0A8F42E9`.
- Eight-task Runtime batch: `zircon-validation-runtime-rust-followup-eight.ps1`, SHA-256
  `551117BF4F52C78DF7FE566319A365CBEFD9FC22EB13CB5888C9F1AADC54D5EA`.
- Both scripts parse with zero PowerShell AST errors.

## Remaining scope

This closes only `FND-P1-039`. Foundation's three P0 items, multi-runtime path scope, checked epoch
exhaustion, typed config authority, durable projection, retry, shutdown receipts, and the remaining
Runtime55 qualification gates stay open.
