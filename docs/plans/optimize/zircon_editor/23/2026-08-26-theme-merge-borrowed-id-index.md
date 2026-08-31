# Editor23 Theme Merge Borrowed ID Index Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: Editor23 theme authoring and P1-54 large-asset qualification
- Status: implementation and release gate authored; batched managed validation pending

## Problem

Imported-theme merge cloned every existing style import, local stylesheet ID, and token name into
three ordered sets before resolving collisions. These sets provide membership only; final document
order is owned by import vectors, stylesheet vectors, and token maps.

## Change

- Add a capacity-sized `UsedIdentifierSet` with borrowed existing IDs and owned newly admitted IDs.
- Reuse it for style imports, stylesheet collision resolution, and imported-token rename planning.
- Complete import selection before mutating the source vector, then drop the borrowed index.
- Preserve import insertion order, collision suffixes, stylesheet order, and token rename order.
- Keep the ordered `imported_theme_rules` return contract unchanged.

## Deterministic Performance Evidence

| Three existing-ID indexes x 32,768 long IDs | Before | After | Reduction |
|---|---:|---:|---:|
| Existing ID string clones | 98,304 | 0 | 100% removed |
| Existing ID bytes copied | 4,718,592 | 0 | 100% removed |
| Ordered-tree admissions | 98,304 | 0 | 100% removed |
| Borrowed hash admissions | 0 | 98,304 | average O(1) indexes |
| Merge/collision order | deterministic | deterministic | unchanged |

The ignored release gate alternates 17 three-tree and three borrowed-hash samples. It emits
`EDITOR23_THEME_MERGE_BORROWED_ID_INDEX_BENCH_V1`; acceptance requires optimized P95 to be at most
60% of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826m_editor23_borrowed_merge_indexes_preserve_collisions` covers import
  deduplication, token suffix collision, and stylesheet suffix/order behavior.
- `optimization_batch_20260826m_editor23_theme_merge_indexes_borrow_existing_ids` requires all
  three borrowed indexes, exact capacity planning, and removal of existing-ID clone collection.
- `optimization_batch_20260826m_editor23_theme_merge_borrowed_hash_performance_evidence` emits
  clone/copy counts and both P95 values, then enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

This slice does not close Editor23's typed theme tokens, cascade authority, lossless document
editing, compiled UI artifacts, dependency invalidation, virtualization, or complete
1k/10k/100k authoring qualification.
