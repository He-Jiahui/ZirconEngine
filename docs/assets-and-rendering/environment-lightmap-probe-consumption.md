---
related_code:
  - zircon_runtime/src/core/framework/render/environment/ambient.rs
  - zircon_runtime/src/core/framework/render/environment/lightmap.rs
  - zircon_runtime/src/core/framework/render/environment/lightmap/tests.rs
  - zircon_runtime/src/core/framework/render/environment/extract.rs
  - zircon_runtime/src/core/framework/render/environment/mod.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/graphics/runtime/offline_bake/offline_bake_frame.rs
  - zircon_runtime/src/asset/assets/texture/lightmap_asset.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/lightmap_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/lightmap_binding/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_bind_group_layout/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_lightmap.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_forward.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_deferred_gbuffer.wgsl
  - zircon_runtime/src/graphics/tests/render_product_baked_lighting.rs
  - zircon_runtime/src/graphics/tests/fixtures/plan11_baked_lighting_v1.json
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/baked_lighting.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/environment/ambient.rs
  - zircon_runtime/src/core/framework/render/environment/lightmap.rs
  - zircon_runtime/src/core/framework/render/environment/lightmap/tests.rs
  - zircon_runtime/src/core/framework/render/environment/extract.rs
  - zircon_runtime/src/core/framework/render/environment/mod.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/graphics/runtime/offline_bake/offline_bake_frame.rs
  - zircon_runtime/src/asset/assets/texture/lightmap_asset.rs
  - zircon_runtime/src/asset/assets/texture/upload_support.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_texture/gpu_texture_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/lightmap_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/lightmap_binding/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_bind_group_layout/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_lightmap.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_forward.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_deferred_gbuffer.wgsl
  - zircon_runtime/src/graphics/tests/render_product_baked_lighting.rs
  - zircon_runtime/src/graphics/tests/fixtures/plan11_baked_lighting_v1.json
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/baked_lighting.rs
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
  - user: 2026-07-13 improve Hybrid GI baked-static and dynamic-lighting usage from the 35-article graphics collection
tests:
  - tools/tests/test_environment_lightmap_contract.py
  - tools/tests/test_environment_lightmap_gpu_binding_contract.py
  - zircon_runtime/src/core/framework/render/environment/lightmap/tests.rs
  - zircon_runtime/src/core/framework/render/environment/extract.rs::baked_environment_requires_one_light_set_generation
  - zircon_runtime/src/graphics/tests/render_product_baked_lighting.rs
doc_type: module-detail
---

# Environment Lightmap And Probe Consumption

## Current State

EL-M3 is complete. It has a renderer-neutral serializable contract, an importable raw RGBA16F array asset, stable GPUScene slot metadata, and shared Forward+/Deferred shader consumption. The resource owner creates a neutral `rgba16float` 2D-array fallback, filtering sampler, and SH9 probe storage buffer at bindings 24, 28, and 23 respectively. The external JSON bake fixture, numeric WGPU readback parity, and product screenshot gates pass.

Hybrid GI still exposes only `DynamicOnly`: EL-M3 now provides the validated baked baseline required by HGI-M4, but HGI-M4 has not yet implemented participation, source-ledger dynamic delta, invalidation epochs, or product presets. `BakedStaticDynamic` therefore remains unavailable until those HGI-owned gates pass.

The old `RenderBakedLightingExtract` ambient color is not a lightmap. It must not be used as evidence that static baked lighting is complete, and it must not be combined with full HGI bounce energy without the HGI source-ledger policy.

## Ownership

- `LightmapConsumeContract` owns the imported atlas asset, its RGBA16F array descriptor, stable renderer-instance-to-slot mapping, contract version, and `light_set_generation`.
- `LightmapInstanceSlot` maps UV2 into one RGBA16F atlas-array page through `scale.xy + offset.xy`.
- `LightProbeGridData` owns a uniform world-space grid and one SH L2 RGB value per grid point.
- `LightmapBakeRequest` carries a versioned scene snapshot payload, atlas budget, texel density, static instance set, probe bounds, request identity, and light-set generation.
- `LightmapBakeOutput` carries importable little-endian RGBA16F atlas pages, slots, and an optional probe grid. `validate_against(request)` rejects stale/mismatched work before import; `into_consume_contract(atlas_asset)` is only called after the asset layer registers the page array.
- `SceneLightmapResources` owns the current GPU ABI. Probe storage uses three `vec4` header records followed by nine RGB coefficient `vec4` records per grid point.
- `EnvironmentExtract` is the sole baked lightmap/probe owner and accepts only matching validated generations. The old lighting snapshot retains only its legacy color/intensity shape for ABI compatibility and is forced neutral; offline bake no longer synthesizes a fake full-screen ambient result. The rendering plugin owns baking; the runtime owns validation and consumption.

