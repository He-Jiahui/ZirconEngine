# Editor Template Timeline Strip Static Review

- Date: 2026-07-17
- Scope: `paint_template_nodes/{template_timeline_strip.rs,template_timeline_strip/**,template_timeline_strip_tests/**}`
- Rust files read: 11/11
- Lines read: 627
- Acceptance state: `static_complete_dynamic_pending`
- Plan item: `PERF-MVP-215`
- Fixing plan: `docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`

## Files reviewed

| module | files | result |
|---|---:|---|
| entry/identity/geometry | 3 | theme snapshots are acquired once, but ticks are not shared and static/dynamic output rebuilds together |
| surface | 1 | `timeline_ticks` is allocation-producing, unbounded, cumulative-float iteration and can fail to advance |
| text | 1 | generates the complete tick Vec a second time and formats every tick plus footer/track text every paint |
| keys/playhead | 1 | scans all keys and expands every diamond into scanline quads; playhead-only change rebuilds static keys |
| metrics/palette | 2 | one projection each per entry is the correct local pattern |
| tests | 3 | palette and coarse pixel/scale coverage exists; no malformed interval, count, allocation, command, or stable-generation budgets |

## Bottleneck evidence

`surface::timeline_ticks` starts at zero and repeatedly executes `time += interval` until duration. There is no count or pixel-density bound. A tiny positive interval can allocate an enormous Vec. At large time magnitudes or subnormal intervals, floating-point addition can leave `time` unchanged, making the loop non-terminating. This is a direct main-thread hang and memory-growth defect.

The surface and text paths each call `timeline_ticks`, so normal paints allocate and compute the same list twice. Text formats every tick into a new String. A changing playhead also causes surface ticks, tick labels, track label, footer, and all key scanline commands to rebuild. Key and playhead diamonds share PERF-MVP-214's command-amplification root.

## Direct fix and architecture follow-up

The direct fix computes ticks once by integer index, clamps to the number of distinguishable plot pixel columns and a shared hard limit, always terminates, and passes one slice to surface and text. Tests cover tiny/subnormal/invalid intervals, endpoint semantics, and the bound.

Editor07 then publishes an immutable, preformatted timeline generation. EditorUI08 compiles static surface/ticks/labels/keys separately from the dynamic playhead/selection segment. Godot's animation timeline redraws on state invalidation and expresses markers/lines as drawing primitives; Zircon should not rebuild string and marker fragments solely because current time changed.

## Dynamic acceptance still required

- Land the direct bounded single-generation tick fix and focused unit tests, then re-run current-source `zircon_editor --lib performance_tests` plus timeline tests.
- Test invalid, NaN, infinite, subnormal, tiny, normal, and duration-larger-than-interval inputs; no case may exceed pixel/hard budget or hang.
- Record 1/100/10,000 keys, 300 stable frames, and 1,000 playhead scrubs: tick generation, formats, key visits, static/dynamic builds, allocations, commands, CPU p50/p95/p99.
- Preserve endpoint, footer percent, progress, track, selected key, z-order, clip, and Softbuffer/RenderDoc pixels before moving the folder to `review.md`.
