# Editor Template Material Feedback Static Review

- Date: 2026-07-17
- Scope: `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/{template_material_feedback.rs,template_material_feedback/**,template_material_feedback_tests/**}`
- Rust files read: 21/21
- Acceptance state: `static_complete_dynamic_pending`
- Plan item: `PERF-MVP-212`

## Files reviewed

| module | files | result |
|---|---:|---|
| dispatcher | 1 | backdrop/progress identity dispatch is bounded, but specialized paint still owns dynamic raster work |
| backdrop | 1 | closed/invisible paths return early; visible path repeats variant probes and palette resolution |
| circular progress | 5 | every paint allocates/formats/rasterizes; image identity is incomplete |
| linear progress | 3 | command count is bounded at 1-3, but state/palette/metrics are resolved through separate calls |
| state, palette, metrics | 8 | small helpers are individually bounded but reacquire global theme snapshots and rescan string roles/variants |
| tests | 3 | pixel/palette/metric/backdrop coverage exists; no circular cache identity, stable-frame allocation, scale, or indeterminate-update budget test |

## Bottleneck evidence

`circular_progress/entry.rs` creates a fresh RGBA vector for every visible circular progress on every paint. `pixels.rs` executes nested `size^2` loops with `sqrt`, `atan2`, and `rem_euclid`, even when size, progress, colors, and theme generation are unchanged. The command then owns that vector and formats another `String` key. This is main-thread O(nodes * pixels) CPU and allocation work in a stable retained frame.

`circular_progress/key.rs` only includes `track[0]` and `fill[0]`, so distinct colors with the same red channel collide. The pixel path uses the resolved indeterminate percent, while the key is built from `progress_percent(node)`; indeterminate nodes can therefore publish pixels and identity from different state. A cache consumer cannot safely reuse this key.

Linear progress emits a bounded number of quads, but it resolves corner radius, metrics, track color, fill color, indeterminate mode, and percent through independent helpers. Backdrop similarly probes component variants more than once and resolves the palette only after style lookup. These are secondary and should converge into one changed-node `MaterialFeedbackPaintSpec` rather than receive a local cache in every helper.

## Reference-engine direction

- Slint's material circular progress uses retained `Rectangle` and `Path`/`ArcTo` geometry in `dev/slint/ui-libraries/material/src/ui/components/progress_indicator.slint`; indeterminate motion updates path parameters rather than CPU-rasterizing a new image.
- Material UI's `CircularProgress.js` keeps one SVG circle and varies transform/stroke dash parameters; determinate progress is geometry/state, not a per-frame bitmap rebuild.
- Godot's `TextureProgressBar` submits texture regions or radial polygon geometry from retained textures in `dev/godot/scene/gui/texture_progress_bar.cpp`; it does not rebuild an RGBA texture by evaluating every destination pixel during control paint.

The Zircon target is therefore typed ring/arc geometry owned by Render13/host rendering. A bounded raster cache is only a transitional fallback and must use complete identity plus theme generation.

## Current-source direct mitigation

The raster fallback now normalizes the actual resolved progress once and uses that same value for pixels and resource identity. The key contains exact progress bits and all track/fill RGBA channels. A four-entry thread-local size LRU retains only ring topology `(pixel offset, normalized turn)`: squared-distance membership plus `atan2`/`rem_euclid` run when a size first enters the cache, while stable sizes allocate the required owned output buffer and recolor only ring pixels. Tests lock complete key identity, resolved-percent use, same-size topology reuse, and invalid-progress normalization. This is intentionally transitional: owned RGBA allocation/copy, key formatting, and indeterminate full-image changes remain until typed arc geometry lands.

Local module-wrapper validation with Rust 1.94.1 passed all four focused tests. This proves the standard-library topology/key modules compile and behave as intended in isolation; it does not replace the pending `zircon_editor` crate build, paint tests, or product pixels.

## Dynamic acceptance still required

- Re-run current-source `zircon_editor --lib performance_tests` after the two source-guard corrections.
- Run current-source tests for complete key identity, resolved-percent use, topology reuse, and invalid-progress normalization; add counters for topology build/transcendental operations and preserve 0/NaN/disabled/workbench/style override pixels.
- Record 1/10/100 circular controls over 300 stable frames: raster calls, transcendental pixel operations, allocation bytes, key formats, commands, uploads, CPU p50/p95/p99.
- Record indeterminate animation separately: only dynamic angle/dash parameters may change; the path must not upload a full new RGBA image every frame.
- Run Softbuffer pixel parity and the current-source RenderDoc/editor capture before moving this folder to `review.md`.
