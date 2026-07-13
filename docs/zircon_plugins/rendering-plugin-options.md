---
related_code:
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_feature_bundle_manifest.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/post_process_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_runtime_feature_flags/scene_runtime_feature_flags.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/rendering_features/rows.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/requires_explicit_opt_in.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/advanced_slot.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_data_document.rs
  - zircon_runtime/src/graphics/tests/advanced_followup_slots.rs
  - zircon_runtime/src/graphics/tests/plugin_feature_compile.rs
  - zircon_runtime/src/graphics/tests/plugin_feature_compile/particle.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_plugin_feature_compile_particle_tests.rs
  - zircon_runtime/src/graphics/tests/renderer_data_asset.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_plugins/rendering/editor/src/lib.rs
  - zircon_plugins/rendering/features/post_process/runtime/src/lib.rs
  - zircon_plugins/rendering/features/post_process/editor/src/lib.rs
  - zircon_plugins/rendering/features/ssao/runtime/src/lib.rs
  - zircon_plugins/rendering/features/ssao/editor/src/lib.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/contact_shadow.wgsl
  - zircon_plugins/rendering/features/contact_shadow/editor/src/lib.rs
  - zircon_plugins/rendering/features/decals/runtime/src/lib.rs
  - zircon_plugins/rendering/features/decals/editor/src/lib.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/lib.rs
  - zircon_plugins/rendering/features/reflection_probes/editor/src/lib.rs
  - zircon_plugins/rendering/features/baked_lighting/runtime/src/lib.rs
  - zircon_plugins/rendering/features/baked_lighting/editor/src/lib.rs
  - zircon_plugins/rendering/features/ray_tracing_policy/runtime/src/lib.rs
  - zircon_plugins/rendering/features/ray_tracing_policy/editor/src/lib.rs
  - zircon_plugins/rendering/features/shader_graph/runtime/src/lib.rs
  - zircon_plugins/rendering/features/shader_graph/editor/src/lib.rs
  - zircon_plugins/rendering/features/vfx_graph/runtime/src/lib.rs
  - zircon_plugins/rendering/features/vfx_graph/editor/src/lib.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/scene_bind_group_bundle/create_scene_bind_group_bundle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/motion_vector_camera.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/skinning/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/skinning/joint_palette_storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/is_skinned.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/queue_profile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
implementation_files:
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_feature_bundle_manifest.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/plugin_render_features.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/post_process_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/create_bind_group/bind_group_entries.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/bind_group_layouts/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/construct/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_runtime_feature_flags/scene_runtime_feature_flags.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/rendering_features/rows.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/requires_explicit_opt_in.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/advanced_slot.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_data_document.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/scene_bind_group_bundle/create_scene_bind_group_bundle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/motion_vector_camera.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/skinning/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/skinning/joint_palette_storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/is_skinned.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/queue_profile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_plugins/rendering/editor/src/lib.rs
  - zircon_plugins/rendering/features/post_process/runtime/src/lib.rs
  - zircon_plugins/rendering/features/post_process/editor/src/lib.rs
  - zircon_plugins/rendering/features/ssao/runtime/src/lib.rs
  - zircon_plugins/rendering/features/ssao/editor/src/lib.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/contact_shadow.wgsl
  - zircon_plugins/rendering/features/contact_shadow/editor/src/lib.rs
  - zircon_plugins/rendering/features/decals/runtime/src/lib.rs
  - zircon_plugins/rendering/features/decals/editor/src/lib.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/lib.rs
  - zircon_plugins/rendering/features/reflection_probes/editor/src/lib.rs
  - zircon_plugins/rendering/features/baked_lighting/runtime/src/lib.rs
  - zircon_plugins/rendering/features/baked_lighting/editor/src/lib.rs
  - zircon_plugins/rendering/features/ray_tracing_policy/runtime/src/lib.rs
  - zircon_plugins/rendering/features/ray_tracing_policy/editor/src/lib.rs
  - zircon_plugins/rendering/features/shader_graph/runtime/src/lib.rs
  - zircon_plugins/rendering/features/shader_graph/editor/src/lib.rs
  - zircon_plugins/rendering/features/vfx_graph/runtime/src/lib.rs
  - zircon_plugins/rendering/features/vfx_graph/editor/src/lib.rs
