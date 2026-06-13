---
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/shadow_settings.rs
  - zircon_runtime/src/core/framework/render/light/gpu_light.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/bindings.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/shadow_map.wgsl
  - zircon_runtime/src/graphics/visibility/view_context/mod.rs
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/post_process_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_runtime_feature_flags/scene_runtime_feature_flags.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/contact_shadow.wgsl
  - zircon_plugins/rendering/features/contact_shadow/editor/src/lib.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_bind_group_layout/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_pbr_wgsl.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/shadow_settings.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/bindings.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl
  - zircon_runtime/src/graphics/visibility/view_context/mod.rs
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/post_process_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_runtime_feature_flags/scene_runtime_feature_flags.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/contact_shadow.wgsl
  - zircon_plugins/rendering/features/contact_shadow/editor/src/lib.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_bind_group_layout/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_pbr_wgsl.rs
plan_sources:
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/AdditionalLightsShadowAtlasLayout.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ShadowUtils.cs
  - dev/bevy/crates/bevy_light/src/cascade.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shadow/csm.rs
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs::tests::render_shadow_atlas_allocates_tiers_descending
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs::tests::render_shadow_atlas_global_downgrade_fits_pressure
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs::tests::render_shadow_atlas_evicts_lowest_priority_on_pressure
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs::tests::render_shadow_atlas_hysteresis_prevents_flapping
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs::tests::render_shadow_atlas_preempts_after_confirmed_priority_margin
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs::tests::render_shadow_atlas_scale_bias_matches_slice_transform
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs::tests::render_shadow_cascade_splits_blend_log_linear
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs::tests::render_shadow_cascade_ranges_are_monotonic_and_have_fade_bands
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs::tests::render_shadow_cascade_snapping_quantizes_origin
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs::tests::render_shadow_cascade_view_projection_is_stable_under_half_texel_motion
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs::tests::render_shadow_cascade_bounds_follow_camera_slice_depth
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs::tests::render_shadow_slot_layout_matches_plan_05_std430_contract
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs::tests::render_shadow_slot_disabled_has_no_valid_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs::tests::render_shadow_slot_from_allocation_writes_atlas_slice_and_flags
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs::tests::render_shadow_slot_encodes_pcf_quality_in_flags
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/slot.rs::tests::render_shadow_globals_layout_and_atlas_params_are_stable
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs::tests::render_shadow_atlas_resource_config_normalizes_zero_values
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs::tests::render_shadow_atlas_resource_config_uses_plan_05_defaults
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs::tests::render_shadow_atlas_resource_config_downgrades_to_capability_limit
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs::tests::render_shadow_atlas_upload_report_describes_cleared_tail
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/bindings.rs::tests::render_shadow_atlas_group1_bindings_avoid_legacy_shadow_and_light_grid_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/bindings.rs::tests::render_shadow_atlas_group1_layout_entries_match_plan_05_resource_types
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_receives_shadow_atlas_resources
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs::deferred_lighting_shader_receives_shadow_atlas_resources
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs::deferred_lighting_shader_is_valid_wgsl
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_pbr_wgsl.rs::tests::builtin_pbr_shader_receives_shadow_atlas_resources
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs::tests::render_shadow_frame_plan_assigns_first_directional_cascade_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs::tests::render_shadow_frame_plan_caps_directional_cascade_tier_to_atlas_row
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs::tests::render_shadow_frame_plan_builds_distinct_directional_cascade_matrices
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs::tests::render_shadow_frame_plan_assigns_point_light_contiguous_face_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs::tests::render_shadow_frame_plan_assigns_spot_light_slot_view_key
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs::tests::render_shadow_frame_plan_encodes_per_light_pcf_quality
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs::tests::contact_shadow_feature_registers_hzb_ray_march_pass
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs::tests::contact_shadow_graph_pass_is_absent_when_plugin_feature_is_disabled
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs::tests::contact_shadow_executor_accepts_declared_pass_contract
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs::tests::contact_shadow_executor_requires_gpu_after_contract_validation
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs::tests::contact_shadow_executor_rejects_resource_contract_drift
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs::tests::contact_shadow_shader_declares_expected_compute_bindings
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs::tests::pluginized_rendering_feature_names_drive_runtime_post_process_flags
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs::tests::contact_shadow_runtime_flag_is_encoded_separately_from_ssao
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs::tests::post_process_shader_samples_bound_contact_shadow_occlusion_texture
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs::tests::render_shadow_light_slot_assignments_patch_packed_light_contract
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs::tests::shadow_atlas_view_filter_keeps_only_visible_source_entities
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs::tests::visibility_context_builds_shadow_views_for_atlas_light_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs::tests::mesh_visibility_states_preserve_shadow_only_casters
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_batch_ref_emits_gpu_scene_instance_command
  - zircon_runtime/src/graphics/tests/render_product_shadows.rs::render_product_csm_directional
  - zircon_runtime/src/graphics/tests/render_product_shadows.rs::render_product_multi_spot_shadows
