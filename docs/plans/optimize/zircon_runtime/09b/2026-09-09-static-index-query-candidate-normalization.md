---
title: Runtime213 Static Index Query Candidate Normalization
category: zircon_runtime
report_id: Runtime213-static-index-query-candidate-normalization-2026-09-09
date: 2026-09-09
session_id: astra-optimize-20260909-root
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime213 Static Index Query Candidate Normalization

## Finding

`VisibilityStaticIndex` stored each cell's membership in an ordered set, then
recreated a temporary `BTreeSet` for every bounded bounds/ray query before
collecting the result into a `Vec`. Dense queries therefore paid one tree-node
allocation and one ordered insertion per candidate, followed by a second
linearization pass. The parent Runtime09B/Runtime213 review identifies this
candidate path as a visibility hot path; this slice does not change the grid
budget or fallback semantics.

## Change

- Collect overflow and cell memberships directly into one `Vec<u64>`.
- Sort the contiguous candidates once and deduplicate at the query boundary,
  preserving the sorted, unique stable-key contract.
- Keep the persistent `BTreeMap`/`BTreeSet` index storage, cell budget, overflow
  fallback, and visited-cell accounting unchanged.

## Regression coverage

`visibility_static_index_query_normalizes_overlapping_cell_memberships` checks
cross-cell duplicate suppression and stable ordering. The
`optimization_batch_runtime213_static_index_query_uses_vec_normalization`
source contract requires the production helper to sort/deduplicate in place and
rejects reintroduction of a query-local `BTreeSet`.

## Local evidence

- The touched Rust source passed parse-only `rustfmt --edition 2021 --emit stdout`.
- Scoped `git diff --check`, trailing-whitespace checks, and the Runtime09B/
  Runtime213 visibility contract batch passed.
- No Cargo command was run. Managed Windows correctness and release p50/p95/p99
  comparison remain pending under the immutable validation gate.

## Acceptance boundary

This record is implementation-complete only. It does not claim that the
single-level grid, positive-radius cell budget, persistent scene ownership, or
broader VIS213 performance gates are complete.
