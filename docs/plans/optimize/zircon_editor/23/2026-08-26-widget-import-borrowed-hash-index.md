# Editor23 Widget Import Borrowed Hash Index Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: Editor23 P1-54 and performance qualification 29 for large UI assets
- Status: implementation and release gate authored; batched managed validation pending

## Problem

Widget-import replay cloned every target `String` into a `BTreeSet<String>` before removing stale
imports. Duplicate detection separately rebuilt a borrowed ordered set. Large UI documents paid
tree admission plus target string allocation even though neither path consumes ordered iteration.

## Change

- Build one capacity-sized `HashSet<&str>` borrowing the target import strings.
- Use borrowed lookup while retaining the existing ordered `working` command sequence.
- Use a capacity-sized borrowed hash set for duplicate detection.
- Preserve remove, move, insert, and duplicate-input full-set fallback semantics.

## Deterministic Performance Evidence

| 32,768 target imports | Before | After | Reduction |
|---|---:|---:|---:|
| Target `String` clones | 32,768 | 0 | 100% removed |
| Target string bytes copied | 1,179,648 | 0 | 100% removed |
| Ordered-tree admissions | 32,768 | 0 | 100% removed |
| Borrowed hash admissions | 0 | 32,768 | average O(1) index |
| Replay command order | deterministic | deterministic | unchanged |

The ignored release gate alternates 17 cloned-tree and borrowed-hash samples. It emits
`EDITOR23_WIDGET_IMPORT_BORROWED_HASH_INDEX_BENCH_V1`; acceptance requires borrowed-hash P95 to be
at most 60% of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826k_editor23_widget_import_hash_index_preserves_replay` covers ordered
  remove/move/insert commands and duplicate-input fallback.
- `optimization_batch_20260826k_editor23_widget_import_index_borrows_hash_keys` requires exact
  capacity, borrowed keys, and zero target-index clones while rejecting production `BTreeSet` use.
- `optimization_batch_20260826k_editor23_widget_import_borrowed_hash_performance_evidence` emits
  both P95 values and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

This slice does not close Editor23's document repository, lossless V2 editing, cascade authority,
theme/token refactors, compiled UI artifacts, virtualization, or complete 1k/10k/100k authoring
qualification.