plan_sources:
  - "user: 2026-05-02 Rendering 插件选项补齐计划"
  - .codex/plans/Rendering 插件选项补齐计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - .codex/plans/多插件组合可选功能规则设计.md
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
tests:
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/graphics/tests/advanced_followup_slots.rs
  - zircon_runtime/src/graphics/tests/plugin_feature_compile.rs
  - zircon_runtime/src/graphics/tests/plugin_feature_compile/particle.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_plugin_feature_compile_particle_tests.rs::runtime_15_plugin_feature_compile_particle_tests_are_child_owner
  - zircon_runtime/src/graphics/tests/renderer_data_asset.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
  - zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_selects_mesh_lod_by_camera_distance
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs::tests::prepared_queue_stats_count_skinned_gpu_draws_separately_from_cpu_fallbacks
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs::tests::prepared_queue_stats_count_cpu_morphed_gpu_skinning_source_as_dynamic_geometry
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs::tests::prepared_queue_stats_count_gpu_skinned_motion_vectors_with_previous_palette
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs::tests::object_motion_history_keeps_dynamic_skinned_pose_sideband
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/motion_vector_camera.rs::tests::successful_submit_records_dynamic_object_history_for_next_frame
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs::tests::skinned_gpu_source_candidate_requires_palette
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs::tests::previous_palette_morph_weights_accept_matching_active_shared_source_weights
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs::tests::previous_palette_morph_weights_reject_changed_active_weights
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/scene_bind_group_bundle/create_scene_bind_group_bundle.rs::tests::model_bind_group_layout_reserves_skinned_joint_palette_bindings
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs::tests::model_uniform_appends_motion_and_skinning_flags_without_moving_existing_fields
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs::tests::joint_palette_composes_pose_world_against_bind_world_matrices
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs::tests::joint_palette_reports_missing_parent_bone_reference
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs::tests::joint_palette_uniform_packs_gpu_matrices_and_count
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs::tests::joint_palette_uniform_rejects_current_uniform_limit_overflow
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs::tests::prepared_skinned_model_primitive_keeps_cpu_skinning_when_palette_exceeds_uniform_limit
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs::tests::prepare_skinned_mesh_asset_primitive_keeps_morphed_shader_source_before_cpu_skinning
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_exposes_skinning_vertex_channels
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_exposes_object_motion_vector_entries
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_executes_skinned_joint_palette_behind_draw_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/tests/mesh_gpu_scene.rs::render_product_diagnostics_record_skinned_mesh_queue_count
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_plugins/rendering/features/post_process/runtime/src/lib.rs
  - zircon_plugins/rendering/features/ssao/runtime/src/lib.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/lib.rs
  - zircon_plugins/rendering/features/contact_shadow/runtime/src/contact_shadow.wgsl
  - zircon_plugins/rendering/features/decals/runtime/src/lib.rs
  - zircon_plugins/rendering/features/reflection_probes/runtime/src/lib.rs
  - zircon_plugins/rendering/features/baked_lighting/runtime/src/lib.rs
  - zircon_plugins/rendering/features/ray_tracing_policy/runtime/src/lib.rs
  - zircon_plugins/rendering/features/shader_graph/runtime/src/lib.rs
  - zircon_plugins/rendering/features/vfx_graph/runtime/src/lib.rs
doc_type: module-detail
---

# Rendering Plugin Options

## Owner model

`rendering` is the umbrella plugin package for the Rendering option pool. It owns
nine optional feature bundles:

- `rendering.post_process`
- `rendering.ssao`
- `rendering.contact_shadow`
- `rendering.decals`
- `rendering.reflection_probes`
- `rendering.baked_lighting`
- `rendering.ray_tracing_policy`
- `rendering.shader_graph`
- `rendering.vfx_graph`

The runtime catalog exposes the package as `RuntimePluginId::Rendering`, with
target modes limited to `client_runtime` and `editor_host`. The package category
is `rendering`, and `PluginPackageManifest` carries that category through TOML
round-trips and descriptor-derived manifests.

