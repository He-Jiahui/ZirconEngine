---
related_code:
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/font_registry.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/text/raster/mod.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/tests/text_shaper.rs
  - zircon_runtime/src/ui/tests/text_pipeline.rs
  - zircon_runtime/src/ui/tests/text_layout.rs
  - zircon_runtime/src/ui/tests/text_hit_testing.rs
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
implementation_files:
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/font_registry.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/text/raster/mod.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/tests/text_shaper.rs
  - zircon_runtime/src/ui/tests/text_pipeline.rs
plan_sources:
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/engine-architecture/runtime-tech-stack.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - user: 2026-06-12 implement editor UI architecture from docs/plans/zircon_editor/editor_ui
  - dev/bevy/crates/bevy_text/src/lib.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
tests:
  - rustfmt --edition 2021 --config skip_children=true --check zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/shaper.rs zircon_runtime/src/ui/tests/text_shaper.rs zircon_runtime/src/ui/tests/mod.rs (2026-05-23: passed after UiTextShaper boundary addition)
  - cargo test -p zircon_runtime --lib text_shaper --offline --jobs 1 --target-dir D:\cargo-targets\zircon-text-shaper-20260523 --message-format short --color never (2026-05-23: deferred while unrelated Cargo/rustc processes were active)
  - rustfmt --edition 2021 zircon_runtime/src/ui/text/font_registry.rs zircon_runtime/src/ui/text/resolved_layout.rs zircon_runtime/src/ui/text/measure_cache.rs zircon_runtime/src/ui/text/raster/mod.rs zircon_runtime/src/ui/text/mod.rs zircon_runtime/src/ui/tests/text_pipeline.rs zircon_runtime/src/ui/tests/mod.rs (2026-06-12: passed)
  - cargo test -p zircon_runtime --lib style_mapping --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-editor-ui-runtime-coremin --message-format short --color never -- --nocapture --test-threads=1 (2026-06-12: timed out after 604 seconds while compiling runtime test binary; matching target processes stopped, no Rust diagnostics emitted)
  - target/codex-editor-ui-runtime/debug/deps/zircon_runtime-de6f737e1b69a0f9.exe text_pipeline --nocapture --test-threads=1 (2026-06-12: passed, 5 passed)
  - cargo test -p zircon_runtime --lib runtime_input_manager --locked --jobs 1 --target-dir target/codex-editor-ui-runtime --message-format short --color never (2026-06-12: rebuild blocked by unrelated unresolved import crate::core::frame_clock in zircon_runtime/src/core/runtime/state/runtime_inner.rs)
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs::runtime_text_doc_records_three_layer_stack_and_cross_reference
doc_type: module-detail
---

# Runtime UI Text

## Purpose

Runtime UI text owns the shared layout, edit, hit-test, and shaping boundary used by render extraction, TextInput behavior, accessibility text geometry, and future SDF/native text backends.

The current backend is intentionally heuristic. It preserves Zircon's existing behavior for grapheme-aware wrapping, rich-run ranges, low-fidelity bidirectional visual order, ellipsis, measurement, caret/selection geometry, and hit testing. It is now accessed through `UiTextShaper` instead of making the public `layout_text(...)` entry point call `layout_engine` directly.

## Shaper Boundary

`UiTextShapeRequest` carries the source text, resolved style, layout frame, and optional clip frame. `UiTextShaper` provides two operations:

- `shape_text(...)` returns `UiResolvedTextLayout`, the shared geometry DTO consumed by render extract and hit testing.
- `measure_text(...)` returns `UiSize`, the same measurement used by layout callbacks.

`UiHeuristicTextShaper` is the current default implementation. It delegates to `layout_engine.rs`, so this slice changes ownership boundaries without changing text geometry. Future Parley/Swash/HarfBuzz integration should implement the same trait and preserve the `UiResolvedTextLayout` contract before diverging into native or SDF raster/cache backends.

## Backend Responsibility Matrix