doc_type: module-detail
---

# Scene Renderer Shadow Support

This module is the Plan 05 LS-M3/LS-M4 shadow foundation. The compiled graph declares a `shadow-atlas` pass using the `shadow.atlas` executor and writes the persistent `shadow-atlas` depth resource. The atlas, cascade, slot ABI, WGPU resource owner, group1 bindings, multi-light sampling WGSL, slot-depth replay path, and per-light PCF quality contract are in place; the legacy single `SHADOW_MAP` receiver/resource has been removed.

## Current Boundary

- `shadow_map_renderer.rs` owns the depth-only/alpha-mask caster replay path and records atlas slot depth passes by updating the scene uniform per planned slot, setting atlas viewport/scissor to the slot rect, and replaying the shadow command stream into the shared atlas view. Atlas slot passes carry Plan 04 `VisibilityViewKey` values for directional cascades, point faces, and spot shadows; when a matching view exists on the frame, replay skips shadow commands whose source entity is not visible in that shadow view. The old single-map receiver uniform and direct single-map recording path have been deleted.
- `shadow/atlas/allocator.rs` owns frame-local atlas slot planning only. It does not create WGPU textures or record shadow passes yet.
- `shadow/atlas/bindings.rs` fixes the final group1 atlas binding ABI as 8/9/10/11. Forward and deferred lighting bind groups now include only the atlas receiver entries plus light-grid buffers; the old single-shadow receiver bindings are gone.
- `shadow/atlas/resources.rs` owns the persistent WGPU atlas texture/view, comparison sampler, `shadow_slots` storage buffer, and `shadow_globals` uniform buffer. `SceneRendererCore` creates it and uploads the current `ShadowFramePlan` payload each render.
- `shadow/cascade.rs` owns CSM split/fade/snapping math and camera frustum slice bounds. The slice-bounds helper is crate-reexported through `graphics::scene` so `ShadowFramePlan` and Plan 04 visibility build the same directional cascade coverage without exposing the private shadow module tree.
- `shadow/plan.rs` bridges the full viewport frame to atlas allocation, `GpuShadowSlot`/`GpuShadowGlobals` upload payloads, `ShadowAtlasSlotPass` depth-write descriptors, and `GpuLightData.shadow_slot_layer` patching. It derives directional cascade, spot, and point-face view-projection matrices, and tags atlas slot passes with `VisibilityViewKey::ShadowCascade`, `VisibilityViewKey::ShadowPointFace`, or `VisibilityViewKey::ShadowSpot`.
- `shadow/slot.rs` owns the GPU POD layout for shadow slots/globals. Buffer ownership exists in `ShadowAtlasResources`, and the forward/deferred group1 bindings now expose those buffers to fragment shaders.
- `shadow/shaders/zr_shadow.wgsl` owns the shader-side atlas sampling helper. It reads `GpuLightData.shadow_slot_layer`, chooses directional cascades or point faces, projects through `ZrShadowSlot.view_proj`, and selects Low/Medium/High comparison-sampler PCF kernels from the slot quality flags.
- `PostProcessGraphResourceNames::SHADOW_ATLAS` names the graph-visible external atlas resource. The built-in `shadow-atlas` pass writes it, and forward mesh/deferred lighting/deferred transparent mesh declare reads so graph ordering keeps atlas depth production before atlas sampling. `PostProcessGraphResourceNames::SHADOW_MAP` is no longer part of the runtime graph contract.
- `PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION` names the optional screen-space contact shadow output. The `rendering.contact_shadow` plugin owns the HZB-driven `contact-shadow` pass and its minimal WGPU compute executor/shader, so atlas shadows remain built-in while short-distance screen-space shadowing stays opt-in. The executor reads `scene-depth`, `gbuffer-normal`, and `hzb-furthest`, writes the Rgba8Unorm visibility output, and records its compute dispatch through the public plugin-facing GPU context method. The built-in `post.stack` pass declares a read from that texture and the post-process shader samples binding 27 under `contact_shadow_enabled`; feature-off or missing-resource paths bind a white fallback, so no visual multiplier is applied.
- `core/framework/render/light/shadow_settings.rs` remains the framework-facing authoring contract for `casts_shadow`, bias, strength, resolution preference, and `ShadowPcfQuality`.

