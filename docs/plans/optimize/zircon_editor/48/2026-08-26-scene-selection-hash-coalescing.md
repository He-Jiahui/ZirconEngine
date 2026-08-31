# Editor48 Scene Selection Hash Coalescing Optimization Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: Editor48 E-MSG-P1-23 continuous Scene Inspection delta coalescing and performance
  qualification
- Status: implementation and release gate authored; batched managed validation pending

## Problem

Continuous Scene Inspection selection deltas were coalesced through two `BTreeSet` accumulators.
Every retained addition/removal therefore paid balanced-tree update cost and node traffic even when
a later delta cancelled nearly every earlier entity, a common drag-selection and resync-adjacent
shape.

## Change

- Preallocate two `HashSet<EntityId>` accumulators from the incoming delta sizes.
- Preserve the existing add-versus-remove cancellation rules and revision continuity checks.
- Sort only the surviving output vectors, preserving deterministic ascending serialization and
  all existing duplicate elimination semantics.
- Keep generation gaps and explicit resync inputs on the unchanged compact resync path.

## Deterministic Performance Evidence

| 32,768 add-then-remove entities | Before | After | Structural result |
|---|---:|---:|---:|
| Accumulator updates | 65,536 balanced-tree updates | 65,536 average O(1) hash updates | tree traversal removed |
| Surviving entities to sort | 0 | 0 | unchanged |
| Output order | ascending | ascending | unchanged |
| Temporary per-entity tree nodes | required by `BTreeSet` storage | none | removed |

The ignored release gate alternates 17 tree and hash samples. It emits
`EDITOR48_SCENE_SELECTION_HASH_COALESCING_BENCH_V1`; acceptance requires hash P95 to be at most 60%
of tree P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826f_editor48_hash_coalescing_preserves_semantics`
  covers unordered duplicates and cross-delta cancellation.
- `optimization_batch_20260826f_editor48_hash_coalescing_uses_hash_accumulation` requires
  preallocated hashing, final deterministic sorting, and no production `BTreeSet` path.
- `optimization_batch_20260826f_editor48_hash_coalescing_performance_evidence` publishes
  both P95 values and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and all source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Qualified selection scope, hierarchy/focused-field delta composition, page and byte budgets,
acknowledgement, failed-apply retry, and full resync protocol remain open Editor48 work.
