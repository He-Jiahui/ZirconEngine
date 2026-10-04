---
related_code:
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/text/font/decoration_metrics.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_decorations.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/decorations.rs
implementation_files:
  - zircon_runtime/src/text/font/decoration_metrics.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_decorations.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/decorations.rs
plan_sources:
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/superpowers/specs/2026-07-13-runtime-text-sdf-effects-decoration-design.md
  - docs/superpowers/plans/2026-07-13-runtime-text-sdf-effects-decoration.md
tests:
  - zircon_runtime/src/text/font/decoration_metrics/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/text_style_decorations.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/decoration_geometry.rs
doc_type: module-detail
---

# Runtime font decoration metrics

## Ownership

`text/font/decoration_metrics.rs` is the single owner of display-pixel underline and strikeout metrics. It converts authoritative face units from the font asset metadata or directly from `ttf-parser`; render planning does not invent a second fixed-pixel policy.

The public `UiTextDecorations` contract remains a presentation style. `render/text_decorations.rs` resolves authored colors and the resolved line baseline, while `sdf_render/decorations.rs` converts those values into clipped solid quads. Selection, caret, composition underline and table decorations continue to use `UiTextPaintDecorationKind` and are not mixed into this path.

## Metric contract

For a valid face, position and thickness are scaled by `display_px / units_per_em`:

- underline reads the `post` underline position/thickness exposed by `ttf-parser`;
- strikeout reads the OS/2 strikeout position/thickness;
- ascender is scaled from the same face and is used only when no resolved line baseline is available.

Missing tables use the Text05 fallback contract: underline position `-0.1em`, line thickness `0.05em`, strikeout position `0.3em`, and ascender `0.8em`. A face table that is present is not silently replaced by those values. Raster geometry enforces a one-device-pixel visible minimum after retaining the unrounded face ratio for tests and cache values.

`TextDecorationMetricsCache` keys entries by `(FontFaceId, normalized display-size bits)`. Run aggregation keeps the primary face position and takes the maximum thickness across actual fallback faces, so mixed-script runs remain visible without moving the primary baseline relationship.

## Geometry and draw order

Horizontal text converts font y-up positions with `center_y = baseline - position_px`; negative underline positions therefore appear below the baseline. `VerticalRl` applies the same signed rule on the cross axis and emits a vertical rectangle. V1 intentionally does not implement skip-ink.

Rich HTML/BBCode underline, strike and links are projected into the same batch decoration contract as plain style. The former `run.bottom() - 1px` rich-text quad is removed. Both Native and distance-field batches are resolved by the text system after font registration; solid decoration vertices use the SDF renderer's explicit solid primitive branch and never sample an atlas texel.

## Validation

On 2026-07-13 the managed graphics production check passed under job `1e33672fe7ab4bbcb73389cb223752c8`. The current-source `text_decoration` test filter then passed 7/7 under job `d1728fe48ced4ef19c5b54e1a95f095d`, covering real FiraSans face tables, fallback and scaling, horizontal/VerticalRl frames, rich/plain projection, resolved baseline, distinct colors, solid vertices and WGSL parsing.

The broader target-client check was separately blocked by concurrent plugin bridge re-export migration after the text code compiled; that external failure is not counted as a Text05 green gate.