## Atlas Allocator

`ShadowAtlasConfig::default()` follows Plan 05: a 4096x4096 atlas with the top 1024 pixels reserved for directional CSM. Tests use smaller square atlases through `new_square`.

Allocation input is `ShadowSlotRequest`:

- `ShadowSlotKey { light_id, face_index }` identifies one shadow slice. A point light will use six keys in the later GPU pass slice.
- `requested_tier` and `minimum_tier` use `ShadowResolutionTier` from the framework contract.
- `priority` is a finite non-negative score used after global downgrade.

The allocator first applies the URP-style total-area estimate: if the requested area exceeds the available atlas area, all requests are downshifted by powers of two before packing, while respecting each request's minimum tier. Packing then uses a deterministic free-rectangle shelf variant ordered by y/x, so allocations are stable and non-overlapping.

Hysteresis is local to `ShadowAtlasAllocator`:

- A previous allocation is reused for the same key while the request stays present, the tier is unchanged, and the rect remains free.
- Slot retention uses the Plan 05 constants: 8 frames retention, 4 consecutive contention frames, and a 25% priority margin before confirmed preemption.
- The frame result records both `allocations` and `rejected` slots so later graph diagnostics can expose downgrade/pressure behavior.

`ShadowSlotAllocation::atlas_scale_bias()` mirrors URP `ApplySliceTransform`: `[scale_x, scale_y, offset_x, offset_y]` in normalized atlas UV space.

## WGPU Atlas Resources

`bindings.rs` reserves the final group1 range for atlas sampling without conflicting with light-grid bindings:

| binding | resource |
|---------|----------|
| 8 | `shadow_atlas` depth texture |
| 9 | comparison sampler |
| 10 | `shadow_slots` read-only storage buffer |
| 11 | `shadow_globals` uniform buffer |

These constants are code-owned and active in the forward/deferred group1 layouts. Bindings 0/1/2 in the forward shadow receiver group and 5/6/7 in deferred lighting are no longer declared by the atlas receiver layouts.

`ShadowAtlasResources` is the persistent resource owner for the graph-declared `shadow-atlas` path. It creates:

- A depth atlas texture using the renderer depth format, with render-attachment and texture-binding usages.
- A default atlas view and comparison sampler.
- A storage buffer sized as `slot_capacity * GPU_SHADOW_SLOT_STRIDE`.
- A uniform buffer initialized with disabled `GpuShadowGlobals`.

`ShadowAtlasResourceConfig::default()` follows Plan 05: 4096x4096 and 256 shadow slots. Construction clamps the atlas to device capability and falls back to 2048x2048 when a device cannot host the requested 4096 dimension. This keeps LS-M3 compatible with lower-limit WGPU adapters while preserving the default design for capable devices.

