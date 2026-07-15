---
related_code:
  - zircon_runtime/src/core/framework/render/advanced_lighting/irradiance_volume.rs
  - zircon_runtime/src/core/framework/render/light/gpu_light.rs
  - zircon_runtime/src/graphics/feature/render_feature_descriptor
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_advanced_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/irradiance_volume
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/light_cookie
  - zircon_runtime/src/graphics/shader/wgsl/zr_irradiance_volume.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_light_cookie.wgsl
  - zircon_plugins/rendering/features/irradiance_volumes
  - zircon_plugins/rendering/features/light_cookies
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/light_cookie/frame_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/light_cookie/projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/light_cookie/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/light_cookie/blit_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/light_cookie/executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/irradiance_volume/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/irradiance_volume/executor.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_advanced_lighting.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/shader/ide_env_generation.rs
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_light_cookie.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_irradiance_volume.wgsl
plan_sources:
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
tests:
  - render_cookie_atlas_plan_is_sorted_deduplicated_and_fixed_cell
  - render_cookie_uv_three_projections_match_reference
  - render_cookie_gpu_light_data_extension_offsets
  - render_cookie_metadata_aligns_with_packed_light_ids
  - render_irrvol_world_to_uvw_roundtrip
  - render_irrvol_selection_prefers_priority_inside
  - render_irrvol_view_selection_ignores_unrelated_higher_priority_volume
  - render_irrvol_gpu_normal_matrix_handles_rotation_and_nonuniform_scale
  - shader_ide_standard_pbr_stub_resolves_transitive_advanced_lighting_dependencies
  - render_product_af_m2_feature_off_matches_graph_baseline_exactly
  - render_product_af_m2_frame_without_volume_clears_previous_volume_state
  - render_product_af_m2_cookie_and_volume_execute_and_change_wgpu_frame
  - export_render_product_af_m2_light_cookie_irradiance_volume_png
doc_type: module-detail
---

# Light Cookies And Irradiance Volumes

This module is the AF-M2 advanced-lighting slice. It adds optional light-cookie
projection and one selected local irradiance volume without changing frames
that do not enable either feature.

## Light Cookie Flow

The frame plan sorts cookie users by the stable `light_id`, removes duplicate
IDs, and assigns at most 64 cells in a fixed 8 by 8 atlas. The atlas is a
persistent 1024 by 1024 `Rgba8Unorm` texture. Every rebuild first restores
white, so a missing or disabled cookie is an exact multiplicative identity.
Loaded source textures are copied through the GPU fullscreen blit pipeline.

`GpuLightData` carries an appended 32-byte cookie tail. Existing fields retain
their offsets, while the total storage stride becomes 128 bytes. The tail
contains the atlas UV rectangle plus projection, wrapping, slot, and active
metadata. Directional, spot, and point lights use planar, perspective, and
octahedral projection respectively. Both forward Standard PBR and deferred
lighting multiply direct-light radiance by the sampled cookie.

## Irradiance Volume Flow

CPU view selection first rejects volumes outside the camera render layers and
then requires a candidate to contain at least one visible mesh world position.
This prevents an unrelated high-priority volume from displacing the local
volume that can actually shade the view. Remaining candidates are ordered by
descending priority and then ascending stable volume ID. Per-pixel WGSL
containment determines whether the selected volume affects a surface, so a
camera outside the volume does not suppress lighting for visible geometry
inside it. The selected world-to-volume transform, inverse-transpose normal
matrix, and intensity are uploaded through the volume uniform.

Volume textures reuse the resource streamer's validated RGBA8 3D upload path.
The physical texture dimensions are `(R.x, 2 * R.y, 3 * R.z)`, which stores the
six ambient-cube lobes as positive and negative X, Y, and Z regions. WGSL
clamps sampling to half-texel interiors, selects lobes from the normal sign,
and combines them with squared normal components.

The indirect-light priority is local irradiance volume, baked lightmap, then
the global irradiance-probe grid. A black 3D fallback and a disabled uniform
make an absent volume neutral.

## Bindings And Feature Activation

The current material group reserves bindings `33` and `34` for the cookie
atlas and sampler. Bindings `35`, `36`, and `37` hold the irradiance-volume
texture, sampler, and uniform. This allocation follows the already-landed
Plan 18 bindings through transmission at `31` and `32`; it supersedes the
older draft table that assumed those slots were free.

The `light_cookies` and `irradiance_volumes` rendering features are optional
and default off. Their graph descriptors and executors are retained only when
the matching frame extract contains data. Fixed white/black fallback bindings
remain part of the stable material layout, while feature-off graph shape and
rendered output remain unchanged. Every frame resets the selected volume before
draw bind groups are built, preventing a removed volume from leaking into the
next frame.

The compiled-graph cache fingerprint includes cookie descriptors and
irradiance-volume presence, so changing either feature cannot reuse an
incompatible graph. Forward and deferred shader assembly both consume the
single `ShaderModuleRegistry` dependency graph. Shader IDE stub validation
walks that graph recursively, including the lightmap-to-volume and Standard
PBR-to-PBR-extras-to-volumetric chains, before Naga parses each standalone
stub. The traversal keeps separate active and completed sets, so a circular
include reports its complete import chain instead of being mistaken for an
already-resolved duplicate.

## Current Verification State

The managed Windows Runtime build and the focused cookie/irradiance CPU suites
pass. The DX12 WGPU product passes both the controlled visible-delta test and
the cross-frame stale-volume reset test with WGPU debug and validation enabled.
The exported 1286 by 360 PNG has SHA-256
`340DF40E59E71956E8FC6BAF9B186B659196479EF350650B7CE788A02B1A200A`.
The 12,085,874-byte RenderDoc capture has SHA-256
`F5A85DCCEAA109CF6C5E7C72F5A920BD5B843FF3C98A1DCB9B4BE43B4B19F26C`
and replays once with exit code 0.
