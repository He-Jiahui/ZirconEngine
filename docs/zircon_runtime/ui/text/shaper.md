---
related_code:
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/surface/render/text_measure.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime/src/ui/tests/text_shaper.rs
implementation_files:
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/surface/render/text_measure.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
plan_sources:
  - .codex/plans/ZirconEngine 宿主编辑器 UI 基础能力计划.md
  - user: 2026-06-06 continue host editor UI foundation with font/text focus
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateTextShaper.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontMeasure.cpp
tests:
  - zircon_runtime/src/ui/tests/text_shaper.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/text/shaper.rs zircon_runtime/src/ui/tests/text_shaper.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-keyboard-clipboard-extract-0605 --message-format short --color never
  - cargo test -p zircon_runtime --lib text_shaper --locked --jobs 1 --target-dir D:\cargo-targets\zircon-keyboard-clipboard-extract-0605 --message-format short --color never -- --nocapture --test-threads=1
  - latest focused test attempt is blocked before executing text tests by unrelated render-framework test-only E0277 in zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
doc_type: module-detail
---

# Runtime UI Text Shaper

`shaper.rs` is the shared runtime boundary between resolved UI style and text geometry. Widget code and render extraction ask for `UiResolvedTextLayout` or a measured `UiSize`; they do not directly choose a font crate or duplicate text measurement.

This follows the Unreal Slate split where `SlateTextShaper` owns shaping and `FontMeasure` owns cached measurement before draw elements consume shaped text. Zircon keeps that boundary while routing layout and measurement through the shared runtime text service.

## Shared Layout Boundary

`UiTextShaperStack` holds the `UiSharedTextShaper` adapter only. It has no pseudo-backend selection, fallback reason, or Native/SDF layout branch. Native/SDF/Auto remain raster-routing policy in the graphics text path; they cannot alter line breaks, source ranges, advances, or cached layout geometry.

The shared service owns text segmentation, shaping, layout, and measurement. `glyphon` and the SDF atlas consume the resulting geometry for native and SDF submission respectively. A later shaping replacement must implement `UiTextShaper` at this boundary rather than introduce a second layout route keyed by render mode.

## Contract

`layout_text(...)` and `measure_text_size(...)` route through the stack instead of directly reaching the layout engine. `resolve_ui_text_render_mode(...)` remains owned by the graphics text path so Auto render routing cannot drift from font-manifest policy. This gives future glyphon, SDF atlas, or cosmic-text/fontdue-backed implementations one attachment point without changing widgets, layout pass measurement, render style resolution, or the `UiResolvedTextLayout` DTO.

The current output contract remains unchanged:

- layout lines, source ranges, baselines, wrapping, ellipsis, and visual order are produced by the shared text/layout owners
- hit testing still consumes the same resolved layout rather than measuring independently
- render paint payloads still carry `UiTextRenderMode` through `UiTextPaint` and synthetic `UiShapedText`
- the graphics text renderer still resolves Auto batches through font manifests and routes to glyphon-native or SDF batches
- Native/SDF raster routing is outside layout; it does not create a second text shaper or cache key

## Tests

`text_shaper.rs` covers the stack as the public layout/measurement entrypoint and asserts direct parity with `UiSharedTextShaper`. It also verifies Auto plus font-default render-mode resolution. The graphics text system's module-local tests cover font-manifest Auto routing into native or SDF render batches.