`upload_frame()` writes packed `GpuShadowSlot` data and `GpuShadowGlobals`. When a later frame uploads fewer slots than the previous frame, the stale tail is explicitly overwritten with disabled slots so shader-visible storage cannot retain old valid flags. `SceneRendererCore::render_compiled_scene()` and the legacy `render_scene()` path build a `ShadowFramePlan`, upload its slots/globals, pass its light-slot assignment table into GPUScene light packing, and make the uploaded atlas resources available to forward/deferred graph execution. The compiled graph import step binds `SHADOW_ATLAS` to this persistent atlas view so the shadow executor can write it as an external graph resource.

## Slot ABI

`GpuShadowSlot` is the CPU-side `ZrShadowSlot` ABI from Plan 05:

- `view_proj` at offset 0, 64 bytes.
- `atlas_scale_bias` at offset 64.
- `params` at offset 80, with `x = depth_bias`, `y = normal_bias`, `z = slot_texel_size`, and `w = bitcast flags`.

`GpuShadowSlot::from_allocation()` converts a `ShadowSlotAllocation` plus a light-space matrix into the atlas slice transform and sets the valid flag. Slot flags distinguish directional cascades, spot slices, and point-light cube faces; bits 8..9 encode `ShadowPcfQuality` as Low=1 tap, Medium=5 tap, and High=9 tap. `ShadowPcfQuality::default()` is Low, while tests/product contracts that preserve the older fixed 3x3 behavior set High explicitly.

`GpuShadowGlobals` is a 48-byte uniform block for cascade far splits, fade lengths, and atlas size/inverse-size. The buffer exists in `ShadowAtlasResources`, and the atlas binding ABI is fixed at group1 bindings 8/9/10/11. Forward/deferred shaders include `zr_shadow.wgsl` and assert the legacy single-shadow receiver symbols are absent.

## Cascades

`CascadeSplitConfig` supports up to `MAX_SHADOW_CASCADES == 4` and computes Plan 05's log/linear blended split distances:

```text
split_i = lerp(linear_i, logarithmic_i, lambda)
```

`compute_cascade_ranges()` turns the split array into per-cascade near/far/fade bands. `cascade_shadow_bounds_from_camera_slice()` derives the eight world-space frustum corners for one camera slice, averages them into a cascade center, and stores a stabilized radius for downstream orthographic coverage. `snap_light_space_center_to_texel()` and `snapped_cascade_view_projection()` provide the texel-grid stabilization used by the directional CSM view construction. The snapping follows the Bevy/UE pattern of quantizing light-space x/y to world-units-per-texel before building the orthographic matrix.

## Frame Plan And Light Writeback

`build_shadow_frame_plan()` is the current LS-M3 bridge between authored lights and GPU light-buffer writeback:

- The first shadow-casting directional light reserves contiguous cascade slots at the start of the shadow slot table, writes `GpuShadowGlobals` split/fade data, and builds texel-snapped orthographic matrices around the current camera frustum slice bounds.
- Point lights request six atlas allocations and only receive a light assignment when all six faces fit; each face writes a 90-degree perspective matrix for the corresponding cube direction.
- Spot lights request one atlas allocation and write a perspective matrix from authored position, direction, cone angle, and range.
- Each slot copies the source light's `pcf_quality` into the GPU flags, so point and spot lights can choose different PCF kernels in the same frame.
- Every emitted GPU slot also emits a `ShadowAtlasSlotPass { slot_index, rect, view_proj, view_key }`. The compiled graph shadow stage passes the frame plan into `RenderPassGpuExecutionContext`; `ShadowMapRenderer::record_atlas_commands_with_attachment_ops()` then writes each slot by setting viewport/scissor to `rect` and replaying the shadow command stream with that slot's `view_proj`. Directional, point-face, and spot slots filter replay through the matching Plan 04 shadow view when present. The filtering remains command-by-command for view-filtered slots so the unfiltered shadow indirect args stream cannot be applied to a narrowed per-slot entity set.
- `ShadowLightSlotAssignments::apply_to_packed_lights()` patches packed `GpuLightData.shadow_slot_layer.x` with the first slot and `shadow_params.w` with the slot count before `GpuScene::write_lights()`.

