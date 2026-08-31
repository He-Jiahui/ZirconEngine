# Editor23 Theme Replay Borrowed Hash Index Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: Editor23 P1-54 and performance qualification 29 for large UI assets
- Status: implementation and release gate authored; batched managed validation pending

## Problem

Theme replay cloned style imports, stylesheet IDs, and rule selectors into three independent
`BTreeSet<String>` indexes. Three duplicate checks separately used borrowed ordered sets. Large UI
documents therefore paid string allocation and O(N log N) admission even though none of these paths
consume ordered iteration.

## Change

- Centralize the three lookup indexes on a capacity-sized `HashSet<&str>` helper.
- Centralize import, stylesheet-ID, and selector duplicate detection on a second borrowed hash
  helper.
- Borrow keys from immutable target documents while retaining the ordered `working` replay vectors.
- Preserve deterministic remove, move, insert, replacement, and duplicate-input fallback behavior.
- Keep performance tests in a semantic child module so `theme_state.rs` remains at 833 lines.

## Deterministic Performance Evidence

| Three indexes x 32,768 long keys | Before | After | Reduction |
|---|---:|---:|---:|
| Target string clones | 98,304 | 0 | 100% removed |
| Target string bytes copied | 5,013,504 | 0 | 100% removed |
| Ordered-tree admissions | 98,304 | 0 | 100% removed |
| Borrowed hash admissions | 0 | 98,304 | average O(1) indexes |
| Replay command order | deterministic | deterministic | unchanged |

The ignored release gate alternates 17 three-tree and three borrowed-hash samples. It emits
`EDITOR23_THEME_REPLAY_BORROWED_HASH_INDEX_BENCH_V1`; acceptance requires optimized P95 to be at
most 60% of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826l_editor23_theme_hash_indexes_preserve_replay_semantics` covers
  import, stylesheet, and rule remove/insert/move behavior plus duplicate fallback.
- `optimization_batch_20260826l_editor23_theme_indexes_borrow_all_string_keys` requires all three
  borrowed indexes and all three hash duplicate checks while rejecting production trees/clones.
- `optimization_batch_20260826l_editor23_theme_borrowed_hash_performance_evidence` emits clone and
  copied-byte counts, both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

This slice does not close Editor23's typed theme tokens, cascade authority, document repository,
lossless V2 editing, compiled UI artifacts, virtualization, or complete 1k/10k/100k authoring
qualification.
