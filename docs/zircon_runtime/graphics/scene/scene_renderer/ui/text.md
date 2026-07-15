---
related_code:
  - zircon_runtime/assets/fonts/default.font.toml
  - zircon_runtime/assets/fonts/ZirconDefaultComposite-subset.ttc
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs
  - zircon_runtime/src/text/font/composite_resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
implementation_files:
  - zircon_runtime/assets/fonts/default.font.toml
  - zircon_runtime/assets/fonts/ZirconDefaultComposite-subset.ttc
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs
  - zircon_runtime/src/text/font/composite_resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/runtime/15/fixed-2026-07-14-ui-text-manager-access-cross-frame-retention.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - tools/tests/test_frameworks_05_manager_access_lifetime.py
  - tools/tests/test_text_01_composite_activation.py
  - zircon_runtime/src/graphics/text/font/database/tests.rs::text_font_runtime_default_composite_selects_checked_in_zh_hans_face
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs
  - python -m unittest tools.tests.test_frameworks_05_manager_access_lifetime tools.tests.test_frameworks_05_layer_direction -v (24/24 passed)
  - managed Windows default-feature cargo build -p zircon_runtime --locked (job c2db4e7bfe0647678e6334648b6df811; passed)
doc_type: module-detail
---

# Screen-Space UI Text Manager Access Lifetime

## Purpose

The screen-space UI text subsystem shapes, resolves, uploads, and renders native bitmap plus SDF text for a viewport. It requires project assets for font manifests and glyph sources, but its renderer and caches live across frames. The manager access boundary therefore separates long-lived identity from short-lived concrete manager use.

## Ownership And Lifetime

`ScreenSpaceUiTextSystem` stores `ProjectAssetManagerAccess`, which contains a weak Core reference and a versioned manager handle. It never stores `Arc<ProjectAssetManager>`. Construction resolves the access only while loading the initial default-font record; the Arc is dropped when construction ends. Every `prepare` call resolves again and reuses that Arc only for the current frame's batch resolution, SDF generation, fallback measurement, and native bitmap atlas work.

This boundary preserves unload and generation semantics. If the manager is unavailable or the stored identity is stale, the next construction or frame prepare fails instead of continuing through an old manager instance.

## Error Flow

`ScreenSpaceUiTextSystem::{new,prepare}` return `CoreError` for access resolution failures. `ScreenSpaceUiRenderer` maps those failures to `GraphicsError::Asset`. Ordinary `SceneRendererCore::render_scene` propagates the graphics error with `?`; render-graph execution converts the same error to its existing string error channel. Neither path silently drops text, substitutes a fallback manager, or keeps an Arc adapter.

## Data Flow

1. Renderer construction passes the versioned access into the text system.
2. Text construction resolves once to load `res://fonts/default.font.toml`, then stores only the access.
3. A frame with UI text enters `ScreenSpaceUiRenderer::record` and calls text `prepare`.
4. `prepare` resolves once, uses the Arc throughout that bounded frame operation, and drops it before returning.
5. Resolution failure aborts the renderer operation through the caller's normal error result.

## Constraints

- Long-lived UI objects may store versioned access or handles, not concrete asset managers.
- One bounded operation may reuse one resolved Arc to avoid repeated registry resolution in hot inner loops.
- Tests may construct a real test CoreRuntime through `ProjectAssetManagerAccess::for_test`; production has no standalone manager path.
- Retired named resolvers, implicit Arc conversions, compatibility modules, and silent fallbacks are forbidden.

## Test Coverage

`tools.tests.test_frameworks_05_manager_access_lifetime` scans the current production owners and locks access storage, constructor/per-frame resolution, absence of concrete manager storage, and both error propagation paths. Together with the Frameworks05 layer suite it passed 24/24. Managed Windows job `c2db4e7bfe0647678e6334648b6df811` passed the default-feature Runtime compile gate for the signature changes and all callers.

## Remaining Scope

This module document covers manager lifetime and error propagation. Font shaping, native bitmap atlas retry, SDF material policy, and glyph cache behavior remain owned by their existing focused module documents and tests.

## Default CompositeFont Activation

Text01 FR-M3 gives this long-lived system the only UI-side project activation point. Construction loads the default font record through the short-lived resolved manager, registers its faces, then calls `FontDatabase::set_project_composite_font` exactly once with that record's descriptor. When the record is absent, it passes `None` and clears stale state. `font_assets.rs` may load any number of secondary assets for a frame, but those loads only add faces and publish the shared database; they cannot replace the project CompositeFont.

The checked-in default manifest selects `Zircon Noto Sans CJK SC Proof` from TTC face 1 for `zh-Hans`, before optional system fallbacks. The product fixture verifies the real face bytes cover its Chinese content and then renders through the normal screen-space WGPU path. Acceptance therefore requires framebuffer pixels from the engine path; manifest text, candidate diagnostics, or policy-only screenshots are insufficient.

Managed GPU job `f320e76017714cfe97b9b52f92f69b52` passed the exact ignored exporter 1/1. The accepted framebuffer is `docs/tests/runtime/text/runtime_text_composite_font_cjk_product_framebuffer_20260714.png` (1080×1840, SHA256 `754A7C1CC64D98B50D6FB798F702353C4BABB7EAAA5B722657529B4641BB9C40`), and approved target roots contain no same-name copy. Independent review returned Critical 0 / Important 0 / Accept.