`SceneRendererCore` owns both `ShadowAtlasAllocator` and `ShadowAtlasResources`. The allocator config is derived from the actual atlas resource size so capability fallback to 2048 keeps atlas rects and GPU resource dimensions aligned. Directional cascade slot tier is capped by the actual atlas width divided by cascade count and by the reserved CSM row height.

This is not yet the final shadow renderer. The plan now produces real `view_proj` matrices and proves slot assignment, buffer upload, light-buffer ABI writeback, group1 binding, forward/deferred atlas sampling entry points, graph-visible `shadow-atlas` ordering, and per-slot atlas depth writes. Directional cascade, point-face, and spot atlas writes are narrowed by Plan 04 shadow-view visibility when matching views exist. Directional cascades now use camera frustum slice bounds for both atlas slot matrices and visibility shadow cameras. `mesh_visibility_states_preserve_shadow_only_casters` now guards the mesh-draw mapping layer so a caster culled from the main view but visible in a shadow view remains eligible for the shadow pass. `render_product_csm_directional` and `render_product_multi_spot_shadows` add product-contract source coverage for CSM slot generation and multi-spot atlas coexistence; remaining precision work is receiver coverage/product capture validation and any real-scene caster expansion found by that validation.

## Next Integration

The next LS-M3 slice should validate the atlas-only receiver path under real captures:

- Directional cascades occupy the reserved atlas row and write slots `0..cascade_count`.
- Spot shadows allocate one slot; point shadows allocate six cube-face slots.
- `GpuLightData.shadow_slot_layer` writeback and shader sampling now have atlas depth input, but captured forward/deferred parity still has to prove correctness.
- `GpuShadowSlot`, `GpuShadowGlobals`, and `ShadowAtlasSlotPass` are populated from real allocation/cascade data; directional cascade, point-face, and spot slots consume Plan 04 shadow-view visibility, and directional cascades use camera-frustum slice bounds. Source coverage now preserves shadow-only casters at mesh-draw visibility mapping, but the next CSM refinement is still capture validation plus any required real-scene caster expansion.
- Pass/executor naming has converged to `shadow-atlas`/`shadow.atlas`; the graph/resource hard cut is complete. Source contracts now cover multi-spot coexistence and CSM slot generation, while validation still needs real-capture multi-spot coexistence, CSM stability, forward/deferred parity, and receiver coverage.

## Validation State

The 2026-06-13 directional cascade slice follow-up passed the core-min library check in `E:\cargo-targets\zircon-render-vc3-compact-replay-coremin`, compiled the shared lib-test target with `--no-run`, then ran `cargo test ... render_shadow_ -- --nocapture` successfully for 27 filtered shadow tests. The same target also ran `visibility_context_builds_shadow_views_for_atlas_light_slots` successfully as one filtered visibility test. During debugging, the pre-existing half-texel snapping test data was corrected because the old negative Y sample crossed a texel floor boundary; implementation behavior was unchanged.

The 2026-06-13 pass/executor naming slice converged the graph contract to `shadow-atlas` / `shadow.atlas` while retaining the legacy `SHADOW_MAP` resource as the receiver compatibility surface. Validation passed `cargo fmt --all -- --check`, scoped `git diff --check`, and the core-min library check in `E:\cargo-targets\zircon-render-vc3-compact-replay-coremin` with existing warnings. The focused `shadow_atlas` Cargo filter passed 16 tests and ignored the WGPU submit stats test because the current HZB occlusion path exceeds an 8-storage-buffer adapter limit on this machine. Direct execution of the built core-min lib-test binary passed all 4 `render_product_shadows` contract tests.

