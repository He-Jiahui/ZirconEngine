---
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/mod.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/volumetric.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/volumetric/tests.rs
  - zircon_runtime/src/core/framework/render/post_process/resolved_stack.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component/params.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_registry.rs
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
implementation_files:
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
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/volumetric_history.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_volumetric.wgsl
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

The graph chain is `volumetric.media_inject -> volumetric.light_scatter -> volumetric.integrate`. It shares the existing LightGrid, ShadowAtlas, plan-06 camera jitter, frame-history lifecycle, and shader-quality contracts. The compiled-scene product test owns window geometry, shadowed directional lighting, temporal priming, upload/dispatch performance counters, a side-by-side PNG, and a spatial light-shaft acceptance gate. Current-source WGPU execution passes that gate, so AF-M3 slices 1-3 are accepted; broader Plan 18 milestones remain active.

## Ownership Boundaries

The framework owner contains no WGPU types. `VolumetricFogSettings`, `FogVolumeData`, `FroxelGridQuality`, and `FroxelGridParams` describe authored and extracted data that other renderer layers may consume. The existing `VolumeComponentRegistry` registers `lighting.volumetric-fog`, so camera/volume blending uses the same evaluator and parameter interpolation path as post-process settings.

The scene-renderer owner contains the GPU ABI, WGPU pipelines, validation, dynamic dispatch sizing, history copy/binding, and WGSL. The three pipelines remain crate-private behind registered executors. The rendering plugin contributes only descriptors and executor registrations; it does not own WGPU implementation details or duplicate the scene renderer.

The persistent history owner is `SceneFrameHistoryTextures`. It allocates one `Rgba16Float` D3 texture only for High/Ultra quality when volumetric temporal is enabled, invalidates it when quality/dimensions change, binds it as `history.previous.volumetric.scattering`, and copies current scattering after graph execution.

## Contracts

`VolumetricFogSettings` contains global density, albedo, Henyey-Greenstein asymmetry `phase_g`, exponential height falloff, scattering intensity, depth distribution exponent, and temporal preference. Sanitization keeps density, albedo, falloff, and intensity non-negative; clamps `phase_g` to `[-0.9, 0.9]`; and keeps the depth exponent positive.

`FogVolumeData` represents an extracted world-space axis-aligned local medium. It carries stable volume identity, bounds, density, albedo, and a render-layer mask. Media injection currently consumes bounds, density, and albedo. Layer filtering belongs to the extraction/graph integration slice, where the active view mask is available.

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
6. Light scatter reads the media texture, existing `GpuLightData`, LightGrid z-bin/tile masks, and existing ShadowAtlas texture/sampler/slot/globals. It iterates only lights selected by the grid, evaluates directional and punctual attenuation, applies the shared shadow visibility function, and multiplies incident radiance by the normalized HG phase.
7. Integrate dispatches one invocation per XY froxel column and walks Z from near to far. Every step uses `exp(-extinction * step_length)` and the analytic source integral `(1 - step_transmittance) / extinction`; every output slice stores cumulative radiance in RGB and cumulative transmittance in A of an `Rgba16Float` 3D texture.
8. `zr_volumetric_apply(color, uv, view_depth)` owns shading composition. It samples the integrated 3D texture through the fixed group1 bindings 26/27 and returns `color * transmittance + radiance`. Integrate never reads or writes scene color, so no post-process node is introduced.
9. The render graph derives media/scatter dispatch from the owned D3 resource (`FroxelGrid`) and integrate dispatch from its XY extent (`FroxelGridXy`). Resource alias planning reuses the media physical slot for integrated output after media lifetime ends.
10. High-quality scattering jitters XY from the camera temporal sample and Z from base-5 Halton. It reprojects the current world-space froxel center through the previous unjittered clip matrix, reconstructs the previous exponential slice, rejects out-of-volume coordinates and extinction discontinuities, then blends RGB with history while preserving current extinction.

## Skybox Composition Ordering

The compiled scene order is `shadow/light-grid -> volumetric media/scatter/integrate -> opaque or deferred lighting -> sky -> transparent`. The sky pass belongs to `Transparent3d` but is declared before `transparent-mesh`. It reads `SCENE_DEPTH`, writes `SCENE_COLOR` with load/store attachment operations, and reads `VOLUMETRIC_INTEGRATED` when the optional volumetric plugin contributes that resource. It no longer clears or writes scene depth.

The full-screen sky triangle writes clip-space depth `1.0`; its pipeline uses `LessEqual` with depth writes disabled. It therefore fills only pixels that remain at far depth after opaque geometry. The fragment shader applies `zr_volumetric_apply(..., 1.0)`, so the sky receives the fully integrated view ray instead of the fallback texture that was previously observed when `preview-sky` ran in `DepthPrepass` before the volumetric compute chain.

Forward opaque rendering and deferred lighting declare the graph's first `SCENE_COLOR` write as clear/store so transient-resource validation has an explicit producer. `ViewportCameraStackAttachmentPolicy` converts that first write to load/store after the frame-level `scene_clear`, preserving the camera clear at execution time. Deferred lighting discards pixels without G-buffer geometry, preserving that clear until the later sky pass fills them. The obsolete deferred `FINAL_COLOR` background texture dependency and `sky.preview-final-color` executor are removed; `FINAL_COLOR` remains owned by the post-process/output path.

The pipeline follows Unreal's separation of common media injection, `LightScatteringCS`, and front-to-back integration. It reuses Zircon's own lighting and shadow contracts rather than copying Unreal's scene bindings. The local-volume data boundary also follows Bevy's separation between extracted fog volumes and volumetric-light participation.

## Validation and Failure Modes

