---
related_code:
  - zircon_runtime/src/asset/assets/scene/lighting.rs
  - zircon_runtime/src/asset/assets/scene/post_process.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/mod.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/extract.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/extract/tests.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/volumetric.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/volumetric/tests.rs
  - zircon_runtime/src/core/framework/render/post_process/resolved_stack.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component/params.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_registry.rs
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs
  - zircon_runtime/src/scene/components/scene/lighting.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/project_io/post_process.rs
  - zircon_runtime/src/scene/world/render/lights.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/media_inject.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/media_inject/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/media_inject/shaders/media_inject.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter/shaders/types.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter/shaders/main.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/integrate.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/integrate/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/integrate/tests/temporal_product.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/integrate/shaders/integrate.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/apply_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/resolved_settings.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/temporal_reprojection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/view_reconstruction.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/executors/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/executors/media_inject.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/executors/light_scatter.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/executors/integrate.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/volumetric_history.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_volumetric.wgsl
  - zircon_runtime/src/graphics/shader/template/assemble.rs
  - zircon_runtime/src/graphics/shader/template/pass_specialization.rs
  - zircon_runtime/src/graphics/shader/template/tests.rs
  - zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/preview_sky_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/construct/create_sky_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_bind_group_layout/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/core/framework/render/post_process/graph_resource_names.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/lib.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/tests.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/wgpu_product_tests.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/wgpu_product_tests/renderdoc_capture.rs
implementation_files:
  - zircon_runtime/src/asset/assets/scene/lighting.rs
  - zircon_runtime/src/asset/assets/scene/post_process.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/extract.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/volumetric.rs
  - zircon_runtime/src/core/framework/render/post_process/resolved_stack.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component/params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/media_inject.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/media_inject/shaders/media_inject.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter/shaders/types.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter/shaders/main.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/integrate.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/integrate/shaders/integrate.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/temporal_reprojection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/executors/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/executors/light_scatter.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/volumetric_history.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_volumetric.wgsl
  - zircon_runtime/src/graphics/shader/template/assemble.rs
  - zircon_runtime/src/graphics/shader/template/pass_specialization.rs
  - zircon_runtime/src/graphics/shader/includes/zr_pbr_extras.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/scene/components/scene/lighting.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/project_io/post_process.rs
  - zircon_runtime/src/scene/world/render/lights.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/preview_sky_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/construct/create_sky_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_bind_group_layout/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/lib.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/wgpu_product_tests.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/wgpu_product_tests/renderdoc_capture.rs
plan_sources:
  - user: 2026-07-11 continue the WGPU-to-render-pipeline implementation and update completion status
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - dev/UnrealEngine/Engine/Shaders/Private/VolumetricFog.usf
  - dev/bevy/crates/bevy_light/src/volumetric.rs
tests:
  - zircon_runtime/src/core/framework/render/advanced_lighting/volumetric/tests.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/extract/tests.rs
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/media_inject/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/integrate/tests.rs
  - zircon_runtime/tests/runtime_volumetric_shading_contract.rs
  - zircon_runtime/tests/runtime_volumetric_temporal_wgpu_contract.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/default_pipelines.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/feature_descriptors.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/plugin_features.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/temporal_and_ops.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs
  - zircon_runtime/src/graphics/shader/template/tests.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/tests.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/wgpu_product_tests.rs
  - docs/tests/runtime/render/plan18_volumetric_media_inject_wgpu_20260711.png
  - docs/tests/runtime/render/plan18_volumetric_media_inject_wgpu_20260711.txt
  - docs/tests/runtime/render/plan18_volumetric_light_scatter_integrate_shadow_wgpu_20260711.png
  - docs/tests/runtime/render/plan18_volumetric_light_scatter_integrate_shadow_wgpu_20260711.txt
  - docs/tests/runtime/render/plan18_volumetric_temporal_reprojection_wgpu_20260711.png
  - docs/tests/runtime/render/plan18_volumetric_temporal_reprojection_wgpu_20260711.txt
  - docs/tests/runtime/render/plan18_volumetric_compiled_scene_window_light_shaft_perf_wgpu_20260711.png
  - docs/tests/runtime/render/plan18_volumetric_compiled_scene_window_light_shaft_perf_wgpu_20260711.txt
doc_type: module-detail
---

# Volumetric Froxel Pipeline

## Purpose

This module covers Render Plan 18 AF-M3 slices 1-3. It defines renderer-neutral volumetric-fog contracts, three scene-renderer WGPU compute owners, render-graph executors, persistent High-quality history, and the fixed forward/deferred/sky composition ABI.

