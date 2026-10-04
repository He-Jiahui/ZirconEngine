---
title: Editor57 Active Dropdown Toolbar Control Snapshot
category: zircon_editor
date: 2026-09-26
implementation_status: implemented
validation_status: batched_validation_pending
---

# Editor57 Active Dropdown Toolbar Control Snapshot

## Product path and scope

The current `zircon_editor/assets/ui/editor/asset_browser.zui` declares `AssetBrowserKindFilterDropdown`, and `layout_single_toolbar_row` uses that dropdown branch. Before laying out a row, the toolbar read its panel frames, import/view/locate widths, dropdown width, and dropdown presence with eight separate first-match scans over `ViewTemplateNodeData`. This is part of the current Asset Browser layout path. It is separate from the fallback chips work in the adjacent Editor57 record.

`ToolbarControlSnapshot::from_nodes` now captures the seven distinct controls in one scan and stops after their first occurrences are found. The layout uses that snapshot for both dropdown width and presence. Each width still falls back when absent or non-positive; panel admission, all-match frame writes, responsive collapse, and duplicate-ID first-match reads retain their prior behavior. The source lives in `toolbar_layout/control_snapshot.rs` to keep the toolbar module bounded.

## Regression and Release comparison

- `toolbar_control_snapshot_preserves_first_match_and_missing_dropdown` compares the one-scan projection with the retired eight-search projection. A duplicate dropdown and unknown control occur before the last required control, so the test exercises first-match admission; it also checks zero-width fallback, all-match dropdown frame writes through the actual layout function, and missing-dropdown behavior.
- Existing `narrow_toolbar_keeps_search_and_one_kind_filter_trigger_reachable`, `unsupported_current_kind_remains_visible_selected_and_disabled`, and ultra-narrow containment regressions exercise the actual template and dropdown branch in the combined Editor batch.
- The new regression and retired projection were written before the snapshot implementation; the missing `ToolbarControlSnapshot` symbol was observed by a source probe. Dynamic RED/GREEN execution is deferred to the managed batch.
- Ignored Windows Release `editor57_toolbar_control_snapshot_release_benchmark` runs 17 alternating pairs of 256 projections on a 64-prefix small node table and a 4,096-prefix large node table. `PERF_RESULT EDITOR57_ACTIVE_DROPDOWN_TOOLBAR_READ_BENCH_V1` reports old and new p50/p95/p99 for both workloads. The pending p95 gates are optimized ≤ 110% of old on the small table and ≤ 70% on the large table. Both workloads print before either gate asserts.

The benchmark measures only the toolbar control-read projection, not frame writes, full Asset Browser layout, 100,000-asset navigation, or input latency. Those product percentiles remain separate acceptance work; no Release pass is claimed until the managed batch produces output.

## Validation manifest

| Gate | Scope | State |
|---|---|---|
| Source | `toolbar_layout.rs`, `toolbar_layout/control_snapshot.rs`, `toolbar_layout/control_snapshot_tests.rs` | Rustfmt, scoped diff, and UTF-8/whitespace checks passed |
| Cargo | one managed Windows `zircon_editor` package check and focused toolbar/Asset Browser regressions with the shared Editor batch | pending coordinator batch |
| Performance | ignored `editor57_toolbar_control_snapshot_release_benchmark`; p95 ≤ 110% small table and ≤ 70% large table | pending coordinator batch |

This is an implemented optimization candidate pending compilation, behavior regressions, and actual Release measurements.
