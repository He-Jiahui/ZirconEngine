---
title: Runtime11c Glyph Page Single-pass Index
category: zircon_runtime
report_id: Runtime11c-glyph-page-single-pass-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11c Glyph Page Single-pass Index

## Scope

This slice removes repeated full page scans while selecting the smallest free glyph-atlas page
index. Per-format limits, smallest-gap allocation, other-format isolation, rebuild eviction
priority, LRU behavior, current-frame pinning, generation, and reservation output remain unchanged.

## Change

- Build one per-format occupancy summary with page count and a 128-bit low-index mask.
- Resolve the common small-page case directly with `trailing_ones` and no allocation.
- For 128 or more pages, fill one bounded boolean bitmap in a second linear pass.
- Reuse the same occupancy contract in normal allocation and rebuild fallback.

## Deterministic Performance Evidence

| 2,048 dense pages, two allocation decisions per sample | Before | After |
|---|---:|---:|
| Resident-page visits per sample | 4,204,544 | 8,192 |
| Bitmap probes per sample | 0 | 4,098 |
| Candidate-driven full page rescans | 4,098 | 0 |
| Small-case heap allocations below 128 pages | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME11C_GLYPH_PAGE_SINGLE_PASS_INDEX_BENCH_V1`. Acceptance requires single-pass page indexing
P95 to be at least 90% below repeated page scans. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826az_glyph_page_index_preserves_smallest_sparse_gap` covers format
  isolation, low/high sparse indices, and smallest-gap allocation.
- `optimization_batch_20260826az_glyph_page_index_uses_single_occupancy_summary` requires the
  inline mask, linear bitmap fallback, and removal of candidate-driven page scans.
- `optimization_batch_20260826az_glyph_page_single_pass_index_p95` reports paired P50/P95 samples
  and enforces the 90% P95 reduction gate.

## Remaining Parent-plan Work

Runtime11c still owns unified atlas budgets, device generation, pressure, cross-renderer identity,
upload ordering, fallback policy, and product-scale GPU receipts. This slice only converges glyph
page-index selection.
