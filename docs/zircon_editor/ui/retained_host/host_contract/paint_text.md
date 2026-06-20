---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/blend.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw.rs
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
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract paint-text draw/clip/font/raster/blend/test ownership scan
  - scoped whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Host Contract Paint Text

`paint_text.rs` is the retained-host text paint entry. It keeps the legacy import surface stable for chrome command replay, template render commands, and primitive text markers while moving concrete text drawing responsibilities into focused child modules.

## Draw Ownership

`paint_text/draw.rs` owns the software text draw entry points, font-size and line-height clamping, retained recording handoff, fontdue layout setup, glyph traversal, style-based emphasis offset, strong-text overdraw, and the final per-pixel dispatch into the blend owner.

The public boundary remains intentionally narrow: callers keep using `draw_text(...)`, `draw_text_with_size(...)`, or `draw_text_with_size_and_style(...)` through the root module, while the root file no longer owns layout or pixel loops.

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

This slice used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming `paint_text.rs` no longer owns draw, clip, font, raster, blend, or inline test bodies, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo check/test validation remains deferred because current package checks are blocked before editor diagnostics by unrelated `zircon_runtime` render-history errors, and the active instruction is to implement functionality first.
