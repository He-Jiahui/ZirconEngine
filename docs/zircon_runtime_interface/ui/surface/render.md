---
related_code:
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/layout/metrics.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/list.rs
  - zircon_runtime_interface/src/ui/surface/render/paint.rs
  - zircon_runtime_interface/src/ui/surface/render/brush.rs
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime_interface/src/ui/surface/render/cache.rs
  - zircon_runtime_interface/src/ui/surface/render/debug.rs
  - zircon_runtime_interface/src/ui/surface/render/parity.rs
  - zircon_runtime_interface/src/ui/surface/render/visualizer.rs
  - zircon_runtime_interface/src/ui/surface/render/editable_text.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/visual_asset_ref.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/tests/ui_geometry_metrics.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/text.rs
implementation_files:
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/layout/metrics.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/list.rs
  - zircon_runtime_interface/src/ui/surface/render/paint.rs
  - zircon_runtime_interface/src/ui/surface/render/brush.rs
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime_interface/src/ui/surface/render/cache.rs
  - zircon_runtime_interface/src/ui/surface/render/debug.rs
  - zircon_runtime_interface/src/ui/surface/render/parity.rs
  - zircon_runtime_interface/src/ui/surface/render/visualizer.rs
  - zircon_runtime_interface/src/ui/surface/render/editable_text.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/tests/ui_geometry_metrics.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/text.rs