The graph chain is `volumetric.media_inject -> volumetric.light_scatter -> volumetric.integrate`. It shares the existing LightGrid, ShadowAtlas, plan-06 camera jitter, frame-history lifecycle, and shader-quality contracts. The compiled-scene product test owns window geometry, shadowed directional lighting, temporal priming, upload/dispatch performance counters, a side-by-side PNG, and a spatial light-shaft acceptance gate. Historical WGPU products establish the visual baseline; the current AF-M3 closeout still requires a fresh product frame, RenderDoc capture/replay, post-fix review, and managed milestone commit.

## Ownership Boundaries

The framework owner contains no WGPU types. `VolumetricFogSettings`, `FogVolumeData`, `FroxelGridQuality`, and `FroxelGridParams` describe authored and extracted data that other renderer layers may consume. The existing `VolumeComponentRegistry` registers `lighting.volumetric-fog` for global volume settings. A non-global profile is extracted as bounded `FogVolumeData` and its volumetric override is removed from the ordinary post-process evaluator, preventing local density from also changing the whole view.

The scene-renderer owner contains the GPU ABI, WGPU pipelines, validation, dynamic dispatch sizing, history copy/binding, and WGSL. The three pipelines remain crate-private behind registered executors. The rendering plugin contributes only descriptors and executor registrations; it does not own WGPU implementation details or duplicate the scene renderer.

The persistent history owner is `SceneFrameHistoryTextures`. It allocates one `Rgba16Float` D3 texture only for High/Ultra quality when volumetric temporal is enabled, invalidates it when quality/dimensions change, binds it as `history.previous.volumetric.scattering`, and copies current scattering after graph execution.

## Contracts

`VolumetricFogSettings` contains global density, albedo, Henyey-Greenstein asymmetry `phase_g`, exponential height falloff, scattering intensity, depth distribution exponent, and temporal preference. Sanitization keeps density, albedo, falloff, and intensity non-negative; clamps `phase_g` to `[-0.9, 0.9]`; and keeps the depth exponent positive.

`FogVolumeData` represents an extracted world-space axis-aligned local medium. A scene entity contributes one local medium when it contains both `PostProcessVolumeComponent` and a finite, positive-extent `ColliderComponent`; extraction converts the collider to world bounds, applies the authored volume weight to local density, preserves stable identity and render-layer mask, and skips incomplete, non-finite, or degenerate entities. Ordinary post-process effects are selected with the camera `volume_mask`. Local-fog candidate extraction unions the selected base camera and its referenced overlay cameras' `culling_mask` values so stack-only media is retained; each camera's media executor then applies that camera's own final mask before upload.

Scene light assets and components expose a serde-default `volumetric` switch. `AdvancedLightingExtract` records the stable IDs of participating directional, point, spot, and rect lights after layer filtering. The GPU packer writes this state to `GpuLightData.cookie_misc.z` without moving the existing cookie projection/wrap fields; light-scatter WGSL returns zero for non-participating lights.

Authored ambient lights provide the view-independent radiance floor for participating media. The executor sums only finite, non-negative `color * intensity` contributions while preview lighting is enabled; a non-finite contribution or accumulator overflow is discarded instead of poisoning the froxel volume. `GpuLightScatterParams` preserves its 288-byte ABI by storing HG `phase_g` in `phase_and_ambient.x` and ambient radiance in `.yzw`. The shader initializes each froxel from that ambient radiance, then adds LightGrid-selected, shadowed direct lights with the HG phase term. Ambient radiance is therefore independent of directional shadow visibility while still being multiplied by the injected scattering coefficient.

The quality table is fixed by Plan 18:

| Quality | Froxel dimensions | Local volumes | Temporal capability |
|---|---:|---|---|
| Low | `160 x 90 x 48` | disabled | disabled |
| Medium | `160 x 90 x 64` | enabled | disabled |
| High | `160 x 90 x 96` | enabled | enabled |

`ShaderQualityTier` is part of `RenderPipelineCompileOptions`. Low maps to 48 slices, Medium to 64, and High/Ultra to 96. Runtime compilation injects the same quality value used by the compiled-pipeline cache key, so graph descriptors, dispatch dimensions, and executor pipeline settings cannot diverge.

Depth slices use the plan's exponential distribution:

```text
depth = near * (far / near) ^ (((slice + 0.5) / slice_count) ^ exponent)
```

The CPU contract clamps the requested slice to the valid range and sanitizes near/far values before evaluating the formula. It also owns the normalized Henyey-Greenstein phase function and the homogeneous-medium integration step used as closed-form references for WGSL tests.

## GPU Data Flow

