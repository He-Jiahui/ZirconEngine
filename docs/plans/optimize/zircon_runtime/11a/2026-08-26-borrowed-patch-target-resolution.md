---
title: Runtime11A Borrowed Patch Target Resolution
category: zircon_runtime
report_id: Runtime11A-borrowed-patch-target-resolution-2026-08-26
date: 2026-08-26
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A Borrowed Patch Target Resolution

## Scope

This slice removes per-patch `UiTreeId` cloning from reflection-patch validation and target
resolution. It preserves validate-before-mutate atomicity, input ordering, changed-node
deduplication, one diff per changed tree, notification behavior, and all public runtime contracts.

## Implementation

`apply_reflection_patches` previously cloned the indexed `(UiTreeId, UiNodeId)` tuple for every
patch before validation. The optimized resolved list borrows each tree ID from the stable
`node_index`, copies only the compact node ID and patch index, mutates the disjoint `trees` field,
and clones a tree ID only when creating the owned changed-tree map entry.

The ignored release benchmark applies 4,096 valid no-op patches under a 16 KiB tree identifier. It
compares the retired two-phase implementation with the borrowed target-resolution path while
keeping validation and mutation traversal equivalent.

## Performance Contract

| Evidence for 4,096 no-op patches | Retired path | Optimized gate |
| --- | ---: | ---: |
| Tree ID clones during target resolution | 4,096 | 0 |
| Tree ID bytes cloned during target resolution | 67,108,864 | 0 |
| Validate-before-mutate passes | 2 | 2 |
| Alternating release benchmark | 21 paired samples | optimized P95 <= 75% of retired P95 |

The benchmark emits `RUNTIME11A_BORROWED_PATCH_TARGET_RESOLUTION_BENCH_V1` with patch/tree-ID
counts, structural clone bytes, paired P50/P95 timings, and all raw samples for coordinator-owned
WeCom reporting.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, and source-structure gates are required before
submission. One managed Runtime11A Cargo invocation filtered by `runtime11a_` covers this benchmark
together with the direct property-query regression and benchmark. Dynamic P95 evidence,
integration SHA, and automatic WeCom performance delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime11A still requires a live-surface reflection generation, bounded notification fanout,
transactional write routing, incremental indexes/deltas, and native accessibility integration. This
focused allocation optimization does not claim those milestones complete.
