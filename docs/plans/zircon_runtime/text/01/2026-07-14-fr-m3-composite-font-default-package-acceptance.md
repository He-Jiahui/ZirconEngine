---
related_code:
  - zircon_runtime/assets/fonts/default.font.toml
  - zircon_runtime/assets/fonts/ZirconDefaultComposite-subset.ttc
  - zircon_runtime/src/graphics/text/font/database.rs
  - zircon_runtime/src/graphics/text/font/composite_resolve.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs
plan_sources:
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python -m unittest tools.tests.test_text_01_composite_activation tools.tests.test_frameworks_05_manager_access_lifetime
  - managed Windows zircon_runtime lib-test text_font
  - managed Windows runtime_text_multilingual_product_framebuffer exact ignored WGPU exporter
doc_type: milestone-acceptance
status: accepted
---

# Text01 FR-M3 CompositeFont default package acceptance

Plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md

Milestone: M3

Status: accepted

Files: ["zircon_runtime/assets/fonts/default.font.toml", "zircon_runtime/assets/fonts/ZirconDefaultComposite-subset.ttc", "zircon_runtime/assets/fonts/OFL-NotoSansSC.md", "zircon_runtime/src/asset/tests/assets/font.rs", "zircon_runtime/src/graphics/text/font/database.rs", "zircon_runtime/src/graphics/text/font/database/tests.rs", "zircon_runtime/src/graphics/text/font/fallback.rs", "zircon_runtime/src/graphics/text/font/fallback/tests.rs", "zircon_runtime/src/graphics/text/font/composite_resolve.rs", "zircon_runtime/src/graphics/text/font/mod.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/product_project_fixture.rs", "tools/tests/test_frameworks_05_manager_access_lifetime.py", "tools/tests/test_text_01_composite_activation.py", "docs/zircon_runtime/asset/assets/font.md", "docs/zircon_runtime/graphics/text.md", "docs/zircon_runtime/graphics/scene/scene_renderer/ui/text.md", "docs/plans/zircon_runtime/text/01/2026-07-14-fr-m3-composite-font-default-package-acceptance.md", "docs/tests/runtime/text/runtime_text_composite_font_cjk_product_framebuffer_20260714.png"]

## Scope delivered

- `FontDatabase::register_font_asset` registers faces and ordinary fallback families without installing a project CompositeFont or leaking culture-specific CompositeFont families into generic fallback.
- `ScreenSpaceUiTextSystem` is the explicit project activation owner: construction activates the loaded default record exactly once, `None` clears stale policy, and secondary font loads cannot replace it.
- Composite candidate enumeration is isolated in the folder-backed `composite_resolve.rs` leaf and preserves sub-font order before the default/query/global fallback chain.
- The checked-in TTC contains Fira Mono face 0 plus a deterministic Noto Sans SC proof subset at face 1. The first `zh-Hans` route selects face 1, and the SIL OFL is stored beside the package.
- The manager lifetime failure handoff is retained in the same UI path: the long-lived text system stores versioned `ProjectAssetManagerAccess`, resolves one bounded Arc per construction/prepare operation, and propagates resolution errors.

## Fresh testing evidence

- Static TDD guards pass 4/4 and scoped `rustfmt` plus `git diff --check` pass.
- Managed Windows job `74a9c013aa364584b96832d65098538c` first exposed eight owned test compile errors after the resolver split and record extension; the imports/literals were fixed.
- Managed Windows retry `f419e52a69994e2cb6a849aac8b3786b` compiled past all Text01 owners and then stopped only on two concurrently introduced, non-Text `advanced_lighting` field-path errors in `light_grid_pass.rs` and `build_mesh_draws/build/build.rs`.
- Managed Windows GPU job `f320e76017714cfe97b9b52f92f69b52` built the current integration target and completed the exact ignored WGPU exporter with exit 0 (1 passed / 0 failed). The renderer exercised multilingual native/SDF text, inline textures, BiDi, vertical layout, tables, and the default `zh-Hans` CompositeFont route before readback.
- Accepted artifact: `docs/tests/runtime/text/runtime_text_composite_font_cjk_product_framebuffer_20260714.png`, 1080×1840, 353,953 bytes, 2,442 colors, SHA256 `754A7C1CC64D98B50D6FB798F702353C4BABB7EAAA5B722657529B4641BB9C40`.
- Original-resolution visual inspection confirms the `中文排版：引擎文本与布局` row, Arabic/Hebrew/emoji, vertical columns, rich inline images, horizontal/vertical tables, Native/SDF parity, and variable-width samples render without tofu or overlap.
- Same-name scan across repository `target` plus `D:\cargo-targets`, `E:\cargo-targets`, and `F:\cargo-targets` returned zero duplicates.

## Review

- Independent review: Critical 0, Important 0, `Accept`.
- The reviewer verified explicit activation ownership, secondary-load isolation, TTC face-1 Chinese coverage, real WGPU capture, manager lifetime, and structure constraints; the concurrent advanced-lighting E0609 failure was correctly classified as external.

## Acceptance

FR-M3 is accepted. The project default package, explicit CompositeFont activation boundary, candidate data plane, manager lifetime repair, focused guards, and real product framebuffer are complete. A manifest, candidate list, or strategy-only image was not used as visual evidence.
