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

This follows the Unreal Slate split where `SlateTextShaper` owns shaping and `FontMeasure` owns cached measurement before draw elements consume shaped text. Zircon currently keeps the same ownership boundary, but the active implementation is still the fixed-advance heuristic in `layout_engine.rs`.

## Backend Selection

`UiTextShaperStack` records the backend requested by `UiResolvedStyle.text_render_mode` before it delegates to an active backend. `Auto` is resolved through the same policy used by the runtime graphics text system: a font manifest may force SDF or native rendering, and a missing or still-auto font default resolves to native/glyphon intent.

| Requested mode | Font default | Effective mode | Intended backend | Active layout backend |
| --- | --- | --- | --- | --- |
| `Auto` | none / `Auto` / `Native` | `Native` | `NativeGlyphon` | `Heuristic` fallback |
| `Auto` | `Sdf` | `Sdf` | `SdfAtlas` | `Heuristic` fallback |
| `Native` | any | `Native` | `NativeGlyphon` | `Heuristic` fallback |
| `Sdf` | any | `Sdf` | `SdfAtlas` | `Heuristic` fallback |

The fallback is intentional. The graphics renderer already has a glyphon native path and an SDF atlas path fed by font manifests, but layout and measurement still use the fixed-advance scaffold. `glyphon` and `fontsdf` are available dependencies in the runtime, `fontdue` currently lives in the editor crate, and `cosmic-text` is not yet part of the runtime dependency graph. Until a real native/SDF layout shaper is connected, Native and SDF requests must stay observable as intent while producing the same layout geometry as the current heuristic backend.

## Contract

The shaper stack owns backend selection. `layout_text(...)` and `measure_text_size(...)` route through the stack instead of directly calling the heuristic helper. `resolve_text_render_mode(...)` is also consumed by `graphics::scene::scene_renderer::ui::text` so Auto resolution does not drift between layout-time metadata and render-time batch routing. This gives future glyphon, SDF atlas, or cosmic-text/fontdue-backed implementations one attachment point without changing widgets, layout pass measurement, render style resolution, or the `UiResolvedTextLayout` DTO.

The current output contract remains unchanged:

- layout lines, source ranges, baselines, wrapping, ellipsis, and visual-order scaffolds are still produced by `layout_engine.rs`
- hit testing still consumes the same resolved layout rather than measuring independently
- render paint payloads still carry `UiTextRenderMode` through `UiTextPaint` and synthetic `UiShapedText`
- the graphics text renderer still resolves Auto batches through font manifests and routes to glyphon-native or SDF batches
- Native/SDF layout backend selection is recorded, but not yet a claim of real layout-time glyph shaping, font fallback, ligature support, or final cluster metrics

## Tests

`text_shaper.rs` covers the stack as the public layout/measurement entrypoint, verifies Native and SDF backend intent, verifies Auto plus font-default resolution, and asserts that those modes still fall back to the heuristic layout backend until the real layout shapers land. The graphics text system's module-local tests continue to cover font-manifest Auto routing into native or SDF render batches.