plan_sources:
  - docs/assets-and-rendering/runtime-ui-slate-rendering-gap-audit.md
  - user: 2026-05-06 restore preedit text when composition is canceled
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
tests:
  - cargo check -p zircon_runtime_interface
  - cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --message-format short --color never
  - rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/surface/render/debug.rs" "zircon_runtime_interface/src/ui/surface/render/parity.rs" "zircon_runtime_interface/src/ui/surface/render/visualizer.rs" "zircon_runtime_interface/src/ui/surface/render/mod.rs" "zircon_runtime_interface/src/ui/surface/mod.rs" "zircon_runtime_interface/src/tests/render_contracts.rs"
  - CARGO_TARGET_DIR=E:\zircon-build\targets\ui-render-r4-interface cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\zircon-build\targets\ui-render-r4-interface cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --message-format short --color never -- --nocapture (R7 review fixes: 19 passed, 0 failed; R8 shared parity seam: 21 passed, 0 failed)
  - rustfmt --edition 2021 --check "zircon_runtime_interface/src/tests/render_contracts.rs" (R9 interface parity harness: passed with no output)
  - cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-r9-offscreen-parity --message-format short --color never (R9 interface parity harness: passed)
  - cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-r9-offscreen-parity --message-format short --color never -- --nocapture (R9 interface parity harness: 23 passed, 0 failed, 32 filtered out)
  - rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/layout/geometry.rs zircon_runtime_interface/src/ui/surface/render/command.rs zircon_runtime_interface/src/ui/surface/render/list.rs zircon_runtime_interface/src/tests/mod.rs zircon_runtime_interface/src/tests/ui_geometry_metrics.rs
  - cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-interface-geometry --message-format short --color never
  - cargo test -p zircon_runtime_interface --lib ui_geometry_metrics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-interface-geometry --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never (passed with existing warnings)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (attempted; timed out during dependency compilation and background cargo/rustc continued)
  - cargo test -p zircon_editor --lib rust_owned_host_painter_draws_runtime_render_commands --locked --jobs 1 --message-format short --color never -- --nocapture (attempted; timed out during dependency compilation)
  - rustfmt --edition 2021 zircon_runtime_interface/src/ui/surface/render/command.rs zircon_runtime_interface/src/tests/render_contracts.rs
  - cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m2-interface --message-format short --color never -- --nocapture (M2 SVG/icon target-size contract: 24 passed, 0 failed, 35 filtered out)
  - cargo test -p zircon_runtime --lib asset_value_nodes_render_as_image_or_icon_not_text --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture (M2 render command image/icon source contract: 1 passed, 0 failed)
  - cargo test -p zircon_editor --lib runtime_svg_icon_pixels_follow_requested_target_size --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture (M2 native painter SVG scaling: 1 passed, 0 failed)
  - cargo test -p zircon_editor --lib template_svg_icon_pixels_follow_requested_target_size --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture (M2 template SVG scaling: 1 passed, 0 failed)
  - cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never (M6 rich paint run DTO: passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never (M6 runtime rich paint run consumer: passed with existing warnings)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never (M6 editor rich paint run consumer: first cold run timed out at the tool boundary after printing Finished; warm rerun passed)
  - cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture (M6 rich paint run DTO: 27 passed, 0 failed)
  - cargo test -p zircon_runtime --lib screen_space_ui_plan --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture (M6 rich paint run planner: 6 passed, 0 failed)
  - cargo test -p zircon_runtime --lib text_attrs --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture (M6 glyphon rich attrs: 1 passed, 0 failed)
  - cargo test -p zircon_editor --lib native_runtime_text_painter --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture (M6 editor text painter: 1 passed, 0 failed)
doc_type: module-detail
---

# Runtime Interface UI Render Contracts

## Purpose

`zircon_runtime_interface::ui::surface::render` is the neutral DTO layer for retained UI paint data. Runtime, editor, plugins, and future replay/debug tools must meet through this module instead of inventing renderer-specific visual schemas.

The current public `UiRenderCommand` remains the migration source for existing `Group` / `Quad` / `Text` / `Image` command producers. New code can derive `UiPaintElement` records from those commands without adding required fields to the old struct literals. This gives the renderer stack a Slate-like paint element, batch, resource, cache, shaped text draw, and debug visualizer contract while keeping the current shared command boundary stable during R1-R7.

## Paint Element Layer

`paint.rs` defines `UiPaintElement`, `UiPaintPayload`, `UiClipState`, and `UiPaintEffects`.

Each paint element carries `node_id`, `z_index`, `paint_order`, `UiGeometry`, optional `UiClipState`, a typed payload, effect state, and cache/debug placeholders. `UiRenderCommand::to_paint_element(...)` returns the legacy-compatible single payload view. `UiRenderCommand::to_paint_elements(...)` returns the fuller migration view: a button-like command with both background and text can produce a brush element plus a text element, preserving the behavior that glyph/text drawing is not mutually exclusive with quad drawing.

The M1 geometry slice adds `UiGeometry::from_frame_with_metrics(...)` plus metric-aware render command/list conversion. `absolute_frame` remains the unsnapped arranged/layout frame used by hit testing and debug authority, while `render_bounds` and `UiClipState.frame` can be snapped to the DPI pixel grid for painting. This makes render-side crispness explicit without giving editor or runtime consumers permission to replace `UiSurfaceFrame.arranged_tree` with a second coordinate source.

## Brush And Resource Layer

`brush.rs` defines `UiBrushPayload`, `UiBrushSet`, `UiRenderResourceKey`, `UiRenderResourceState`, and `UiResourceUvRect`.

The brush payload is intentionally broader than the current renderer implementation. It covers solid color, rounded color, border, image, box/9-slice-ready image, vector, gradient, and material brushes. `UiRenderResourceKey` includes resource kind, stable id, optional revision, optional atlas page, optional atlas UV rect, and an optional fallback resource. `UiRenderResourceState` mirrors the resolved resource fields used by brushes and adds pixel size, so atlas placement, image dimensions, fallback texture, and material revisions remain visible to cache/debug code without replacing the stable key. Batch keys consume this resource key so atlas moves, UV changes, fallback substitutions, and material revisions can split batches and invalidate caches deliberately.

The M2 SVG/icon slice makes that pixel-size field part of the render command conversion contract. `UiRenderCommand::to_paint_element_with_metrics(...)` now derives image/icon brush `resource_state.pixel_size` from the command's render bounds and `UiLayoutMetrics.dpi_scale`; `absolute_frame` stays as the layout/hit frame, while the resource state carries the physical target size that vector rasterizers, icon caches, and atlas allocators need. Runtime/editor callers can therefore scale SVG icons to the control frame without falling back to a fixed source pixel size or keeping a painter-local size table.

## Text Shape Layer

`text_shape.rs` defines `UiTextPaint`, `UiShapedText`, `UiShapedGlyph`, and `UiTextPaintDecoration` as the shared shape artifact boundary.

The first version derives from `UiResolvedTextLayout`, preserving source text, source ranges, visual ranges, run kinds, line frames, baseline, direction, overflow, and render mode. R6 extends that artifact with optional font resource keys, text atlas resources, ellipsis range, per-line glyph records, glyph ids, source ranges, visual frames, advances, atlas resources, and atlas UV rects. `UiTextPaint` also carries optional selection, caret, composition, and decoration records so selection fills, caret strokes, composition underline, and outline payloads can be shared with runtime/editor painters instead of recreated per backend.

The M6 text convergence slice makes the fallback shaped artifact cluster-aware instead of leaving `glyphs` empty. `UiShapedText::from_resolved_layout(...)` now derives one synthetic glyph record per Unicode grapheme cluster, including source range, visual frame, and advance. The glyph id is still a deterministic placeholder until a real font backend provides native glyph ids, but the range/frame facts are now shared by render visualizers, editor painter tests, and runtime renderer planning. `UiRenderCommand::text_paint(...)` also snaps selection, caret, and composition underline geometry to grapheme cluster edges, so a selection inside `a\u{0301}` or an emoji ZWJ sequence expands to the visible cluster instead of splitting an accent or emoji component.

The rich text paint slice adds `UiTextPaintRun` and `UiTextRunPaintStyle` beside the shaped line/cluster records. Each shared paint run carries the resolved text fragment, source and visual ranges, grapheme-snapped frame, inherited font/color data, and Strong/Emphasis/Code style flags. Runtime screen-space UI planning consumes those runs as text batches, and the native glyphon backend maps the style flags to bold, italic, and monospace attributes. The editor native painter consumes the same run DTO and applies its own software fallback style pass, so rich text no longer depends on renderer-local run parsing.

`editable_text.rs` defines the neutral editable text DTOs and edit action vocabulary. `UiTextComposition::restore_text` is optional because paint-only snapshots may only need the visible preedit range and text, but runtime editable state fills it while a composition is active. That snapshot lets `CancelComposition` restore the text that existed before visible preedit replacement, while `CommitComposition` keeps the already-visible preedit text without double insertion.

These fields do not claim final HarfBuzz or platform IME parity yet. They create the shared slots that glyphon, SDF, future HarfBuzz-backed shaping, and editor preview painting must converge on.

## Batch, Cache, And Debug Layer

`batch.rs` defines `UiBatchKey`, `UiBatchPlan`, batch ranges, primitive/shader classes, opacity classes, and split reasons. `UiBatchPlan::from_paint_elements(...)` performs stable consecutive merging and records why a new batch began: layer, clip, primitive, shader, resource, text backend, or opacity changed.

`cache.rs` defines the R5 neutral cache contract: `UiRenderCachePlan`, per-paint and per-batch cache entries, hit/rebuild status, invalidation reasons, and cache stats. The contract can represent reused paint elements/batches when cache generations are present and can explain recache causes such as node dirtiness, clip changes, resource revision changes, text shape changes, or forced rebuilds. This is intentionally a shared DTO/debug contract first; it does not make runtime reuse cached GPU buffers yet.

`debug.rs` defines `UiRenderDebugSnapshot`, which converts a `UiRenderExtract` into paint elements, batch debug entries, a cache debug plan, the R7 visualizer packet, and the R8 renderer parity packet. This is the shared debug payload shape that `UiSurfaceDebugSnapshot.render_batches` can carry; runtime capture and editor presentation remain separate consumer work.

`parity.rs` defines `UiRendererParitySnapshot` and row DTOs for the R8 shared seam. The packet records canonical paint order, batch order, clip identity, payload kind, batch key, resource key, text render mode, opacity, and summary counts. Runtime WGPU, editor native painter, and future offscreen capture tests can compare this packet to prove that they consumed the same neutral paint/batch contract before diagnosing backend-specific pixel differences.

R9 uses that parity packet as the first golden-test gate. Future runtime and editor adapters should emit comparable paint and batch rows before any pixel snapshot is accepted; pixel differences are only actionable after paint order, batch order, clip/resource/text identities, opacity, and parity stats match. This keeps backend rasterization, font sampling, antialiasing, and color-conversion differences from masking a lower shared contract drift.

2026-05-06 R8 validation stayed interface-only. The final closeout rerun saw E: below the 50 GB cleanup threshold, so `cargo clean --target-dir "E:\zircon-build\targets\ui-render-r4-interface"` removed 755.6 MiB before validation. `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/layout/slot.rs" "zircon_runtime_interface/src/ui/layout/linear_sizing.rs" "zircon_runtime_interface/src/ui/surface/render/debug.rs" "zircon_runtime_interface/src/ui/surface/render/parity.rs" "zircon_runtime_interface/src/ui/surface/render/mod.rs" "zircon_runtime_interface/src/ui/surface/mod.rs" "zircon_runtime_interface/src/tests/render_contracts.rs"` passed with no output. With `CARGO_TARGET_DIR=E:\zircon-build\targets\ui-render-r4-interface`, `cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --message-format short --color never` finished successfully and `cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --message-format short --color never -- --nocapture` passed with `21 passed; 0 failed; 31 filtered out`.

2026-05-06 R9 validation also stayed interface-only. D: had 125,702,815,744 bytes free, above the 50 GB cleanup threshold, so no target clean was needed. `rustfmt --edition 2021 --check "zircon_runtime_interface/src/tests/render_contracts.rs"` passed with no output. `cargo check -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-render-r9-offscreen-parity" --message-format short --color never` finished successfully, and `cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-render-r9-offscreen-parity" --message-format short --color never -- --nocapture` passed with `23 passed; 0 failed; 32 filtered out`.

`visualizer.rs` defines `UiRenderVisualizerSnapshot` and the DTOs a Widget Reflector style render panel needs without reading renderer-private state: paint-element rows, batch-group rows, overlay primitives, overdraw regions, resource bindings, text backend/glyph/decorator counters, and summary stats. The derived overlay set includes wireframe bounds, clip/scissor frames, batch bounds, overdraw heat, resource/atlas markers, text baselines, and glyph bounds when the shaped text artifact carries glyph data. This is a shared contract and export/replay artifact first; it does not implement the editor panel UI, runtime overlay toggles, GPU overdraw pass, or live renderer instrumentation.

## Consumer Boundary

R1-R6 included runtime/editor consumer wiring outside this crate: runtime diagnostics derive `render_batches` from `UiSurfaceFrame.render_extract`, and the editor Rust-owned host painter consumes derived `UiPaintElement` / `UiPaintPayload` records instead of treating `UiRenderCommandKind` as the only visual authority.

R7 does not add a live editor visualizer panel, runtime overlay toggles, or renderer instrumentation. It only adds the shared export/replay packet consumed by later runtime/editor debug surfaces.

## Current Boundaries

This slice does not replace the WGPU screen-space draw planner with `UiBatchPlan`, `UiRenderCachePlan`, `UiRenderVisualizerSnapshot`, or `UiRendererParitySnapshot` yet. The runtime renderer still plans vertices and text batches from `UiRenderCommand`; the new batch/cache/text-shape/visualizer/parity plans are shared contracts and debug/replay sources for R3-R8 wiring.

This slice also does not add material shaders, line/gradient/custom-vertex GPU draw paths, final glyph shaping, or live editor visualizer panels. Atlas UV/resource revision/fallback fields, cache invalidation reasons, shaped glyph/decorator fields, and render visualizer overlay/resource/stat fields are now represented in shared DTOs and batch/cache/text/debug keys. The current M6 glyphs are cluster-safe DTO placeholders, not backend glyph ids from HarfBuzz/glyphon; actual image/icon/SVG/font atlas allocation, runtime cache reuse, platform IME rendering, GPU overdraw measurement, and editor/runtime visualizer UI remain later implementation steps named in the rendering audit.
