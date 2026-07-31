# Editor Template Weight Heatmap Static Review

- Date: 2026-07-17
- Scope: `paint_template_nodes/{template_weight_heatmap.rs,template_weight_heatmap/**,template_weight_heatmap_tests/**}`
- Rust files read: 10/10
- Lines read: 401
- Acceptance state: `static_complete_dynamic_pending`
- Plan item: `PERF-MVP-216`
- Fixing plan: `docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`

## Files reviewed

| module | files | result |
|---|---:|---|
| entry/identity/geometry | 3 | O(1), but field and markers independently access the source model and all work rebuilds together |
| field | 1 | unbounded rows/columns; every cell clones every source and runs one exponential influence calculation |
| markers | 1 | traverses sources again and expands markers into scanline quads |
| palette/text | 2 | bounded interpolation, but labels are copied every paint and palette is hard-coded |
| tests | 3 | coarse multicolor/scale pixels exist; no dimension/source/compute/command/stable budget or malformed-input coverage |

## Bottleneck evidence

`field.rs` converted authored `columns` and `rows` directly to `usize` after only applying a minimum of one. It emitted one quad for every cell. `heat_intensity` then looped over `sources` for each cell, calling `row_data` and evaluating `exp`. Work was therefore O(columns * rows * sources), source DTO clones had the same multiplier, and command memory was O(columns * rows). Authored `i32::MAX` dimensions could not be admitted safely.

After the field, `markers.rs` traverses and clones all sources again and emits seven or eleven scanline quads per marker. Stable frames recompute the entire heat field, legend, markers, and label Strings even when no source or frame generation changed.

## Direct fix and architecture follow-up

The current-source direct fix projects source DTOs once and lends one slice to field and marker builders. Grid dimensions are reduced by plot pixel dimensions, a 4,096-cell hard cap, and a 65,536 influence-evaluation budget while retaining every source in the intensity calculation. Legend commands are bounded by pixels and 64 steps. Each source marker is now one rounded quad instead of seven or eleven scanline quads. Unit guards cover extreme and non-positive dimensions, small-axis preservation, source-adaptive budgets, legend bounds, source preservation, and one-command markers. This bounds command construction and removes cell-times-source model cloning without changing the max-influence formula; dynamic acceptance is still pending.

Editor07 should publish an immutable heat generation and schedule high-source CPU evaluation off the UI thread. Render13 should prefer a retained texture or compute path plus a bounded marker batch; EditorUI08 submits the generation handle and dynamic selection segment.

## Dynamic acceptance still required

- Run current-source tests for the landed single-source-projection, adaptive bounded-grid, bounded-legend, and single-command marker direct fixes.
- Re-run current-source `zircon_editor --lib performance_tests` and heatmap pixel tests.
- Measure 1/16/256 authored dimensions crossed with 1/100/10,000 sources: source row_data, `exp` evaluations, CPU p50/p95/p99, allocations, commands, uploads, and frame memory.
- Prove stable generation compute/upload/build=0 and source move/selection invalidates only the intended generation/marker segment.
- Preserve max influence, gradient/legend, marker selection/position, clipping, and Softbuffer/RenderDoc pixels before moving the folder to `review.md`.
