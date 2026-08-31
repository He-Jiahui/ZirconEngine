# Runtime132 SDF Dirty-Page Binary Merge

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Status: `implementation_complete / managed_validation_pending`
- Batch: `optimization_batch_20260826co_`

## Problem

`merge_sdf_bake_dirty_pages` linearly searched every cached dirty page for every bake page and then
sorted the complete vector after the merge. Large paged glyph atlases therefore paid quadratic
lookup work even though the function already published its page reports in key order.

## Optimization

- Verify the existing vector is ordered and repair only unordered legacy input with one unstable
  sort.
- Use `binary_search_by_key` for every bake-page lookup and insert new pages at the returned ordered
  position.
- Preserve dirty-rectangle union behavior, unique page identity, ascending page order, and the
  legacy page-zero `dirty_rect` projection.

## Test And Performance Contract

- The behavior regression starts from unordered page reports, adds a middle page, merges an
  existing page, and checks exact order and rectangle union.
- The source regression requires the dirty-page binary lookup and rejects the old linear find and
  unconditional stable sort.
- Ignored release evidence prints `RUNTIME132_SDF_DIRTY_PAGE_BINARY_MERGE_BENCH_V1` for 21
  alternating sample pairs over 2,048 existing and 2,048 matching bake pages.
- Acceptance requires `optimized_p95_ns * 100 <= legacy_p95_ns * 70`.

## Validation State

Rust 1.94.1 formatting and scoped static checks are required before submission. Cargo results,
exact P50/P95 values, commit SHA, push result, and WeCom delivery remain coordinator-owned terminal
evidence and are not claimed by this pending record.

