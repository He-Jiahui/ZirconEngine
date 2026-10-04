# Editor Template Sample Grid Static Review

- Date: 2026-07-17
- Scope: `paint_template_nodes/{template_sample_grid.rs,template_sample_grid/**,template_sample_grid_tests/**}`
- Rust files read: 11/11
- Lines read: 641
- Acceptance state: `static_complete_dynamic_pending`
- Plan item: `PERF-MVP-214`
- Fixing plan: `docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`

## Files reviewed

| module | files | result |
|---|---:|---|
| entry/identity/geometry | 3 | bounded identity and value normalization; complete static/dynamic output is rebuilt together |
| surface | 1 | dashed grid expands by panel pixel dimensions and tick count |
| points | 1 | every diamond expands into per-scanline quads; selected marker can emit about 25 marker quads before its label |
| text | 1 | every tick is formatted into a new String and axis/selected labels are copied on every paint |
| constants/palette | 2 | O(1), but hard-coded theme prevents shared generation ownership |
| tests | 3 | basic pixels, selected marker/text, and frame scaling exist; no command, allocation, tick, point, or stable-generation budgets |

## Bottleneck evidence

`surface.rs` walks every x/y tick and calls `push_dashed_vertical`/`push_dashed_horizontal`. Each helper advances by `GRID_DASH_LENGTH + GRID_DASH_GAP` and emits one `HostPaintCommand::quad` per dash. Command count therefore grows with both tick count and panel pixel dimensions. The default 360x260 fixture with five ticks per axis creates hundreds of host commands for a static grid before text or points.

`points.rs` draws a diamond as one horizontal quad per scanline. A normal radius-four marker emits nine body lines plus three center lines; a selected radius-six marker adds thirteen halo lines, reaching about 25 marker commands. Every point is cloned through `row_data`, and selected labels count chars and copy the String again. `text.rs` separately revisits both tick models and formats every value every paint.

The static surface, ticks, labels, grid geometry, and normal points are mixed with selected/drag state. Stable or point-only changes thus rebuild the complete command stream. The issue is command representation and generation ownership, not a need for a faster inner while-loop.

## Reference-engine direction

Godot's `animation_blend_space_2d_editor.cpp` invalidates its canvas when blend-space or interaction state changes and submits whole lines/circles/text through drawing primitives. Fyrox's blend-space field uses retained point widgets whose draw submits a filled circle to a drawing context, with point moves sent as widget messages. Both avoid representing each dash or marker scanline as an independent high-level paint command.

Editor07 should publish one immutable grid generation with typed ticks, preformatted labels, and points. EditorUI08 separates static and dynamic segments. Render13/host should batch dashed lines and markers as geometry or instanced primitives.

## Dynamic acceptance still required

- Re-run current-source `zircon_editor --lib performance_tests` and sample-grid pixel tests.
- Add deterministic counters for tick visits, label formats/copies, static/dynamic builds, host/compiled/RHI commands, vertices, batches, and allocation bytes.
- Test 1/10/100 ticks, 1/100/10,000 points, 300 stable frames, and 1,000 selected-point drags; report CPU p50/p95/p99.
- Prove static grid/axis/text build and formatting are zero on stable frames and point drags patch only the necessary dynamic segment.
- Preserve invalid range normalization, selected marker/label clamp, zero axes, z-order, clipping, and Softbuffer/RenderDoc pixels before moving the folder to `review.md`.