1. The caller supplies sanitized settings, froxel dimensions, finite world bounds, local volume data, and the quality-derived local-volume switch.
2. The pipeline uploads one uniform block and a packed storage buffer of local boxes. An empty local list still binds one zeroed sentinel element while exposing a logical count of zero, satisfying WGPU's non-empty binding requirements without changing shader behavior.
3. A `4 x 4 x 4` compute workgroup writes one texel per froxel into a `Rgba16Float` three-dimensional storage texture.
4. The shader maps the froxel center into the supplied world bounds, evaluates exponential global height density, and adds every containing local box.
5. RGB stores the scattering coefficient (`albedo * density * scattering_intensity`); alpha stores extinction density.
6. Light scatter reads the media texture, existing `GpuLightData`, LightGrid z-bin/tile masks, and existing ShadowAtlas texture/sampler/slot/globals. It starts from the finite authored ambient-radiance floor, then iterates only participating lights selected by the grid, evaluates directional and punctual attenuation, applies the shared shadow visibility function, and multiplies direct incident radiance by the normalized HG phase.
7. Integrate dispatches one invocation per XY froxel column and walks Z from near to far. Every step uses `exp(-extinction * step_length)` and the analytic source integral `(1 - step_transmittance) / extinction`; every output slice stores cumulative radiance in RGB and cumulative transmittance in A of an `Rgba16Float` 3D texture.
8. `zr_volumetric_apply(color, uv, view_depth)` owns shading composition. It samples the integrated 3D texture through the fixed group1 bindings 26/27 and returns `color * transmittance + radiance`. Integrate never reads or writes scene color, so no post-process node is introduced.
9. The render graph derives media/scatter dispatch from the owned D3 resource (`FroxelGrid`) and integrate dispatch from its XY extent (`FroxelGridXy`). Resource alias planning reuses the media physical slot for integrated output after media lifetime ends.
10. High-quality scattering jitters XY from the camera temporal sample and Z from base-5 Halton. It reprojects the current world-space froxel center through the previous unjittered clip matrix, reconstructs the previous exponential slice, rejects out-of-volume coordinates and extinction discontinuities, then blends RGB with history while preserving current extinction.

## Feature-off Variants

`ShaderFeatureBits::VOLUMETRIC_FOG` and `PipelineKey::volumetric_fog` carry the optional feature through mesh draw construction and pipeline caching. Shader assembly selects the production volumetric include only when the bit is enabled; otherwise it emits a no-binding implementation whose apply function returns the original color. Forward, deferred, and sky pipelines all use this pass-specialized source, so disabling volumetric fog removes its bindings and sampling work rather than retaining a runtime branch.

The feature-off tests define assembly and Naga validation for both variants, assert that disabled sources contain no volumetric texture or sampler declaration, and cover forward, deferred, and sky consumers independently. The Shader IDE Standard-PBR stub intentionally excludes the pass-specialized volumetric dependency while retaining registered PBR and cookie dependencies. Current execution of these tests remains gated by the routed foreign Plugins05 exhaustive-match compile failure recorded in the AF-M3 closeout.

## Skybox Composition Ordering

The compiled scene order is `shadow/light-grid -> volumetric media/scatter/integrate -> opaque or deferred lighting -> sky -> transparent`. The sky pass belongs to `Transparent3d` but is declared before `transparent-mesh`. It reads `SCENE_DEPTH`, writes `SCENE_COLOR` with load/store attachment operations, and reads `VOLUMETRIC_INTEGRATED` when the optional volumetric plugin contributes that resource. It no longer clears or writes scene depth.

The full-screen sky triangle writes clip-space depth `1.0`; its pipeline uses `LessEqual` with depth writes disabled. It therefore fills only pixels that remain at far depth after opaque geometry. The fragment shader applies `zr_volumetric_apply(..., 1.0)`, so the sky receives the fully integrated view ray instead of the fallback texture that was previously observed when `preview-sky` ran in `DepthPrepass` before the volumetric compute chain.

Forward opaque rendering and deferred lighting declare the graph's first `SCENE_COLOR` write as clear/store so transient-resource validation has an explicit producer. `ViewportCameraStackAttachmentPolicy` converts that first write to load/store after the frame-level `scene_clear`, preserving the camera clear at execution time. Deferred lighting discards pixels without G-buffer geometry, preserving that clear until the later sky pass fills them. The obsolete deferred `FINAL_COLOR` background texture dependency and `sky.preview-final-color` executor are removed; `FINAL_COLOR` remains owned by the post-process/output path.

The pipeline follows Unreal's separation of common media injection, `LightScatteringCS`, and front-to-back integration. It reuses Zircon's own lighting and shadow contracts rather than copying Unreal's scene bindings. The local-volume data boundary also follows Bevy's separation between extracted fog volumes and volumetric-light participation.

## Validation and Failure Modes

Pipeline encoding rejects non-finite world bounds and bounds without positive extent. Settings, grid dimensions, and local media are sanitized before upload. Dispatch dimensions use integer ceiling division, so non-multiple extents cannot leave trailing froxels unwritten. Shader bounds checks protect over-dispatched invocations.