Each Rendering optional feature has a runtime capability
`runtime.feature.rendering.<feature>` and a matching editor module capability
`editor.feature.rendering.<feature>`. The static `plugin.toml`, generated
runtime provider manifest, built-in catalog row, and feature editor crate
descriptor now all project the same editor capability, so the editor-host package
projection can show feature-specific authoring surfaces instead of only the
umbrella `rendering.editor` extension.

## Default policy

The default-enabled feature set is intentionally limited to the options that
preserve the previous frame graph behavior:

- `post_process`
- `ssao`
- `reflection_probes`
- `baked_lighting`

`contact_shadow`, `decals`, `ray_tracing_policy`, `shader_graph`, and
`vfx_graph` are opt-in. VFX Graph depends on `particles` plus the
`runtime.feature.rendering.shader_graph` capability; the catalog reports it as
blocked when those dependencies are not selected, and it does not implicitly
enable either dependency.

## Runtime boundary

`zircon_runtime` still owns only neutral contracts: plugin catalog metadata,
`RenderFeatureDescriptor`, render pass executor registration, RHI capability
requirements, and graph compilation. It does not depend on `zircon_plugins`.

Feature implementations live under `zircon_plugins/rendering/features/*`. The
existing-backed features register descriptors matching the old pass contracts:
SSAO keeps the ambient-occlusion history binding, reflection probes and baked
lighting keep their post-process composite slots, and post process keeps
`post.stack`. The default forward/deferred pipeline assets no longer embed those
features directly; applying the default rendering feature descriptors restores
the legacy pass order.

The `contact_shadow` feature is the Plan 05 LS-M4 screen-space shadow extension.
Its runtime crate registers a `contact-shadow` AmbientOcclusion-stage async
compute pass with executor `lighting.contact-shadow`; the pass reads
`scene-depth`, `gbuffer-normal`, and the shared Plan 04 `hzb-furthest` mip chain,
then writes the transient `contact-shadow-occlusion` storage texture. The
executor is a plugin-owned WGPU compute executor that caches its pipeline, binds
the graph resources directly, runs `contact_shadow.wgsl`, and records the actual
dispatch through `RenderPassGpuExecutionContext::record_compute_dispatch(...)`.
The built-in `post.stack` pass declares a normal texture read from
`contact-shadow-occlusion`, so when the plugin is enabled the compiled graph keeps
the transient lifetime alive until final compositing. When the feature is absent,
execution binds a white fallback. The final post-process shader samples binding
27 under `SceneRuntimeFeatureFlags::contact_shadow_enabled` /
`PostProcessParams::lighting_flags.x` and multiplies it independently from SSAO.
The feature is disabled by default, and compile options that disable
`contact_shadow` leave no `contact-shadow` producer pass in the compiled graph.

The runtime built-in `Decal` feature is deliberately narrower than the
`rendering.decals` plugin. It is a descriptor-only advanced slot for renderer-data
sources, quality gates, and profile opt-in; enabling it only reserves the
`decals` extract section and does not register a graph pass, executor, history
binding, or backend capability requirement. Descriptor-only advanced built-ins
are registered through the shared catalog in
`builtin_render_feature/advanced_slots.rs`, so descriptor dispatch, explicit
opt-in, sparse capability metadata, and tests use the same slot table.

The runtime built-in `Particle` slot follows the same descriptor-first policy.
Core scene particles can request the neutral `particles` extract section and
compile the built-in billboard pass only when particle sprites are present,
while the executable external particle feature still comes from a plugin
descriptor named `particle`. Plugin feature compile particle tests owner split
keeps `graphics/tests/plugin_feature_compile.rs` as the generic plugin/advanced/SMAA
compile parent and moves particle plugin/core-scene particle compile guards into
`graphics/tests/plugin_feature_compile/particle.rs`. Guard
`runtime_15_plugin_feature_compile_particle_tests_are_child_owner` records this
shape under
`render_plugin_feature_compile_particle_tests_owner_split_static_passed_cargo_deferred_implementation_cadence`.

The runtime built-in `MeshLod` feature follows the same descriptor-first policy
at the SRP layer, but conventional scene data now has a first runtime extraction
path. Scene mesh instances can carry `lods`, each with a finite distance
threshold and the same model/direct-mesh/material/primitive binding shape as the
base mesh renderer. Scene extraction chooses the highest matching
`min_distance` level for the active camera and emits ordinary flat mesh snapshots
for the selected source. Enabling `MeshLod` still only reserves the neutral
`mesh_lod` extract section; it does not register a plugin executor, graph pass,
backend capability, streaming policy, cross-fade, screen-error selector, or
Virtual Geometry/Nanite path.

