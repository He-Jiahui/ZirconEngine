---
title: Editor57 Compact Thumbnail Card Frame Cache
category: zircon_editor
date: 2026-09-27
implementation_status: implemented
validation_status: batched_validation_pending
performance_status: release_measurement_pending
---

# Editor57 Compact Thumbnail Card Frame Cache

## Product path and scope

The current Asset Browser calls `apply_compact_thumbnail_grid_layout` when a thumbnail grid uses the compact layout. Each materialized asset contributes nine thumbnail nodes. The previous loop called `thumbnail_card_frames` for every node, rebuilding all nine role frames up to nine times per card. This work is on the active visible-list layout path, after the materialized item budget is chosen.

The layout now lazily caches the optional frame set by card index for one layout call. It computes a card's geometry once when its first recognized node is reached, including the collapsed-width `None` result. Grid extent, type-label measurement, filename compaction, sparse-index handling, and unrelated nodes keep their previous behavior. The cache is local to the layout call, so a later viewport or text change is recomputed. Its size follows the existing maximum Card index plus one, like the pre-existing layout input vector.

## Regression and Release comparison

- `compact_thumbnail_layout_matches_per_node_geometry_for_interleaved_sparse_cards` compares every frame, text, and extent against the retired per-node layout for interleaved card parts, different type labels and continuation lines, a missing Card slot, an out-of-range node, and both usable and collapsed widths. The old implementation is retained in the test module as an executable behavior and performance reference. The regression was written before the cache change; managed Cargo execution remains pending.
- Ignored Windows Release `editor57_compact_thumbnail_frame_cache_release_benchmark` measures the complete `apply_compact_thumbnail_grid_layout` call against the retired call on the same pre-cloned node vectors. After five warmups it alternates old/new order across 31 samples and prints `PERF_RESULT EDITOR57_COMPACT_THUMBNAIL_FRAME_CACHE_BENCH_V1` with p50/p95/p99, the raw samples in acquisition order, OS, architecture, and package version. Cloning and fixture construction are outside the timed region; count, input extraction, geometry, text work, and cache allocation are inside. The managed receipt must supply compiler, build, machine, and cache metadata.
- The 48-card fixture represents the materialized maximum for a 900 by 620 grid with two overscan rows on each side. Its pending p95 non-regression gate is new at most 105% of old. The 240-card fixture stresses five times that materialized work and has a pending p95 improvement gate of new at most 85% of old. Both fixtures contain all nine nodes per card and representative name, type, and metadata text. The benchmark prints each fixture's samples and percentiles before asserting its gate.

These measurements cover the compact thumbnail layout only. They do not measure the surrounding Asset Browser update, retained host, interaction latency, or the Editor57 100,000-asset navigation p95/p99 and 1,000,000-asset paged-provider gates. No Release or product pass is claimed until the combined managed validation produces evidence.

## Validation manifest

| Gate | Scope | State |
|---|---|---|
| Source | `zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_layout.rs` | Rustfmt and scoped diff checks passed; Cargo pending |
| Behavior | `compact_thumbnail_layout_matches_per_node_geometry_for_interleaved_sparse_cards` plus existing thumbnail layout tests | queued for combined Editor Cargo batch |
| Performance | ignored `editor57_compact_thumbnail_frame_cache_release_benchmark`; p95 at most 105% at 48 cards and at most 85% at 240 cards | queued for combined Windows Release batch |

This is an implementation candidate pending compilation, behavior regressions, and actual Release measurements.
