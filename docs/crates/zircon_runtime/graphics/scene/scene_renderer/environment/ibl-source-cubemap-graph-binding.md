---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_environment_ibl_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_compiled_scene_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_environment_ibl_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_compiled_scene_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/review_guard_maps/sources/folder_backed.rs zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_environment_ibl_graph_resources.rs zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_compiled_scene_graph_resources.rs zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/mod.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-runtime-writeback-0706 --message-format short --color never (exit 0)
  - cargo test -p zircon_runtime --lib environment_ibl_source_binder --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-runtime-writeback-0706 --message-format short --color never -- --nocapture --test-threads=1 (not counted; timed out after 600s)
  - docs/tests/runtime/render/plan11_ibl_source_binding_validation_20260706.png
doc_type: module-detail
---

# IBL Source Cubemap Graph Binding

## Purpose

`bind_environment_ibl_graph_resources.rs` is the compiled-scene resource binding bridge for Plan 11 / Shader 06 IBL bake input. The IBL bake graph declares `environment.ibl.source_cubemap` as a required external texture. This helper binds that logical graph resource to the source cubemap view already uploaded by `SceneEnvironmentCubemap`.

The module sits in `scene_renderer_core_render_compiled_scene/render` because it is part of frame graph resource binding. It does not build IBL requests, create compute pipelines, read back artifacts, or write cache blobs.

## Contract

- The helper only imports `environment.ibl.source_cubemap` when the compiled graph declares it as an external texture.
- If another owner already bound that graph resource, the helper leaves it unchanged.
- If the current frame has no source cubemap view, the helper leaves the required external missing. The materialization report must expose that missing resource instead of receiving a dummy fallback.
- `render.rs` calls the helper after `write_scene_uniform(...)`, so the source cubemap has validated its immutable artifact, appended its staging range to the outer frame upload batch, and recorded its scene-encoder copies before the borrowed view is imported. The revision remains pending until scene submission succeeds.
- `SceneEnvironmentCubemap::source_view()` is a narrow renderer-core accessor for this graph binding path.

This closes the source-input side of the live WGPU IBL bake chain. The output resource acquisition and artifact readback remain owned by `ibl_bake_wgpu_readback.rs` and `ibl_bake_runtime_writeback.rs`.

## Verification

Counted evidence:

- Targeted rustfmt passed for the new binding helper, render binding call sites, and the review-guard compile-blocker fix.
- Core-min `cargo check -p zircon_runtime --lib` passed with target dir `E:\cargo-targets\zircon-ibl-runtime-writeback-0706`.
- Screenshot/text validation artifacts were written under `docs/tests/runtime/render/plan11_ibl_source_binding_validation_20260706.*`.

Not counted as passing:

- Focused `environment_ibl_source_binder` lib-test exceeded the 600s tool window during broader lib-test build. The timeout is recorded under `docs/tests/runtime/render/plan11_ibl_source_binding_tests_20260706.*`.

## Reference Audit

The Lumen reference scan covered `dev/LumenInUE5.5.4WithComputeShader/Res/Shader/LumenSceneLighting/Radiosity/*.hlsl`. Those passes use `[numthreads(8,8,1)]`, read from SRV/StructuredBuffer-style inputs, and write RWTexture outputs. That matches the current Zircon direction: imported graph inputs plus transient graph outputs feed renderer-owned WGPU compute dispatch.

## Open Issues

Open gates remain: focused test pass after the lib-test queue clears, production scheduler queue ownership, product graph injection, readback draining before transient release, runtime cache writeback from GPU outputs, product second-launch dispatch=0 proof, RenderDoc/product capture, optimized SH9 reduction, strict roughness/SSIM/seam validation, 4K/16K offline bake coverage, and full CI.