## Generation Rules

`light_set_generation` is non-zero and must match across the atlas contract, probe grid, bake output, and later Hybrid GI frame metadata. A generation changes when the authoritative baked light set or its produced assets change. Runtime intensity tweaks that do not alter baked content must not silently rewrite the generation.

Mixed generations are rejected before GPU upload. HGI history, Surface Cache pages, and dynamic-delta weights must be invalidated when this generation changes.

`LightmapInstanceSlot::transform_uv2` and `inverse_transform_uv2` are the CPU reference for shader UV transformation. `LightProbeGridData::sample_trilinear` is the CPU reference for world-position SH interpolation, including finite/bounds checks and exact grid indexing.

## Validation

The contract rejects:

- unsupported contract versions;
- zero light-set generations;
- duplicate stable instance IDs;
- non-finite, zero-area, negative, or out-of-page UV rectangles;
- missing/unsupported scene snapshots, invalid bake identity, atlas budgets, texel density, or RGBA16F payload sizes;
- duplicate, missing, or out-of-range atlas pages and slots;
- outputs whose request ID, scene revision, generation, budget, or instance set differs from the originating request;
- zero probe dimensions, invalid bounds/cell sizes, coefficient-count mismatch, overflow, or non-finite SH values;
- bake outputs whose atlas and probe generations disagree.

Probe indexing is performed with checked `usize` multiplication/addition, so the sampling path uses the same address width and overflow policy as capacity validation.

## GPU ABI

| Group 1 binding | Resource | Current state |
|---:|---|---|
| 23 | read-only SH9 probe-grid storage buffer | packing and generation-aware upload implemented |
| 24 | filterable `texture_2d_array<f32>` lightmap atlas | RGBA16F fallback and imported atlas resolution implemented |
| 28 | filtering sampler | implemented; 25/26/27 are reserved for volumetric apply |

`texture_asset_from_lightmap_bake_output` sorts pages by index and creates the `zircon-lightmap-rgba16f-le-v1` container. Upload readiness rejects truncated or incorrectly shaped data; `GpuTextureResource` uploads every page to a `Rgba16Float` array layer. `ResourceStreamer::ensure_scene_resources` prepares the contract atlas, and `SceneLightmapResources::prepare` validates the GPU descriptor, uploads the matching probe generation, then refreshes both Forward and Deferred bindings.

The lightmap resource always exposes a `D2Array` view, including a one-page atlas. Its legacy generic texture bind group separately retains a `D2` view of page zero because that bind group layout predates array textures; the environment binding never consumes that compatibility view. WGPU tests lock both dimensions and the real GBuffer binding path.

`GpuInstanceData` stores the stable UV rect, atlas page, enabled bit, and 64-bit light-set generation. `zr_lightmap.wgsl` samples the atlas for mapped static instances and falls back to trilinear SH9 grid irradiance for unmapped/dynamic instances. Forward+ adds the baked diffuse term in its fragment lighting path. Deferred geometry computes the same per-instance term while UV2 and instance id are available, then stores it in the existing HDR emissive MRT for the fullscreen lighting pass to restore. This reuse is intentional transport of already-lit indirect radiance, not a second baked owner.

## EL-M3 Product Evidence

- Fixture: `zircon_runtime/src/graphics/tests/fixtures/plan11_baked_lighting_v1.json`.
- Product test: `render_product_baked_lightmap_and_dynamic_probe_match_forward_deferred`.
- Focused current binary: `lightmap` 20/20 with one ignored exporter, `baked_indirect` 2/2, GBuffer WGPU 1/1, exporter 1/1.
- Static region baseline delta MAE: Forward `9.5296`, Deferred `9.5507`.
- Dynamic probe region baseline delta MAE: Forward `3.6566`, Deferred `3.6784`.
- Forward/Deferred parity: MAE `0.0214`, maximum channel error `1`.
- Evidence: `docs/tests/runtime/render/plan11_lightmap_probe_forward_deferred_wgpu_20260713.png`, SHA-256 `386909A40E13EB4C0B8E27B354D05AC0DAEE2113FA2EC9A564F787B9B30FAB22`.

The next owner is HGI-M4. It must consume this baseline and generation contract without copying the bake DTOs or adding a second baked owner.
