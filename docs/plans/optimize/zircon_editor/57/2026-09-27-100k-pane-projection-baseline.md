---
title: Editor57 100k Asset Browser Pane Projection Baseline
category: zircon_editor
date: 2026-09-27
implementation_status: implemented
validation_status: batched_validation_pending
performance_status: baseline_measurement_pending
---

# Editor57 100k Asset Browser Pane Projection Baseline

## Scope and frozen input

The existing Asset Browser virtualization tests project 10,000 assets through `asset_browser_pane_nodes` and verify bounded retained slots and complete logical scroll extent. This record extends that same Rust seam to 100,000 `AssetItemSnapshot` values in both list and thumbnail modes. The viewport is fixed at 900 × 620, the catalog revision at 1, and the selection variants at none, first item, item 50,000, and item 100,000. The source generation is constructed before timing and shared across selection variants.

`asset_browser_pane_data` is the measured production entry. It includes pane cache admission, logical paint generation, template composition, responsive layout, virtual slot creation, and paint metadata construction. The initial phase alone clears the pane and logical paint caches before the timer, so each timed call rebuilds the 100,000-item logical paint projection while static template resources can stay warm. The select-first phase primes an unselected pane before timing selection of the first asset; select-middle and select-last prime panes with selected assets 1 and 50,000, respectively. These selection phases exercise pane recomposition while the immutable logical paint source is reusable. Snapshot creation, cache reset, prior-state priming, and output inspection are outside the timed region.

## Regression and Release baseline

- `hundred_thousand_asset_pane_projection_preserves_extent_pool_and_selection` checks all four selection variants in both modes. It requires paint metadata to report exactly 100,000 logical items and active virtualization, the selection locator to resolve even for the last logical item, the initial first row or card and slot binding to be selected only when asset 1 is selected, and middle and tail virtual slot bindings to resolve the selected logical index at the corresponding scroll position while neighboring bindings stay unselected. It also checks list row count against the conservative viewport budget, thumbnail card count against the final grid budget, and each pool and paint metadata's materialized count at or below 64 items. The pools must be nonempty and the list or grid extent must exceed the viewport and include all 100,000 logical assets. These slot checks inspect pane metadata at synthetic scroll offsets; they do not dispatch native scroll input or render those scrolled frames. The existing 10,000-asset tests remain intact.
- Ignored Windows Release `editor57_hundred_thousand_asset_pane_projection_release_benchmark` emits `PERF_RESULT EDITOR57_100K_ASSET_PANE_PROJECTION_BENCH_V1` for initial, select-first, select-middle, and select-last phases in each view mode. Every phase performs five warmups and records 31 individual timed projections. The output includes all raw nanosecond samples in acquisition order, nearest-rank p50/p95/p99, viewport, item count, OS, architecture, and package version. With 31 samples, nearest-rank p99 is the largest observation; this is a reproducible baseline, not a statistically stable tail-latency acceptance claim. The managed receipt must add compiler, build, machine, and cache metadata.

The structural pool and extent assertions are executable requirements. Editor57-G37 defines a 100,000-asset navigation, selection, and action p95/p99 baseline but gives no numeric pane-projection latency budget; this benchmark does not invent one. It does not drive native pointer events, scroll, action dispatch, retained paint, GPU submission, or present. The existing product `asset_browser_scroll` profile accepts at most 10,000 catalog files and does not report p99, so the 100,000-asset product qualification remains pending.

## Validation manifest

| Gate | Scope | State |
|---|---|---|
| Source | `zircon_editor/src/ui/layouts/views/asset_browser/tests/virtualization.rs` | Rustfmt, scoped diff, UTF-8, LF, and trailing-whitespace checks passed; Cargo pending |
| Behavior | `hundred_thousand_asset_pane_projection_preserves_extent_pool_and_selection`; existing 10,000-asset virtualization regressions | queued for combined Editor Cargo batch |
| Performance | ignored `editor57_hundred_thousand_asset_pane_projection_release_benchmark`; eight 100,000-item phase/mode outputs | queued for combined Windows Release batch; numeric p99 budget undefined |

This fixture is implemented pending compilation and actual Release samples. Product Editor57-G37 remains open.
