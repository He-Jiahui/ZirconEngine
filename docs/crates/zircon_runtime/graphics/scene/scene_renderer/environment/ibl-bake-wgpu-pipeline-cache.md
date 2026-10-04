---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
plan_sources:
  - user: 2026-07-06 continue Shader 06 / Plan 11 true HDRI PBR reflection implementation
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - cargo fmt --package zircon_runtime
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-wgpu-dispatch-coverage-0706 CARGO_INCREMENTAL=0 cargo check -p zircon_runtime --lib --tests --no-default-features --features core-min --locked --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-wgpu-dispatch-coverage-0706 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib ibl_bake_wgpu_pipeline_cache --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-wgpu-dispatch-coverage-0706 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib ibl_bake_wgpu_dispatch --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# IBL Bake WGPU Pipeline Cache

## Purpose

`ibl_bake_wgpu_pipeline_cache.rs` owns the renderer-lifetime WGPU pipeline cache for Plan 11 / Shader 06 IBL baking. It keeps IBL bake pipeline ownership inside the environment subsystem instead of mixing it into the mesh pipeline cache.

The cache stores:

- the two IBL bake bind group layouts from `ibl_bake_wgpu_binding.rs`,
- WGSL shader modules keyed by `ComputePipelineCacheKey`,
- pipeline layouts keyed by output binding class,
- compute pipelines keyed by `ComputePipelineCacheKey` plus storage-texture/storage-buffer layout kind.

This means PMREM mip commands reuse the same shader module, pipeline layout, and compute pipeline across all mip levels. SH9 uses its own storage-buffer layout and pipeline. IEM uses the storage-texture layout but its own shader/pipeline key.

## Runtime Ownership

`SceneRendererCore` constructs one `IblBakeWgpuPipelineCache` with the renderer. `execute_graph_stage(...)` passes the cache into `RenderPassGpuExecutionContext`, and the IBL bake dispatch bridge requires that cache when recording a graph-context WGPU dispatch.

This is an intentional production-path requirement. If an IBL bake graph pass reaches `record_ibl_bake_wgpu_pass_for_request(...)` without a renderer cache, it fails with a renderer-context error instead of silently creating temporary pipelines per dispatch.

## Dispatch Use

`ibl_bake_wgpu_dispatch.rs` still exposes a direct `create_ibl_bake_wgpu_compute_pipeline(...)` helper for focused low-level tests. The graph-context path uses `IblBakeWgpuPipelineCache::ensure_compute_pipeline(...)` and receives a cloned WGPU pipeline handle from the cache before encoding the compute pass.

The bind group still uses per-dispatch params buffers and output views because those depend on the command and graph resource. The shared layouts and compute pipelines are cache-owned.

## Verification

Focused cache coverage verifies that PMREM mip0 and mip1 share one shader module, one storage-texture pipeline layout, and one compute pipeline. It then adds SH9 and verifies that the storage-buffer path creates a second shader module, layout, and compute pipeline.

The existing dispatch tests now attach an `IblBakeWgpuPipelineCache` to the test GPU context, so PMREM, SH9, and IEM graph-context dispatches exercise the production cache path.

## Open Issues

This closes renderer-lifetime pipeline/layout ownership and reuse for IBL bake WGPU dispatch. The remaining production gates are scheduler-owned request injection, async scheduling/readback/cache writeback, GPU final 1x1 face average, optimized SH9 reduction, product second-launch dispatch=0 proof, strict screenshot/SSIM/roughness/seam validation, RenderDoc/product capture, 4K/16K offline bake, and full CI.