This matrix mirrors the runtime dependency decision in [Runtime Tech Stack](../../engine-architecture/runtime-tech-stack.md#text-stack-boundary). It is the runtime text ownership boundary until a later milestone replaces the active layout backend through `UiTextShaper`.

| Layer | Input | Output | Owner modules | Current status |
|---|---|---|---|---|
| Shaping, segmentation, layout, and measurement | Source text, resolved style, layout frame, optional clip frame, grapheme segmentation | `UiResolvedTextLayout`, line boxes, glyph boxes, caret/selection geometry, `UiSize` measurement | `zircon_runtime/src/ui/text/{shaper.rs,grapheme.rs,layout_engine.rs}` with public access through `layout_text(...)`, `measure_text_size(...)`, and `UiTextShaper` | Active backend is `UiHeuristicTextShaper` plus `unicode-segmentation 1.13.2`; `heuristic_text_shaper_matches_public_layout_entrypoint` locks public-layout parity. |
| Font registry, raster, and SDF policy | Runtime `FontAsset` records, inferred or explicit families, render-mode preference, fallback chain, resolved text layout | Neutral font records, raster policy decisions, future bitmap/SDF cache inputs | `zircon_runtime/src/ui/text/{font_registry.rs,raster/mod.rs,measure_cache.rs,resolved_layout.rs}` | Runtime owns the policy boundary. `fontsdf 0.5.3` stays a runtime text/raster dependency; `fontdue 0.9.3` remains editor-only retained-host fallback debt. |
| GPU/native text submission | Extracted UI text primitives and resolved layout output from the text subsystem | Render-side text draw submission and native glyph resources | Runtime graphics/UI render paths including `rhi_wgpu/ui_surface/text.rs`, `rhi_wgpu/ui_surface/geometry.rs`, `graphics/scene/scene_renderer/ui/text.rs`, `ui/surface/render/resolve.rs`, plus `ui/text/shaper.rs` and `ui/tests/text_shaper.rs` for intent/fallback coverage | `glyphon 0.11.0` is 渲染侧已用, but layout 后端未接. `active_layout_backend_for_intent` keeps `NativeGlyphon` and `SdfAtlas` explicit while both fallback to heuristic layout until a real backend lands. |

## Current Guarantees

`layout_text(...)` now routes through `UiHeuristicTextShaper`, while `measure_text_size(...)` routes through the same backend. This keeps layout measurement and render extraction on one source of text geometry.

The existing `layout_engine` still handles grapheme cluster wrapping, rich text run splitting, ellipsis range preservation, and the low-fidelity BiDi scaffold. `text_hit_testing.rs` consumes the resolved layout rather than re-estimating line geometry.

`cosmic-text`, Parley, Swash, or HarfBuzz remain governed by the runtime tech-stack decision: they may only enter as replacement `UiTextShaper` implementations, not as duplicate public text layout entry points.

## Validation

`text_shaper.rs` focused tests assert that the default shaper returns the same layout as the public `layout_text(...)` entry point and the same measurement as `measure_text_size(...)`, including a combining-mark ellipsis case. `heuristic_text_shaper_matches_public_layout_entrypoint` locks public layout parity, while `text_shaper_stack_uses_current_heuristic_backend_until_font_backends_land` locks the current fallback from NativeGlyphon/SdfAtlas intents back to the heuristic backend. Runtime Cargo execution is still deferred when unrelated Cargo/Rust compiler processes are active in the shared checkout.

## Font Registry

`UiFontRegistry` is the runtime owner for font family registration before the real shaping backend lands. It records `FontAsset` sources, inferred or explicit family names, render-mode preference, and a fallback chain that includes Latin and CJK-friendly defaults. The registry deliberately stores neutral records instead of exposing a shaping library type. A later cosmic-text integration can populate its `FontSystem` from the same records without changing the call sites that register editor/runtime font assets.

## Layout Request And Cache

`UiTextLayoutRequest` wraps the existing `layout_text(...)` path with the extra information the editor UI plan needs: style key extraction, optional clip frame, source hashing, and an optional preedit span. `UiPreeditSpan` replaces a byte range in the temporary layout input only; it does not mutate the source text. This preserves the IME invariant that preedit text is visual state until commit.

`UiTextMeasureCache` is the Stage A cache boundary. Its key contains the source/preedit hash, a style key, and a width bucket. For the current heuristic layout engine, the width bucket is the wrapped character capacity, matching the existing `text_advance(font_size) = font_size * 0.5` rule. The cache records per-frame shape count so later frame reports can assert that the same text/style/wrap bucket is shaped once and reused by measure, arrange, and render extraction.

## Raster Path Policy

`UiGlyphRasterPolicy` captures the SDF-versus-bitmap routing decision without introducing a new rasterizer dependency. Static small UI text defaults to bitmap, large text defaults to SDF, and explicitly scalable text prefers SDF. This keeps the future fontsdf/bitmap atlas choice local to the text subsystem while existing `UiResolvedTextLayout` and render extraction remain unchanged.
