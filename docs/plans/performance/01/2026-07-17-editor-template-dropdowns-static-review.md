# Editor Template Dropdowns Static Review

- Date: 2026-07-17
- Scope: `paint_template_nodes/{template_dropdowns.rs,template_dropdowns/**,template_dropdowns_tests/**,template_dropdown_glyphs.rs,template_dropdown_glyphs/**,template_dropdown_metrics.rs}`
- Rust files read: 19/19
- Acceptance state: `static_complete_dynamic_pending`
- Plan item: `PERF-MVP-213`

## Files reviewed

| module | files | result |
|---|---:|---|
| dropdown entry/identity/geometry/layers | 5 | bounded identity and O(1) geometry; valid zero-size early return |
| style/surface/text/commands | 4 | style is selected once, but label and metrics are independently recomputed by child layers |
| glyph and metrics | 4 | shipped dropdown asset is preferred; fallback is three quads; chevron size/right/reserve each project the full metrics struct |
| tests | 6 | identity, geometry, state, style, asset, paint, order, offset, and brightness coverage exists; no stable-frame work budget |

## Bottleneck evidence

`commands.rs` resolves the style before emitting three layers. `dropdown_style()` calls `dropdown_label_is_placeholder()`, which constructs a `String` through `template_node_label`. `push_dropdown_label()` constructs the label again and may clone `options.row_data(0)` into another `String`. The final text command owns the label, so a changed control needs one owned label, not two independent projections.

The surface projects `WorkbenchDropdownMetrics` once. Text projects it again and calls `dropdown_chevron_reserve()`, which projects it a third time. The glyph separately calls `dropdown_chevron_size()` and `dropdown_chevron_right()`, producing two more full projections. A normal dropdown therefore projects the same eight derived metrics about five times before emitting one quad, one text command, and one image command.

The option list is not painted here; open popup rows are owned by the already recorded PERF-MVP-205 path. This module's root issue is repeated changed-node derivation and stable-frame command rebuild, not an unbounded dropdown-option loop.

## Reference-engine direction

- Slint's material `combobox.slint` binds one retained `current-value` to one `Text`, one retained icon resource, and theme/layout properties; the popup explicitly caps its visible height to six items.
- Godot `OptionButton` stores the selected item text on selection changes and queues a cached minimum-size refresh; draw reuses the retained button text, arrow theme cache, and icon rather than querying the option model to reconstruct the label every frame.

Zircon should first pass one label and one metrics snapshot through the three layers. The final `DropdownPaintSpec` belongs to the shared changed-generation compilation path, while popup row virtualization remains a separate owner.

The current-source direct fix now resolves `(label, placeholder)` and `WorkbenchDropdownMetrics` exactly once in `commands.rs`, then lends the metrics snapshot through surface, text, and chevron layers. The former glyph metrics forwarding module was deleted, and no child layer retains a `workbench_dropdown_metrics()` call. A focused source guard fixes the one-label/one-metrics budget and a behavioral regression test preserves the first-option fallback. Dynamic acceptance is still pending.

## Dynamic acceptance still required

- Run current-source tests for the landed one-label/one-metrics source guard and first-option fallback regression; add runtime counters for zero stable-generation builds.
- Re-run current-source `zircon_editor --lib performance_tests` and the dropdown pixel suite after the local consolidation.
- Measure 1/100/10,000 dropdowns and 300 stable frames: label bytes, theme/metrics reads, resource resolves, host commands, CPU p50/p95/p99.
- Preserve placeholder/fallback-option behavior, disabled/hover/focus/open state priority, declared style, offsets, brightness, asset fallback, z-order, clip, and pixels.
- Include the module in the current-source Softbuffer/RenderDoc editor capture before moving it to `review.md`.