The focused CPU tests cover the exponential slice formula, exact quality table, shared Volume evaluator registration, HG isotropic/directional behavior, homogeneous integration closed form, temporal jitter values, graph ordering, feature-off graph identity, dynamic dispatch, resource aliasing, and all four shader quality tiers. Naga parse/validation is mandatory for assembled forward, deferred, sky, light-scatter, and integration WGSL.

The media WGPU test allocates a real `Rgba16Float` 3D texture, executes the production compute pipeline, reads it back, and verifies global-only versus global-plus-local media. The full-chain WGPU test then executes media inject, one LightGrid-selected directional light, a reversed-Z `Depth32Float` ShadowAtlas slot, HG scatter, and integration. Its synthetic shadow projection covers the left half and leaves the right half outside the slot, proving that shadow visibility changes the integrated output.

The ambient-radiance CPU contract covers enabled/disabled preview lighting, negative component clamping, non-finite contributions, accumulator overflow, phase/ambient packing, and the unchanged 288-byte upload ABI. Managed job `87b0728d17194711a2de95b4179455be` completed with exit code 0 and ran the exact `render_volumetric_light_scatter` filter: 8 passed, 0 failed, and 8,169 filtered out. The broader Runtime volumetric filter and plugin/product stages remain separate acceptance gates.

On Windows, the ignored RenderDoc product test uses the injected RenderDoc 1.0 API to set an artifact path, explicitly bracket two offscreen submissions with `StartFrameCapture` and `EndFrameCapture`, and wait for the registered `.rdc` file. A layout test locks the function-pointer offsets used by this test-only FFI owner. This explicit boundary is required because the headless WGPU product has no swapchain `Present` event for `renderdoccmd` to use as an implicit frame boundary.

Historical products remain useful regression baselines, but they are not the current closeout evidence. The previous compiled-scene frame recorded three volumetric dispatches, `44,400` dispatch groups, `624` uploaded bytes, `8,475` brighter pixels, and corridor-over-control contrast `73.506`; its PNG SHA-256 is `2AF87DABF39EA5485F579D6300615E6A5DA380A166C5AFF28DF84A03EBACDB0D`, and the report SHA-256 is `9AC7B7B29BA80A6F4311E71FE1732479B46B571075E42D009622D546EFAA969C`.

Prior focused Runtime validation passed 78 advanced-lighting tests with 8 ignored, the Shader IDE dependency test passed 1/1, and a real-WGPU seven-binding shadow scene-group regression passed 1/1 in managed job `4034b57165444f9298cfafe5065a7cc0`. A prior plugin binary passed 8 non-ignored tests with one ignored exporter. These results predate the final review remediation and therefore must be rerun.

Current managed Runtime rerun job `f9e4addefebd4b9f9ef6915d9e51cff8` stopped before Render18 tests on a foreign exhaustive-match error: `UiBindingExpression::ControlPropRef` was added without a corresponding arm in `zircon_runtime/src/ui/template/asset/binding/validation.rs`. The coordinator routed that gate to `plugins05-control-prop-binding-ref-20260715`; Render18 owns neither path and does not claim the current testing stage passed.

Managed graphics-only job `b34eeebc14034295b0bd2ece23c0624f` compiled the production `zircon_runtime` library but then stopped before Render18 unit execution on active Frameworks05/Text03 test drift. That partial compile establishes source-level compatibility for the production graphics library only; it is not a substitute for the pending unit, plugin, WGPU product, or RenderDoc gates.

Final independent re-review closed the source remediation at 0 Critical, 0 Important, and 0 Minor. Product acceptance remains open: exact managed DX12 jobs `c49eaa41d7324593b92ff24e92ddfa9a` and `aefb18e771394b42adce1ae9fa0cbc7c` each ran one ignored exporter and failed its spatial shaft gate. The first run confirmed the full dispatch/LightGrid/shadow counter path but produced no brighter samples in the shaft corridor. The second run disproved a comparison-sampler hypothesis, which was fully reverted. The current PNG/report are diagnostic failures; RenderDoc inspection of the light buffer, shadow atlas, froxel scattering, and integrated volume is required before a fresh artifact may be accepted.

## Follow-up

AF-M3 source implementation now includes media injection, LightGrid/ShadowAtlas scatter, 3D integration/apply, temporal reprojection, graph ordering, quality tiers, feature-off identity, local/global volume separation, camera-stack layer union, bounds validation, and viewport-local transmission UV. Independent review is closed at 0 Critical/0 Important/0 Minor. Acceptance remains open until the Render18-owned spatial product regression is fixed, the routed foreign compile gates land, current managed Runtime/plugin tests pass, a fresh passing WGPU product frame and RenderDoc replay are recorded, and the coordinator creates the M3 milestone commit. Plan 18 remains active after M3 for AF-M4 through AF-M6.
