---
title: Editor01 Palette Selection Index Fast Path
category: zircon_editor
report_id: Editor01-palette-selection-index-fast-path-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Palette Selection Index Fast Path

## Scope

This slice reuses the retained palette selection index after a catalog rebuild when that index
still names an entry exactly equal to the retained selected entry. The stable refresh path now
performs one equality check instead of scanning the full palette from the first row.

If the index is absent, out of range, or names a different entry after reordering, the existing
linear equality scan remains the fallback. Selection clamping, selected-entry cloning, drag-state
reset, layout/v2 catalog construction, and typed conversion errors are unchanged.

## Deterministic Work Model

The release workload builds 512 unique palette entries with long shared prefixes and performs 4,096
stable reconciliation lookups for the last entry.

| Work per workload | Before | After |
|---|---:|---:|
| Entry equality comparisons | 2,097,152 | 4,096 |
| Full palette scans | 4,096 | 0 |
| Catalog rebuilds or entry clones in benchmark | 0 | 0 |
| Reorder fallback-policy changes | 0 | 0 |

Stable comparison work falls by 99.8%. The ignored release gate runs 17 alternating sample pairs
and emits `EDITOR01_PALETTE_SELECTION_INDEX_FAST_PATH_BENCH_V1`. Acceptance requires index
fast-path P95 to be at least 90% below the full scan. Exact Windows P50/P95 timings remain pending
the coordinator run.

## Acceptance

- `optimization_batch_20260826bt_palette_selection_index_fast_path_preserves_stable_index` covers
  retained and out-of-range previous indexes.
- `optimization_batch_20260826bt_palette_selection_index_fast_path_preserves_reorder_fallback`
  covers reordered entries and locks exact-index comparison before the fallback scan.
- `optimization_batch_20260826bt_palette_selection_index_fast_path_p95` reports paired release
  P50/P95 samples and enforces the 90% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained invalidation, layout/paint scaling, shared visual materialization,
accessibility, and product-scale interaction evidence. This slice only converges palette selection
reconciliation after catalog refresh.
