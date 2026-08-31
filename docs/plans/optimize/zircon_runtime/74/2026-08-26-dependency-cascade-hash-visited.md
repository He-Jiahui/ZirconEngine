---
title: Runtime74 Dependency Cascade Hash Visited Membership
category: zircon_runtime
report_id: Runtime74-dependency-cascade-hash-visited-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime74 Dependency Cascade Hash Visited Membership

## Scope

This slice removes logarithmic visited-membership checks from UI dependency cascade traversal. The
reverse dependency index remains an ordered map of ordered sets, so direct dependent order and BFS
publication order remain unchanged. Only the non-published visited index changes.

## Change

- Replace the cascade-local `BTreeSet<&str>` with `HashSet<&str>`.
- Continue borrowing graph-owned asset IDs in the visited set and queue.
- Allocate owned strings only when publishing rebuild targets.
- Keep reverse dependency storage, query ordering, and reference deduplication unchanged.

## Deterministic Performance Evidence

| Representative 65,536 visits / 8,192 unique dependency nodes | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Graph identity clones for visited/queue | 0 | 0 |
| Published target order | sorted-dependent BFS | sorted-dependent BFS |

The ignored release gate runs 17 alternating samples and emits
`RUNTIME74_DEPENDENCY_CASCADE_HASH_VISITED_BENCH_V1`. Acceptance requires hash-visited P95 to be at
most 60% of ordered-visited P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826x_runtime74_dependency_cascade_preserves_sorted_bfs_order` verifies
  ordered siblings and shared-parent deduplication through the product index.
- `optimization_batch_20260826x_runtime74_dependency_cascade_uses_borrowed_hash_visited` updates
  the existing source boundary contract to require borrowed hash membership.
- `optimization_batch_20260826x_runtime74_dependency_cascade_hash_visited_performance_evidence`
  checks admission equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime74 still needs dependency-complete compiled generations, atomic tree replacement, state
migration, binding reinstall, rollback, and old-generation subscription retirement. This slice
only improves the dependency closure traversal primitive.