Pipeline encoding rejects non-finite world bounds and bounds without positive extent. Settings, grid dimensions, and local media are sanitized before upload. Dispatch dimensions use integer ceiling division, so non-multiple extents cannot leave trailing froxels unwritten. Shader bounds checks protect over-dispatched invocations.

The focused CPU tests cover the exponential slice formula, exact quality table, shared Volume evaluator registration, HG isotropic/directional behavior, homogeneous integration closed form, temporal jitter values, graph ordering, feature-off graph identity, dynamic dispatch, resource aliasing, and all four shader quality tiers. Naga parse/validation is mandatory for assembled forward, deferred, sky, light-scatter, and integration WGSL.

The media WGPU test allocates a real `Rgba16Float` 3D texture, executes the production compute pipeline, reads it back, and verifies global-only versus global-plus-local media. The full-chain WGPU test then executes media inject, one LightGrid-selected directional light, a reversed-Z `Depth32Float` ShadowAtlas slot, HG scatter, and integration. Its synthetic shadow projection covers the left half and leaves the right half outside the slot, proving that shadow visibility changes the integrated output.

The accepted artifact reports average extinction `0.032955` in the global-only half and `0.152908` in the half containing the local box. Its PNG SHA-256 is `D40B09EFA5423E2C0C458D4E821139D459BA29B38C04B3DC191C8F30332629FA`.

The light-shaft artifact reports left shadowed average RGB `0.036987, 0.043182, 0.061676` and right unshadowed average RGB `1.393555, 0.844238, 0.232788`. It executes a test shading consumer through `zr_volumetric_apply` after the production 3D integration pass. Its PNG SHA-256 is `D836AB5300F066E2BAFA9CB03E61375CC2AC62107BB8FADB8599B968AA59D681`; the report SHA-256 is `E2298952B0561FAB9E4AB58EB30520FFAFE9D41EE03AF7731AC2DCBFF6489A37`.

The temporal WGPU contract assembles and executes the production LightScatter WGSL with full group-0/group-1 ABI. Its three panels are temporal off, matched-history accumulation, and extinction-change rejection. All 1024 froxels accumulate history, average RGB distance to history falls from `1.375000` to `0.137634`, and rejection differs from the current frame by `0.000000`. The 770x128 PNG SHA-256 is `0666A56364FD645A768A5E37A8CBED790F4E645C23A5B5795DB8409F811852AA`; report SHA-256 is `1FB2C52EC7E01EC49ACAED85893012A6F3679372C91F60B95B1FB7EC1ED47540`.

The sky-order contract first failed against the old compiled graph with `preview-sky` before `volumetric.integrate`. After the hard cut, the current production include order for sky and deferred WGSL passes independent Naga parsing and full validation 2/2. The first current-source graph test then exposed an owned first-writer RED: `opaque-mesh` loaded transient `scene-color` before a graph producer. That contract is corrected as described above and covered by forward/deferred attachment assertions.

The first shadowed compiled-scene rerun still produced extinction only even though the unshadowed diagnostic proved LightGrid, HG scatter, integrate, and apply. The shared shadow report showed four directional cascades and four caster draws, which isolated the failure to atlas command recording. `ShadowMapRenderer` previously queued four `queue.write_buffer` calls into one scene-uniform buffer while recording all cascade passes into one command encoder. WGPU executes those copies before the render passes at submission, so every pass observed the last cascade matrix. The renderer now creates an immutable initialized uniform buffer and matching scene bind group per slot. This preserves each cascade's view-projection matrix until submission and fixes both surface and volumetric consumers without changing the comparison-sampler contract.

The compiled-scene product gate no longer accepts full-frame change alone. It measures a normalized trapezoidal window-light corridor and two side shadow-control bands. Acceptance requires more than 20% of the corridor to brighten, positive average corridor radiance, and at least `1.5` luma units of corridor-over-control contrast in addition to the existing changed-pixel, chromaticity, and aggregate RGB-delta thresholds. Synthetic-frame tests prove that a concentrated shaft passes while a uniform fog tint fails. The report records the corridor/control sample counts, average luma deltas, and spatial contrast so the PNG result remains auditable.

The accepted current-source compiled-scene product executes three volumetric dispatches, `44,400` dispatch groups, and `624` uploaded bytes. It records one LightGrid light, `524,160` non-empty clusters, one shadow-atlas write, four caster draws, and one ready directional light. The shadowed volumetric frame has `8,475` brighter pixels; `4,241 / 4,398` corridor samples brighten, corridor average luma delta is `76.264`, side-control delta is `2.758`, and spatial contrast is `73.506`. The 385x128 PNG SHA-256 is `2AF87DABF39EA5485F579D6300615E6A5DA380A166C5AFF28DF84A03EBACDB0D`; report SHA-256 is `9AC7B7B29BA80A6F4311E71FE1732479B46B571075E42D009622D546EFAA969C`.

Current-source `cargo build -p zircon_runtime --lib --locked --offline --jobs 1` passes. The current volumetric plugin test binary passes 8/8 non-ignored tests with one ignored exporter, and the ignored compiled-scene WGPU exporter passes 1/1. Plugin structure audit and locked/offline metadata also pass. A separate runtime lib-test link attempt reached the linker but failed with `LNK1180` because the shared E drive lacked space; this does not invalidate the successful runtime library build or product binary, but broad full-workspace acceptance remains outside AF-M3.

## Follow-up

AF-M3 slices 1-3 are complete: media injection, LightGrid/ShadowAtlas scatter, 3D integration/apply, temporal reprojection, graph ordering, quality tiers, feature-off identity, performance counters, and the current-source compiled-scene spatial light-shaft product are accepted. Continue with AF-M4 OIT, planar reflections, and SSS, then broad runtime/full-workspace and RenderDoc acceptance. Plan 18 therefore remains active.