The 2026-06-14 receiver hard cut removed the legacy `SHADOW_MAP` graph resource, forward/deferred single-map receiver bindings, `ShadowReceiverUniform` shader structs, Rust receiver uniform buffers, and old single-map depth recording helper. The shadow graph now writes only external `SHADOW_ATLAS`, and render stats count `shadow_atlas_write_count`. Validation passed `cargo fmt --all`, `cargo fmt --all -- --check`, scoped `git diff --check`, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-shadow-atlas-cutover-coremin --message-format short --color never` with existing warnings. A focused Cargo test run for `shadow_atlas` could not start under `--locked` because the current `Cargo.lock` does not satisfy test-target resolution; the lock file was not modified.

The 2026-06-14 caster/receiver source-guard slice added `mesh_visibility_states_preserve_shadow_only_casters`, proving that mesh draw visibility states keep a shadow-view-visible caster even when the main view culled it. Validation passed `cargo fmt --all`, `cargo fmt --all -- --check`, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-caster-receiver-coremin --message-format short --color never` with existing warnings. A focused Cargo test run for the new test timed out after 904 seconds while building/running the shared lib-test target; target-dir cargo/rustc processes were stopped, and `Cargo.lock` remained unchanged.

The 2026-06-14 product shadow contract slice added `render_product_csm_directional` and `render_product_multi_spot_shadows`, proving the product test suite now has source coverage for directional CSM four-slot generation and at least three simultaneous spot shadow atlas slots without overlap. Validation passed `cargo fmt --all`, `cargo fmt --all -- --check`, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-product-shadow-contracts-coremin --message-format short --color never` with existing warnings. A focused `cargo test --no-run` attempt for `render_product_multi_spot_shadows` timed out after 904 seconds during shared lib-test compilation, so no filtered test result is claimed.

The 2026-06-14 LS-M4 PCF quality source-contract slice added `ShadowPcfQuality`, encoded per-light PCF quality into `GpuShadowSlot.params.w` flags, and switched `zr_shadow.wgsl` from a fixed 3x3 PCF kernel to slot-selected 1/5/9 tap kernels. Validation passed `cargo fmt --all -- --check`, scoped `git diff --check`, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-ls-m4-pcf-coremin --message-format short --color never` with existing warnings. `cargo test -p zircon_runtime --lib pcf_quality --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-ls-m4-pcf-coremin --message-format short --color never --no-run` timed out after 904 seconds while compiling/linking the shared lib-test target; no filtered test result is claimed. The target-dir cargo/rustc leftovers were stopped, and a source-contract scan confirmed the key enum, flag, frame-plan, WGSL, and shader-source symbols are present.

The 2026-06-14 LS-M4 contact shadow descriptor slice added the
`rendering.contact_shadow` optional plugin feature, its runtime/editor crates,
and the `contact-shadow` async compute graph pass descriptor. The pass reads
`scene-depth`, `gbuffer-normal`, and Plan 04 `hzb-furthest`, then writes
`contact-shadow-occlusion`. Scoped rustfmt and diff checks passed, plugin
metadata confirmed both contact-shadow packages, and a 16-symbol source-contract
scan covered the pass, manifest, catalog, graph insertion, and docs contracts.
Locked runtime/plugin cargo checks were blocked before compilation because the
current root/plugin lock files need refresh under `--locked`/`--frozen`; no lock
file was modified in this slice.

The 2026-06-14 LS-M4 contact shadow executor slice replaced the placeholder
executor with a plugin-owned WGPU compute executor and `contact_shadow.wgsl`.
The executor caches its compute pipeline, binds the graph depth/normal/HZB/output
resources directly, dispatches 8x8 workgroups, and records the dispatch plus the
`contact-shadow-occlusion` storage write through
`RenderPassGpuExecutionContext::record_compute_dispatch(...)`. Scoped rustfmt,
scoped diff, and a 15-symbol source-contract scan passed. Locked cargo checks
for root runtime and the contact-shadow plugin still stopped before compilation
because both lock files need refresh; no lock file was modified while other Cargo
tasks were active.

The 2026-06-14 LS-M4 contact shadow post-process consumption slice made
`post.stack` declare a read from `contact-shadow-occlusion`, added the runtime
flag and `PostProcessParams::lighting_flags.x`, expanded the post-process bind
group with binding 27, and multiplied the sampled contact visibility into final
color independently from SSAO. Scoped rustfmt, scoped diff, a 12-symbol
source-contract scan, and a 4-call-site SSR fallback scan passed. Locked Cargo
validation was retried for root runtime and the contact-shadow plugin package,
but both commands stopped before compilation because the corresponding lock files
need refresh under `--locked`; no lock file was modified in this slice.
