# Editor57 Creation Menu Hash Index Optimization Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: Editor57 asset type registry preservation and 100K/1M performance gates
- Status: implementation and release gate authored; batched managed validation pending

## Problem

`AssetCreationMenuGeneration::compile` used a `BTreeMap` for label counts, a
`BTreeSet` for used labels, and another `BTreeMap` for suffix cursors. These
structures are only queried for count, membership, and keyed cursor updates;
they are never iterated to determine output order. Candidate iteration already
owns the deterministic menu order, so balanced-tree comparisons added
`O(N log N)` index work without providing an ordering contract.

## Change

- Use capacity-sized `HashMap`/`HashSet` indexes for label counts and membership.
- Keep suffix cursors in a `HashMap`; cursor values, not map iteration, determine
  the visible suffix sequence.
- Preserve candidate order, generation/ordinal action IDs, label disambiguation,
  collision suffixes, action lookup, and final immutable menu entries.
- Retain `BTreeMap`/`BTreeSet` in registry domains where deterministic iteration
  is part of the actual contract.

## Deterministic Performance Evidence

| 32,768 unique creation labels | Before | After |
|---|---:|---:|
| Count/membership index | balanced trees | capacity-sized hash tables |
| Expected index complexity | `O(N log N)` | average `O(N)` |
| Candidate/output order | registry order | registry order |
| Collision projection | deterministic suffix cursor | identical |

The ignored release gate runs 17 alternating tree/hash index sample pairs.
Acceptance requires hash-index nearest-rank P95 to be at most 70% of tree-index
P95, a minimum 30% reduction. Exact Windows timings remain pending the batched
coordinator run.

## Acceptance

- `optimization_batch_20260826e_editor57_creation_menu_uses_hash_membership_indexes`
  locks all three non-ordering indexes to hash ownership.
- `optimization_batch_20260826e_editor57_creation_menu_hash_indexes_preserve_labels`
  compares tree/hash outputs across repeated bases and suffix collisions.
- `optimization_batch_20260826e_editor57_creation_menu_hash_index_performance_evidence`
  emits `EDITOR57_CREATION_MENU_HASH_INDEX_BENCH_V1`, raw samples, label count,
  index kinds, and the 30% P95 threshold.
- Exact-file Rust 1.94.1 rustfmt with child traversal disabled, source contracts,
  and scoped diff checks must pass before managed validation submission.

## Remaining Plan Work

This slice does not close Editor57. Exact asset type propagation, Browser create
surfaces, per-instance state, paged/lazy source trees, multi-selection, mutation
operations, provider capabilities, history/favorites/collections, async import,
preview residency, and 100K/1M product qualification remain open.
