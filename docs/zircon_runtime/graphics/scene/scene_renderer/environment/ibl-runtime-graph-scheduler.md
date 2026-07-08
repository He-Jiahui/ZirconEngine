---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/environment_ibl_compile_options.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_compile_options.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_construct/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/project/paths.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes.rs
implementation_files:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/environment_ibl_compile_options.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_construct/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes.rs
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - rustfmt --edition 2021 --check (touched Rust files)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-runtime-scheduler-0706 --message-format short --color never (exit 0)
  - cargo test -p zircon_runtime --lib environment_ibl --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-runtime-scheduler-0706 --message-format short --color never (not counted; timed out after 600s during test-harness build/link)
  - docs/tests/runtime/render/plan11_ibl_runtime_scheduler_validation_20260706.png
  - cargo test -p zircon_runtime --lib export_runtime_render_ibl_cache_second_launch_dispatch_zero_png --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-product-cache-0707 --message-format short --color never -- --ignored --nocapture --test-threads=1 (exit 0)
  - docs/tests/runtime/render/plan11_ibl_product_second_launch_dispatch_zero_20260707.png
  - docs/tests/runtime/render/plan11_ibl_product_second_launch_dispatch_zero_20260707.txt
  - direct lib-test binary run: export_runtime_render_ibl_cache_second_launch_dispatch_zero_png --ignored --nocapture --test-threads=1 (exit 0)
  - docs/tests/runtime/render/plan11_ibl_product_wgpu_capture_second_launch_dispatch_zero_20260707.png
  - docs/tests/runtime/render/plan11_ibl_product_wgpu_capture_second_launch_dispatch_zero_20260707.txt
doc_type: module-detail
---

# IBL Runtime Graph Scheduler

## Purpose

This slice connects the source-cubemap IBL bake graph to the product render pipeline. When the current extracted environment has a source cubemap and the project runtime cache does not already contain the requested IBL artifact, the final render-pipeline compile options carry an `IblBakeArtifactRequest`. Pipeline authoring then appends the PMREM/SH9/IEM bake passes into the compiled `RenderGraph`.

The goal is to move the IBL bake path from manual graph-context tests into the normal frame submission path without collapsing asset cache ownership, graph authoring, and WGPU execution into one module.

## Runtime Flow

- `environment_ibl_compile_options.rs` derives the request from `EnvironmentExtract::source_cubemap_ibl_bake_request(...)`.
- `ProjectAssetManager::ibl_bake_artifact_cache_store()` exposes the project runtime cache store without moving filesystem logic into renderer core.
- Cache misses keep `environment_ibl_bake_request` in `RenderPipelineCompileOptions`; cache hits, missing source cubemaps, or missing project stores clear it.
- `pass_authoring.rs` calls `append_ibl_bake_artifact_graph_plan(...)` and records generated IBL passes in `RenderPassStage::AmbientOcclusion`, which currently runs before Lighting.
- `SceneRenderer::new_with_icon_source(...)` registers the default IBL compute executors so the authored graph can execute in the product renderer.
- `submit_compiled_scene_frame(...)` writes runtime cache artifacts after queue submission and before transient graph resources are released.
- `project_render::export_runtime_render_ibl_cache_second_launch_dispatch_zero_png` proves the same project/source cubemap executes IBL bake executors on the first product framework submit and omits them on the second submit after the runtime cache artifact exists.
- The product screenshot proof uses a perspective Core3D capture snapshot because the reusable PBR matrix scene's orthographic camera intentionally resolves to the Core2D product pipeline. With the Core3D snapshot, the second frame is captured through `WgpuRenderFramework::capture_frame(...)` and validates as nonblack.

## Ownership Boundaries

The scheduler does not define artifact bytes, cache filenames, or backend readback formats. Those remain in `core/framework/render/environment` and `asset/artifact`. The scheduler also does not duplicate pass names or executor ids; pipeline authoring reuses the existing environment IBL graph-plan constants through a crate-internal graphics boundary.

`CompiledRenderPipeline::environment_ibl_bake_request` is intentionally a narrow frame closeout carrier. It tells the renderer whether the compiled graph was authored for a runtime bake and gives writeback the same request shape used by cache selection.

## Verification

Counted evidence:

- Targeted rustfmt check passed for the touched Rust files.
- Core-min `cargo check -p zircon_runtime --lib` exited 0 with target dir `E:\cargo-targets\zircon-ibl-runtime-scheduler-0706`.
- Validation PNG was written and visually inspected at `docs/tests/runtime/render/plan11_ibl_runtime_scheduler_validation_20260706.png`.
- Ignored product test `export_runtime_render_ibl_cache_second_launch_dispatch_zero_png` exited 0 with target dir `E:\cargo-targets\zircon-ibl-product-cache-0707`. The report records first-frame IBL executor count 9, second-frame IBL executor count 0, and post-first environment dispatch count 0.
- Product visual PNG was written and visually inspected at `docs/tests/runtime/render/plan11_ibl_product_second_launch_dispatch_zero_20260707.png`.
- Direct run of the built lib-test binary for `export_runtime_render_ibl_cache_second_launch_dispatch_zero_png --ignored --nocapture --test-threads=1` exited 0. The Wgpu capture report records first-frame IBL executor count 9, second-frame IBL executor count 0, first/second pipeline `RenderPipelineHandle(1)`, post-first environment dispatch count 0, and a `FrameworkOffscreen` 1280x960 capture.
- Wgpu product capture PNG was written and visually inspected as nonblack at `docs/tests/runtime/render/plan11_ibl_product_wgpu_capture_second_launch_dispatch_zero_20260707.png`.

Not counted as passing:

- Focused `environment_ibl` lib-test exceeded the 600s tool window while building/linking the lib-test harness. The timeout record is `docs/tests/runtime/render/plan11_ibl_runtime_scheduler_tests_20260706.exit.txt`.
- Full `--tests` compile for the 2026-07-07 product proof exceeded 600s and is retained as process evidence, not a passing result.
- The first 2026-07-07 mixed proof PNG used the same project/source cubemap through `SceneRenderer`; it remains process evidence but is superseded by the Wgpu product capture PNG for the capture gate.

## Open Issues

Open gates remain: RenderDoc product trace, asset-derived artifact production, strict roughness/SSIM/seam validation, 4K/16K offline bake, and full CI.
