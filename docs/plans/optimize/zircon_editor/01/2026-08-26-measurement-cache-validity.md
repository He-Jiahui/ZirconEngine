# Incremental Measurement Cache Validity

## Outcome

The incremental measurement planner no longer uses `UiFrame::default()` as an
"unmeasured" sentinel. A legitimate 0x0 frame can now remain reusable, while
new, collapsed, structurally changed, directly dirtied, responsive, and pooled
nodes explicitly invalidate their measurement cache.

## Implementation

- `UiLayoutCache.measure_valid` is a serde-compatible validity bit. Successful
  measurement calls `complete_measure`; collapsed measurement calls
  `invalidate_measure`.
- The post-order planner expands required descendants only when the cache is
  invalid or the node carries a structural/input dirty flag. A parent reached
  only through child invalidation remains sparse; a direct layout source still
  invalidates its own measure.
- `UiTreeNodes::mark_layout_dirty_source` invalidates the source cache. Parent
  insertion and detach, surface layout/style/text/visible-range invalidation,
  runtime layout invalidation, and responsive layout/visibility updates all use
  that authority.
- Reused and reinserted pooled nodes invalidate retained measurement state so a
  previous template instance cannot satisfy a new node's inputs.

## Complexity Contract

For a required path through a valid 0x0 parent, measurement work is `O(K)` for
the required nodes rather than `O(N)` for all descendants. Direct parent
layout/structure changes remain conservative and expand the necessary subtree;
this preserves correctness instead of treating sparse reuse as universal.

## Evidence

- Focused source contract: `python -m unittest tools.tests.test_runtime_incremental_layout_snapshot_performance_contract`
  -> **19/19 passed**.
- Runtime performance-contract discovery (`test_runtime_*performance_contract.py`)
  -> **155/155 passed**; Editor discovery (`test_editor_*performance_contract.py`)
  -> **392/392 passed**; full `test_*performance_contract.py` discovery
  -> **634/634 passed**.
- `rustfmt --edition 2021 --check --config skip_children=true` passed for the
  touched cache, tree, measurement, responsive, rebuild, and node-pool paths.
- `git diff --check` passed for the touched paths.
- Source regressions cover valid zero-frame sparse measurement, cache validity
  independent from geometry, parent invalidation on child structure changes,
  collapsed invalidation, and pooled-node invalidation.

Cargo/managed Rust and product profiling were intentionally not run in this
slice. CPU, allocator, RSS, and p50/p95/p99 measurements remain unclaimed until
the current-source validation lane is available.
