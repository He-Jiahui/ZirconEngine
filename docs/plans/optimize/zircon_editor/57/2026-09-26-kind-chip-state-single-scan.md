---
title: Editor57 Kind Chip State Single Scan
category: zircon_editor
date: 2026-09-26
implementation_status: implemented
validation_status: batched_validation_pending
---

# Editor57 Kind Chip State Single Scan

## Scope and target

The Asset Browser toolbar's fallback kind-chip branch computes selected state and width for six chips during responsive layout. Its previous fixed-array projection still made one full `ViewTemplateNodeData` search per chip. With `N` preceding content nodes, that repeats the prefix scan six times on every layout pass. The target is one forward node scan, while preserving the first matching node for duplicate control IDs, fallback width for missing or non-positive-width chips, selected-chip priority, and final visible-chip order. The current `zircon_editor/assets/ui/editor/asset_browser.zui` declares `AssetBrowserKindFilterDropdown`, so `toolbar_layout.rs` selects the dropdown branch and does not invoke this chip projector in the current template. This is fallback-path evidence, not a current MVP product-path gate.

This is a bounded Editor57 asset-workspace layout improvement. It does not close the parent plan's navigation, activation, source, or 100,000-asset product latency gates.

## Implementation and regression

- `kind_chip_states` initializes the six fallback states, visits each node at most once, skips unrelated IDs, and records only the first matching chip. It stops once all six first matches are known.
- The existing visibility regression still compares the complete selection against the retired vector algorithm across selected/missing states and five width limits. The new `kind_chip_state_scan_preserves_first_match_and_missing_fallback` places a duplicate and an unknown chip before the last first-match chip, checks their behavior, and uses an inspected iterator to prove the scan stops before a trailing node. It also checks zero width, missing chips, and equality with the immediately prior six-search projector.
- The test and Release benchmark were written before the production helper. A source probe was RED while `kind_chip_states` was absent; dynamic Rust RED/GREEN execution is deferred to the managed Editor batch.
- Tests and the baseline live in `zircon_editor/src/ui/layouts/views/asset_browser/toolbar_layout/kind_chip_single_scan_tests.rs`, keeping the toolbar source below the module size warning threshold.

## Performance gate

`editor57_kind_chip_single_scan_release_benchmark` creates 4,096 unrelated nodes before the six kind chips, runs 256 projections per sample for 21 alternating pairs, and checks full state equality. `PERF_RESULT EDITOR57_KIND_CHIP_SINGLE_SCAN_BENCH_V1` reports old six-search and new one-scan p50/p95/p99 nanoseconds. The pending Windows Release fallback-helper gate is optimized p95 at most 70% of the six-search p95. It does not measure the active dropdown branch, full Asset Browser layout, or input latency, and must not be counted toward MVP product-path performance acceptance.

The older `EDITOR57_LINEAR_KIND_CHIP_SELECTION_BENCH_V1` benchmark remains an independent whole-selection comparison against the earlier vector algorithm. Its current output now reports 32 searches versus one scan; its original plan table is a historical checkpoint.

## Validation manifest

| Gate | Scope | State |
|---|---|---|
| Source | toolbar source and new test module | Rustfmt, scoped diff check, and source structure check passed |
| Cargo | one managed Windows `zircon_editor` check and focused `kind_chip`/Asset Browser regressions, coalesced with other Editor changes | pending coordinator batch |
| Performance | ignored `editor57_kind_chip_single_scan_release_benchmark`, Windows Release, fallback-only p95 ratio ≤ 0.70 and p50/p95/p99 output | pending coordinator batch; not an MVP product-path gate |

The implementation remains pending acceptance until the current source compiles, the regressions pass, and the actual Release marker meets its gate.
