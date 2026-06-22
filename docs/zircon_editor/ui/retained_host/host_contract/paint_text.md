---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/blend.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/clip_rect.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/text_markers.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/blend.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/clip_rect.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract paint-text draw/clip/font/raster/blend/test ownership scan
  - host_contract paint-text draw recording/metrics ownership scan
  - host_contract paint-text draw clip-rect ownership scan
  - host_contract paint-text glyph row pixel ownership scan
  - host_contract paint-text draw entry ownership scan
  - scoped whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Host Contract Paint Text

`paint_text.rs` is the retained-host text paint entry. It keeps the legacy import surface stable for chrome command replay, template render commands, and primitive text markers while moving concrete text drawing responsibilities into focused child modules.

## Draw Ownership

`paint_text/draw.rs` owns the text draw pipeline after entry normalization and the draw-order handoff to the focused clip, layout, recording, and glyph children.

`paint_text/draw/entry.rs` owns the public text draw entry chain, default font metrics, and default text-style selection before forwarding into the pipeline owner.

`paint_text/draw/clip_rect.rs` owns effective text-clip resolution plus frame-to-pixel clip conversion for draw calls.

`paint_text/draw/layout.rs` owns fontdue layout construction for a retained text run, including the positive-Y-down coordinate system, vertical centering within the target frame, maximum width/height bounds, fallback font selection, and text-style append.

`paint_text/draw/recording.rs` owns font-size and line-height clamping plus retained recording handoff. It converts the clipped pixel bounds back into a frame-space recording rectangle before forwarding the existing text/style payload to `HostRgbaFrame::record_text(...)`.

`paint_text/draw/glyphs.rs` owns glyph traversal, cached-raster lookup, per-row clip gating, and row dispatch.

`paint_text/draw/glyphs/row.rs` owns row/column clipping, style-based emphasis offset, strong-text overdraw, coverage-scaled alpha, and the final per-pixel dispatch into the blend owner.

The public boundary remains intentionally narrow: callers keep using `draw_text(...)`, `draw_text_with_size(...)`, or `draw_text_with_size_and_style(...)` through the root module, while the root file no longer owns layout or pixel loops and `draw.rs` no longer owns fontdue setup, retained recording rectangle construction, or glyph pixel loops directly.

## Clip Ownership

`paint_text/clip.rs` owns active paint-clip and explicit text-clip resolution. It intersects the frame clip with the call-site clip when both exist and returns `None` for disjoint clipping so the draw owner can stop before layout or raster work.

## Font And Raster Ownership

`paint_text/font.rs` owns the embedded fallback editor font and lazy `fontdue::Font` construction.

`paint_text/raster.rs` owns glyph raster caching by glyph index and pixel size. It stores metrics with shared bitmap data so repeated retained-host text draws can reuse glyph bitmaps without re-rasterizing the same font-size pair.

## Blend Ownership

`paint_text/blend.rs` owns RGBA write and alpha blending for text pixels. The draw owner only calculates coverage-adjusted color and pixel coordinates; channel overwrite and alpha composition stay in the blend child.

## Root Boundary

The root `paint_text.rs` declares child modules, re-exports the existing text draw entry points, and attaches the external test module. It should not regain font bytes, fontdue imports, glyph cache state, clip math, pixel blending, layout traversal, or inline tests.

## Test Ownership

`paint_text_tests.rs` owns local regressions for glyph raster cache reuse and disjoint active/explicit clip suppression. The tests stay outside production files so the root can remain a structural owner boundary.

## Validation Notes

This slice used `cargo fmt -p zircon_editor --check`, root/draw ownership scans confirming `paint_text.rs` no longer owns draw, clip, font, raster, blend, or inline test bodies and `paint_text/draw.rs` no longer owns fontdue layout setup or glyph pixel loops directly, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo check/test validation remains deferred because current package checks are blocked before editor diagnostics by unrelated runtime compile lanes, and the active instruction is to implement functionality first.

The 2026-06-21 draw recording/metrics split reduced `paint_text/draw.rs` from 105 lines to a 101-line draw entry. `draw/recording.rs` owns text metric clamping and retained recording handoff, while `draw.rs` keeps the public entry chain, clip resolution, layout invocation, record-only early return, and glyph draw dispatch. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint-text draw recording/metrics ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 draw clip-rect split reduced `paint_text/draw.rs` from 101 lines to a 93-line draw entry. `draw/clip_rect.rs` owns active/explicit text clip resolution and `PixelRect` conversion, while `draw.rs` keeps the public entry chain, metric clamping, recording, layout, and glyph dispatch order. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint-text draw clip-rect ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 glyph row pixel split reduced `paint_text/draw/glyphs.rs` from 88 lines to a 52-line glyph traversal entry. `draw/glyphs/row.rs` owns row/column clipping, emphasis offset, strong overdraw, coverage alpha, and blend dispatch, while `glyphs.rs` keeps cached raster lookup and per-row dispatch. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint-text glyph row pixel ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 draw entry split reduced `paint_text/draw.rs` from 85 lines to a 50-line text pipeline owner. `draw/entry.rs` owns the public draw entry chain, default font metrics, and default style selection, while `draw.rs` keeps clip resolution, metric clamping, recording handoff, layout, and glyph dispatch. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a paint-text draw entry ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.