The runtime built-in `SkinnedMesh` feature follows the same descriptor-first
policy for future skeletal mesh rendering. Renderer data can reference
`SkinnedMesh` as a source or quality gate, but enabling it only reserves the
neutral `skinned_mesh` extract section. Runtime mesh preparation now computes a
renderer-private joint palette for the CPU-skinned fallback path, packs that
palette through the mesh-level fixed 256-matrix uniform ABI, uploads it into a
renderer-owned per-draw buffer when the current ABI fits, marks those fallback
draws, and exposes `render.mesh.queue.skinned_draw_count`,
`render.mesh.queue.skinned_palette_upload_count`, and
`render.mesh.queue.skinned_gpu_source_candidate_count`. When the candidate has
a palette uniform plus a shader-skinning source, and the built-in fallback
shader pipeline, draw construction switches the active geometry to that source,
sets the model-uniform shader-skinning flag, and exposes
`render.mesh.queue.skinned_gpu_skinning_draw_count`. Successful submissions also
record dynamic skinned pose history for the next frame. When the next draw has a
matching previous skeleton pose, a previous transform, and either no current or
previous morph weights or a direct CPU-morphed GPU-skinning source whose finite
current and previous morph weights match, draw construction uploads a previous palette, sets the
previous-palette model-uniform flag, and exposes
`render.mesh.queue.skinned_previous_palette_upload_count` plus
`render.mesh.queue.skinned_gpu_motion_vector_draw_count`. Direct `MeshAsset`
draws with active morph weights can now use a CPU-morphed-but-unskinned source
primitive for current-frame shader skinning; that subset is exposed as
`render.mesh.queue.skinned_gpu_cpu_morphed_source_candidate_count`. Morphed
previous-palette velocity with changed weights still waits for a GPU-visible previous morph source.
The mesh pipeline cache resolves that fallback shader id to the renderer
fallback WGSL source, and custom material shader authored non-empty layouts now get renderer ABI readiness diagnostics for the same skinning ABI. Actual custom shader execution remains on the CPU fallback
until GPU-skinning and velocity policy lands.
GPU-skinned prepared draws still count as prepared geometry, while CPU-morphed
shader-skinning sources count as dynamic geometry. Both are excluded from
ordinary direct prepared batch and GPU instancing candidates because the current
batch key does not include the per-draw palette. Oversized palettes keep the CPU
fallback draw and skip only the palette upload. The fallback mesh shader
now declares the joint index, joint weight, tangent, and vertex color channels;
deferred geometry declares the same tangent/color inputs, and both fallback and
deferred albedo multiply authored vertex color. Runtime glTF import now carries
primitive `TANGENT` and `COLOR_0` into the model vertices and labeled mesh
subasset attributes that feed those channels, and the CPU-morphed source applies
standard tangent.xyz plus vertex-color morph deltas before shader skinning. Fallback mesh,
deferred geometry, normal prepass, and shadow map WGSL now declare the same
group 1 binding 1 current-palette ABI and execute palette skinning only behind
the draw-level flag and palette joint-count bounds. The fallback mesh
motion-vector shader also consumes group 1 binding 2 as the previous-palette ABI
when `ModelUniform.motion_params.z` is set, so the built-in fallback path can
compute current and previous clip positions from matching skinned poses. The
mesh material texture set now binds base color and normal map separately, uses a
neutral normal fallback, and lets fallback lighting plus the normal prepass sample
material normal maps through the skinned tangent frame. The existing model bind
group now reserves vertex-stage bindings 1 and 2 and every draw binds either the
per-draw palette buffers or shared empty palette buffers, but graph pass/executor
ownership, full GPU morph deformation, storage-buffer large-skeleton support,
custom material shader GPU-skinning execution and velocity policy, plugin-owned skinned velocity output,
and any plugin-owned renderer extension remain dedicated renderer work.

## V1 feature surfaces

