---
title: Editor01 Projection Composition Hash Index
category: zircon_editor
report_id: Editor01-projection-composition-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Projection Composition Hash Index

## Scope

This slice removes the nested linear scan used to map source projection rows back to composed
output rows. The mapping remains source ordered, unmatched rows remain `None`, and duplicate
composed identities still resolve to the first output row.

## Change

- Build one capacity-sized `HashMap<(&str, &str), usize>` over composed `(node_id, control_id)`
  identities.
- Admit rows with `or_insert`, preserving the previous `Iterator::position` first-match rule.
- Resolve each source row through borrowed composite-key lookup and publish the existing
  `Vec<Option<usize>>` shape.
- Leave projection cache ownership, row patching, generation comparison, and composition output
  order unchanged.

## Deterministic Performance Evidence

| Representative 2,048 reversed source/composed rows | Before | After |
|---|---:|---:|
| Candidate-row visits | 2,098,176 | 2,048 index inserts + 2,048 lookups |
| Time complexity | `O(S * C)` | expected `O(S + C)` |
| Identity string clones | 0 | 0 |
| Source order / first duplicate | preserved | preserved |

The ignored release gate runs 17 alternating samples and emits
`EDITOR01_PROJECTION_COMPOSITION_HASH_INDEX_BENCH_V1`. Acceptance requires hash-index P95 to be at
most 60% of nested-linear P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826ah_editor01_hash_index_preserves_source_order_and_first_match`
  covers reversed output order, an unmatched source row, and duplicate composed identities.
- `optimization_batch_20260826ah_editor01_projection_composition_uses_first_row_hash_index`
  requires capacity-sized hash indexing with first-row admission and rejects `.position` scanning.
- `optimization_batch_20260826ah_editor01_projection_composition_hash_index_performance_evidence`
  checks output equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts passed before managed
  validation submission.

## Remaining Parent-plan Work

Editor01 still needs generation-bound async loading, cancellation, retained-tree patching,
product-scale startup and interaction qualification, and complete retained UI performance
evidence. This slice only replaces the source/output row lookup algorithm during full composition.
