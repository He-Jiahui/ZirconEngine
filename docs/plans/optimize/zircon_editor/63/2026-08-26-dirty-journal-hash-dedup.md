---
title: Editor63 Dirty Journal Hash Dedup
category: zircon_editor
report_id: Editor63-dirty-journal-hash-dedup-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor63 Dirty Journal Hash Dedup

## Scope

This slice reduces the cost of projecting a bounded dirty-history journal into an ordered delta.
It also removes redundant ordered-tree construction from Reset projection. It does not change the
engine-wide operation gate, transaction serialization, history capacity, or save authority.

## Change

- Deduplicate replayable journal changes in a preallocated `HashSet<HistoryContextId>`.
- Materialize only unique histories into a vector and sort once at the public batch boundary.
- Project Reset histories directly from the already ordered generation map into a vector.
- Preserve ascending history order, one state per history, journal visit accounting, and cursor
  semantics.

## Deterministic Performance Evidence

| 65,536 journal changes / 8,192 unique histories | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered dedup admissions | 65,536 | 0 | 100% removed |
| Hash dedup admissions | 0 | 65,536 | average O(1) admission |
| Values sorted at output | implicit in tree | 8,192 | unique values only |
| Public delta order | ascending unique | ascending unique | unchanged |

Reset projection also changes from rebuilding a `BTreeSet` from already ordered map keys to a
single vector collection. The ignored release gate alternates 17 ordered-dedup and hash-plus-sort
samples. It emits `EDITOR63_DIRTY_JOURNAL_HASH_DEDUP_BENCH_V1`; acceptance requires hash-plus-sort
P95 to be at most 60% of ordered-dedup P95. Exact Windows timings remain pending the batched
coordinator run.

## Acceptance

- `optimization_batch_20260826o_editor63_hash_dedup_preserves_sorted_dirty_delta` covers duplicate
  changes, ascending unique output, and journal visit accounting.
- `optimization_batch_20260826o_editor63_dirty_journal_uses_hash_dedup` requires preallocated hash
  dedup, output sorting, and vector projection while rejecting an ordered production set.
- `optimization_batch_20260826o_editor63_dirty_journal_hash_dedup_performance_evidence` emits
  workload counts and both P95 values, then enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor63 still serializes otherwise independent histories through one operation gate. Per-document
admission, async transaction ownership, save/autosave coordination, object-generation validation,
and full multi-document scale qualification remain open.