`decals` registers a `rendering.Component.DecalProjector` descriptor plus the
`decal-projector-composite` screen/deferred-compatible composite pass and the
explicit `decals.projector-composite` executor. That plugin remains the owner of
executable projector decal rendering; the runtime `Decal` token only gives
renderer data and quality profiles a stable built-in feature name before a
dedicated built-in decal renderer exists.

`contact_shadow` registers no component surface in V1. It contributes the
short-distance screen-space ray-march pass descriptor, a minimal WGPU compute
executor/shader pair, the post-process binding/flag path that consumes its
visibility output, and the matching editor capability row so projects can enable
the feature explicitly while authoring controls and visual tuning remain future
work.

`ray_tracing_policy` provides a policy report over acceleration structure,
inline ray query, and ray pipeline gates. It does not implement a hardware ray
tracer in V1.

`shader_graph` provides a local asset DTO and a minimal WGSL compiler for
constants, texture samples, math nodes, color output, and material output.

`vfx_graph` provides a VFX asset DTO, compile report, emitter component, an
async simulation pass, and a transparent render pass.

## Reference evidence

The module split follows Unreal's separation between `Renderer`, `RenderCore`,
and `RHI`, plus the plugin-shaped examples in
`PostProcessMaterialChainGraph`, `GPULightmass`, and `Niagara`. Unreal decal
actor metadata and sort-order handling inform the plugin-owned projector surface.
Unity Graphics is the secondary reference for SRP `ScriptableRendererFeature`,
RenderGraph, ShaderGraph, VFX Graph, Decal Projector authoring, SSAO, and
post-process pass organization. Bevy's clustered decal path is the local
WGPU-oriented reminder that real decal execution needs explicit clustered/storage
resources rather than a descriptor-only slot.

## Validation

Focused checks that passed for this slice:

- 2026-05-31 editor-module capability parity:
  `cargo test --manifest-path zircon_plugins\rendering\runtime\Cargo.toml
  rendering_feature_manifests_declare_editor_capabilities --locked --offline
  --jobs 1 --target-dir D:\cargo-targets\zircon-rendering-feature-editor-capability
  --quiet` passed for the generated Rendering provider manifests.
- 2026-05-31 editor-module capability parity:
  `cargo test --manifest-path Cargo.toml -p zircon_runtime --lib
  builtin_rendering_optional_features_declare_editor_capabilities --locked
  --offline --jobs 1 --target-dir
  D:\cargo-targets\zircon-rendering-feature-editor-capability --quiet` passed
  for the built-in catalog row.
- 2026-05-31 editor-module capability parity:
  `cargo test --manifest-path Cargo.toml -p zircon_runtime --lib
  rendering_plugin_toml_roundtrips_owner_features_and_modules --locked --offline
  --jobs 1 --target-dir
  D:\cargo-targets\zircon-rendering-feature-editor-capability --quiet` passed
  for static `zircon_plugins/rendering/plugin.toml` round-trip parity.
- 2026-05-31 editor-module capability parity: all eight then-existing Rendering feature editor
  crates passed `cargo check --locked --offline --jobs 1` against
  `D:\cargo-targets\zircon-rendering-feature-editor-capability` after their
  descriptors were wired to the runtime `EDITOR_CAPABILITY` constants.
- 2026-06-14 contact shadow feature row:
  scoped `rustfmt --edition 2021 --check` passed, scoped `git diff --check`
  passed with only CRLF warnings, `cargo metadata --manifest-path
  zircon_plugins\Cargo.toml --no-deps --format-version 1 --locked` confirmed
  the `zircon_plugin_rendering_contact_shadow_runtime` and
  `zircon_plugin_rendering_contact_shadow_editor` workspace packages, and a
  16-symbol source-contract scan covered the pass, manifest, catalog, graph
  insertion, and docs contracts. Locked `cargo check` for the runtime package
  was blocked before compilation because `zircon_plugins/Cargo.lock` currently
  needs refresh; that lock file was not modified while another plugin Cargo
  task was active.
