# Editor59 Highlight Vec Normalization Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: Editor59 M5 large-selection scale/performance qualification
- Status: implementation and release gate authored; batched managed validation pending

## Problem

Every `EditorRuntimeHighlightSet` normalized entity IDs by inserting them into a temporary
`BTreeSet`, then collecting a second final vector. Large multi-selection overlay submissions paid
ordered-tree admission and transient tree storage even though the product contract only needs a
sorted unique slice.

## Change

- Collect the input directly into the final entity `Vec`.
- Normalize in place with `sort_unstable` and `dedup`.
- Preserve ascending deterministic entity order, duplicate removal, viewport/generation values,
  visual settings, and validity behavior.

## Deterministic Performance Evidence

| 32,768 unique entity IDs | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-tree insertions | 32,768 | 0 | 100% removed |
| Transient ordered-set entries | 32,768 | 0 | 100% removed |
| Normalization structures | `BTreeSet` then final `Vec` | final `Vec` only | tree removed |
| Final entity order | ascending unique | ascending unique | unchanged |

The ignored release gate alternates 17 tree-normalization and in-place vector samples over a
deterministic pseudo-random permutation. It emits `EDITOR59_HIGHLIGHT_VEC_NORMALIZATION_BENCH_V1`;
acceptance requires vector P95 to be at most 60% of legacy P95. Exact Windows timings remain
pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826h_editor59_highlight_normalization_preserves_sorted_unique_entities`
  covers ordering, deduplication, metadata, and validity.
- `optimization_batch_20260826h_editor59_highlight_normalization_uses_one_vec` rejects production
  `BTreeSet` use and requires in-place sort/dedup.
- `optimization_batch_20260826h_editor59_highlight_vec_normalization_performance_evidence` emits
  both P95 values and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

This slice does not close Highlight runtime consumption, overlay revision/currentness, base-frame
failure isolation, per-view receipt states, selectable spatial-product convergence, or the full
100k/1m selection qualification matrix.
