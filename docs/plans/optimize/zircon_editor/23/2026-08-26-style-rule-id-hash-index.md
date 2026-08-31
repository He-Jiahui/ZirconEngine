# Editor23 Style Rule ID Hash Index Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: Editor23 P1-54 and performance qualification 29 for large rule sets
- Status: implementation and release gate authored; batched managed validation pending

## Problem

Creating or extracting a stylesheet rule rebuilt a borrowed `BTreeSet<&str>` over every rule ID,
then performed ordered-tree membership checks while choosing the base or first free numeric suffix.
Large UI assets therefore paid O(N log N) tree admission even though ID generation does not consume
ordered iteration.

## Change

- Sum the existing stylesheet rule counts without walking each rule twice.
- Allocate one `HashSet<&str>` with that capacity and borrow every present rule ID.
- Preserve selector stem normalization and the first available suffix sequence beginning at 2.
- Keep rule IDs borrowed from the document; no string clones are introduced.

## Deterministic Performance Evidence

| 32,768 rule IDs | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-tree admissions | 32,768 | 0 | 100% removed |
| Hash admissions | 0 | 32,768 | average O(1) index |
| Rule ID string clones | 0 | 0 | unchanged |
| Generated ID semantics | base / first free suffix | base / first free suffix | unchanged |

The ignored release gate alternates 17 borrowed-tree and capacity-hash samples over long rule IDs.
It emits `EDITOR23_STYLE_RULE_ID_HASH_INDEX_BENCH_V1`; acceptance requires hash P95 to be at most
60% of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826j_editor23_style_rule_id_preserves_first_free_suffix` covers the base
  and continuous suffix cases.
- `optimization_batch_20260826j_editor23_style_rule_id_uses_capacity_hash_index` requires exact
  capacity and borrowed string keys while rejecting the production tree.
- `optimization_batch_20260826j_editor23_style_rule_id_hash_index_performance_evidence` emits both
  P95 values and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

This slice does not close Editor23's document repository, lossless V2 editing, cascade authority,
theme/token refactors, compiled UI artifacts, virtualization, or complete 1k/10k/100k authoring
qualification.