- 2026-06-14 contact shadow executor slice: scoped `rustfmt --edition 2021
    --check`, scoped `git diff --check`, and a 15-symbol source-contract scan
  passed for the WGPU executor, `contact_shadow.wgsl` bindings, storage texture
    output, and plugin dispatch-recording entry. Locked runtime/plugin cargo checks
    were retried and still stopped before compilation because the root and plugin
    lock files need refresh; no lock file was modified while other Cargo work was
    active.
- 2026-06-14 contact shadow post-process consumption slice: scoped `rustfmt
  --edition 2021`, scoped `git diff --check`, a 12-symbol source-contract scan,
  and a 4-call-site SSR fallback scan passed for the `post.stack` graph read,
  runtime flag, binding 27 layout/entry, WGSL sampling path, and plugin-side
  post-process read dependency assertion. Locked Cargo validation was retried
  with target dirs `D:\cargo-targets\zircon-runtime-contact-shadow-post-0614`
  and `D:\cargo-targets\zircon-contact-shadow-post-0614`; both checks stopped
  before compilation because the root/plugin lock files need refresh under
  `--locked`. No lock file was modified by this slice.

- `cargo metadata --manifest-path zircon_plugins/Cargo.toml --no-deps --format-version 1`
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_rendering_runtime --locked --jobs 1`
- `cargo check --manifest-path zircon_plugins/Cargo.toml` for all rendering
  feature runtime crates with `--locked --jobs 1`
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_rendering_runtime --locked --jobs 1`
- `cargo test --manifest-path zircon_plugins/Cargo.toml` for all rendering
  feature runtime crates with `--locked --jobs 1`
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_rendering_editor --locked --jobs 1`
- `cargo check --manifest-path zircon_plugins/Cargo.toml` for all rendering
  feature editor crates with `--locked --jobs 1`
- `cargo test -p zircon_runtime --lib --locked rendering_plugin_default_features_restore_legacy --jobs 1`
- `cargo test -p zircon_runtime --lib --locked builtin_rendering_catalog_declares_owner_features_and_defaults --jobs 1`
- `cargo test -p zircon_runtime --lib --locked compile_options_can_disable_clustered_history_and_rendering_plugin_features --jobs 1`
- `cargo test -p zircon_runtime --lib --locked rendering_ --jobs 1`, covering
  the rendering catalog, `plugin.toml` roundtrip, VFX dependency diagnostics,
  server-target blocking, runtime feature flags, and pipeline pass order
- `cargo test -p zircon_editor --lib --locked
  editor_manager_plugin_status_lists_rendering_owner_features_and_defaults
  --jobs 1`, covering editor plugin status projection for the Rendering owner
  package and its feature rows
- `cargo test -p zircon_runtime --lib --locked plugin_extensions --jobs 1`,
  covering plugin catalog, manifest, dependency, export-template, native loader,
  extension registry, and Rendering option integration rows
- `cargo test -p zircon_runtime --lib --locked graphics::tests::pipeline_compile
  --jobs 1`, covering default/disabled/enabled render feature graph behavior
- `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked
  --jobs 1`, covering the full plugin workspace, including the Rendering
  umbrella and all Rendering feature runtime/editor crates
- `cargo test -p zircon_runtime --lib --locked --jobs 1`, covering the full
  runtime lib test binary after the Rendering option changes; this includes
  project-render behavior, M4 behavior layers, plugin extension tests, and
  runtime absorption fixtures
- `.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -RepoRoot
  E:/Git/ZirconEngine -TargetDir
  E:/cargo-targets/zircon-rendering-plugin-runtime-check`, covering repository
  workspace build and test with locked dependencies

The first focused editor status-listing run timed out while concurrent Cargo
jobs were active, but the retry passed once the build queue cleared enough to
compile the editor crate.
The first full plugin workspace attempt exposed a missing `toml` test dependency
in the glTF importer package; after adding that dev-dependency and refreshing
`zircon_plugins/Cargo.lock` offline, the full plugin workspace test passed.
The first repository validator passes exposed two shared support gaps outside
the Rendering package itself: runtime absorption fixtures still had a stale
string-based manager descriptor shape, and parallel full-suite graphics tests
could collide with other workspace processes through timestamp-only temporary
project roots. The closeout changed those fixtures to typed `qualified_name(...)`
manager descriptors and made graphics temporary project roots include process
and per-process sequence components. After those fixes, the full runtime lib
test and final validator passed.
