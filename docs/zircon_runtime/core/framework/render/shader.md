---
related_code:
  - dev/bevy/crates/bevy_shader/src/lib.rs
  - dev/bevy/crates/bevy_shader/src/shader.rs
  - dev/bevy/crates/bevy_shader/src/shader_cache.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_render/src/render_resource/bind_group_layout.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/environment/mod.rs
  - zircon_runtime/src/core/framework/render/environment/extract.rs
  - zircon_runtime/src/core/framework/render/environment/skybox.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/environment_brdf_lut.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/page_payload.rs
  - zircon_runtime/src/core/framework/render/shader/mod.rs
  - zircon_runtime/src/core/framework/render/shader/ide_env.rs
  - zircon_runtime/src/bin/zircon_shader_ide_env/args.rs
  - zircon_runtime/src/bin/zircon_shader_ide_env/run.rs
  - zircon_runtime/src/bin/zircon_shader_ide_env/main.rs
  - zircon_runtime/src/graphics/shader/ide_env_generation.rs
  - zircon_runtime/src/graphics/shader/ide_env_generation/tests.rs
  - zircon_runtime/src/graphics/shader/ide_preview.rs
  - zircon_runtime/src/graphics/shader/ide_validation.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_runtime/src/core/framework/render/shader/module_import.rs
  - zircon_runtime/src/core/framework/render/shader/asset_kind.rs
  - zircon_runtime/src/core/framework/render/shader/render_state.rs
  - zircon_runtime/src/core/framework/render/shader/queue.rs
  - zircon_runtime/src/core/framework/render/shader/resource.rs
  - zircon_runtime/src/core/framework/render/shader/stage.rs
  - zircon_runtime/src/core/framework/render/shader/entry_point.rs
  - zircon_runtime/src/core/framework/render/shader/geometry_source.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_geometry_source_descriptor.rs
  - zircon_runtime/src/core/framework/render/shader/definition_value.rs
  - zircon_runtime/src/core/framework/render/shader/dependency.rs
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/core/framework/render/shader/pipeline_layout.rs
  - zircon_runtime/src/core/framework/render/shader/compute_dispatch.rs
  - zircon_runtime/src/core/framework/render/shader/fullscreen_pass.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/asset/assets/material/alpha_mode.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/readiness.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies.rs
  - zircon_runtime/src/asset/project/shader_resource_records.rs
  - zircon_runtime/src/asset/tests/project/zmeta/compound_shader.rs
  - zircon_runtime/tests/shader_import_dependency_contract.rs
  - zircon_runtime/tests/material_shader_redirect_dependency_contract.rs
  - zircon_runtime/src/asset/assets/shader/entry_point.rs
  - zircon_runtime/src/asset/assets/shader/dependency.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests/readiness_diagnostics/shader_redirect.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/shader_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_shader.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_shader_quality.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/extension_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/target_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/mod.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/create/create_render_framework.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_construct/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_brdf_lut.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/half_float.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/procedural_environment.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/construct.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/pipeline_profiles.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/deferred.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/prewarm_pipeline_validation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests/runtime_pipeline.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/binding.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/morph.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/virtual_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/geometry_source_selection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/morph_payload_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/virtual_geometry_resident_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/extract_item.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/virtual_geometry_submission_detail.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/virtual_geometry_execution_projection.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/extract_output.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/page_payload.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/automatic_extract.rs
  - zircon_plugins/virtual_geometry/runtime/src/provider.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources/virtual_geometry_imported_extract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/virtual_geometry_asset_payload_decode.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/virtual_geometry_cluster_payload_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/geometry_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/queue_profile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/tests/combined_validation_tests.rs
  - zircon_runtime/src/graphics/shader/template/mod.rs
  - zircon_runtime/src/graphics/shader/template/assemble.rs
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
  - zircon_runtime/src/graphics/shader/template/material_surface.rs
  - zircon_runtime/src/graphics/shader/template/pass_specialization.rs
  - zircon_runtime/src/graphics/shader/template/deferred_gbuffer.rs
  - zircon_runtime/src/graphics/shader/template/taa_reactive_mask.rs
  - zircon_runtime/src/graphics/shader/template/validation.rs
  - zircon_runtime/src/graphics/shader/template/tests.rs
  - zircon_runtime/src/graphics/material/shading_models/include_sources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_shading_models.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests/runtime_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/create.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_surface_types.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_scene_runtime.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_static.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_virtual_geometry.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_forward.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_gbuffer.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_deferred_gbuffer.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_depth.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_depth_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_taa_reactive_mask.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_gbuffer_encode_standard_pbr.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shade_deferred_standard_pbr.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shade_deferred_blinn_phong.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shade_deferred_unlit.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests/runtime_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/tests/pipeline_compile/dynamic_resolution.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_gbuffer_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/test_support.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/shading_model_parity.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/staged_prewarm/mod.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/staged_prewarm/material_passes.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/mod.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/case.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/manifest.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/fixture.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/pipeline.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/assertions.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/second_launch.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/custom_shading_model.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/custom_second_launch.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/custom_product_png.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/product_png.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_staged_cache.rs
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/render_plan08_three_shading_models_forward_deferred_parity.rs
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/material_sources.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/module_dependencies.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/paths.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/permutation_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/revision.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests/module_dependencies.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests/raw_revision.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests/resource_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/error.rs
  - zircon_runtime/src/core/resource/manager/registry_export.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/resource_revisions.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_shader_permutation_manifest.rs
  - zircon_plugins/virtual_geometry/runtime/src/plugin.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/native_dynamic_fixture/assets/shader.wgsl.zmeta
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/geometry_source.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_live_resource_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_permutation_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_permutation_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_permutation_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_geometry_source_descriptor.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_shading_model_descriptor.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_revision.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_export_file.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_cli_dry_run.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_cli_selection.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_command.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_fixture.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_live_wgpu.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_wrapper_no_proxy.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_wrapper_orchestration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_runtime_staged_cache_hit.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_material_passes_staged_cache.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_product_staged_cache.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_live_asset_roots.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/depth_prepass_pure_depth_product_migration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_module_validation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_pipeline_validation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_validation_report_summary.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_report_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_summary.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_report_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_export_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_report_correlation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_report_dimension_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/taa_reactive_shader_pass_identity.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_multi_root_dedupe.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_asset_root_plan_visibility.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_staged_wgpu_handoff_command_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/virtual_geometry_page_cluster_shader_bindings.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/virtual_geometry_resident_buffers_upload.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/morph_storage_buffers_upload.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/morph_payload_projection.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/morph_payload_slot_indexing.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/morph_geometry_source_selection.rs
  - zircon_runtime/src/graphics/shader/template/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_mesh_pass_processors.rs
  - tools/zircon_build.py
  - tools/zircon_build_shader_prewarm.py
  - tools/zircon_build_shader_resource_registry.py
  - tools/zircon_build_shader_prewarm_report_contract.py
  - tools/zircon_build_shader_prewarm_cache_artifacts.py
  - tools/zircon_build_shader_prewarm_acceptance.py
  - tools/zircon_build_shader_prewarm_written_variants.py
  - tools/tests/test_zircon_build_shader_prewarm.py
  - tools/tests/shader_prewarm_test_support.py
  - tools/tests/test_zircon_build_shader_prewarm_wrapper_orchestration.py
  - tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_command_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_source_provenance_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_cache_contract.py
  - tools/tests/test_zircon_build_plugin_carriers.py
implementation_files:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/environment/mod.rs
  - zircon_runtime/src/core/framework/render/environment/extract.rs
  - zircon_runtime/src/core/framework/render/environment/skybox.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs
  - zircon_runtime/src/core/framework/render/environment/environment_brdf_lut.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/shader/mod.rs
  - zircon_runtime/src/core/framework/render/shader/ide_env.rs
  - zircon_runtime/src/bin/zircon_shader_ide_env/args.rs
  - zircon_runtime/src/bin/zircon_shader_ide_env/run.rs
  - zircon_runtime/src/bin/zircon_shader_ide_env/main.rs
  - zircon_runtime/src/graphics/shader/ide_env_generation.rs
  - zircon_runtime/src/graphics/shader/ide_env_generation/tests.rs
  - zircon_runtime/src/graphics/shader/ide_preview.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies.rs
  - zircon_runtime/src/asset/project/shader_resource_records.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_runtime/src/core/framework/render/shader/asset_kind.rs
  - zircon_runtime/src/core/framework/render/shader/render_state.rs
  - zircon_runtime/src/core/framework/render/shader/queue.rs
  - zircon_runtime/src/core/framework/render/shader/resource.rs
  - zircon_runtime/src/core/framework/render/shader/stage.rs
  - zircon_runtime/src/core/framework/render/shader/entry_point.rs
  - zircon_runtime/src/core/framework/render/shader/geometry_source.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_geometry_source_descriptor.rs
  - zircon_runtime/src/core/framework/render/shader/definition_value.rs
  - zircon_runtime/src/core/framework/render/shader/dependency.rs
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/core/framework/render/shader/pipeline_layout.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/asset/assets/material/alpha_mode.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/readiness.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/shader/entry_point.rs
  - zircon_runtime/src/asset/assets/shader/dependency.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/shader_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_shader.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_shader_quality.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/extension_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/target_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/mod.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/create/create_render_framework.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_construct/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_brdf_lut.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/half_float.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/procedural_environment.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/preview_sky_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/deferred.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/morph_payload_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/extract_item.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/geometry_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/queue_profile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/graphics/shader/template/mod.rs
  - zircon_runtime/src/graphics/shader/template/assemble.rs
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
  - zircon_runtime/src/graphics/shader/template/material_surface.rs
  - zircon_runtime/src/graphics/shader/template/pass_specialization.rs
  - zircon_runtime/src/graphics/shader/template/deferred_gbuffer.rs
  - zircon_runtime/src/graphics/shader/template/taa_reactive_mask.rs
  - zircon_runtime/src/graphics/shader/template/validation.rs
  - zircon_runtime/src/graphics/shader/template/tests.rs
  - zircon_runtime/src/graphics/material/shading_models/include_sources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_shading_models.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests/runtime_pipeline.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_surface_types.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_scene_runtime.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_static.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_virtual_geometry.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_forward.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_gbuffer.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_deferred_gbuffer.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_depth.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_depth_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_taa_reactive_mask.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_gbuffer_encode_standard_pbr.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shade_deferred_standard_pbr.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shade_deferred_blinn_phong.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shade_deferred_unlit.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests/runtime_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/dynamic_resolution.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_gbuffer_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/test_support.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly.rs
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/module_dependencies.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/paths.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/permutation_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/revision.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests/module_dependencies.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests/resource_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - zircon_runtime/src/core/resource/manager/registry_export.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_shader_permutation_manifest.rs
  - zircon_plugins/virtual_geometry/runtime/src/plugin.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/native_dynamic_fixture/assets/shader.wgsl.zmeta
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/geometry_source.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_live_resource_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_permutation_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_permutation_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_asset_roots_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/depth_prepass_pure_depth_product_migration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_module_validation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_pipeline_validation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_validation_report_summary.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_report_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_summary.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_report_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_export_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_report_correlation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_report_dimension_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/taa_reactive_shader_pass_identity.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_revision.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_multi_root_dedupe.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_asset_root_plan_visibility.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_staged_wgpu_handoff_command_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_export_file.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_cli_dry_run.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_cli_selection.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_command.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_fixture.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_live_wgpu.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_wrapper_no_proxy.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_wrapper_orchestration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_runtime_staged_cache_hit.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_live_asset_roots.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_acceptance_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_mesh_pass_processors.rs
  - tools/zircon_build.py
  - tools/zircon_build_shader_prewarm.py
  - tools/zircon_build_shader_prewarm_report_contract.py
  - tools/zircon_build_shader_prewarm_cache_artifacts.py
  - tools/zircon_build_shader_prewarm_acceptance.py
  - tools/zircon_build_shader_prewarm_written_variants.py
  - tools/tests/test_zircon_build_shader_prewarm.py
  - tools/tests/test_zircon_build_shader_prewarm_wrapper_orchestration.py
  - tools/tests/test_zircon_build_shader_prewarm_command_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_source_provenance_contract.py
  - tools/tests/test_zircon_build_shader_prewarm_cache_contract.py
  - tools/tests/test_zircon_build_plugin_carriers.py
plan_sources:
  - user: 2026-05-20 Bevy rendering completion plan continuation
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - docs/superpowers/specs/2026-05-24-shader-readiness-report-design.md
  - docs/superpowers/specs/2026-05-25-typed-shader-definitions-design.md
  - docs/superpowers/plans/2026-05-24-shader-readiness-report.md
  - docs/superpowers/plans/2026-05-25-typed-shader-definitions.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/shader/01-shader-asset-kinds-and-zshader-v2.md
  - docs/plans/zircon_runtime/shader/03-module-imports-and-cross-references.md
  - docs/plans/zircon_runtime/shader/04-material-binding-and-renderer-contract.md
  - docs/plans/zircon_runtime/shader/05-ide-and-authoring-dx.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/superpowers/specs/2026-07-04-skybox-ibl-pbr-matrix-design.md
  - docs/superpowers/plans/2026-07-04-skybox-ibl-pbr-matrix.md
  - docs/superpowers/plans/2026-07-04-real-hdri-pbr-reflection.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::deferred::lighting_pipeline --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m3-pbr-0704 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-04 Plan 11 PBR environment reflection: passed, 13/13)
  - cargo test -p zircon_runtime --lib scene_renderer::mesh::mesh_pipeline::fallback_mesh_shader_source --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m3-pbr-0704 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-04 Plan 11 fallback mesh environment include/source coverage: passed, 14/14)
  - cargo test -p zircon_runtime --lib graphics::shader::template --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m3-pbr-0704 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-04 Plan 11 shader template environment include coverage: passed, 21/21)
  - cargo test -p zircon_runtime --lib graphics::tests::project_render::project_scenes::export_runtime_shader_pbr_metallic_smoothness_matrix_png --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m4-pbr-matrix-0704 --message-format short --color never -- --ignored --exact --nocapture --test-threads=1 (2026-07-04 Plan 11 8x8 PBR metallic/smoothness matrix export: passed, 1/1; wrote docs/tests/runtime/shader/runtime_shader_pbr_metallic_smoothness_matrix_skybox_20260704.png, 1280x960, 109556 bytes, SHA256 E883A3BDF657025EAD16A7F39B1F8BE5D7FFCDA1FDEF0243A8636A05C217030D; same-name repo target and E/F/D cargo-target root scan returned 0)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-real-hdri-reflection-0704 --message-format short --color never (2026-07-04 Plan 11 real HDRI sampled environment reflection: passed with existing warnings)
  - E:\cargo-targets\zircon-real-hdri-reflection-0704-server\debug\deps\zircon_runtime-0a7825d39d44b0c4.exe graphics::tests::project_render::project_scenes::export_runtime_shader_pbr_real_hdri_reflection_png --ignored --exact --nocapture --test-threads=1 (2026-07-04 Plan 11 real HDRI reflection export: passed, 1/1; wrote docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png, 1280x960, 132232 bytes, SHA256 958E3B200EC56BCA16BF9596B1F05D872179F51CEB9A64925E10FC2D41792DEE; source HDR docs/tests/runtime/shader/assets/polyhaven_lakes_1k.hdr size 1464859, MD5 B615491D315A3D4E23BB09C2C96C9E03, SHA256 FAF3ECE79216E568A29F0D8FC176A795C66EB9C312C3CF3EE18D9AC04A71DECB; same-name repo target and E/F/D cargo-target root scan returned 0)
  - cargo test -p zircon_runtime --lib source_cubemap_linear_sampling_bleeds_across_face_edges --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-hdr-pmrem-export-it-0705 --message-format short --color never -- --nocapture (2026-07-05 Plan 06 EC-M3d cross-face source-cubemap CPU sampling guard: passed, 1/1)
  - CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --test runtime_environment_source_cubemap_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-hdr-pmrem-export-it-0705 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-05 Plan 06 EC-M3d source-cubemap PMREM contract after cross-face/high-roughness changes: passed, 9/9)
  - CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-hdr-pmrem-export-it-0705 --message-format short --color never -- --ignored --exact export_runtime_shader_pbr_real_hdri_2k_reflection_png --nocapture --test-threads=1 (2026-07-05 Plan 06 EC-M3d Poly Haven lakes 2K HDRI source-cubemap PMREM export: passed, 1/1; wrote docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_2k_hdr_pmrem_reflection_20260705.png, 1280x960, 1009731 bytes, SHA256 920A028DC6B0BB64A45F1798E89BF5E0FBE2BABF3A90BED22FFBA842DD1714F0; source HDR docs/tests/runtime/shader/assets/polyhaven_lakes_2k.hdr size 5918432, SHA256 B2506E0EE912C4C599FF013566FBD3ECAAC2F4B176319D450CCE0DE5758FED98; same-name target and E:\cargo-targets scan returned 0)
  - CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-png-metrics-0706 --message-format short --color never runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics -- --exact --nocapture --test-threads=1 (2026-07-06 Plan 06 EC-M3e saved 2K HDRI PNG metrics regression: passed, 1/1, 0 ignored, 2 filtered; test body 0.23s, build 8m43s; reused docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_2k_hdr_pmrem_reflection_20260705.png, 1009731 bytes, SHA256 920A028DC6B0BB64A45F1798E89BF5E0FBE2BABF3A90BED22FFBA842DD1714F0; same-name target and E:\cargo-targets scan returned 0)
  - CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --test runtime_environment_source_irradiance_cubemap_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-iem-contract-0706 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-06 Plan 06 EC-M2d CPU source IEM bridge: passed, 2/2 after adding root render facade exports; first focused run failed on missing facade re-export; no screenshot generated)
  - cargo test -p zircon_runtime --lib --no-run --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-combined-validation-0704 --message-format short --color never (2026-07-04 shader prewarm combined WGPU module+pipeline validation owner split: passed with existing warnings)
  - E:\cargo-targets\zircon-shader-combined-validation-0704\debug\deps\zircon_runtime-6bef7a696c15c9a5.exe shader --nocapture (2026-07-04 shader/material focused lib filter after combined validation split: passed, 366 passed / 0 failed / 1 ignored / 6065 filtered)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm_wrapper_orchestration.ZirconBuildShaderPrewarmWrapperOrchestrationTests.test_public_runtime_wrapper_exports_project_plugin_registry_with_live_wgpu_pipelines (2026-07-04 public wrapper live WGPU prewarm: passed; report requested/written/failed 18/18/0, WGPU module validation 18/18, WGPU pipeline validation 18/18)
  - cargo build -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-wrapper-pipeline-0704 --message-format short --color never (2026-07-04 combined validation CLI wrapper rebuild: passed with existing warnings)
  - python -m unittest -v tools.tests.test_zircon_build_shader_prewarm_wrapper_orchestration.ZirconBuildShaderPrewarmWrapperOrchestrationTests.test_public_runtime_wrapper_exports_project_plugin_registry_with_live_wgpu_pipelines (2026-07-04 final recheck after docs sync: skipped because `ZR_TEST_SHADER_PREWARM_EXE` was unset and no `zircon_shader_prewarm.exe` was present; not counted as additional live-WGPU evidence)
  - cargo build -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-wrapper-pipeline-0704 --message-format short --color never (2026-07-04 final recheck after docs sync: timed out after 244s while other cargo/rustc lanes were active; not counted as additional rebuild evidence)
  - rustfmt --edition 2021 zircon_runtime/src/asset/mod.rs zircon_runtime/src/asset/assets/mod.rs zircon_runtime/src/asset/assets/shader/mod.rs zircon_runtime/src/asset/assets/shader/zshader.rs zircon_runtime/src/asset/tests/assets/shader_readiness.rs zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/core/framework/render/shader/mod.rs zircon_runtime/src/core/framework/render/shader/asset_kind.rs zircon_runtime/src/core/framework/render/shader/queue.rs zircon_runtime/src/core/framework/render/shader/render_state.rs zircon_runtime/src/core/framework/render/shader/resource.rs (2026-07-02 SH01-M1 zshader v2 contract parse: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/mod.rs (2026-07-03 Frameworks 02 M3 plugin workspace ShaderIdeModuleSource facade backfill: passed)
  - wsl.exe bash -lc "cd /mnt/e/Git/ZirconEngine && cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --jobs 1 --target-dir /home/hejiahui/zircon-targets/frameworks-plugins-workspace-build-0703 --message-format short --color never" (2026-07-03 Frameworks 02 M3 plugin workspace ShaderIdeModuleSource facade backfill: first rerun exposed missing facade export; final rerun passed in 23m04s with existing warnings)
  - cargo test -p zircon_runtime --lib zshader_v2 --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh01-m1 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-02 SH01-M1 zshader v2 contract parse: passed, 3 passed, 5948 filtered; existing repository warnings only)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records/zshader.rs (2026-07-02 SH01-M2 zshader v2 importer cutover review guard: passed)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never (2026-07-02 SH01-M2 zshader v2 importer cutover: passed with existing repository warnings)
  - cargo check -p zircon_runtime --tests --locked --jobs 1 --message-format short --color never (2026-07-02 SH01-M2 zshader v2 importer cutover test compile: passed with existing repository warnings)
  - cargo test -p zircon_runtime --lib zshader_v2 --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh01-m2 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-02 SH01-M2 zshader v2 importer cutover: passed, 5 passed, 5952 filtered; existing repository warnings only)
  - git ls-files '*.zshader' plus static scan for version = 1, pipeline_layout, shader_defs, and shader_def_values (2026-07-02 SH01-M2 repository zshader migration: passed, no tracked matches)
  - cargo check -p zircon_runtime --bin zircon_shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never (2026-07-03 SH05-M2 default shader IDE preview output: passed with existing repository warnings after an earlier cold-build timeout produced no counted result)
  - cargo test -p zircon_runtime --bin zircon_shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 SH05-M2 default shader IDE preview and incremental stub diff: passed 5/5, covering --variants parsing, module_map/stubs/generated material, default preview WGSL, segment JSON, and one-byte module diff rewrites)
  - cargo check -p zircon_runtime --bin zircon_shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never (2026-07-03 SH05-M2 Naga stub parse/default preview validate gate: passed with existing repository warnings; follow-up bin test reruns timed out twice at 304s due concurrent external cargo/rustc lanes and are not counted)
  - cargo test -p zircon_runtime --lib shader_ide_preview_paths_are_scoped_by_source_uri_and_variant --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 SH05-M2 lib helper check: blocked by unrelated missing `graphics/scene/scene_renderer/ui/text/sdf_fallback.rs` test module; not counted as passed)
  - rustfmt --edition 2021 --check --config skip_children=true on `graphics/shader/ide_env_generation.rs`, `graphics/shader/ide_env_generation/tests.rs`, `bin/zircon_shader_ide_env/run.rs`, graphics facade exports, editor project sync hook, and ResourceStreamer accessor import repair (2026-07-03 SH05-M2 shared generator/editor refresh structure split: passed)
  - cargo check -p zircon_runtime --bin zircon_shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never (2026-07-03 SH05-M2 shared generator/runtime bin recheck: passed with existing repository warnings)
  - cargo test -p zircon_runtime --bin zircon_shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 SH05-M2 post-split CLI args test: passed 1/1; generator behavior tests moved to `graphics/shader/ide_env_generation/tests.rs`)
  - cargo check -p zircon_editor --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-editor-refresh --message-format short --color never (2026-07-03 SH05-M2 editor refresh hook typecheck: passed with existing warnings after SDF test-owner imports and SH05 WGSL contract convergence)
  - cargo test -p zircon_runtime --lib shader_ide_env --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 SH05-M2 post-split library generator tests: passed 6/6, covering generator output, incremental diff, validation context, and invalid-stub diagnostics)
  - cargo test -p zircon_runtime --lib shader_template_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 SH05 shared surface/GBuffer contract regression: passed 20/20 after moving `ZrDeferredGBufferOutput` and deferred material flag helpers into `zr_surface_types.wgsl`)
  - cargo test -p zircon_runtime --lib shader_module --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-shader-module --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 SH05-M1/SH03 focused shader_module gate: passed 6/6, covering module import directive parsing/stripping, builtin token classification, and module registry transitive resolution/cycle/include stripping regressions)
  - rustfmt --edition 2021 --check --config skip_children=true zircon_runtime\src\asset\project\manager\scan_and_import.rs zircon_runtime\src\asset\project\manager\scan_and_import\shader_import_dependencies.rs zircon_runtime\tests\shader_import_dependency_contract.rs (2026-07-04 SH03 source-only import dependency propagation: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-live-import-deps-check --message-format short --color never (2026-07-04 SH03 source-only import dependency propagation: passed with existing warnings)
  - cargo test -p zircon_runtime --test shader_import_dependency_contract --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-live-import-deps-check --message-format short --color never -- --nocapture --test-threads=1 (2026-07-04 SH03 source-only import dependency propagation: passed 1/1)
  - cargo test -p zircon_runtime --test material_shader_redirect_dependency_contract --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-redirect-diagnostics-check --message-format short --color never -- --nocapture --test-threads=1 (2026-07-04 SH03 redirect shader import dependency readiness diagnostics: passed 2/2, including ProjectManager scan/import artifact coverage)
  - E:\cargo-targets\zircon-shader-redirect-product-streamer\debug\deps\zircon_runtime-6bef7a696c15c9a5.exe shader_redirect --nocapture --test-threads=1 (2026-07-04 SH03 product ResourceStreamer redirect shader import diagnostics: passed 2/2; Cargo wrapper timed out after 1204s and is not counted)
  - cargo test -p zircon_runtime --lib property_layout --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-nondefault-preview --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 SH02 property layout and generated-material Naga focused gate: passed 4/4)
  - cargo test -p zircon_runtime --lib render_shader_template --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-nondefault-preview --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 SH02/SH05 template focused gate: first run exposed a status-support mirror gap, final rerun passed 18/18 after restoring the render shader template assembly support anchors)
  - cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never (2026-07-01 shader prewarm source-hash helper support fix: passed with existing warnings)
  - cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never (2026-07-01 editor build gate after shader prewarm source-hash helper support fix: passed with existing warnings)
  - zircon_runtime/src/graphics/shader/template/tests.rs::render_shader_template_uses_shading_model_descriptor_forward_include (2026-07-01 Descriptor-driven forward shading include dispatch: added; Cargo result not claimed)
  - zircon_runtime/src/graphics/shader/template/tests.rs::render_shader_template_rejects_unknown_shading_model_forward_include (2026-07-01 Descriptor-driven forward shading include dispatch: added; Cargo result not claimed)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/shader/template/assemble.rs zircon_runtime/src/graphics/shader/template/include_registry.rs zircon_runtime/src/graphics/shader/template/tests.rs (2026-07-01 Descriptor-driven forward shading include dispatch: passed)
  - zircon_runtime/src/graphics/shader/template/tests.rs::render_shader_template_uses_custom_shading_model_forward_include_source (2026-07-01 Custom forward shading include source dispatch: added; Cargo result not claimed)
  - zircon_runtime/src/graphics/shader/template/tests.rs::render_deferred_gbuffer_template_rejects_unknown_shading_model_gbuffer_include (2026-07-01 Deferred GBuffer shading include source dispatch: added; Cargo result not claimed)
  - zircon_runtime/src/graphics/shader/template/tests.rs::render_deferred_gbuffer_template_uses_custom_shading_model_gbuffer_include_source (2026-07-01 Deferred GBuffer shading include source dispatch: added; Cargo result not claimed)
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs::deferred_lighting_shader_rejects_unknown_shading_model_deferred_include (2026-07-01 Deferred lighting include source dispatch: added; Cargo result not claimed)
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs::deferred_lighting_shader_uses_custom_shading_model_deferred_include_source (2026-07-01 Deferred lighting include source dispatch: added; Cargo result not claimed)
  - zircon_runtime/src/graphics/material/shading_models/include_sources.rs::tests::exported_include_source_set_feeds_forward_and_gbuffer_template_requests (2026-07-02 selected plugin/source-registry Cargo-wrapper backfill: passed 1/1 with 5839 filtered)
  - zircon_runtime/src/graphics/material/shading_models/include_sources.rs::tests::project_shader_records_match_include_tokens_without_wgsl_extension (2026-07-01 Project/plugin shading-model include source set: added; Cargo result not claimed)
  - zircon_runtime/src/graphics/material/shading_models/include_sources.rs::tests::project_shader_records_report_missing_plugin_include_source (2026-07-01 Project/plugin shading-model include source set: added; Cargo result not claimed)
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs::runtime_custom_shading_model_sources_compile_as_wgpu_modules (2026-07-01 Custom shading-model runtime WGPU module validation: passed 1/1)
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests/runtime_pipeline.rs::custom_shading_model_deferred_lighting_pipeline_creates_with_project_include_source (2026-07-01 Deferred lighting custom include WGPU pipeline validation: passed 1/1)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/shader/template/assemble.rs zircon_runtime/src/graphics/shader/template/include_registry.rs zircon_runtime/src/graphics/shader/template/tests.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/docs_anchors.rs (2026-07-01 Custom forward shading include source dispatch: passed; Cargo/Naga/WGPU deferred while external cargo/rustc lanes were active)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/shader/template/deferred_gbuffer.rs zircon_runtime/src/graphics/shader/template/include_registry.rs zircon_runtime/src/graphics/shader/template/tests.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/{assembly_assertions,docs_anchors,gbuffer_cache}.rs (2026-07-01 Deferred GBuffer shading include source dispatch: passed; source/docs anchors, line budgets, whitespace/conflict scan, and scoped diff-check passed; Cargo/Naga/WGPU deferred while external cargo/rustc lanes were active)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs zircon_runtime/src/graphics/tests/pipeline_compile/dynamic_resolution.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/{deferred_lighting_include,docs_anchors,wgsl_contracts}.rs (2026-07-01 Deferred lighting include source dispatch: passed; source/docs anchors, line budgets, whitespace/conflict scan, and scoped diff-check passed; Cargo/Naga/WGPU deferred while external cargo/rustc lanes were active)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/material/shading_models/include_sources.rs zircon_runtime/src/graphics/material/mod.rs zircon_runtime/src/graphics/material/shading_models/mod.rs zircon_runtime/src/graphics/shader/template/assemble.rs zircon_runtime/src/graphics/shader/template/deferred_gbuffer.rs zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs (2026-07-01 Project/plugin shading-model include source set: passed; source anchors and line budgets passed; Cargo/Naga/WGPU deferred while external cargo/rustc lanes were active)
  - rustfmt --edition 2021 --check on runtime shading-model include source handoff files passed on 2026-07-01; `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-shading-runtime-handoff-0701 --message-format short --color never` passed with repository warnings only after one earlier timeout attempt
  - cargo test -p zircon_runtime --lib runtime_custom_shading_model_sources_compile_as_wgpu_modules --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-custom-shading-wgpu-nodefault-0701 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 Custom shading-model runtime WGPU module validation: passed 1/1 with repository warnings only after earlier default-feature lib-test compile/link timeouts)
  - cargo test -p zircon_runtime --lib custom_shading_model_deferred_lighting_pipeline_creates_with_project_include_source --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-deferred-lighting-custom-pipeline-0701 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 Deferred lighting custom include WGPU pipeline validation: passed 1/1 with repository warnings only)
  - cargo test -p zircon_runtime --lib render_product_custom_shading_model_registry_material_passes_use_staged_prewarm_without_compile_miss --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-custom-shading-product-0701 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 Custom shading-model product material-pass staged cache WGPU validation: passed 1/1 with repository warnings only)
  - cargo test -p zircon_runtime --lib render_product_custom_shading_model --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-02 Custom shading-model product group Cargo-wrapper WGPU backfill: passed 3/3 with 5850 filtered and repository warnings only)
  - E:\cargo-targets\zircon-plan08-custom-product-png-0704\debug\deps\zircon_runtime-0a7825d39d44b0c4.exe graphics::tests::render_product_mesh_cache::project_plugin_registry_material_passes_staged_cache::custom_product_png::export_custom_shading_model_deferred_lighting_product_png --ignored --exact --nocapture --test-threads=1 (2026-07-04 Custom shading-model deferred-lighting product readback PNG: passed 1/1 with 6309 filtered, 9.66s; PNG 641x240, 3871 bytes, SHA256 21188825B3FCEC7089BC198CDF89B53527332583FFAF5B3755317BF11EAD66F2, 4794 non-black pixels, 4554 dominant green pixels)
  - cargo test -p zircon_runtime --lib render_product_project_plugin_registry_material_passes_use_staged_prewarm_without_compile_miss --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-custom-shading-product-0701 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 Standard registry material-pass staged-cache regression after custom shading-model product path: passed 1/1 with repository warnings only)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_generated_shader_permutation_registry_document_exports_selected_plugin_shading_model_descriptors (2026-07-01 Plugin shading-model descriptor registry export: RED then passed)
  - python -m unittest tools.tests.test_zircon_build_plugin_carriers tools.tests.test_zircon_build_shader_prewarm (2026-07-01 Plugin shading-model descriptor registry export: passed, 41 tests)
  - python -m py_compile tools\zircon_build.py tools\zircon_build_shader_prewarm.py tools\tests\test_zircon_build_shader_prewarm.py tools\tests\test_zircon_build_plugin_carriers.py (2026-07-01 Plugin shading-model descriptor registry export: passed)
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/permutation_registry.rs::tests::shader_prewarm_permutation_registry_merges_custom_shading_model_descriptors (2026-07-01 Plugin shading-model descriptor registry export: added; Cargo result not claimed)
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_shading_model_descriptor.rs::runtime_15_shader_prewarm_plugin_shading_model_descriptor_registration_is_wired (2026-07-01 Plugin shading-model descriptor registry export: updated; Cargo result not claimed)
  - rustfmt --edition 2021 --check zircon_runtime/src/asset/tests/pipeline/manager/resource_revisions.rs (2026-07-01 Edited shader ResourceRecord revision export: passed; focused Cargo deferred while unrelated cargo/rustc lanes were active)
  - zircon_runtime/src/asset/tests/pipeline/manager/resource_revisions.rs::shader_reimport_exports_updated_revision_for_prewarm_registry (2026-07-02 selected plugin/source-registry Cargo-wrapper backfill: passed 1/1 with 5839 filtered)
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs.rs::tests::plugin_registration_inputs_collect_shading_model_descriptors (2026-07-02 selected plugin/source-registry Cargo-wrapper backfill: passed 1/1 with 5839 filtered)
  - rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph.rs; cargo test -p zircon_runtime render_product_skinned_mesh_gpu_morph_matches_cpu_baked_reference_pixels --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-morph-parity-0630 --message-format short --color never -- --nocapture --test-threads=1; generated test binary direct morph regression filter render_product_direct_mesh_ passed 2/2 (2026-06-30 Skinned Morph GPU-vs-CPU product parity: passed)
  - rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph.rs; cargo test -p zircon_runtime render_product_direct_mesh_ --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-morph-parity-0630 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-30 Morph GPU-vs-CPU product parity: passed 2/2, covering GPU source guard and CPU-baked reference pixel/readback parity)
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-support-repair-0630 --message-format short --color never (2026-06-30 Project/plugin registry product/material-pass staged-cache WGPU closeout: passed with repository warnings; status render_plan08_project_plugin_registry_product_material_passes_staged_cache_wgpu_passed_renderdoc_deferred)
  - rustfmt --edition 2021 --check --config skip_children=true on resource limits, request-device limits, GPUScene binding, HZB occlusion, temporal descriptor filtering, pipeline compile temporal tests, frame extract, product material-pass assertions, and UI text layout helper files (2026-06-30 Project/plugin registry product/material-pass staged-cache WGPU closeout: passed)
  - cargo test -p zircon_runtime --lib --target-dir E:\cargo-targets\zircon-plan08-support-repair-0630 offscreen_device_limits -- --nocapture --test-threads=1 (2026-06-30 HZB/mesh storage-buffer device-limit synthesis: passed, 3/3)
  - cargo test -p zircon_runtime --lib --target-dir E:\cargo-targets\zircon-plan08-support-repair-0630 hzb_occlusion_limit_gate -- --nocapture --test-threads=1 (2026-06-30 HZB occlusion storage-buffer layout gate: passed, 2/2)
  - cargo test -p zircon_runtime --lib --target-dir E:\cargo-targets\zircon-plan08-support-repair-0630 taa_resolve_ -- --nocapture --test-threads=1 (2026-06-30 temporal history binding filtering: passed, 10/10)
  - cargo test -p zircon_runtime --lib --target-dir E:\cargo-targets\zircon-plan08-support-repair-0630 render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings -- --nocapture --test-threads=1 (2026-06-30 GPUScene compute-visible storage layout budget: passed, 1/1)
  - cargo test -p zircon_runtime --lib --target-dir E:\cargo-targets\zircon-plan08-support-repair-0630 render_view_select_camera_descriptor_preserves_explicit_anti_alias -- --nocapture --test-threads=1 (2026-06-30 frame extract explicit anti-alias preservation: passed, 1/1)
  - cargo test -p zircon_runtime --lib --target-dir E:\cargo-targets\zircon-plan08-support-repair-0630 render_product_project_plugin_registry_materials_use_staged_prewarm_without_compile_miss -- --nocapture --test-threads=1 (2026-06-30 raw project/plugin registry product staged-cache WGPU closeout: passed, 1/1)
  - cargo test -p zircon_runtime --lib --target-dir E:\cargo-targets\zircon-plan08-support-repair-0630 render_product_project_plugin_registry_material_passes_use_staged_prewarm_without_compile_miss -- --nocapture --test-threads=1 (2026-06-30 material-pass project/plugin registry staged-cache WGPU closeout: passed, 1/1; a later duplicate rerun was inconclusive during Cargo rebuild/lock and is not counted as a regression)
  - cargo test -p zircon_runtime render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-registry-second-launch-0701 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 Project/plugin registry material-pass second-launch miss=0: timed out after about 1204s with no test result and no reusable `zircon_runtime` test binary; matching target-dir cargo/rustc processes were stopped, so no WGPU pass is claimed; a short rerun using `E:\cargo-targets\zircon-plan08-support-repair-0630` was stopped at 2026-07-01 04:56 +08:00 while still compiling dependencies, with no stdout test result; a second bounded rerun using `E:\cargo-targets\zircon-plan08-registry-second-launch-0701` was stopped at 2026-07-01 05:03 +08:00 while still compiling/linking `zircon_runtime`, with warning-only stderr and no stdout test result; a final bounded rerun launched from `target\codex-plan08-rerun-0701-final` against the same target dir was stopped at 2026-07-01 05:15 +08:00 while still compiling/linking `zircon_runtime`, with no `zircon_runtime` test binary produced, empty stdout, and warning-only stderr)
  - E:\cargo-targets\zircon-plan08-custom-shading-second-launch-guard-0701\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss --test-threads=1 --nocapture (2026-07-01 Project/plugin registry material-pass second-launch direct-binary WGPU backfill: passed, 1/1, 5806 filtered, 14.95s; status render_plan08_project_plugin_registry_material_passes_second_launch_direct_binary_wgpu_passed_renderdoc_deferred)
  - E:\cargo-targets\zircon-plan08-custom-shading-second-launch-guard-0701\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe runtime_15_shader_prewarm_project_plugin_registry_material_passes_staged_cache_is_wired --test-threads=1 --nocapture (2026-07-01 Project/plugin registry material-pass structure guard after direct-binary backfill: passed, 1/1, 5806 filtered, 0.44s)
  - cargo test -p zircon_runtime --lib render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-02 Project/plugin registry material-pass second-launch Cargo-wrapper WGPU backfill: passed 1/1 with 5842 filtered in 12.86s; status render_plan08_project_plugin_registry_material_passes_second_launch_cargo_wrapper_wgpu_passed_renderdoc_deferred)
  - cargo test -p zircon_runtime --lib render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-product-default-0703 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 Project/plugin registry material-pass second-launch default-feature WGPU backfill: passed 1/1 with 6177 filtered in 14.96s after fixing the standard registry fixture to use registry_material_pass_runtime_surface_source / fn zr_material_surface(; status render_plan08_project_plugin_registry_material_passes_second_launch_default_features_wgpu_passed_renderdoc_deferred)
  - cargo test -p zircon_runtime --lib render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-product-default-0703 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 Project/plugin registry material-pass second-launch no-default regression after surface-source fix: passed 1/1 with 6171 filtered in 12.75s)
  - cargo test -p zircon_runtime --lib registry_material_pass_runtime_surface_source_uses_surface_entry_contract --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-product-default-0703 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 registry material-pass runtime surface entry contract: passed 1/1 with 6171 filtered)
  - cargo test -p zircon_runtime --lib graphics::tests::render_product_mesh_cache::project_plugin_registry_staged_cache::render_product_project_plugin_registry_materials_use_staged_prewarm_without_compile_miss --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-product-default-0703 --message-format short --color never -- --exact --nocapture --test-threads=1 (2026-07-03 Plan 08 staged-prewarm product sweep default-feature current WGPU refresh: passed 1/1 with 6197 filtered, test body 5.65s after raw registry runtime surface/key fix and a 10m 04s build)
  - cargo test -p zircon_runtime --lib staged_prewarm_without_compile_miss --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-product-default-0703 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 Plan 08 staged-prewarm product sweep default-feature current WGPU refresh: initial current rerun failed 10/11 at raw project/plugin registry product; final rerun passed 11/11 with 6187 filtered, test body 48.90s after a 7m 36s build; status render_plan08_staged_prewarm_product_sweep_default_features_current_wgpu_refresh_passed_renderdoc_deferred)
  - E:\cargo-targets\zircon-plan08-project-plugin-png-0703\debug\deps\zircon_runtime-0a7825d39d44b0c4.exe graphics::tests::render_product_mesh_cache::project_plugin_registry_material_passes_staged_cache::product_png::export_project_plugin_registry_material_passes_product_png --ignored --exact --nocapture --test-threads=1 (2026-07-04 Project/plugin registry material-pass product readback PNG: passed 1/1 with 6290 filtered and 6.58s; status render_plan08_project_plugin_registry_material_passes_product_readback_png_passed_renderdoc_deferred)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs zircon_runtime/src/graphics/tests/render_product_mesh_cache/shading_model_parity.rs zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/render_plan08_three_shading_models_forward_deferred_parity.rs (2026-07-01 Plan 08 three shading-model forward/deferred product parity: passed; status render_plan08_three_shading_models_forward_deferred_parity_wgpu_passed_light_grid_fallback_renderdoc_deferred; guard runtime_15_render_plan08_three_shading_models_forward_deferred_parity_is_wired; RenderDoc/product capture deferred)
  - cargo test -p zircon_runtime --lib light_grid_external_fallback_buffers_satisfy_materialization_report --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 disabled clustered-lighting light-grid execution fallback: passed 1/1)
  - cargo test -p zircon_runtime --lib render_product_three_shading_models_forward_deferred_parity --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 Plan 08 three shading-model Forward + Deferred product parity: passed 1/1, 5818 filtered out)
  - cargo test -p zircon_runtime --lib render_product_three_shading_models_forward_deferred_parity --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-three-shading-default-0702 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-02 Plan 08 three shading-model forward/deferred product parity default-feature WGPU backfill: passed 1/1, 5876 filtered out, 11.81s; status render_plan08_three_shading_models_forward_deferred_parity_default_features_wgpu_passed_renderdoc_deferred; initial default-feature compile blocker fixed by a test-scope `SdfAtlasRect` import in `graphics/scene/scene_renderer/ui/text.rs`)
  - cargo test -p zircon_runtime --lib runtime_15_render_plan08_three_shading_models_forward_deferred_parity_is_wired --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 Plan 08 three shading-model structure/docs guard: passed 1/1, 5822 filtered after helper/fixture guard expansion; direct generated-binary rerun after final docs sync passed 1/1, 5825 filtered)
  - E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe runtime_15_render_plan08_three_shading_models_forward_deferred_parity_is_wired --test-threads=1 --nocapture (2026-07-02 Plan 08 three shading-model default-feature status/docs guard: direct generated-binary passed 1/1, 5873 filtered, 0.41s after the Cargo wrapper timed out during compile/link)
  - cargo test -p zircon_runtime --lib deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-01 Plan 08 Deferred project-shader/GBuffer probe fixture repair: passed 1/1, 5825 filtered; status render_plan08_deferred_project_shader_gbuffer_probe_wgpu_passed_renderdoc_deferred; earlier direct generated-binary rerun also passed 1/1 after an initial 120s Cargo build timeout)
  - python -m py_compile tools\zircon_build.py tools\tests\test_zircon_build_shader_prewarm_wrapper_orchestration.py (2026-06-30 Project/plugin registry production wrapper no-proxy WGPU run: passed; status render_plan08_project_plugin_registry_production_wrapper_no_proxy_wgpu_passed_product_renderdoc_deferred)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm_wrapper_orchestration.ZirconBuildShaderPrewarmWrapperOrchestrationTests.test_runtime_server_wrapper_uses_client_features_for_preview_binary (2026-06-30 Project/plugin registry production wrapper no-proxy WGPU run: passed; guard runtime_15_shader_prewarm_project_plugin_registry_wrapper_no_proxy_is_wired)
  - cargo test -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-wrapper-no-proxy-0630-guard --message-format short --color never runtime_15_shader_prewarm_project_plugin_registry_wrapper_no_proxy_is_wired -- --nocapture --test-threads=1 (2026-06-30 Project/plugin registry production wrapper no-proxy WGPU run: passed, 1/1 with repository warnings)
  - python -u tools\zircon_build.py --targets runtime --plugins native_dynamic_fixture --out target\codex-plan08-wrapper-no-proxy-0630 --mode debug --runtime-features target-server --jobs 1 --prewarm-shaders --validate-wgpu-shaders --shader-asset-root target\codex-plan08-wrapper-no-proxy-0630\project_assets --shader-quality-tier medium --shader-geometry-source static (2026-06-30 Project/plugin registry production wrapper no-proxy WGPU run: real no-proxy public command passed; runtime lib target-server, preview executable target-client, prewarm cargo run target-server, report 18/18 written and 18/18 WGPU module validated; RenderDoc/product remains deferred)
  - rustfmt --edition 2021 --check on render_product_mesh_cache.rs, render_product_mesh_cache/project_plugin_registry_staged_cache.rs, runtime_15_shader_prewarm_project_plugin_registry_product_staged_cache_is_wired, and test_file_budget/mod.rs (2026-06-30 Project/plugin registry product staged-cache miss=0: source/static pass; Cargo/WGPU execution timed out after about 904s with no result)
  - rustfmt --edition 2021 --check on mesh_pipeline_cache/ensure_pipeline.rs, runtime_15_shader_prewarm_project_plugin_registry_runtime_staged_cache_hit_is_wired, and test_file_budget/mod.rs (2026-06-30 Project/plugin registry runtime staged-cache hit: source/static pass; Cargo/WGPU execution deferred)
  - rustfmt --edition 2021 --check on VG model primitive ordinal encoding, mesh asset conversion, importer assertions, VG WGSL include, mesh shader source guard, and runtime_15_virtual_geometry_meshlet_vertex_ordinal_is_wired (2026-06-29 VirtualGeometry meshlet vertex ordinal: static pass; Cargo deferred)
  - Direct generated-binary focused runs on `virtual_geometry_vertex_ordinals`, `model_primitive_converts_to_mesh_asset_with_builtin_attributes`, `importer_decodes_obj_into_model_asset`, `importer_backfills_virtual_geometry_for_model_toml_without_dropping_base_mesh`, `importer_decodes_triangle_gltf_into_model_asset`, `default_importer_decodes_gltf_without_first_wave_plugin_fixture`, and `mesh_pipeline_virtual_geometry_template_source_declares_page_cluster_fetch_bindings` (2026-07-02 VirtualGeometry meshlet vertex ordinal: direct-binary asset/shader pass; ProjectAssetManager fixture source fixed, Cargo rerun deferred)
  - ProjectAssetManager VG fixture source guard and stale-binary audit (2026-07-02 status `render_plan08_virtual_geometry_project_asset_manager_fixture_source_guarded_cargo_rerun_deferred`; `asset_manager_imports_model_toml_with_virtual_geometry_payload` source expected fixture uses `assign_virtual_geometry_vertex_ordinals()`, stale binary timestamp `2026-07-02 05:27:25 +08:00` failed and is not counted)
  - cargo test -p zircon_runtime --lib asset::tests::pipeline::manager::model_import::asset_manager_imports_model_toml_with_virtual_geometry_payload --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-readback-0702 --message-format short --color never -- --exact --nocapture --test-threads=1 (2026-07-02 ProjectAssetManager VG fixture Cargo-wrapper rerun: status `render_plan08_virtual_geometry_project_asset_manager_fixture_cargo_wrapper_passed_renderdoc_deferred`; passed 1/1, 5911 filtered, repository-existing warnings only)
  - rustfmt --edition 2021 --check on runtime VG extract sidecar/context/snapshot builder, virtual_geometry plugin nanite page payload/automatic extract/provider/imported extract test, and runtime_15_virtual_geometry_asset_payload_decode_is_wired (2026-06-29 VirtualGeometry asset payload decode: static pass; Cargo deferred)
  - cargo check -q -p zircon_plugin_virtual_geometry_runtime --lib --target-dir target\codex-plan08-vg-asset-payload-decode-0629 --locked --jobs 1 (2026-06-29 VirtualGeometry asset payload decode: timed out after about 304s; no check result, not counted as passed)
  - rustfmt --edition 2021 --check on virtual_geometry_debug_snapshot payload DTO/re-export, production VG debug snapshot builder, GPUScene virtual-geometry ABI, mesh build resident upload owner, and runtime_15_virtual_geometry_cluster_payload_upload_is_wired (2026-06-29 VirtualGeometry cluster payload upload: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir target\codex-plan08-product-material-pass-cache-0629 --locked --jobs 1 (2026-06-29 VirtualGeometry cluster payload upload: timed out after about 304s; no check result, not counted as passed)
  - E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe graphics::scene::scene_renderer::mesh::build_mesh_draws::build::virtual_geometry_resident_upload::tests::virtual_geometry_cluster_words_follow_resident_page_payloads --exact --test-threads=1 --nocapture (2026-07-02 VirtualGeometry cluster payload upload direct-binary WGPU backfill: passed 1/1, 5881 filtered)
  - cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::mesh::build_mesh_draws::build::virtual_geometry_resident_upload::tests::virtual_geometry_cluster_words_follow_resident_page_payloads --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-upload-cargo-0703 --message-format short --color never -- --exact --nocapture --test-threads=1 (2026-07-03 VirtualGeometry resident/cluster upload Cargo-wrapper backfill: status `render_plan08_virtual_geometry_resident_cluster_upload_cargo_wrapper_passed_renderdoc_deferred`; passed 1/1, 6063 filtered, repository-existing warnings only)
  - rustfmt --edition 2021 --check on GPUScene layout/runtime/mod/virtual_geometry upload owner, mesh build resident upload, gpu_scene_sync, VirtualGeometry submission/execution projection, and runtime_15_virtual_geometry_resident_buffers_upload_is_wired (2026-06-29 VirtualGeometry resident buffers upload: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir target\codex-plan08-product-material-pass-cache-0629 --locked --jobs 1 (2026-06-29 VirtualGeometry resident buffers upload: passed in warmed target with existing warnings)
  - cargo test -q -p zircon_runtime --lib --target-dir target\codex-plan08-product-material-pass-cache-0629 --locked --jobs 1 render_gpu_scene_uploads_virtual_geometry_resident_buffers -- --exact --nocapture --test-threads=1 (2026-06-29 VirtualGeometry resident buffers upload: timed out after about 304s while compiling lib-test; no result, not counted as passed)
  - E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe graphics::scene::gpu_scene::virtual_geometry::tests::render_gpu_scene_uploads_virtual_geometry_resident_buffers --exact --test-threads=1 --nocapture (2026-07-02 VirtualGeometry resident buffers upload direct-binary WGPU backfill: passed 1/1 with 5881 filtered)
  - E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe graphics::scene::scene_renderer::mesh::build_mesh_draws::build::virtual_geometry_resident_upload::tests::virtual_geometry_page_rows_follow_submission_slots --exact --test-threads=1 --nocapture (2026-07-02 VirtualGeometry resident buffers upload direct-binary WGPU backfill: passed 1/1 with 5881 filtered)
  - cargo test -p zircon_runtime --lib graphics::scene::gpu_scene::virtual_geometry::tests::render_gpu_scene_uploads_virtual_geometry_resident_buffers --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701 --message-format short --color never -- --exact --nocapture --test-threads=1 (2026-07-03 VirtualGeometry resident/cluster upload Cargo-wrapper backfill: status `render_plan08_virtual_geometry_resident_cluster_upload_cargo_wrapper_passed_renderdoc_deferred`; passed 1/1, 6037 filtered, repository-existing warnings only)
  - cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::mesh::build_mesh_draws::build::virtual_geometry_resident_upload::tests::virtual_geometry_page_rows_follow_submission_slots --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-upload-cargo-0703 --message-format short --color never -- --exact --nocapture --test-threads=1 (2026-07-03 VirtualGeometry resident/cluster upload Cargo-wrapper backfill: status `render_plan08_virtual_geometry_resident_cluster_upload_cargo_wrapper_passed_renderdoc_deferred`; passed 1/1, 6060 filtered, repository-existing warnings only)
  - cargo test -p zircon_runtime --lib graphics::scene::gpu_scene::virtual_geometry::tests::render_gpu_scene_uploads_virtual_geometry_resident_buffers --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-upload-default-0703 --message-format short --color never -- --exact --nocapture --test-threads=1 (2026-07-03 VirtualGeometry resident/cluster upload default-feature Cargo-wrapper backfill: status `render_plan08_virtual_geometry_resident_cluster_upload_default_features_cargo_wrapper_passed_renderdoc_deferred`; warmed rerun passed 1/1, 6094 filtered, repository-existing warnings only after the initial compile/link timeout was not counted)
  - cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::mesh::build_mesh_draws::build::virtual_geometry_resident_upload::tests::virtual_geometry_page_rows_follow_submission_slots --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-upload-default-0703 --message-format short --color never -- --exact --nocapture --test-threads=1 (2026-07-03 VirtualGeometry resident/cluster upload default-feature Cargo-wrapper backfill: status `render_plan08_virtual_geometry_resident_cluster_upload_default_features_cargo_wrapper_passed_renderdoc_deferred`; passed 1/1, 6097 filtered, repository-existing warnings only)
  - cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::mesh::build_mesh_draws::build::virtual_geometry_resident_upload::tests::virtual_geometry_cluster_words_follow_resident_page_payloads --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-upload-default-0703 --message-format short --color never -- --exact --nocapture --test-threads=1 (2026-07-03 VirtualGeometry resident/cluster upload default-feature Cargo-wrapper backfill: status `render_plan08_virtual_geometry_resident_cluster_upload_default_features_cargo_wrapper_passed_renderdoc_deferred`; passed 1/1, 6097 filtered, repository-existing warnings only)
  - rustfmt --edition 2021 --check on GeometrySource descriptor, virtual_geometry plugin descriptor/manifest fixtures, GPUScene binding/runtime, mesh pipeline shader source fixtures, and runtime_15_virtual_geometry_page_cluster_shader_bindings_are_wired (2026-06-29 VirtualGeometry page/cluster shader bindings: passed)
  - python -m py_compile tools/tests/test_zircon_build_shader_prewarm.py (2026-06-29 VirtualGeometry page/cluster shader bindings: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_generated_shader_permutation_registry_document_exports_selected_plugin_descriptors (2026-06-29 VirtualGeometry page/cluster shader bindings: passed, 1 test)
  - cargo check -q -p zircon_runtime --lib --target-dir target\codex-plan08-vg-page-cluster-bindings-0629 --locked --jobs 1 (2026-06-29 VirtualGeometry page/cluster shader bindings: timed out after about 300s while compiling; no result, not counted as passed)
  - cargo test -q -p zircon_runtime --lib --target-dir target\codex-plan08-vg-page-cluster-bindings-0629 --locked --jobs 1 mesh_pipeline_virtual_geometry_template_source_declares_page_cluster_fetch_bindings -- --exact --nocapture --test-threads=1 (2026-06-29 VirtualGeometry page/cluster shader bindings: timed out after about 300s while compiling; no result, not counted as passed)
  - cargo test -q -p zircon_runtime --lib --target-dir target\codex-plan08-vg-page-cluster-bindings-0629 --locked --jobs 1 render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings -- --exact --nocapture --test-threads=1 (2026-06-29 VirtualGeometry page/cluster shader bindings: timed out after about 300s while compiling; no result, not counted as passed)
  - E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::shader_source::tests::mesh_pipeline_virtual_geometry_template_source_declares_page_cluster_fetch_bindings --exact --test-threads=1 --nocapture (2026-07-02 VirtualGeometry page/cluster shader bindings direct-binary backfill: passed 1/1, 5881 filtered)
  - E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe graphics::scene::gpu_scene::binding::tests::render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings --exact --test-threads=1 --nocapture (2026-07-02 VirtualGeometry page/cluster shader bindings direct-binary WGPU-layout backfill: passed 1/1, 5881 filtered)
  - cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::shader_source::tests::mesh_pipeline_virtual_geometry_template_source_declares_page_cluster_fetch_bindings --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701 --message-format short --color never -- --exact --nocapture --test-threads=1 (2026-07-03 VirtualGeometry page/cluster shader bindings Cargo-wrapper WGPU-layout backfill: passed 1/1, 5984 filtered, build 17m45s; an earlier same-command compile failed before tests on a concurrent `swash` dependency snapshot and is not counted)
  - cargo test -p zircon_runtime --lib graphics::scene::gpu_scene::binding::tests::render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701 --message-format short --color never -- --exact --nocapture --test-threads=1 (2026-07-03 VirtualGeometry page/cluster shader bindings Cargo-wrapper WGPU-layout backfill: passed 1/1, 5992 filtered, build 14m59s)
  - cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::shader_source::tests::mesh_pipeline_virtual_geometry_template_source_declares_page_cluster_fetch_bindings --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-upload-default-0703 --message-format short --color never -- --exact --nocapture --test-threads=1 (2026-07-03 VirtualGeometry page/cluster shader bindings default-feature Cargo-wrapper WGPU-layout backfill: status `render_plan08_virtual_geometry_page_cluster_shader_bindings_default_features_cargo_wrapper_wgpu_layout_passed_renderdoc_deferred`; passed 1/1, 6106 filtered, build 12m55s)
  - cargo test -p zircon_runtime --lib graphics::scene::gpu_scene::binding::tests::render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-upload-default-0703 --message-format short --color never -- --exact --nocapture --test-threads=1 (2026-07-03 VirtualGeometry page/cluster shader bindings default-feature Cargo-wrapper WGPU-layout backfill: status `render_plan08_virtual_geometry_page_cluster_shader_bindings_default_features_cargo_wrapper_wgpu_layout_passed_renderdoc_deferred`; passed 1/1, 6108 filtered, build 13m56s)
  - cargo test -p zircon_runtime --lib runtime_15_virtual_geometry_page_cluster_shader_bindings_are_wired --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-vg-upload-default-0703 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-03 VirtualGeometry page/cluster shader bindings default-feature Cargo-wrapper WGPU-layout backfill: status `render_plan08_virtual_geometry_page_cluster_shader_bindings_default_features_cargo_wrapper_wgpu_layout_passed_renderdoc_deferred`; passed 1/1, 6111 filtered, build 9m51s)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs (2026-06-29 Runtime custom geometry descriptor non-Base staged cache hit WGPU pipelines: passed)
  - cargo test -q -p zircon_runtime --lib --target-dir target\codex-plan08-runtime-custom-geometry-cache-0629 --locked --jobs 1 graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::ensure_pipeline::tests::runtime_custom_geometry_descriptor_non_base_pipelines_use_staged_prewarm_without_compile_miss -- --exact --nocapture --test-threads=1 (2026-06-29 Runtime custom geometry descriptor non-Base staged cache hit WGPU pipelines: passed, 1 test, 0 failed, 5467 filtered out)
  - cargo test -q -p zircon_runtime --lib --target-dir target\codex-plan08-runtime-custom-geometry-cache-0629 --locked --jobs 1 dynamic_api::shader_prewarm::tests::builtin_ -- --nocapture --test-threads=1 (2026-06-29 Runtime custom geometry descriptor non-Base staged cache hit WGPU pipelines: passed, 4 tests, 0 failed, 5464 filtered out)
  - cargo check -q -p zircon_runtime --lib --target-dir target\codex-plan08-runtime-custom-geometry-cache-0629 --locked --jobs 1 (2026-06-29 Runtime custom geometry descriptor staged cache hit WGPU pipeline: passed with existing warnings)
  - cargo test -q -p zircon_runtime --lib --target-dir target\codex-plan08-runtime-custom-geometry-cache-0629 --locked --jobs 1 graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::ensure_pipeline::tests::runtime_custom_geometry_descriptor_pipeline_uses_staged_prewarm_without_compile_miss -- --exact --nocapture --test-threads=1 (2026-06-29 Runtime custom geometry descriptor staged cache hit WGPU pipeline: passed, 1 test, 0 failed, 5463 filtered out)
  - rustfmt --edition 2021 --check on touched runtime module, WGPU construction, SceneRenderer construction, and mesh_pipeline_cache files (2026-06-29 Runtime custom geometry descriptor staged cache hit WGPU pipeline: passed)
  - cargo test -q -p zircon_runtime --lib --target-dir target\codex-plan08-product-material-pass-cache-0629 --locked --jobs 1 render_product_material_mesh_passes_second_launch_use_staged_prewarm_without_compile_miss -- --nocapture --test-threads=1 (2026-06-29 Product material mesh staged prewarm: passed, 1 test, 0 failed, 5480 filtered out)
  - cargo test -q -p zircon_runtime --lib --target-dir target\codex-plan08-product-material-pass-cache-0629 --locked --jobs 1 graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::ensure_pipeline::tests::runtime_custom_geometry_descriptor_non_base_pipelines_use_staged_prewarm_without_compile_miss -- --exact --nocapture --test-threads=1 (2026-06-29 Runtime custom geometry descriptor non-Base staged cache rerun on product target: passed, 1 test, 0 failed, 5480 filtered out)
  - cargo check -q -p zircon_runtime --lib --target-dir target\codex-plan08-product-material-pass-cache-0629 --locked --jobs 1 (2026-06-29 DepthPrepass pure-depth and GBuffer normal-target product graph contract: passed with existing warnings)
  - rustfmt --edition 2021 zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/depth_prepass_pure_depth_product_migration.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-27 Runtime pure-depth DepthPrepass product migration: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-depth-prepass-pure-depth-check --message-format short --color never (2026-06-27 Runtime pure-depth DepthPrepass product migration: passed with existing warnings)
  - cargo test -p zircon_runtime --lib depth_prepass_mesh_pipeline_creates_on_wgpu_device_with_template_shader --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-depth-prepass-pure-depth-check --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27 Runtime pure-depth DepthPrepass product migration: timed out after 15 minutes in Windows lib-test link, no test result)
  - rustfmt --edition 2021 zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs zircon_runtime/src/graphics/shader/variant_cache/mod.rs zircon_runtime/src/graphics/shader/mod.rs zircon_runtime/src/dynamic_api/shader_prewarm.rs zircon_runtime/src/dynamic_api/mod.rs zircon_runtime/src/bin/zircon_shader_prewarm/args.rs zircon_runtime/src/bin/zircon_shader_prewarm/run.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_module_validation.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-27 Prewarm opt-in WGPU shader-module validation: passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-27 Prewarm opt-in WGPU shader-module validation: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-27 Prewarm opt-in WGPU shader-module validation: passed, 13 tests)
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-wgpu-module-prewarm-check --message-format short --color never (2026-06-27 Prewarm opt-in WGPU shader-module validation: passed with existing warnings)
  - cargo check -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-wgpu-module-prewarm-check --message-format short --color never (2026-06-27 Prewarm opt-in WGPU shader-module validation: passed with existing warnings)
  - cargo run -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-wgpu-module-prewarm-run-build --message-format short --color never -- --project-root . --cache-dir target/codex-plan08-wgpu-module-prewarm-run/cache --report target/codex-plan08-wgpu-module-prewarm-run/report.json --builtin-fallback --validate-wgpu-modules --pretty (2026-06-27 Prewarm opt-in WGPU shader-module validation: timed out after 604s in Windows compile/run setup, no report, not counted as passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_acceptance.py tools/zircon_build_shader_prewarm_report_contract.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_command_contract.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py (2026-06-30 Prewarm opt-in WGPU render-pipeline validation: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract (2026-06-30 Prewarm opt-in WGPU render-pipeline validation: passed, 8 tests)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_dimension_summary_lines_format_wgpu_pipeline_validation_counts tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_build_command_forwards_wgpu_shader_pipeline_validation tools.tests.test_zircon_build_shader_prewarm_acceptance_contract.ZirconBuildShaderPrewarmAcceptanceContractTests.test_acceptance_contract_requires_pipeline_validation_when_enabled (2026-06-30 Prewarm opt-in WGPU render-pipeline validation: passed, 3 tests; status render_plan08_prewarm_wgpu_render_pipeline_validation_gate_focused_tests_passed_product_deferred; guard runtime_15_shader_prewarm_wgpu_render_pipeline_validation_is_wired)
  - E:\cargo-targets\zircon-plan08-pipeline-prewarm-0630-tests\debug\deps\zircon_runtime-d7a749791e79d863.exe mesh_prewarm_pipeline_validation --test-threads=1 --nocapture (2026-06-30 Prewarm opt-in WGPU render-pipeline validation: passed, 2 tests; creates Forward/GBuffer/DepthPrepass/Shadow/Velocity/TAA reactive-mask render pipelines)
  - E:\cargo-targets\zircon-plan08-pipeline-prewarm-0630-tests\debug\deps\zircon_runtime-d7a749791e79d863.exe wgpu_pipeline_validation --test-threads=1 --nocapture (2026-06-30 Prewarm opt-in WGPU render-pipeline validation: behavior tests passed 2/2 before docs sync; after docs sync the same filter covers structure guard status render_plan08_prewarm_wgpu_render_pipeline_validation_gate_focused_tests_passed_product_deferred)
  - cargo test -q -p zircon_runtime --lib mesh_prewarm_pipeline_validation --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-pipeline-prewarm-0630-tests -- --test-threads=1 --nocapture (2026-06-30 Prewarm opt-in WGPU render-pipeline validation: Cargo wrapper timed out while rebuilding/relinking the large lib-test binary; direct generated-binary run above is the counted WGPU evidence)
  - rustfmt --edition 2021 zircon_runtime/src/core/resource/manager/registry_export.rs zircon_runtime/src/core/resource/manager/mod.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_live_resource_registry.rs (2026-06-27 Live ResourceManager shader registry export)
  - cargo test -p zircon_runtime --lib resource_manager_exports_ready_records_for_kind_with_live_revisions --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-live-resource-registry-0627 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27 Live ResourceManager shader registry export: wrapper timed out after 1204s with no test result)
  - target/codex-plan08-live-resource-registry-0627/debug/deps/zircon_runtime-09d65f3d4d31577f.exe resource_manager_exports_ready_records_for_kind_with_live_revisions --test-threads=1 --nocapture (2026-06-27 Live ResourceManager shader registry export: passed, 1 test)
  - cargo test -p zircon_runtime --bin zircon_shader_prewarm shader_prewarm_resource_registry_overlay_uses_live_resource_manager_shader_revisions --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-live-resource-registry-0627 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27 Live ResourceManager shader registry export: passed, 1 test)
  - target/codex-plan08-live-resource-registry-0627/debug/deps/zircon_runtime-09d65f3d4d31577f.exe runtime_15_shader_prewarm_live_resource_manager_registry_export_is_wired --test-threads=1 --nocapture (2026-06-27 Live ResourceManager shader registry export: passed, 1 test)
  - rustfmt --edition 2021 zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs zircon_runtime/src/bin/zircon_shader_prewarm/run.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_auto_export.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_multi_root_dedupe.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Staged shader resource registry multi-root dedupe: passed)
  - python -m py_compile tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool shader asset-root plan visibility: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool shader asset-root plan visibility: passed, 15 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_plan_lists_runtime_fallback_handoff_paths (2026-06-28 Build-tool shader asset-root plan visibility fallback handoff extension: RED then passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool shader asset-root plan visibility fallback handoff extension: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_asset_root_plan_visibility.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool shader asset-root plan visibility: passed)
  - python -m py_compile tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Prewarm WGPU validation report summary: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_dimension_summary_lines_accept_rust_count_field_names tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_dimension_summary_lines_format_wgpu_module_validation_counts (2026-06-28 Prewarm WGPU validation report summary: passed, 2 tests)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Prewarm WGPU validation report summary: passed, 17 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs zircon_runtime/src/core/framework/render/shader/mod.rs zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs zircon_runtime/src/dynamic_api/shader_prewarm.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_validation_report_summary.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Prewarm WGPU validation report summary: passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool WGPU validation report contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_prints_summary_before_raising_nonzero_exit tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_report_contract_requires_wgpu_validation_when_requested tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_report_contract_accepts_wgpu_validation_counts (2026-06-28 Build-tool WGPU validation report contract: passed, 4 tests)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool WGPU validation report contract: passed, 20 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_report_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool WGPU validation report contract: passed)
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs::tests::render_shader_variant_prewarm_custom_ids_hit_staged_fallback_root (2026-06-28 Runtime custom id staged fallback lookup contract: added; Cargo deferred under milestone-first cadence)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Runtime custom id staged fallback lookup contract: passed)
  - source/docs anchor scan, conflict marker scan, trailing-whitespace scan, scoped git diff --check (2026-06-28 Runtime custom id staged fallback lookup contract: passed; diff-check only reported LF/CRLF warnings)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_command_contract (2026-06-28 Build-tool staged WGPU handoff command contract: RED then passed, 2 tests; status render_plan08_build_tool_staged_wgpu_handoff_command_contract_python_passed_cargo_deferred; guard runtime_15_shader_prewarm_staged_wgpu_handoff_command_contract_is_wired)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool staged WGPU handoff command contract: passed, 30 tests; test_full_staged_wgpu_handoff_keeps_generated_registries_and_roots locked in separate command-contract owner)
  - python -m py_compile tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Shader prewarm source provenance summary: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_dimension_summary_lines_format_source_provenance (2026-06-28 Shader prewarm source provenance summary: RED then passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Shader prewarm source provenance summary: passed, 21 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs zircon_runtime/src/core/framework/render/shader/mod.rs zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs zircon_runtime/src/dynamic_api/shader_prewarm.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_summary.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Shader prewarm source provenance summary: passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool source provenance report contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_report_contract_requires_source_provenance_when_requested tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_report_contract_accepts_source_provenance_counts (2026-06-28 Build-tool source provenance report contract: RED then passed, 3 tests)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool source provenance report contract: passed, 23 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_report_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool source provenance report contract: passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool shader resource registry export contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_uses_same_acceptance_entry_for_explicit_registry tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_requires_resource_records tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_accepts_wrapped_resources tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_accepts_raw_array (2026-06-28 Build-tool shader resource registry export contract: RED then passed, 5 tests; explicit registry handoff later moved into staged acceptance)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool shader resource registry export contract: passed, 27 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_export_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool shader resource registry export contract: passed)
  - cargo test -p zircon_runtime --lib runtime_15_shader_prewarm_resource_registry_export_contract_is_wired --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-registry-export-contract-0628 --message-format short --color never -- --nocapture (2026-06-28 Build-tool shader resource registry export contract: blocked before compile because Cargo.lock would need update under --locked; not counted as passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool shader resource registry report correlation: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_rejects_missing_report_source_locator tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_accepts_report_source_locator tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_ignores_builtin_report_sources (2026-06-28 Build-tool shader resource registry report correlation: RED then passed, 4 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool staged prewarm acceptance contract: RED then passed, 3 tests; status render_plan08_build_tool_staged_prewarm_acceptance_contract_python_passed_cargo_deferred; guard runtime_15_shader_prewarm_acceptance_contract_is_wired)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool staged prewarm runtime fallback layout contract: RED then passed, 5 tests; status render_plan08_build_tool_staged_prewarm_runtime_fallback_layout_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py (2026-06-28 Build-tool staged prewarm runtime fallback layout contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract.ZirconBuildShaderPrewarmAcceptanceContractTests.test_acceptance_contract_rejects_empty_success_report (2026-06-28 Build-tool staged prewarm nonempty success report acceptance: RED with old source-provenance error)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool staged prewarm nonempty success report acceptance: passed, 7 tests; status render_plan08_build_tool_staged_prewarm_nonempty_success_report_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract.ZirconBuildShaderPrewarmAcceptanceContractTests.test_acceptance_contract_requires_written_variant_identity (2026-06-28 Build-tool staged prewarm written variant identity acceptance: RED with old source-provenance error)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool staged prewarm written variant identity acceptance: passed, 9 tests; status render_plan08_build_tool_staged_prewarm_written_variant_identity_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract.ZirconBuildShaderPrewarmAcceptanceContractTests.test_acceptance_contract_rejects_partial_written_success_report (2026-06-28 Build-tool staged prewarm complete written count acceptance: RED with old source-provenance error, then passed after helper check)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool staged prewarm acceptance contract: passed, 30 tests after success-path handoff moved to acceptance helper)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_permutation_registry_contract (2026-06-28 Build-tool staged prewarm acceptance contract closeout: passed, 3 tests after old build-root report-validator patch was moved to the staged acceptance entry point)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool staged prewarm written variant identity acceptance: passed, 64 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool staged prewarm complete written count acceptance: passed, 65 tests; status render_plan08_build_tool_staged_prewarm_complete_written_count_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_report_contract_rejects_source_provenance_count_mismatch tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_report_contract_accepts_source_provenance_counts (2026-06-28 Build-tool source provenance totals match contract: RED then passed, 2 tests; status render_plan08_build_tool_source_provenance_totals_match_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool source provenance totals match contract: passed, 66 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool source provenance report test owner split: passed, 32 tests; status render_plan08_build_tool_source_provenance_report_tests_owner_split_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool source provenance report test owner split: passed, 66 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_report_contract.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_source_provenance_contract.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py tools/tests/test_zircon_build_shader_prewarm_command_contract.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_permutation_registry_contract.py (2026-06-28 Build-tool source provenance report test owner split: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract (2026-06-28 Build-tool WGPU validation totals match contract: RED first failed with RuntimeError not raised, then passed, 3 tests; status render_plan08_build_tool_wgpu_validation_totals_match_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool WGPU validation totals match contract: passed, 30 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build_shader_prewarm_report_contract.py tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-28 Build-tool WGPU validation totals match contract: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_report_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool WGPU validation totals match contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool product Base pass acceptance contract: RED first failed with unexpected keyword argument expected_pass_types, then passed, 32 tests; status render_plan08_build_tool_product_base_pass_acceptance_contract_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract (2026-06-28 Build-tool product Base pass acceptance contract: passed, 70 tests)
  - python -m py_compile tools/zircon_build_shader_prewarm_report_contract.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/zircon_build_shader_prewarm_acceptance.py (2026-06-28 Build-tool product Base pass acceptance contract: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_report_dimension_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_acceptance_contract.rs (2026-06-28 Build-tool product Base pass acceptance contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool product material mesh pass acceptance contract: RED then passed, 12 tests; status render_plan08_build_tool_product_material_mesh_pass_acceptance_contract_python_passed_cargo_deferred; guard runtime_15_shader_prewarm_acceptance_contract_is_wired)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool product material mesh pass acceptance contract: passed, 90 tests)
  - python -m py_compile tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py (2026-06-28 Build-tool product material mesh pass acceptance contract: passed with PYTHONPYCACHEPREFIX isolation)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_acceptance_contract.rs (2026-06-28 Build-tool product material mesh pass acceptance contract: passed)
  - source/docs anchor scan, conflict marker scan, trailing-whitespace scan, line-count scan, scoped git diff --check (2026-06-28 Build-tool product material mesh pass acceptance contract: passed; diff-check only reported LF/CRLF warnings)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool cache quality/geometry identity contract: RED first failed with unexpected keyword argument expected_quality_tiers/expected_geometry_sources, then passed, 29 tests; status render_plan08_build_tool_cache_quality_geometry_identity_contract_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build_shader_prewarm_cache_artifacts.py tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py (2026-06-28 Build-tool cache quality/geometry identity contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract (2026-06-28 Build-tool cache quality/geometry identity contract: passed, 73 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs (2026-06-28 Build-tool cache quality/geometry identity contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool cache dimension combination contract: RED first failed with RuntimeError not raised, then passed, 20 tests; status render_plan08_build_tool_cache_dimension_combination_contract_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool cache dimension combination contract: passed, 30 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract (2026-06-28 Build-tool cache dimension combination contract: passed, 74 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build_shader_prewarm_cache_artifacts.py tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py (2026-06-28 Build-tool cache dimension combination contract: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs (2026-06-28 Build-tool cache dimension combination contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool cache custom id combination contract: RED first failed with RuntimeError not raised, then passed, 21 tests; status render_plan08_build_tool_cache_custom_id_combination_contract_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract (2026-06-28 Build-tool cache custom id combination contract: passed, 31 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract (2026-06-28 Build-tool cache custom id combination contract: passed, 75 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build_shader_prewarm_cache_artifacts.py tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py (2026-06-28 Build-tool cache custom id combination contract: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs (2026-06-28 Build-tool cache custom id combination contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_report_contract.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/zircon_build_shader_prewarm_acceptance.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py tools/tests/test_zircon_build_shader_prewarm_command_contract.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_permutation_registry_contract.py (2026-06-28 Build-tool staged prewarm acceptance contract closeout: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_acceptance_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool staged prewarm acceptance contract closeout: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool shader resource registry report correlation: passed, 30 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_report_correlation.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool shader resource registry report correlation: passed)
  - cargo test -p zircon_runtime --lib runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-registry-report-correlation-0628 --message-format short --color never -- --nocapture (2026-06-28 Build-tool shader resource registry report correlation: timed out after 120 seconds with no test result; no residual cargo/rustc process; not counted as passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool resource registry written source-label correlation: RED first failed with RuntimeError not raised, then passed, 28 tests; status render_plan08_build_tool_resource_registry_written_source_label_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-28 Build-tool resource registry usable shader revision contract: RED first failed with RuntimeError not raised for non-Shader and zero-revision records, then passed, 30 tests; status render_plan08_build_tool_resource_registry_usable_shader_revision_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool resource registry contract test owner split: passed, 30 tests; status render_plan08_build_tool_resource_registry_contract_tests_owner_split_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool resource registry ResourceRecord wire-shape contract: RED first failed with RuntimeError not raised, then passed, 10 tests; status render_plan08_build_tool_resource_registry_record_shape_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool resource registry enum wire-shape contract: RED first failed with TypeError on dict enum, then passed, 11 tests; status render_plan08_build_tool_resource_registry_enum_wire_shape_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool resource registry numeric width contract: RED first failed with RuntimeError not raised for u64/u32 overflow, then passed, 13 tests; status render_plan08_build_tool_resource_registry_numeric_width_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool resource registry locator wire-shape contract: RED first failed with RuntimeError not raised and a rejected valid duplicate-separator path, then passed, 16 tests; status render_plan08_build_tool_resource_registry_locator_wire_shape_python_passed_cargo_deferred)
  - PYTHONPYCACHEPREFIX=%TEMP%\zircon-codex-pycache-plan08-locator python -m py_compile tools\zircon_build_shader_prewarm.py tools\zircon_build_shader_resource_registry.py tools\tests\test_zircon_build_shader_prewarm_resource_registry_contract.py (2026-06-28 Build-tool resource registry locator wire-shape contract: passed after local __pycache__ lock was bypassed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract (2026-06-28 Build-tool resource registry locator wire-shape contract: passed, 87 tests)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool shader resource registry-backed locator correlation: RED first failed with RuntimeError not raised for missing lib/package report source locators, then passed, 18 tests; status render_plan08_build_tool_resource_registry_backed_locator_correlation_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Resource registry ready shader revision contract: passed, 19 tests; status render_plan08_resource_registry_ready_shader_revision_contract_python_static_passed_cargo_deferred; includes test_validate_registry_export_contract_rejects_non_ready_report_source_record)
  - python -m py_compile tools/zircon_build_shader_resource_registry.py tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py (2026-06-28 Resource registry ready shader revision contract: passed)
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs::shader_prewarm_resource_registry_overlay_uses_ready_shader_revisions_only (2026-06-28 Resource registry ready shader revision contract: added; focused Cargo deferred while external cargo/rustc lanes were active)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_acceptance_contract.ZirconBuildShaderPrewarmAcceptanceContractTests.test_acceptance_contract_rejects_duplicate_written_variant_identity (2026-06-28 Build-tool written variant uniqueness contract: RED first failed with RuntimeError not raised, then passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract.ZirconBuildShaderPrewarmCacheContractTests.test_validate_cache_artifact_contract_rejects_duplicate_written_variant_identity (2026-06-28 Build-tool written variant uniqueness contract: RED first failed with RuntimeError not raised, then passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm_dimension_contract (2026-06-28 Build-tool shader prewarm report dimension contract: RED then passed, 4 tests)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py (2026-06-28 Build-tool shader prewarm report dimension contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_dimension_contract (2026-06-28 Build-tool shader prewarm report dimension contract: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_report_dimension_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool shader prewarm report dimension contract: passed)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_dimension_contract (2026-06-28 Build-tool shader prewarm report dimension complete-count contract: RED then passed, 7 tests; status render_plan08_build_tool_report_dimension_complete_counts_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_dimension_contract (2026-06-28 Build-tool shader prewarm report dimension totals match contract: RED then passed, 8 tests; status render_plan08_build_tool_report_dimension_totals_match_python_passed_cargo_deferred)
  - PYTHONDONTWRITEBYTECODE=1 python -m unittest tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_prewarm_acceptance_contract tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_command_contract tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract tools.tests.test_zircon_build_shader_prewarm_source_provenance_contract tools.tests.test_zircon_build_shader_prewarm_wgpu_report_contract tools.tests.test_zircon_build_shader_prewarm_resource_registry_contract (2026-06-28 Build-tool shader prewarm report dimension totals match contract: passed, 92 tests)
  - python -m py_compile tools/zircon_build_shader_prewarm_report_contract.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py (2026-06-28 Build-tool shader prewarm report dimension totals match contract: passed with isolated PYTHONPYCACHEPREFIX)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_report_dimension_contract.rs (2026-06-28 Build-tool shader prewarm report dimension totals match contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_uses_same_acceptance_entry_for_explicit_registry tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool shader prewarm cache artifact contract: RED then passed, 5 tests; explicit registry handoff later moved into staged acceptance)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py (2026-06-28 Build-tool shader prewarm cache artifact contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool shader prewarm cache artifact contract: passed, 38 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Build-tool shader prewarm cache artifact contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Prewarm report cache identity contract: RED then passed, 8 tests)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py (2026-06-28 Prewarm report cache identity contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Prewarm report cache identity contract: passed, 41 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs zircon_runtime/src/core/framework/render/shader/mod.rs zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Prewarm report cache identity contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Prewarm cache runtime layout contract: RED then passed, 10 tests)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py (2026-06-28 Prewarm cache runtime layout contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Prewarm cache runtime layout contract: passed, 43 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Prewarm cache runtime layout contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Prewarm cache hash shape contract: RED then passed, 11 tests)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/zircon_build_shader_prewarm_cache_artifacts.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py tools/tests/test_zircon_build_shader_prewarm_cache_contract.py (2026-06-28 Prewarm cache hash shape contract: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_dimension_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Prewarm cache hash shape contract: passed, 44 tests)
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Prewarm cache hash shape contract: passed)
- rustfmt --edition 2021 zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs zircon_runtime/src/core/framework/render/shader/mod.rs zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/core/runtime/diagnostics/render_stats_store/shader_variant.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/queue_profile.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs zircon_runtime/src/graphics/tests/render_product_mesh_cache/staged_prewarm/mod.rs (2026-06-27 runtime shader variant dimension correlation; current owner path updated by 2026-06-30 folder-backed follow-up)
  - cargo test -p zircon_runtime --lib gpu_skinning --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-runtime-dimension-correlation-check-0627 --message-format short --color never -- --nocapture (2026-06-27 runtime shader variant dimension correlation: passed, 4 tests)
  - target/codex-plan08-runtime-dimension-correlation-check-0627/debug/deps/zircon_runtime-09d65f3d4d31577f.exe shader_variant_miss_report_groups_runtime_outcomes_by_variant_dimensions --test-threads=1 --nocapture (2026-06-27 runtime shader variant dimension correlation: passed)
  - target/codex-plan08-runtime-dimension-correlation-check-0627/debug/deps/zircon_runtime-09d65f3d4d31577f.exe render_product_base_mesh_second_launch_uses_staged_prewarm_without_compile_miss --test-threads=1 --nocapture (2026-06-27 runtime shader variant dimension correlation: passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-27 build-tool prewarm dimension summary: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-27 build-tool prewarm dimension summary: passed, 4 tests)
  - python tools/zircon_build.py --targets runtime --out target/codex-plan08-build-tool-prewarm-summary-dry-run --mode debug --prewarm-shaders --dry-run (2026-06-27 build-tool prewarm dimension summary: passed)
  - rustfmt --edition 2021 zircon_runtime/src/bin/zircon_shader_prewarm/args.rs zircon_runtime/src/bin/zircon_shader_prewarm/run.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/permutation_registry.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_permutation_registry.rs (2026-06-27 Shader permutation registry overlay: passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-27 Shader permutation registry overlay: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-27 Shader permutation registry overlay: passed, 5 tests)
  - python tools/zircon_build.py --targets runtime --out target/codex-plan08-permutation-registry-dry-run --mode debug --prewarm-shaders --shader-permutation-registry Project/shader_permutation_registry.json --dry-run (2026-06-27 Shader permutation registry overlay: passed)
  - cargo test -p zircon_runtime --bin zircon_shader_prewarm shader_prewarm_permutation_registry --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-permutation-registry-0627 --color never -- --nocapture (2026-06-27 Shader permutation registry overlay: passed, 2 tests, existing warnings)
  - cargo test -p zircon_runtime --lib runtime_15_shader_prewarm_permutation_registry_overlay_is_wired --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-permutation-registry-0627 --color never -- --nocapture (2026-06-27 Shader permutation registry overlay: passed, 1 test, existing warnings)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_plugin_carriers.py (2026-06-27 Plugin shader permutation registry auto-export: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_plugin_carriers (2026-06-27 Plugin shader permutation registry auto-export: passed, 13 tests)
  - python tools/zircon_build.py --targets runtime,plugins --plugins virtual_geometry --out target/codex-plan08-plugin-permutation-registry-dry-run --mode debug --prewarm-shaders --dry-run (2026-06-27 Plugin shader permutation registry auto-export: passed)
  - cargo check -q -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-plugin-permutation-registry-check --color never (2026-06-27 Plugin shader permutation registry auto-export: passed with existing warnings)
  - cargo test -q -p zircon_plugin_virtual_geometry_runtime --manifest-path zircon_plugins/Cargo.toml virtual_geometry_registration_contributes_render_feature_descriptor --locked --jobs 1 --target-dir target/codex-plan08-plugin-permutation-registry-plugin --color never -- --nocapture (2026-06-27 Plugin shader permutation registry auto-export: passed, 1 test)
  - python -m py_compile tools/zircon_build.py tools/tests/test_zircon_build_plugin_carriers.py (2026-06-27 Plugin shading-model descriptor registration: passed)
  - python -m unittest tools.tests.test_zircon_build_plugin_carriers (2026-06-27 Plugin shading-model descriptor registration: passed, 3 tests)
  - cargo check -q -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plan08-shading-model-descriptor-check --color never (2026-06-27 Plugin shading-model descriptor registration: passed with existing warnings)
  - cargo test -q -p zircon_runtime --lib shading_model --no-default-features --features target-server --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plan08-shading-model-descriptor-check --color never -- --nocapture (2026-06-27 Plugin shading-model descriptor registration: blocked during lib-test compile by existing `UiInputMethodSurroundingTextError` / `StdError` thiserror source drift; not counted as passed)
  - python -m py_compile tools/zircon_build.py tools/tests/test_zircon_build_plugin_carriers.py (2026-06-27 Plugin geometry-source descriptor registration: passed)
  - python -m unittest tools.tests.test_zircon_build_plugin_carriers (2026-06-27 Plugin geometry-source descriptor registration: passed, 4 tests)
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir D:/cargo-targets/zircon-plan08-geometry-source-descriptor-check --message-format short --color never (2026-06-27 Plugin geometry-source descriptor registration: passed with existing warnings)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime --locked --jobs 1 --target-dir D:/cargo-targets/zircon-plan08-geometry-source-descriptor-plugin-check --message-format short --color never (2026-06-27 Plugin geometry-source descriptor registration: passed with existing warnings)
  - cargo test -p zircon_runtime --lib runtime_15_shader_prewarm_plugin_geometry_source_descriptor_registration_is_wired --no-default-features --features target-server --locked --jobs 1 --target-dir D:/cargo-targets/zircon-plan08-geometry-source-descriptor-check --message-format short --color never -- --nocapture (2026-06-27 Plugin geometry-source descriptor registration: timed out after 1204s during lib-test compile/link; no test result)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_plugin_carriers.py (2026-06-27 Plugin shader asset roots auto-export: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_plugin_carriers (2026-06-27 Plugin shader asset roots auto-export: passed, 20 tests)
  - python tools/zircon_build.py --targets runtime,plugins --plugins native_dynamic_fixture --out target/codex-plan08-plugin-asset-roots-dry-run --mode debug --prewarm-shaders --dry-run (2026-06-27 Plugin shader asset roots auto-export: passed)
  - zircon_runtime/src/tests/plugin_extensions/package_manifest_declarations.rs::plugin_package_manifest_declares_custom_shading_model_descriptors
  - zircon_runtime/src/tests/plugin_extensions/package_manifest_declarations.rs::plugin_package_manifest_declares_custom_geometry_source_descriptors
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_metadata.rs::runtime_plugin_registration_collects_package_manifest_declared_runtime_contributions
  - zircon_runtime/src/graphics/material/shading_models/registry.rs::tests::shading_model_registry_rejects_plugin_descriptor_in_builtin_id_range
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_shading_model_descriptor.rs::runtime_15_shader_prewarm_plugin_shading_model_descriptor_registration_is_wired
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_geometry_source_descriptor.rs::runtime_15_shader_prewarm_plugin_geometry_source_descriptor_registration_is_wired
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_shader_defs_accept_legacy_flags_and_typed_values
  - zircon_runtime/src/asset/tests/assets/shader_readiness.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs::zshader_typed_shader_definition_rows_validate_kind_and_value
  - zircon_runtime/src/asset/tests/project/zmeta.rs::project_manager_imports_compound_zshader_package_with_subassets
  - 2026-05-26 typed shader definitions: rustfmt, focused shader tests, compound zshader test, and runtime lib-test check passed on D:/cargo-targets/zircon-typed-shader-defs
  - cargo test -p zircon_runtime --lib render_product_assets_shader_defs_accept_legacy_flags_and_typed_values --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-25 typed shader definitions: passed, 1 passed)
  - cargo test -p zircon_runtime --lib zshader_typed_shader_definition_rows_validate_kind_and_value --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-25 typed shader definitions: passed, 1 passed)
  - cargo test -p zircon_runtime --lib project_manager_imports_compound_zshader_package_with_subassets --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-25 typed shader definitions: passed, 1 passed)
  - cargo test -p zircon_runtime --lib shader_readiness --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-25 typed shader definitions: passed, 5 passed)
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs --message-format short --color never (2026-05-25 typed shader definitions: passed with existing warnings)
  - cargo test -p zircon_runtime --lib shader_readiness --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1 (2026-05-25 shader readiness report: passed, 5 passed)
  - cargo test -p zircon_runtime --lib shader --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1 (2026-05-25 shader readiness report: passed, 24 passed)
  - cargo test -p zircon_runtime --locked render_product_assets
  - cargo check -p zircon_runtime --lib --locked
  - zircon_runtime/src/core/framework/render/shader/geometry_source.rs::tests::render_shader_geometry_source_ids_reserve_builtin_segment
  - zircon_runtime/src/core/framework/render/shader/geometry_source.rs::tests::render_shader_geometry_source_descriptors_cover_builtin_segment
  - zircon_runtime/src/core/framework/render/shader/geometry_source.rs::tests::render_shader_geometry_source_descriptors_report_shape_requirements
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_geometry_source_descriptor.rs::runtime_15_render_shader_geometry_source_descriptor_contract_is_complete
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs::tests::shader_prewarm_args_default_to_static_geometry_source
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs::tests::shader_prewarm_args_expand_all_builtin_geometry_sources
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs::shader_prewarm_asset_root_manifest_expands_requested_geometry_sources
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs::shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs::shader_prewarm_builtin_fallback_manifest_expands_requested_geometry_sources
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs::runtime_15_shader_prewarm_geometry_source_enumeration_is_wired
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs::runtime_15_shader_prewarm_builtin_standard_material_template_source_is_wired
  - zircon_runtime/src/graphics/shader/template/tests.rs::render_shader_template_assembles_static_and_skinned_geometry_sources
  - zircon_runtime/src/graphics/shader/template/tests.rs::render_shader_template_assembles_standard_material_surface_source
  - zircon_runtime/src/graphics/shader/template/tests.rs::render_shader_template_validates_standard_material_wgsl_with_naga
  - zircon_runtime/src/graphics/shader/template/tests.rs::render_shader_template_clips_alpha_for_masked_standard_material_passes
  - zircon_runtime/src/graphics/shader/template/tests.rs::render_shader_template_specializes_depth_and_velocity_passes
  - zircon_runtime/src/graphics/shader/template/tests.rs::render_shader_template_rejects_reserved_material_symbols
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_standard_material_template_source_assembles_forward_base_source
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_template_source_hashes_include_template_revision
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_standard_material_template_source_uses_requested_geometry_source
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_velocity_template_source_uses_previous_position_vertex_input
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_taa_reactive_mask_template_source_uses_material_surface_without_lighting
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs::tests::mesh_pipeline_template_source_hashes_feed_disk_and_module_keys
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_separates_geometry_sources
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs::render_mesh_draw_processor_uses_batch_geometry_source_for_pipeline_variant_key
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly.rs::runtime_15_render_shader_template_assembly_is_folder_backed
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs::tests::taa_reactive_mask_shader_key_includes_shader_variant_identity_and_source_hash
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs::tests::taa_reactive_mask_pipeline_declares_template_entries_and_static_vertex_layout
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs::tests::shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs::tests::shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_mesh_pass_processors.rs::runtime_15_mesh_pass_processors_are_folder_backed
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs::tests::builtin_fallback_shader_prewarm_manifest_uses_mesh_template_source
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs::tests::builtin_standard_material_shader_prewarm_manifest_projects_material_features
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs::tests::builtin_standard_material_shader_prewarm_manifest_projects_geometry_source
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs::runtime_15_builtin_fallback_prewarm_uses_template_source
  - rustfmt --edition 2021 zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs zircon_runtime/src/bin/zircon_shader_prewarm/run.rs (2026-06-25 builtin fallback multi-geometry prewarm slice: passed)
  - cargo check -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-builtin-fallback-multigeometry-prewarm-check --message-format short --color never (2026-06-25 builtin fallback multi-geometry prewarm slice: progress-only exit 1 twice, no Rust diagnostics, not counted as passed)
  - cargo check -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-builtin-template-multigeometry-prewarm-check --message-format short --color never (2026-06-25 builtin fallback multi-geometry prewarm slice warmed-target retry: timed out after 304s, no runnable binary, not counted as passed)
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs::tests::shader_prewarm_args_parse_custom_shading_model_plugin_ids
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs::tests::shader_prewarm_args_reject_builtin_shading_model_id_range
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs::shader_prewarm_asset_root_manifest_maps_custom_shading_model_plugin_ids
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs::tests::shader_prewarm_args_parse_custom_geometry_source_plugin_ids
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs::tests::shader_prewarm_args_reject_builtin_geometry_source_id_range
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs::shader_prewarm_asset_root_manifest_expands_custom_geometry_source_plugin_ids
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/geometry_source.rs::runtime_15_shader_prewarm_custom_geometry_source_id_is_wired
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs::shader_prewarm_asset_root_manifest_uses_zmeta_source_digest_revision
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs::shader_prewarm_asset_root_manifest_uses_raw_source_hash_revision
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs::shader_prewarm_asset_root_manifest_uses_resource_registry_revision_overlay
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_revision.rs::runtime_15_shader_prewarm_resource_registry_revision_overlay_is_wired
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs::runtime_15_shader_prewarm_custom_shading_model_id_is_wired
  - rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/{args,manifest,manifest/tests,run}.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs (2026-06-25 asset-root custom shading-model id prewarm slice: passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py (2026-06-25 asset-root custom shading-model id prewarm slice: passed)
  - python tools/zircon_build.py --targets runtime --out D:/zircon-shader-custom-id-dry-run --mode debug --prewarm-shaders --shader-shading-model-id custom:subsurface=16 --dry-run (2026-06-25 asset-root custom shading-model id prewarm slice: passed; generated prewarm command contains --shading-model-id custom:subsurface=16)
  - cargo check -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-custom-shading-model-prewarm-check --message-format short --color never (2026-06-25 asset-root custom shading-model id prewarm slice: passed with existing warnings)
  - cargo check -p zircon_runtime --tests --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-custom-shading-model-prewarm-check --message-format short --color never (2026-06-25 asset-root custom shading-model id prewarm slice: blocked by existing virtual_geometry_debug_snapshot_contract.rs RenderLayerSet/u32 type mismatch, not counted as passed)
  - cargo test -p zircon_runtime --bin zircon_shader_prewarm shader_prewarm_asset_root_manifest_maps_custom_shading_model_plugin_ids --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-custom-shading-model-prewarm-check (2026-06-25 asset-root custom shading-model id prewarm slice: repeated timeout / warning-only exit without test result, not counted as passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/args.rs zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/geometry_source.rs (2026-06-27 asset-root custom geometry-source id prewarm slice: passed after formatting)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py (2026-06-27 asset-root custom geometry-source id prewarm slice: passed)
  - python tools/zircon_build.py --targets runtime --out E:/zircon-custom-geometry-id-dry-run --mode debug --prewarm-shaders --shader-geometry-source-id custom:gpu-driven=4 --dry-run (2026-06-27 asset-root custom geometry-source id prewarm slice: passed; generated prewarm command contains --geometry-source-id custom:gpu-driven=4)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-asset-revision-prewarm-0625 --color never (2026-06-27 asset-root custom geometry-source id prewarm slice: passed with existing warnings)
  - cargo test -q -p zircon_runtime --bin zircon_shader_prewarm custom_geometry_source --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-asset-revision-prewarm-0625 --color never -- --nocapture (2026-06-27 asset-root custom geometry-source id prewarm slice: blocked before target tests by unrelated animation asset compile drift: AnimationAssetError/Infallible and AssetImportError::Parse(String) mismatch; not counted as passed)
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs::tests::render_shader_variant_key_packs_dimensions_stably
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs::tests::render_shader_feature_bits_reports_named_flags
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shading-model-check-0616 (2026-06-16 Plan 08 shader variant key contract slice: passed with existing warnings)
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs::tests::pipeline_key_derives_material_shader_variant_key
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_derives_material_shader_variant_key
  - rustfmt --edition 2021 zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs (2026-06-17 PipelineKey to ShaderVariantKey bridge slice: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shading-model-check-0616 (2026-06-17 PipelineKey to ShaderVariantKey bridge slice: passed with existing warnings)
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs::tests::render_shader_variant_cache_hits_disk_after_restart
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs::tests::render_shader_variant_cache_treats_corrupt_entry_as_miss_after_cleanup
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs::tests::render_shader_variant_prewarm_writes_disk_entries
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs::tests::render_shader_variant_prewarm_rejects_invalid_wgsl_before_disk_write
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_counts_variant_misses_and_memory_hits
  - rustfmt --edition 2021 on Plan 08 MS-M4-S1b touched files (2026-06-17 shader variant disk cache slice: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-variant-cache-check-0617 (2026-06-17 shader variant disk cache slice: passed with existing warnings)
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs::tests::velocity_mesh_shader_key_includes_shader_variant_identity_and_source_hash
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs::tests::velocity_mesh_pipeline_declares_template_entries_and_previous_position_vertex_slot
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs::tests::velocity_mesh_pipeline_creates_on_wgpu_device_with_template_shader
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs::tests::taa_reactive_mask_shader_key_includes_shader_variant_identity_and_source_hash
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs::tests::taa_reactive_mask_pipeline_declares_template_entries_and_static_vertex_layout
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs::tests::taa_reactive_mask_mesh_pipeline_creates_on_wgpu_device_with_template_shader
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs::tests::shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs::tests::shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-prewarm-check-0617 (2026-06-17 shader prewarm slice: passed with existing warnings)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-prewarm-bin-check-0617 (2026-06-17 shader prewarm slice: passed with existing warnings)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/shader/variant_key.rs and cargo check -p zircon_editor --lib --locked (2026-06-17 Workbench resize splitter validation exposed shader variant key GeometrySourceId owner import drift; passed after variant_key imports GeometrySourceId from geometry_source)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 (2026-06-17 asset-scanned shader prewarm slice: passed with existing warnings)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets/shaders --pretty (2026-06-17 asset-scanned shader prewarm slice: wrote 4/4 variants)
  - python tools\zircon_build.py --targets runtime --out D:\zircon-shader-asset-prewarm-dry-run --mode debug --prewarm-shaders --dry-run (2026-06-17 asset-scanned shader prewarm slice: command includes --asset-root ZirconEngine/assets)
  - rustfmt --edition 2021 zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs (2026-06-17 asset-scanned multi-pass prewarm slice: passed)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 (2026-06-17 asset-scanned multi-pass prewarm slice: passed with existing warnings)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets/shaders --pretty (2026-06-17 asset-scanned multi-pass prewarm slice: wrote 20/20 variants)
  - rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs (2026-06-17 asset-scanned material-feature prewarm slice: passed)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 (2026-06-17 asset-scanned material-feature prewarm slice: passed with existing warnings)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets --pretty (2026-06-17 asset-scanned material-feature prewarm slice: wrote 40/40 variants)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets/shaders --pretty (2026-06-17 asset-scanned material-feature prewarm regression: wrote 20/20 variants)
  - rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs (2026-06-17 asset-scanned shading-model prewarm slice: passed)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 (2026-06-17 asset-scanned shading-model prewarm slice: passed with existing warnings)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets --pretty (2026-06-17 asset-scanned shading-model prewarm slice: wrote 40/40 variants)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets/shaders --pretty (2026-06-17 asset-scanned shading-model prewarm regression: wrote 20/20 variants)
  - rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs (2026-06-17 asset-scanned initial revision prewarm slice: passed)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 (2026-06-17 asset-scanned initial revision prewarm slice: passed with existing warnings)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets --pretty (2026-06-17 asset-scanned initial revision prewarm slice: wrote 40/40 variants)
  - rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs (2026-06-17 asset-scanned alpha-blend pass filtering slice: passed)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 (2026-06-17 asset-scanned alpha-blend pass filtering slice: passed with existing warnings)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets --pretty (2026-06-17 asset-scanned alpha-blend pass filtering slice: wrote 40/40 variants)
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_separates_shader_quality_tiers
  - rustfmt --edition 2021 touched runtime shader-quality files (2026-06-17 runtime shader quality key wiring slice: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-quality-check-0617 (2026-06-17 runtime shader quality key wiring slice: passed with existing warnings)
  - rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/{args,manifest,run}.rs (2026-06-17 quality-tier prewarm enumeration slice: passed)
  - python -m py_compile tools/zircon_build.py (2026-06-17 quality-tier prewarm enumeration slice: passed)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-quality-prewarm-check-0617 (2026-06-17 quality-tier prewarm enumeration slice: passed with existing warnings)
  - python tools\zircon_build.py --targets runtime --out D:\zircon-shader-quality-prewarm-dry-run --mode debug --prewarm-shaders --shader-quality-tier high --shader-quality-tier ultra --dry-run (2026-06-17 quality-tier prewarm enumeration slice: command includes --quality-tier high --quality-tier ultra)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-quality-prewarm-check-0617 -- --asset-root examples/vampire/assets --quality-tier high --quality-tier ultra (2026-06-17 quality-tier prewarm runtime probe: timed out during build/run; no pass claimed)
  - cargo test -p zircon_runtime --lib runtime_15_shader_prewarm_manifest_tests_are_folder_backed --no-default-features --features core-min --locked: deferred in Runtime 15 M3 shader prewarm manifest test folder split
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --target-dir target\codex-plan08-builtin-template-prewarm-check (2026-06-24 asset-root builtin standard material template prewarm slice: passed with existing warnings)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --target-dir target\codex-plan08-builtin-template-multigeometry-prewarm-check (2026-06-24 asset-root builtin standard material multi-geometry prewarm slice: passed with existing warnings)
  - cargo test -q -p zircon_runtime --bin zircon_shader_prewarm shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source --no-run --no-default-features --features target-server --locked --target-dir target\codex-plan08-builtin-template-multigeometry-prewarm-check (2026-06-24 asset-root builtin standard material multi-geometry prewarm slice: passed test binary compile with existing warnings)
  - cargo check -q -p zircon_runtime --lib --no-default-features --features target-server --locked --target-dir target\codex-plan08-prewarm-wgsl-validation-check (2026-06-24 shader prewarm WGSL validation gate: passed with existing warnings)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --target-dir target\codex-plan08-prewarm-wgsl-validation-check (2026-06-24 shader prewarm WGSL validation gate bin path: passed with existing warnings)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --target-dir target\codex-plan08-prewarm-wgsl-validation-check -- --manifest <invalid> --cache-dir <temp>/cache --report <temp>/report.json --pretty (2026-06-24 shader prewarm WGSL validation gate invalid manifest probe: returned exit 2, report requested=1/written=0/failed=1, cache_files=0)
  - cargo test -q -p zircon_runtime --lib render_shader_variant_prewarm_rejects_invalid_wgsl_before_disk_write --no-run --no-default-features --features target-server --locked --target-dir target\codex-plan08-prewarm-wgsl-validation-check (2026-06-24 shader prewarm WGSL validation gate: timed out after 604s with no test compile result; residual target processes stopped; not counted as passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs zircon_runtime/src/core/framework/render/shader/mod.rs zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs (2026-06-27 prewarm dimension diagnostics: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-prewarm-diagnostics-check-0627 --message-format short --color never (2026-06-27 prewarm dimension diagnostics: passed with existing warnings)
  - cargo test -p zircon_runtime --lib render_shader_variant_prewarm_report_groups_written_and_failed_dimensions --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-prewarm-diagnostics-check-0627 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27 prewarm dimension diagnostics: timed out after 604s on warmed target with no test result and no generated lib-test binary; residual target processes stopped; not counted as passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/{mesh_pipeline_cache,ensure_velocity_pipeline,ensure_taa_reactive_mask_pipeline}.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly.rs (2026-06-24 Velocity/TAA variant-id pipeline cache owner: passed)
  - cargo check -q -p zircon_runtime --lib --no-default-features --features target-server --locked --target-dir target\codex-plan08-velocity-taa-variant-cache-check (2026-06-24 Velocity/TAA variant-id pipeline cache owner: passed with existing warnings)
  - cargo test -q -p zircon_runtime --lib shader_key_includes_shader_variant_identity --no-default-features --features target-server --locked --target-dir target\codex-plan08-velocity-taa-variant-cache-check -- --test-threads=1 --nocapture (2026-06-24 Velocity/TAA variant-id pipeline cache owner: timed out after 608s with no test result; residual target processes stopped; not counted as passed)
  - rustfmt --edition 2021 --check on Velocity template source cache cutover touched Rust files (2026-06-24 Velocity pipeline template source cache cutover: passed)
  - cargo check -q -p zircon_runtime --lib --no-default-features --features target-server --locked --target-dir target\codex-plan08-velocity-template-cache-check (2026-06-24 Velocity pipeline template source cache cutover: passed with existing warnings)
  - cargo test -q -p zircon_runtime --lib runtime_15_render_shader_template_assembly_is_folder_backed --no-run --no-default-features --features target-server --locked --target-dir target\codex-plan08-velocity-template-cache-check (2026-06-24 Velocity pipeline template source cache cutover: timed out after 244 seconds with no test compile result; residual target processes stopped; not counted as passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir target\codex-plan08-taa-reactive-template-cache-check-2 --message-format short --color never (2026-06-24 TAA reactive mask template source cache cutover: passed with existing warnings after an earlier scoped cargo check timed out after 244 seconds with no result)
  - cargo test -p zircon_runtime --lib runtime_15_render_shader_template_assembly_is_folder_backed --no-run --no-default-features --features target-server --locked --jobs 1 --target-dir target\codex-plan08-taa-reactive-template-cache-check-2 --message-format short --color never (2026-06-24 TAA reactive mask template source cache cutover: timed out after 424 seconds with no test compile result; no residual target processes found; not counted as passed)
  - rustfmt --edition 2021 --check on Shadow template source cache cutover touched Rust files (2026-06-24 Shadow pipeline template source cache cutover: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir target\codex-plan08-shadow-template-cache-check --message-format short --color never (2026-06-24 Shadow pipeline template source cache cutover: timed out after 600 seconds with no result; no residual target command found; not counted as passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir target\codex-plan08-shadow-template-cache-check-2 --message-format short --color never (2026-06-24 Shadow pipeline template source cache cutover: passed with existing warnings after the earlier scoped timeout)
  - cargo test -p zircon_runtime --lib runtime_15_render_shader_template_assembly_is_folder_backed --no-run --no-default-features --features target-server --locked --jobs 1 --target-dir target\codex-plan08-shadow-template-cache-check-2 --message-format short --color never (2026-06-24 Shadow pipeline template source cache cutover: passed with existing warnings after folder-backed guard module path attributes were made explicit)
  - cargo check -q -p zircon_runtime --lib --no-default-features --features target-server --locked --target-dir target\codex-plan08-runtime-geometry-key-check (2026-06-24 runtime mesh variant geometry-source key wiring slice: passed with existing warnings)
  - cargo test -q -p zircon_runtime --lib mesh_pipeline_standard_material_template_source_uses_requested_geometry_source --no-run --no-default-features --features target-server --locked --target-dir target\codex-plan08-runtime-geometry-key-test-compile (2026-06-24 runtime mesh variant geometry-source key wiring slice: timed out after 364 seconds with no test compile result; residual target processes stopped; not counted as passed)
  - cargo test -p zircon_runtime --lib render_shader_template --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-render-shader-template -- --test-threads=1 (2026-06-24 Shader template assembly foundation: timed out after 300s with no test result; not counted as passed)
  - rustfmt --edition 2021 zircon_runtime/src/core/framework/render/mod.rs; cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --lib --no-run --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-descriptor-rerun-0624 --message-format short --color never; cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor --lib --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-descriptor-rerun-0624 --message-format short --color never -- --test-threads=1 --nocapture; cargo test -p zircon_editor --lib export_wizard_panel_ --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-export-m6-descriptor-rerun-0624 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-24 root GeometrySource facade restoration: passed 4/4 editor plugin tests and 18/18 editor panel tests)
  - rustfmt --edition 2021 zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/{mesh_pipeline_cache,new,ensure_pipeline}.rs (2026-06-17 base mesh quality-aware cache owner slice: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-quality-cache-check-0617 (2026-06-17 base mesh quality-aware cache owner slice: passed with existing warnings)
doc_type: module-detail
---

# Runtime Render Shader Contracts

## Purpose

`zircon_runtime::core::framework::render::shader` owns the neutral shader contract that assets, material readiness, renderer preparation, and diagnostics can share without depending on WGPU objects or Bevy's ECS render app. It names shader stages, entry points, serialized dependencies, variant keys, and pipeline layout intent.

This module deliberately does not load files, parse WGSL imports, compile shader modules, allocate bind group layouts, or queue GPU pipelines. Asset import stays under `zircon_runtime::asset`, and concrete shader module or render pipeline creation stays under `zircon_runtime::graphics`.

## Bevy Evidence

Bevy keeps the shader asset surface separate from concrete renderer allocation. `dev/bevy/crates/bevy_shader/src/lib.rs:1-8` exposes `Shader` and `ShaderCache` as the shader crate's public surface. `dev/bevy/crates/bevy_shader/src/shader.rs:33-55` stores raw source, import path, imports, extra imports, shader defs, file dependencies, and validation policy on the shader asset. `shader.rs:85-148` constructs WGSL, GLSL, and SPIR-V shader assets, while `shader.rs:323-382` loads source files and records imported shader file handles.

`dev/bevy/crates/bevy_shader/src/shader_cache.rs:59-66` describes a cache that waits for imports and leaves renderer-specific module compilation to the render device. `shader_cache.rs:182-331` resolves imports, applies shader defs, composes the module, and reports pipelines that must be requeued when a shader changes.

The render-side precedent is `dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs:190-217`, where `PipelineCache` stores queued, creating, ready, and failed pipeline states. `pipeline_cache.rs:438-446` exposes cached bind group layout creation, `pipeline_cache.rs:448-466` requeues dependent pipelines when shader assets change, and `pipeline_cache.rs:468-632` creates render or compute pipelines from shader modules and layout descriptors. `dev/bevy/crates/bevy_render/src/render_resource/bind_group_layout.rs:7-14` describes bind group layouts as the shader resource interface.

Zircon copies the boundary, not the implementation: `render::shader` is the stable DTO layer; `asset::assets::shader` projects authoring data into those DTOs; `graphics` remains the only owner of WGPU shader modules, layouts, and render pipelines.

## Product Surface

`RenderShaderStage` is the common stage vocabulary: vertex, fragment, and compute. The enum is serializable with `snake_case` names so `.zshader`, `.zmeta`, tests, and diagnostics can move stage values across asset and runtime boundaries.

`ShaderAssetKind` is the v2 `.zshader` family discriminator. It has four values: `Surface`, `Include`, `Compute`, and `Fullscreen`. Only `Surface` participates in material variant space; include shaders only provide import modules, compute shaders provide compute entry points/resources, and fullscreen shaders provide pass-local fragment resources. `ShaderRenderStateDescriptor`, `ShaderQueueDescriptor`, and `ShaderResourceDescriptor` are backend-neutral authoring contracts for the v2 parser; they remain plain serializable DTOs and do not contain `wgpu` layout objects.

`RenderShaderEntryPointDescriptor` records the public entry-point name plus its `RenderShaderStage`. Asset-side parsing accepts authoring aliases such as `vert`, `vs`, `frag`, `fs`, `comp`, and `cs`, but the framework contract only exposes canonical stage values.

`RenderShaderDependency` records a `ResourceKind` and `AssetReference`. Dependencies are explicit serialized authoring data in the current milestone; they are not inferred from WGSL import syntax by the framework layer.

`RenderShaderDefinitionValue` records Bevy-style shader definition inputs as bool, signed integer, or unsigned integer values. `From<&str>` and `From<String>` create bool-true flag definitions so legacy authoring paths and small tests can stay concise while the runtime contract is no longer string-only.

`RenderShaderVariantKey` records an optional entry point, optional stage, and typed definition list. It is a neutral key for material or pipeline specialization diagnostics and single-module compile requests, not the full material pipeline-cache key.

`GeometrySourceId` is the geometry-source dimension for the material shader variant space. Built-in ids are reserved as `0 = StaticMesh`, `1 = SkinnedMesh`, `2 = MorphedMesh`, and `3 = SkinnedMorphed`; plugin geometry sources start at `GEOMETRY_SOURCE_PLUGIN_ID_START`. This keeps VertexFactory-style geometry source selection in the framework contract without pulling WGPU vertex-buffer declarations into the neutral layer.

The 2026-06-24 GeometrySource descriptor contract foundation extends that owner beyond ids. `GeometrySourceDescriptor` now records the stable token, WGSL include token, vertex attributes, backend-neutral required bindings, and typed shader defines for each geometry source. Built-in descriptor helpers cover static, skinned, morphed, and skinned+morphed meshes; all require the GPUScene instance binding, while skinned descriptors add skinning palette storage and morphed descriptors add morph weight/target storage. This contract intentionally stops at serializable framework data: no `wgpu` types, no pipeline descriptors, and no concrete bind group creation live in this module. The guard `runtime_15_render_shader_geometry_source_descriptor_contract_is_complete` locks the shape under status `render_plan08_geometry_source_descriptor_contract_static_passed_cargo_deferred_implementation_cadence`.

The 2026-06-29 runtime custom geometry descriptor staged-cache slice moves that descriptor contract into the runtime mesh pipeline cache instead of leaving it as an offline prewarm-only artifact. `MeshPipelineCache` owns a geometry-source descriptor registry initialized with built-ins and extended by product/plugin registration. Runtime module assembly collects `RuntimeExtensionRegistry::geometry_sources()`, carries descriptors through `GraphicsModule`, the graphics module host, `WgpuRenderFramework`, and `SceneRenderer`, and registers them in `SceneRendererCore` before mesh variants resolve. Base, GBuffer, DepthPrepass, Shadow, Velocity, and TAA reactive shader-source assembly resolve a `GeometrySourceDescriptor` from the variant's `GeometrySourceId` before asking the disk cache or compiling a WGPU pipeline. Status: `render_plan08_runtime_custom_geometry_descriptor_staged_cache_hit_wgpu_pipeline_passed_product_deferred`. Fresh `zircon_runtime --lib` type checking passed for this wiring, and the focused WGPU lib-test now creates the Base mesh pipeline from the staged fallback root for `custom:virtual_geometry=4` with `request=1`, `disk_hit=1`, `compile_miss=0`, `disk_write=0`, and `disk_error=0`. This closes descriptor-backed runtime cache lookup through the first custom geometry WGPU pipeline hit, but it is not a product virtual-geometry draw-source/page-cluster fetch implementation.

The 2026-06-24 Shader template assembly foundation is the first graphics-side consumer of that descriptor contract. `graphics::shader::template` is folder-backed by `assemble.rs`, `include_registry.rs`, and `pass_specialization.rs`: the assembler consumes a `GeometrySourceDescriptor`, `ShaderPassType`, `ShaderFeatureBits`, and a material surface function, then emits deterministic WGSL plus include tokens, include content hashes, and template revision `zr-material-template-v1`. The include registry maps the built-in descriptor include tokens to `graphics/shader/wgsl/zr_geometry_*.wgsl`, `zr_surface_types.wgsl`, and the pass templates; the pass specializer keeps Forward/GBuffer/DepthPrepass/Shadow/Velocity behavior explicit. This is still an assembly contract, not the runtime pipeline cutover: standard material, fallback shader, Naga/WGPU validation, and actual mesh pipeline consumption remain later Plan 08 slices. The guard `runtime_15_render_shader_template_assembly_is_folder_backed` locks the owner under status `render_plan08_shader_template_assembly_foundation_static_passed_cargo_timeout_no_result`.

The 2026-06-24 root GeometrySource facade restoration keeps that graphics-side template consumer compiling through the `core::framework::render` root facade. `render/mod.rs` now forwards `GeometrySourceDescriptor` and the built-in `GEOMETRY_SOURCE_WGSL_INCLUDE_*` tokens from `render/shader`; no shader assembly behavior, WGPU object creation, or compatibility path moved into the root module. The status anchor is `render_plan08_shader_template_root_geometry_exports_m6_cargo_passed`.

The 2026-06-24 Standard material template surface foundation adds `graphics/shader/template/material_surface.rs` as the built-in StandardPBR material surface source owner for that assembler. `standard_material_surface_source(...)` projects `StandardMaterialDescriptor` alpha mode, shadow reception, and double-sided state into `ShaderFeatureBits`, and returns the `standard_material_surface` WGSL entry that the assembler renames to `zr_material_surface`. The WGSL source declares group 2 material bindings for base-color, normal, metallic-roughness, occlusion, emissive textures/samplers, and binding 10 material properties; the generated header now exposes `ZR_FEATURE_ALPHA_TEST`, `ZR_FEATURE_RECEIVE_SHADOWS`, and `ZR_FEATURE_DOUBLE_SIDED`. This is still not the runtime standard-material or fallback shader cutover. The guard `runtime_15_render_shader_template_assembly_is_folder_backed` also locks this source owner under status `render_plan08_standard_material_template_surface_static_passed_cargo_deferred_implementation_cadence`.

The 2026-06-24 Standard material runtime feature surface source foundation adds `standard_material_surface_source_for_features(...)` below that descriptor projection. Runtime mesh-cache work can later feed `ShaderFeatureBits` plus alpha cutoff directly into this helper instead of rehydrating a full `StandardMaterialDescriptor`. The descriptor path now delegates to it, and `standard_material_surface_source_can_be_built_from_runtime_features` locks `ShaderFeatureBits::RECEIVE_SHADOWS` preservation plus finite/clamped `ZR_STANDARD_MATERIAL_ALPHA_CUTOFF` generation. Status: `render_plan08_standard_material_runtime_feature_surface_source_static_passed_cargo_deferred_implementation_cadence`.

The 2026-06-24 Shader template Naga validation foundation adds `graphics/shader/template/validation.rs` as the Naga parse/validate owner for assembled template WGSL. `validate_material_shader_template_wgsl(...)` returns a small `MaterialShaderTemplateValidation` summary with entry point names and maps parser or validator failures into `ShaderTemplateValidationError`. This is deliberately still below WGPU pipeline creation: it proves a WGSL module can be sent to Naga before later mesh pipeline cache work asks a device to create shader modules or render pipelines. The guard `runtime_15_render_shader_template_assembly_is_folder_backed` locks the owner under status `render_plan08_shader_template_naga_validation_static_passed_cargo_deferred_implementation_cadence`.

The 2026-06-24 Shader template uv1 tangent interpolation foundation extends the template WGSL contract so material surface code can receive both UV channels and tangent basis data. `zr_surface_types.wgsl` now carries `uv1`, `tangent_ws`, and `tangent_handedness` in `ZrVertexOutput`, all built-in pass templates call `fetch_tangent(v, instance_index)` and `fetch_uv1(v)`, and the standard material surface source passes `input.uv1` into its UV-channel selector. The guard `runtime_15_render_shader_template_assembly_is_folder_backed` locks the owner under status `render_plan08_shader_template_uv1_tangent_interpolation_static_passed_cargo_deferred_implementation_cadence`.

The 2026-06-24 Shader template vertex input ABI alignment fixes the template input side before runtime WGPU cutover. `ZrVertexInput` now follows `GpuMeshVertex::layout()` exactly: position 0, normal 1, uv0 2, joints 3, weights 4, tangent 5, color 6, and uv1 7. The standard material template test and `runtime_15_render_shader_template_assembly_is_folder_backed` guard lock that order under status `render_plan08_shader_template_vertex_input_abi_alignment_static_passed_cargo_deferred_implementation_cadence`.

The 2026-06-24 Shader template runtime scene transform foundation fixes the template output side before runtime WGPU cutover. `zr_scene_runtime.wgsl` now declares the runtime group0 `SceneUniform`, the assembler includes current `zr_gpu_scene.wgsl`, and `zr_build_vertex_output(...)` applies `zr_world_from_local(instance_index)` plus `scene.view_proj * position_ws` before emitting `clip_position`; normal and tangent outputs are transformed into world-space. Skinned template geometry now reuses runtime `zr_skinned_joint_matrix(...)` and `zr_previous_skinned_joint_matrix(...)` uniform palette helpers so it does not collide with GPUScene group3 instance/light bindings; storage palette migration remains MS-M2 work. The status anchor is `render_plan08_shader_template_runtime_scene_transform_static_passed_cargo_timeout_no_result`, with focused Cargo/Naga execution timed out and no WGPU/RenderDoc result counted.

The 2026-06-24 Shader template runtime entry alias foundation closes the generic mesh entry-name mismatch before WGPU cutover. Pass templates now route vertex bodies through `zr_vs_main_impl(...)` and fragment bodies through `zr_fs_main_impl(...)`, preserve `zr_vs_main`/`zr_fs_main` for template diagnostics, and add runtime `vs_main`/`fs_main` aliases for the current mesh pipeline creation code. The guard `runtime_15_render_shader_template_assembly_is_folder_backed` locks `zr_vs_main_impl`, `zr_fs_main_impl`, `vs_main`, and `fs_main` under status `render_plan08_shader_template_runtime_entry_alias_static_passed_cargo_timeout_no_result`. Focused Cargo/Naga execution timed out after 300 seconds with no result. Velocity's former dedicated motion-vector entry names are superseded by the Velocity template/source-cache cutover recorded below.

The 2026-06-24 Forward template runtime light-grid shadow shading context foundation moves the Forward template closer to the current WGPU fallback mesh shader without switching the runtime source yet. `pass_specialization.rs` injects `zr_light_grid.wgsl` and `zr_shadow.wgsl` through Forward-only `support_includes`; `zr_surface_types.wgsl` carries runtime `tint`/`shadow_params` and exposes `zr_build_shading_context(input)`; `zr_shading_standard_pbr.wgsl` consumes that context, light-grid functions, and `zr_gpu_light_shadow_visibility(...)`. The receive-shadow branch is explicit as `ZR_FEATURE_RECEIVE_SHADOWS && ctx.shadow_params.z > 0.5`, and the standard material source multiplies sampled base color by `input.tint * input.color`. Status: `render_plan08_forward_template_light_grid_shadow_context_static_passed_cargo_deferred_implementation_cadence`.

The 2026-06-24 Forward template built-in shading model dispatch parity foundation adds the built-in model branch data that the current fallback shader already consumes. `ZrSurfaceOutput` now carries `unlit` and `shading_model_id`, `standard_material_shading_model_id()` decodes `standard_material_properties.data8.y` and respects the `data0.w` unlit override, and `zr_shading_standard_pbr.wgsl` dispatches Unlit, BlinnPhong, and StandardPBR before the runtime source cutover. The BlinnPhong branch is locked by `surface.shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID`; status: `render_plan08_forward_template_builtin_shading_model_dispatch_static_passed_cargo_deferred_implementation_cadence`.

The 2026-06-24 Standard material normal map sampling foundation uses that tangent input in `standard_material_sampled_normal(...)`. The standard material template source samples `standard_material_normal_tex`, builds a tangent frame from `input.tangent_ws`, `input.tangent_handedness`, and `input.normal_ws`, and writes the sampled world normal into `surface.normal_ws`. The guard `runtime_15_render_shader_template_assembly_is_folder_backed` locks the owner under status `render_plan08_standard_material_normal_map_sampling_static_passed_cargo_deferred_implementation_cadence`.

The 2026-06-24 Standard material alpha clip pass foundation closes the template-level `Mask` semantics before runtime WGPU cutover. `zr_surface_types.wgsl` now adds `alpha_cutoff` to `ZrSurfaceOutput` and exposes pure `zr_surface_fails_alpha_clip(...)`; the standard material source emits `ZR_STANDARD_MATERIAL_ALPHA_CUTOFF` from the descriptor cutoff and assigns `surface.alpha_cutoff` through `standard_material_alpha_cutoff()`. That helper reads `standard_material_properties.data8.z` first, then falls back to the descriptor constant so template assembly remains deterministic while the runtime standard-material uniform now owns the production cutoff value. Forward, G-buffer, DepthAlpha, and ShadowAlpha fragment templates each own `zr_apply_alpha_clip(...)` so `discard` stays fragment-local. DepthPrepass and Shadow select `zr_template_depth_alpha.wgsl` / `zr_template_shadow_alpha.wgsl` only when `ShaderFeatureBits::ALPHA_TEST` is set. The status anchors are `render_plan08_standard_material_alpha_clip_pass_static_passed_cargo_deferred_implementation_cadence` and `render_plan08_standard_material_uniform_alpha_cutoff_static_passed_cargo_deferred_implementation_cadence`.

Standard material uniform alpha cutoff foundation is the runtime-side companion to that template helper. `standard_material_uniform_packs_alpha_cutoff_for_template_clip` covers the `data8.z` slot while Cargo/Naga/WGPU execution remains deferred by milestone cadence.

`GeometrySourceId` is owned by `shader/geometry_source.rs`. `shader/mod.rs` may re-export it for public callers, but internal shader submodules that need the type should import it from `super::geometry_source::GeometrySourceId`. That keeps `variant_key.rs` tied to the canonical geometry-source owner instead of relying on a facade re-export that can disappear during hard-cutover module cleanup.

`ShaderVariantKey` is the Plan 08 material pipeline variant key contract. It combines `material_shader`, `material_revision`, `geometry_source`, `shading_model`, `pass_type`, `features`, `quality`, and a backend `platform_token`. `packed_dims()` reserves stable bit segments for fast in-memory specialization dimensions: geometry bits `0..3`, shading model bits `4..11`, pass bits `12..15`, feature bits `16..47`, and quality bits `48..49`. `canonical_string()` serializes the full stable key, including material id/revision and platform token, for later disk-cache hashing and shader prewarm manifests. The type remains backend-agnostic; WGPU shader modules, render pipelines, and cache entries still belong under `graphics`. `RenderQualityProfile::shader_quality` is the runtime-facing quality source and defaults to `ShaderQualityTier::Medium`; callers can override it with `RenderQualityProfile::with_shader_quality(...)`.

`ShaderPassType`, `ShaderFeatureBits`, and `ShaderQualityTier` are the typed subdimensions of that key. Pass type covers forward, G-buffer, depth prepass, shadow, and velocity passes. Feature bits currently reserve alpha-test, receive-shadows, double-sided, LOD dither crossfade, and instanced previous-transform flags. Quality tiers are low, medium, high, and ultra. The names and bit positions are intentionally stable because future mesh pipeline cache and disk-cache code will use them as part of persisted shader variant identity.

`ShaderVariantMissReport` is the neutral diagnostic DTO for variant cache behavior. It records variant requests, memory hits, disk hits, compile misses, disk writes, and disk errors for the last frame so runtime diagnostics can verify whether prewarm and disk-cache slices actually removed runtime compiles.

The 2026-06-27 runtime dimension-correlation slice adds `ShaderVariantMissReport.dimension_summary`, mirroring the prewarm report's pass type, geometry source id, shading model id, and quality tier groups. Runtime count rows carry request, memory-hit, disk-hit, compile-miss, disk-write, and disk-error totals, and serde defaults keep older diagnostic payloads readable. This makes staged-prewarm product evidence inspectable by the same dimensions that build-time prewarm reports print, without needing renderer-private registries at the diagnostics layer. Status: `render_plan08_runtime_shader_variant_dimension_correlation_product_passed_renderdoc_deferred`.

`ShaderVariantPrewarmManifest`, `ShaderVariantPrewarmSource`, `ShaderVariantPrewarmRequest`, and `ShaderVariantPrewarmReport` are the neutral offline-cache DTOs. Schema version 2 stores final WGSL, include/source hashes, and template/compiler version strings exactly once in a content-addressed source table; every request contains only its final `ShaderVariantKey`, optional complete pipeline state, and `source_id`. Manifest integrity rejects duplicate, missing, or non-canonical source ids before graphics-side work begins. This prevents a pass x quality x geometry product from duplicating identical WGSL payloads while keeping WGPU types outside the framework contract. The report records requested, written, and failed counts, per-variant failures, a `dimension_summary` grouped by pass type, geometry source id, shading model id, and quality tier, source provenance, and the configured/resident/in-flight execution budget. The graphics prewarm executor currently uses one serial WGPU worker and rejects a manifest whose resident source table or individual source payload exceeds its configured budget. Geometry and shading dimensions use stable numeric id strings so report readers do not need renderer-private registries to diagnose gaps. The `zircon_shader_prewarm` tool can read an authored schema-2 manifest, emit the built-in fallback manifest, or scan asset roots for `.zmeta` compound shader packages, `.zshader` files, standalone `.wgsl` files, and `.zmaterial` material instances. Automatically generated built-in and asset-root requests can be expanded with repeated `--quality-tier low|medium|high|ultra` or `--quality-tier all`; no explicit tier still defaults to Medium so existing staging size stays stable. Authored manifests must be regenerated into schema 2 rather than using a compatibility request-source shim. This source-table and budget migration is implementation work; WGPU product execution, cold/warm performance evidence, and RenderDoc capture are still pending managed validation.

The 2026-06-24 Shader prewarm WGSL validation gate keeps the DTO unchanged but changes the graphics-side write behavior: `graphics/shader/variant_cache/prewarm.rs` now calls `validate_shader_variant_prewarm_wgsl(...)` before writing a request to `ShaderVariantCacheDisk`. Invalid WGSL increments `ShaderVariantPrewarmReport.failed_count`, stores the variant index and error, and leaves the disk cache at miss. The tool-level invalid manifest probe returned exit 2 with requested=1, written=0, failed=1, and cache_files=0, matching the report path. This uses the existing Naga validation owner under `graphics/shader/template/validation.rs`; it is not WGPU shader-module or render-pipeline creation evidence. Status: `render_plan08_shader_prewarm_wgsl_validation_check_passed_test_compile_timeout_no_result`.

The 2026-06-27 prewarm dimension diagnostics slice makes the staged-cache report show which variant dimensions actually wrote or failed. `graphics/shader/variant_cache/prewarm.rs` now records successful writes and WGSL/write failures through the variant-aware report methods, while schema-level failures can still remain top-level failures without a dimension key. `ShaderVariantPrewarmReport.dimension_summary` is serde-defaulted for older report JSON and exposes `pass_types`, `geometry_source_ids`, `shading_model_ids`, and `quality_tiers`, each with requested/written/failed counters. The focused regression is `render_shader_variant_prewarm_report_groups_written_and_failed_dimensions`; current status is `render_plan08_prewarm_dimension_diagnostics_typecheck_passed_test_timeout_no_result`.

The 2026-06-24 Shader prewarm geometry-source enumeration slice adds the geometry dimension to generated prewarm manifests. Repeated `--geometry-source static|skinned|morphed|skinned-morphed|all` expands asset-root requests across built-in `GeometrySourceId` values while preserving static as the default. `asset_root_manifest_for_quality_tiers_and_geometry_sources(...)` now forms the pass x quality x geometry-source product before writing `ShaderVariantKey.geometry_source`, and the older `asset_root_manifest_for_quality_tiers(...)` remains a static-default compatibility wrapper. `tools/zircon_build.py --prewarm-shaders` forwards quality tiers through `--shader-quality-tier` and geometry sources through `--shader-geometry-source`, which the staged cache command maps back to repeated `--geometry-source` arguments. The guard `runtime_15_shader_prewarm_geometry_source_enumeration_is_wired` locks the CLI parser, run forwarding, manifest product expansion, build-script forwarding, docs/status anchors, and the focused test `shader_prewarm_asset_root_manifest_expands_requested_geometry_sources` under status `render_plan08_shader_prewarm_geometry_source_enumeration_static_passed_cargo_deferred_implementation_cadence`.

Asset-root builtin standard material template prewarm is the follow-up that connects `.zmaterial` references to the runtime template source without changing custom shader scan semantics. When a material shader locator is exactly `builtin://shader/pbr.wgsl`, `zircon_shader_prewarm` calls `dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry(...)` for each requested built-in `GeometrySourceId` instead of looking for an asset-root shader file. The old `dynamic_api::builtin_standard_material_shader_prewarm_manifest(...)` remains the static wrapper. Both builders project `ShaderFeatureBits`, `ShadingModelId`, alpha cutoff, and quality tiers into a `PipelineKey`, then reuse `mesh_pipeline_standard_material_template_source(...)` for static or `mesh_pipeline_standard_material_template_source_for_geometry(...)` for explicit geometry so the request carries matching WGSL, include hashes, source hash, template revision, and `ShaderVariantKey.geometry_source`. Custom `.zshader` and standalone `.wgsl` files remain raw scanned WGSL requests. `shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source` now checks static and skinned builtin template requests, alpha cutoff constant, `ShaderFeatureBits::RECEIVE_SHADOWS`, and geometry-specific includes; `runtime_15_shader_prewarm_builtin_standard_material_template_source_is_wired` locks the dynamic API export, scene facade, manifest expansion, docs/status anchors, and line budgets. Status: `render_plan08_asset_root_builtin_standard_material_template_prewarm_static_passed_cargo_deferred_implementation_cadence`; multi-geometry follow-up status: `render_plan08_asset_root_builtin_standard_material_multi_geometry_prewarm_static_passed_cargo_deferred_implementation_cadence`.

Asset-root custom shading-model id prewarm adds an explicit project/plugin id map without reintroducing a dead plugin registry surface. `zircon_shader_prewarm --shading-model-id custom:subsurface=16` and `tools/zircon_build.py --shader-shading-model-id custom:subsurface=16` normalize the custom token, reject ids below `SHADING_MODEL_PLUGIN_ID_START`, and forward the map into `asset_root_manifest_for_quality_tiers_geometry_sources_and_shading_model_ids(...)`. The staging command assembly lives in `tools/zircon_build_shader_prewarm.py` so `tools/zircon_build.py` remains the build orchestrator instead of accumulating more shader-prewarm detail. A `.zmaterial` with `lighting_model = "custom:subsurface"` then writes `ShaderVariantKey.shading_model = ShadingModelId::new(16)` for builtin standard-material template requests and raw scanned material requests. Unknown custom models still fall back to StandardPBR, keeping the staged cache conservative until a real project/插件 registry exporter exists. The focused guards are `shader_prewarm_args_parse_custom_shading_model_plugin_ids`, `shader_prewarm_asset_root_manifest_maps_custom_shading_model_plugin_ids`, and `runtime_15_shader_prewarm_custom_shading_model_id_is_wired`. Status: `render_plan08_asset_root_custom_shading_model_id_prewarm_static_passed_cargo_deferred_implementation_cadence`.

Asset-root custom geometry-source id prewarm adds the equivalent explicit id path for geometry dimensions. `zircon_shader_prewarm --geometry-source-id custom:gpu-driven=4` and `tools/zircon_build.py --shader-geometry-source-id custom:gpu-driven=4` normalize the custom token, reject ids below `GEOMETRY_SOURCE_PLUGIN_ID_START`, and append the resulting `GeometrySourceId` to the manifest geometry-source list. The manifest path already accepts arbitrary `GeometrySourceId` values, so no renderer-private WGPU descriptors or plugin registry surface are introduced for this slice. A raw asset-root shader can therefore write five pass-specific `ShaderVariantKey.geometry_source = GeometrySourceId::new(4)` requests under the explicit CLI/build-tool input. The focused guards are `shader_prewarm_args_parse_custom_geometry_source_plugin_ids`, `shader_prewarm_asset_root_manifest_expands_custom_geometry_source_plugin_ids`, and `runtime_15_shader_prewarm_custom_geometry_source_id_is_wired`. Status: `render_plan08_asset_root_custom_geometry_source_id_prewarm_typecheck_passed_test_timeout_no_result`.

Asset-root shader edit revision export closes the staged-cache edit-key gap without pretending the offline scan owns the live `ResourceManager` counter. `manifest/revision.rs` turns `.zmeta.source_digest` into a stable non-zero `ShaderVariantKey.material_revision`; raw `.wgsl` and `.zshader` sources without `.zmeta` derive the revision from their include/source content hash list. `shader_source_from_zmeta(...)` and fallback `shader_prewarm_source(...)` now consume those values, so changing a shader package source hash or raw WGSL content produces a new prewarm key instead of reusing revision `1`. `shader_prewarm_asset_root_manifest_uses_zmeta_source_digest_revision` and `shader_prewarm_asset_root_manifest_uses_raw_source_hash_revision` lock the edit boundary, and `runtime_15_shader_prewarm_asset_revision_export_is_wired` locks the module split and docs/status anchors. Status: `render_plan08_asset_root_shader_edit_revision_export_passed_cargo_renderdoc_deferred`.

Asset-root resource registry revision overlay is the explicit handoff point for projects that can already export live `ResourceRecord` data, narrowing the older live project registry exact revision overlay gap to a caller-supplied JSON input. `zircon_shader_prewarm --resource-registry <records.json>` and `tools/zircon_build.py --shader-resource-registry <records.json>` accept a `ResourceRecord` array or a JSON object containing `resources`/`records`; `manifest/resource_registry.rs` filters to shader records with non-zero revisions and indexes them by `ResourceId`, primary locator, and artifact locator. During `.zmeta` shader scanning, `asset_root_manifest_with_resource_registry_revisions(...)` uses the matching `ResourceRecord.revision` for `ShaderVariantKey.material_revision`; unmatched `.zmeta` sources still use `source_digest`, and raw sources without `.zmeta` still use content-hash revision. The focused guards are `shader_prewarm_args_parse_resource_registry_path`, `shader_prewarm_asset_root_manifest_uses_resource_registry_revision_overlay`, and `runtime_15_shader_prewarm_resource_registry_revision_overlay_is_wired`. Status: `render_plan08_asset_root_resource_registry_revision_overlay_typecheck_passed_test_timeout_no_result`.

Staged shader resource registry auto-export closes the build-tool default path between explicit revision overlay and asset-root manifest generation. When `tools/zircon_build.py --prewarm-shaders` has no explicit `--shader-resource-registry`, `tools/zircon_build_shader_prewarm.py` now forwards `--export-resource-registry ZirconEngine/cache/shader_resource_records.json`; explicit registries still use `--resource-registry`. `bin/zircon_shader_prewarm/manifest/resource_registry.rs::shader_resource_records_from_asset_root(...)` reads staged `.zmeta` files, exports shader-only ready `ResourceRecord` rows with source-hash-derived staged revisions, and `run.rs` feeds those rows into `ShaderPrewarmResourceRegistryOverlay::from_records(...)` for the same scan. The manifest scan skips a raw `.wgsl`/`.zshader` file when a sidecar `.zmeta` owns that single-file shader, so generated registry revisions are not mixed with duplicate content-hash fallback variants. `shader_prewarm_asset_root_exports_shader_resource_records` and `runtime_15_shader_prewarm_registry_auto_export_is_wired` guard the handoff. Status: `render_plan08_shader_resource_registry_auto_export_focused_tests_passed_renderdoc_deferred`.

Staged shader resource registry multi-root dedupe keeps that auto-export deterministic when engine and selected plugin asset roots contain the same shader metadata. `shader_resource_records_from_asset_roots(...)` gathers all requested roots and calls `deduplicate_shader_resource_records(...)` before `run.rs` writes `shader_resource_records.json`; exact duplicate `ResourceRecord` rows collapse to one entry, while conflicting id-to-locator or locator-to-id mappings fail the prewarm command instead of creating a last-writer overlay. Status: `render_plan08_shader_resource_registry_multi_root_dedupe_static_passed_cargo_deferred`; `shader_resource_records_from_asset_roots_deduplicates_duplicate_shader_records` and `runtime_15_shader_prewarm_resource_registry_multi_root_dedupe_is_wired` lock the code owner, docs anchors, and 800-line guard. This is not counted as real WGPU runtime execution, RenderDoc capture, full live registry export, or miss=0 product acceptance.

Build-tool shader asset-root plan visibility makes the same root set visible in staged build dry-run output. `print_shader_prewarm_plan(...)` now prints `shader asset roots: ...` through `shader_asset_root_paths_for_prewarm(config)`, so the plan output and `build_shader_prewarm_command(...)` share the engine/plugin root owner before `--export-resource-registry` runs. The same plan visibility owner now also prints `shader prewarm cache root`, `shader prewarm report`, and `shader runtime fallback root`, so dry-run output exposes the staged cache/report handoff path before the acceptance helper reads a real report. Status: `render_plan08_build_tool_shader_asset_root_plan_visibility_python_passed_cargo_deferred`; `test_prewarm_plan_lists_asset_roots_for_registry_export`, `test_prewarm_plan_lists_runtime_fallback_handoff_paths`, `test_build_command_auto_export_registry_scans_all_asset_roots`, and `runtime_15_shader_prewarm_asset_root_plan_visibility_is_wired` lock the plan text, command handoff, docs anchors, and line budget. Closeout verification passed build-helper Python combo 60/60 plus py_compile, rustfmt, anchors, whitespace/conflict, line-budget, and scoped diff-check. This is not counted as real WGPU runtime execution, RenderDoc capture, full live registry export, or miss=0 product acceptance.

Shader permutation registry overlay is the next explicit project/plugin input layer for staged prewarm. `zircon_shader_prewarm --shader-permutation-registry <registry.json>` merges a registry file, and `shader_permutation_registry_paths` also discovers `shader_permutation_registry.json` below each asset root. The registry document's `geometry_source_ids` and `shading_model_ids` records are normalized to custom tokens, range-checked against plugin id starts, and merged before asset-root manifests expand, so external project/plugin ids can populate `ShaderVariantKey.geometry_source` and `ShaderVariantKey.shading_model` without hand-written CLI ids for every build. `tools/zircon_build.py --shader-permutation-registry <path>` forwards explicit registries while staged asset roots keep the sidecar discovery path. The focused guards are `shader_prewarm_permutation_registry_merges_custom_geometry_and_shading_ids`, `shader_prewarm_permutation_registry_discovers_asset_root_registry`, and `runtime_15_shader_prewarm_permutation_registry_overlay_is_wired`. Status: `render_plan08_shader_permutation_registry_overlay_focused_tests_passed_renderdoc_deferred`.

The staged auto-export is intentionally narrower than live project registry export: it records asset-root `.zmeta` shader rows for build-time prewarm, not the running `ResourceManager` revision counter or custom plugin registry ids.

The scan path mirrors the shader package importer by reading `.zshader` `wgsl_files` in order and combining those files into the runtime WGSL payload before writing disk-cache entries. `.zshader` entry-point stages drive StandardPBR pass expansion for each selected geometry source: vertex+fragment sources emit Forward, GBuffer, DepthPrepass, Shadow, and Velocity; vertex-only sources emit DepthPrepass, Shadow, and Velocity; fragment-only sources emit Forward and GBuffer; compute-only sources do not enter the material-variant prewarm space. Standalone `.wgsl` sources default to the full material pass set because they do not carry serialized stage metadata. Scanned shader requests now use source-hash-derived material revisions for edit invalidation while source/include hashes remain part of the disk-cache key payload for stale-entry validation. `.zmaterial` files are parsed through `MaterialAsset`, joined back to scanned shader sources by shader `AssetReference` URL or resource id, and expanded into deduplicated material-dimension variants. The feature mapping matches runtime `PipelineKey`: `AlphaMode::Mask` sets `ShaderFeatureBits::ALPHA_TEST`, `double_sided = true` sets `ShaderFeatureBits::DOUBLE_SIDED`, and runtime `PipelineKey.receive_shadows` now sets `ShaderFeatureBits::RECEIVE_SHADOWS`. Built-in material lighting models also enter the prewarm key through `ShadingModelId::from_lighting_model`: PBR maps to StandardPBR, BlinnPhong maps to BlinnPhong, and Unlit maps to Unlit. `AlphaMode::Blend` material-instance requests are filtered to the Forward pass so transparent materials align with the current runtime transparent queue instead of prewarming unused G-buffer, depth, shadow, or velocity variants for that material instance. Custom lighting models can be mapped through explicit `--shading-model-id` / `--shader-shading-model-id` plugin ids; unknown custom models continue to fall back to StandardPBR until a project shading-model registry exporter can provide those ids automatically.

## Runtime 15 M3 shader prewarm manifest test folder split

状态：`runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred`。

Runtime 15 R4.1/M3 的结构切片先移动 shader prewarm manifest 的测试 owner，后续 Plan 08 geometry-source 枚举、builtin standard material template prewarm、custom shading-model id prewarm、asset-root shader edit revision export 与 resource registry revision overlay 切片继续在同一 folder-backed owner 内扩展覆盖。`bin/zircon_shader_prewarm/manifest.rs` 当前 705 行，父文件只保留生产扫描/manifest 逻辑和 `#[cfg(test)] mod tests;` 挂载；`bin/zircon_shader_prewarm/manifest/revision.rs` 承接 source-hash/content-hash revision projection，`bin/zircon_shader_prewarm/manifest/paths.rs` 承接路径扫描 helper，`bin/zircon_shader_prewarm/manifest/resource_registry.rs` 承接 exported `ResourceRecord` revision overlay；原内联测试迁入 `bin/zircon_shader_prewarm/manifest/tests.rs`，测试子文件当前 563 行。

子文件保留 `shader_prewarm_asset_root_manifest_reads_compound_zshader_package`，继续覆盖 compound `.zshader`、`.zmaterial` feature bits、BlinnPhong/Unlit shading model 映射、material revision 与 alpha-blend Forward-only pass filtering；新增 `shader_prewarm_asset_root_manifest_expands_requested_geometry_sources`，覆盖 asset-root manifest 按 static+skinned geometry source 展开 10 个 pass x geometry 请求；`shader_prewarm_asset_root_manifest_expands_custom_geometry_source_plugin_ids` 覆盖显式 plugin-range geometry-source id 展开；`shader_prewarm_builtin_fallback_manifest_expands_requested_geometry_sources` 覆盖纯 builtin fallback 的多 geometry 展开；`shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source` 覆盖 builtin standard material `.zmaterial` 生成 static+skinned Forward template source；`shader_prewarm_asset_root_manifest_maps_custom_shading_model_plugin_ids` 覆盖 custom lighting-model token 到 plugin-range shading id 的显式 map；`shader_prewarm_asset_root_manifest_uses_zmeta_source_digest_revision` 和 `shader_prewarm_asset_root_manifest_uses_raw_source_hash_revision` 覆盖 source_hash/raw content 编辑后 revision 变化。`structure_convention/test_file_budget/shader_prewarm_manifest.rs::runtime_15_shader_prewarm_manifest_tests_are_folder_backed` 现在锁定 7 个 manifest 测试保留与 revision child owner；`runtime_15_shader_prewarm_geometry_source_enumeration_is_wired` 额外锁定 `--geometry-source` CLI、manifest geometry-source product expansion、`tools/zircon_build.py --shader-geometry-source` 转发和 docs/status anchors；`runtime_15_shader_prewarm_custom_geometry_source_id_is_wired` 锁定 `--geometry-source-id`、`tools/zircon_build.py --shader-geometry-source-id`、manifest plugin-range geometry dimension 和状态锚点 `render_plan08_asset_root_custom_geometry_source_id_prewarm_typecheck_passed_test_timeout_no_result`；`runtime_15_shader_prewarm_builtin_standard_material_template_source_is_wired` 锁定 `dynamic_api::builtin_standard_material_shader_prewarm_manifest(...)`、`dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry(...)`、builtin URI routing、`ShaderFeatureBits::RECEIVE_SHADOWS` 和 statuses `render_plan08_asset_root_builtin_standard_material_template_prewarm_static_passed_cargo_deferred_implementation_cadence` / `render_plan08_asset_root_builtin_standard_material_multi_geometry_prewarm_static_passed_cargo_deferred_implementation_cadence`；`runtime_15_shader_prewarm_custom_shading_model_id_is_wired` 锁定 `--shading-model-id`、`tools/zircon_build.py --shader-shading-model-id`、manifest explicit id map 和状态锚点 `render_plan08_asset_root_custom_shading_model_id_prewarm_static_passed_cargo_deferred_implementation_cadence`；`runtime_15_shader_prewarm_asset_revision_export_is_wired` 锁定 revision child owner、测试和状态锚点 `render_plan08_asset_root_shader_edit_revision_export_passed_cargo_renderdoc_deferred`。

`RenderShaderPipelineLayoutDescriptor` records the intended shader resource interface. Each `RenderShaderBindGroupLayoutDescriptor` stores a group index, optional label, and binding rows. Each `RenderShaderBindingDescriptor` stores binding index, optional label, resource type, and stage visibility. `RenderShaderBindingResourceType` currently names uniform buffers, storage buffers, sampled textures, storage textures, and samplers. `push_constant_ranges` is intentionally a vector of labels or range descriptions rather than a WGPU-native range type because the neutral contract must remain serializable and backend-agnostic.

## Asset Projection

`ShaderAsset::runtime_wgsl_source()` is the runtime source selector. It prefers non-empty emitted `wgsl_source`, then falls back to raw `source` only when `source_language == ShaderSourceLanguage::Wgsl`. Non-WGSL source without emitted WGSL is not render-ready and must fall back or report readiness diagnostics before graphics code attempts to build a shader module.

`ShaderAsset::entry_point_descriptors()` maps serialized `ShaderEntryPointAsset` rows into canonical framework descriptors and filters invalid stage tokens. `ShaderAsset::dependencies()` maps serialized `ShaderDependencyAsset` rows into `RenderShaderDependency`. `ShaderAsset::variant_keys()` derives first-pass keys from entry point names and stage strings. `ShaderAsset::pipeline_layout_descriptor()` clones the serialized layout descriptor so render feature contracts and diagnostics can reason about bind groups without allocating WGPU layouts.

`ShaderAsset::readiness_report()` sits above the neutral render DTOs and below renderer preparation. It validates whether the asset payload has runtime WGSL, canonical entry-point stages, non-empty and non-duplicated shader definition names, and no shader-side validation diagnostics. It deliberately does not compose WGSL imports, create Naga modules, allocate WGPU shader modules, build bind group layouts, or queue pipelines; those remain shader-cache and graphics responsibilities.

`.zshader` documents are asset-layer authoring documents. They store WGSL file references, entry points, import redirects, material property schema, texture slots, and editor hints. The `.zshader` importer may perform authoring diagnostics such as WGSL capture checks, but `render::shader` stays limited to the product DTOs that the renderer and material readiness layer can consume.

## Graphics Integration

`ResourceStreamer::ensure_shader_source(...)` is the current concrete bridge. It resolves the referenced `ShaderAsset`, requires `runtime_wgsl_source()`, stores the selected WGSL in `ShaderRuntime`, and returns a material readiness fallback report when the shader is missing or cannot provide runtime WGSL. This keeps shader-source failure visible to material diagnostics instead of silently using a fallback.

The mesh renderer cache currently creates WGPU shader modules from the prepared WGSL source and caches modules by shader resource id plus revision. `PipelineKey` can now derive the neutral `ShaderVariantKey`, and `MeshPipelineVariantKey` stores that derived key beside the full `PipelineKey` with a WGPU platform token and a pass-type mapping for forward, G-buffer/depth, shadow, and velocity pass kinds. The 2026-06-24 PipelineKey receive-shadows shader feature foundation adds `PipelineKey.receive_shadows` to that bridge and maps it to `ShaderFeatureBits::RECEIVE_SHADOWS`; `material.pipeline_key.receive_shadows` is now the runtime source of truth for rebuilding the standard-material template feature set. Viewport quality now flows from `RenderQualityProfile::shader_quality` through `ViewportRecordState`, `FrameSubmissionContext`, `ViewportRenderFrame`, and `MeshPassBuildContext` before `MeshPipelineVariantRegistry` writes it into `ShaderVariantKey.quality`; distinct quality tiers therefore produce distinct mesh variant ids and sort-key material bits. The runtime mesh variant geometry-source key wiring slice adds the geometry dimension to the same path: `MeshDrawGeometrySource::shader_geometry_source_id()` maps prepared/dynamic batches to static mesh and GPU skinning source batches to skinned mesh, `MeshDrawQueueProfile::shader_geometry_source_id()` keeps prepared GPU-skinning draw statistics on the prepared queue while routing shader variants to `GEOMETRY_SOURCE_ID_SKINNED_MESH`, `MeshPassBuildContext` passes that id into `MeshPipelineVariantResolver::resolve_variant_for_geometry(...)`, and the registry/cache call `PipelineKey::shader_variant_key_for_geometry(...)` so `ShaderVariantKey.geometry_source` participates in variant identity. The base mesh render-command path now resolves pipelines by `MeshPipelineVariantId` and uses the registry-owned `ShaderVariantKey` for both shader-module cache identity and `graphics::shader::variant_cache::ShaderVariantCacheDisk` lookup/write. Disk entries are keyed by `ShaderVariantKey::canonical_string()` plus a WGSL source hash, first checking the runtime writable cache and then the staged prewarm cache produced by `zircon_shader_prewarm` / `tools/zircon_build.py --prewarm-shaders`. Build staging passes `ZirconEngine/assets` to the tool as `--asset-root`, so source packages copied into the staged runtime can contribute disk-cache entries in the same pass as the built-in fallback shader. Velocity/TAA reactive runtime pipeline maps now also use `MeshPipelineVariantId`, and their shader module keys include `ShaderVariantKey::canonical_string()`. Velocity consumes `mesh_pipeline_velocity_template_source_for_geometry(...)`; TAA reactive mask consumes `mesh_pipeline_taa_reactive_mask_template_source_for_geometry(...)`; both reuse the pass-specific disk/source-hash cache path and include the final source hash in module identity. Deferred/template variants still keep their current narrower cache owners. Status: `render_plan08_pipeline_key_receive_shadows_shader_feature_static_passed_cargo_deferred_implementation_cadence`; geometry-source runtime wiring status: `render_plan08_runtime_mesh_variant_geometry_source_key_wiring_static_passed_cargo_deferred_implementation_cadence`; runtime dimension correlation status: `render_plan08_runtime_shader_variant_dimension_correlation_product_passed_renderdoc_deferred`; Velocity/TAA cache-owner status: `render_plan08_velocity_taa_variant_id_pipeline_cache_static_passed_cargo_deferred_implementation_cadence`; Velocity source-cache status: `render_plan08_velocity_pipeline_template_source_cache_static_passed_cargo_deferred_implementation_cadence`; TAA reactive source-cache status: `render_plan08_taa_reactive_mask_template_source_cache_static_passed_cargo_deferred_implementation_cadence`.

Velocity/TAA variant-id pipeline cache owner is the 2026-06-24 non-Base pass convergence slice. `MeshPipelineCache` stores velocity, TAA reactive mask, and TAA reactive material mask WGPU pipelines by `MeshPipelineVariantId`; the `_for_variant` entry points read `(kind, PipelineKey, ShaderVariantKey)` through `pipeline_and_shader_key_for_variant(...)`, and pass shader module keys include the canonical shader variant identity. This slice did not claim template WGSL source, disk-cache writes, WGPU device validation, or RenderDoc acceptance by itself; the later Velocity and TAA source-cache cutovers recorded below move both passes onto template source/hash identity.

Velocity pipeline template source cache cutover moves the object Velocity pass onto the same template DTO and source-hash identity as the Base mesh path. `shader_source.rs` now exposes `mesh_pipeline_velocity_template_source_for_geometry(...)`, `ensure_velocity_pipeline.rs` selects that source through `shader_variant_key.geometry_source`, records disk-cache lookup/write errors through `mesh_pipeline_shader_source_with_cache(...)`, and includes the final template source hash in `velocity_mesh_shader_key(...)`. `zr_template_velocity.wgsl` owns `ZrVelocityVertexInput` with `@location(8) previous_position`, computes current and previous unjittered clip positions through GPUScene transforms, and exposes runtime `vs_main`/`fs_main`; `zr_template_velocity_alpha.wgsl` is selected for alpha-test variants and calls `zr_material_surface(...)` only for discard. `create_velocity_mesh_pipeline.rs` now points WGPU descriptors at those template entry names while keeping `GpuMeshVertex::previous_position_layout()`. Status: `render_plan08_velocity_pipeline_template_source_cache_static_passed_cargo_deferred_implementation_cadence`.

TAA reactive mask template source cache cutover moves the temporal reactive mask pass onto the same source DTO and source-hash identity without adding a new main material pass enum. `template/taa_reactive_mask.rs` assembles scene runtime, GPUScene, surface types, geometry include, standard material surface, and `zr_template_taa_reactive_mask.wgsl` while omitting Forward light/shadow/shading includes. `shader_source.rs` exposes `mesh_pipeline_taa_reactive_mask_template_source_for_geometry(...)`; `ensure_taa_reactive_mask_pipeline.rs` selects that source through `shader_variant_key.geometry_source`, reuses `mesh_pipeline_shader_source_with_cache(...)`, and includes the final source hash in `taa_reactive_mask_mesh_shader_key(...)`. `create_taa_reactive_mask_mesh_pipeline.rs` points WGPU descriptors at `vs_main`, `fs_taa_reactive_mask`, and `fs_taa_reactive_material_mask` while keeping `GpuMeshVertex::layout()`. Status: `render_plan08_taa_reactive_mask_template_source_cache_static_passed_cargo_deferred_implementation_cadence`.

Velocity/TAA WGPU device pipeline validation code now exists for those two template/source-cache paths. `graphics/scene/scene_renderer/mesh/mesh_pipeline/test_support.rs` provides the test-only scene/material/GPUScene pipeline layout fixture, `velocity_mesh_pipeline_creates_on_wgpu_device_with_template_shader` creates the Velocity `Rg16Float` render pipeline from `mesh_pipeline_velocity_template_source_for_geometry(...)`, and `taa_reactive_mask_mesh_pipeline_creates_on_wgpu_device_with_template_shader` creates both TAA reactive and material mask `R8Unorm` pipelines from `mesh_pipeline_taa_reactive_mask_template_source_for_geometry(...)`. The generated WSL lib-test binary passes Velocity, TAA, Shadow, GBuffer, and DepthPrepass device filters under `WGPU_BACKEND=vulkan`, while the old default/GL path exits 139. `graphics/backend/render_backend/config.rs` now defaults no-env offscreen selection to `wgpu::Backends::PRIMARY` and keeps explicit `WGPU_BACKEND=gl` available for GL regression coverage, but the new default-path binary did not recompile within the current WSL window. Status: `render_plan08_velocity_taa_wgpu_device_pipeline_validation_implemented_validation_not_closed`; backend follow-up status: `render_plan08_offscreen_backend_primary_default_implemented_recompile_not_closed`.

Shadow pipeline template source cache cutover moves ShadowDepth and ShadowDepthAlphaMask onto the same source DTO and source-hash identity while deleting the renderer-private shadow shader body. The old renderer-local shadow source owner and inline shadow WGSL body are gone; `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs` exposes `mesh_pipeline_shadow_template_source_for_geometry(...)` for `graphics/shader/wgsl/zr_template_shadow.wgsl` and `graphics/shader/wgsl/zr_template_shadow_alpha.wgsl`; `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs` selects that source through `shader_variant_key.geometry_source`, reuses `mesh_pipeline_shader_source_with_cache(...)`, and includes the final source hash in `shadow_mesh_shader_key(...)`. `graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs` points WGPU descriptors at template `vs_main` and, for alpha-test, `fs_main`, keeps `GpuMeshVertex::layout()`, and preserves the shadow depth-bias constants. Shadow replay now asks `MeshPipelineCache` for `command.pipeline_variant_id` instead of owning fixed render pipelines. `shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash` and `shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias` keep the source/cache and descriptor contracts covered. Scoped lib cargo check and focused structure-guard no-run compile passed with existing warnings; WGPU/RenderDoc validation remains deferred. Status: `render_plan08_shadow_pipeline_template_source_cache_static_passed_cargo_check_test_compile_wgpu_deferred`.

The 2026-06-24 Mesh pipeline standard material template source cutover moves the Base mesh fallback/missing-shader WGSL source to the template assembler while leaving ready custom WGSL on the raw source path. `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs` builds `MeshPipelineShaderSource` through `mesh_pipeline_standard_material_template_source(...)` for the static wrapper and `mesh_pipeline_standard_material_template_source_for_geometry(...)` for explicit built-in geometry sources. The source owner uses `standard_material_surface_source_for_features(...)` and `assemble_material_shader_template(...)`; it now selects the requested `GeometrySourceDescriptor` instead of assuming every runtime template source is static mesh. `PipelineKey::shader_feature_bits()` is crate-visible so this path reuses the same alpha-test, receive-shadows, and double-sided feature projection as `ShaderVariantKey`. Disk-cache inputs now include template include content hashes plus the final WGSL source hash, and `mesh_shader_module_cache_key(...)` also includes that source hash so template/cutoff/source changes cannot reuse an old shader module. The focused source tests are `mesh_pipeline_standard_material_template_source_assembles_forward_base_source`, `mesh_pipeline_standard_material_template_source_uses_requested_geometry_source`, `mesh_pipeline_template_source_hashes_include_template_revision`, and `mesh_pipeline_template_source_hashes_feed_disk_and_module_keys`; statuses: `render_plan08_mesh_pipeline_standard_material_template_source_static_passed_cargo_deferred_implementation_cadence` and `render_plan08_runtime_mesh_variant_geometry_source_key_wiring_static_passed_cargo_deferred_implementation_cadence`.

The Mesh pipeline shader source owner split keeps template assembly out of `ensure_pipeline.rs`. `shader_source.rs` owns raw WGSL wrapping, final source hashing, fallback/missing standard material template source generation, requested built-in geometry descriptor selection, and the shared `MeshPipelineShaderSource` DTO that dynamic prewarm consumes. `ensure_pipeline.rs` now calls `mesh_pipeline_shader_source(...)` with `shader_variant_key.geometry_source` and remains responsible for WGPU shader module creation, render pipeline creation, disk-cache lookup/write accounting, and source-hash-aware module keys. `runtime_15_render_shader_template_assembly_is_folder_backed` locks the split by requiring `shader_source.rs` to contain the assembly imports and explicit geometry-source builder, and requiring `ensure_pipeline.rs` not to contain `builtin_geometry_source_descriptor`, `assemble_material_shader_template`, or `standard_material_surface_source_for_features`. Status: `render_plan08_mesh_pipeline_shader_source_owner_split_static_passed_cargo_deferred_implementation_cadence`.

The Runtime 15 M3 mesh pipeline shader source tests child-owner split keeps that production source owner below the R4.3 budget without changing source assembly behavior. `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs` now only mounts `#[cfg(test)] #[path = "shader_source/tests.rs"] mod tests;`; `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs` owns the module-local source assembly tests and mounts `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs` for the custom shading-model WGPU module validation. Status: `runtime_15_mesh_pipeline_shader_source_tests_child_owner_split_static_passed_cargo_deferred`; guard: `runtime_15_render_shader_template_assembly_support_children_are_folder_backed`.

Builtin fallback prewarm template source alignment makes the staged built-in fallback cache use that same source DTO. `dynamic_api/shader_prewarm.rs::builtin_fallback_shader_prewarm_manifest()` now calls `mesh_pipeline_standard_material_template_source(...)` through the crate-visible scene facade and writes the returned template WGSL, include/source hashes, and `zr-material-template-v1` revision into `ShaderVariantPrewarmRequest`. It no longer imports or writes `FALLBACK_MESH_SHADER`; if the controlled template assembly fails, the built-in manifest is empty rather than producing a stale wrong cache entry. `bin/zircon_shader_prewarm/manifest.rs::builtin_fallback_manifest_for_quality_tiers_and_geometry_sources(...)` extends the pure `--builtin-fallback` path across requested built-in geometry sources by reusing `dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry(...)`, while the older quality-only wrapper remains static by default. The focused tests are `builtin_fallback_shader_prewarm_manifest_uses_mesh_template_source`, `shader_prewarm_builtin_fallback_manifest_expands_requested_geometry_sources`, and `runtime_15_builtin_fallback_prewarm_uses_template_source`; statuses: `render_plan08_builtin_fallback_prewarm_template_source_static_passed_cargo_deferred_implementation_cadence` and `render_plan08_builtin_fallback_multi_geometry_prewarm_static_passed_cargo_no_result`.

Asset-root builtin standard material template prewarm extends that DTO path to material-authored builtin references and now emits five pass templates for each requested quality/geometry pair. `dynamic_api::builtin_standard_material_shader_prewarm_manifest(...)` keeps the static default, while `dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry(...)` takes the material-derived feature bits, shading model id, optional alpha cutoff, explicit `GeometrySourceId`, and quality tier list. Both route through `mesh_pipeline_standard_material_template_source_for_shader_pass(...)` so Forward, GBuffer, DepthPrepass, Shadow, and Velocity prewarm entries use the same mesh source owner and source-hash inputs as runtime template consumers. DepthPrepass prewarm is intentionally pure depth-only: opaque variants select `zr_template_depth.wgsl` without material fragment code, and alpha-test variants select `zr_template_depth_alpha.wgsl` with alpha clip but no normal-target encode. `bin/zircon_shader_prewarm/manifest.rs` uses the explicit builder for each requested geometry source when `.zmaterial` references `builtin://shader/pbr.wgsl`; raw asset-root `.zshader` and standalone `.wgsl` sources continue through the scanned-source path. The focused tests are `builtin_standard_material_shader_prewarm_manifest_projects_material_features`, `builtin_standard_material_shader_prewarm_manifest_projects_geometry_source`, `shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source`, `mesh_pipeline_standard_material_shader_pass_source_keeps_depth_only_contract`, and `runtime_15_shader_prewarm_builtin_standard_material_template_source_is_wired`; statuses: `render_plan08_asset_root_builtin_standard_material_template_prewarm_static_passed_cargo_deferred_implementation_cadence`, `render_plan08_asset_root_builtin_standard_material_multi_geometry_prewarm_static_passed_cargo_deferred_implementation_cadence`, and `render_plan08_builtin_material_multi_pass_depth_only_prewarm_tests_passed_renderdoc_deferred`.

The staged-cache acceptance slice now covers the next step for builtin standard material requests. `builtin_standard_material_prewarm_writes_restart_hits_and_wgpu_modules` writes the generated static and skinned five-pass manifests through `prewarm_shader_variants(...)`, reopens `ShaderVariantCacheDisk` to simulate a restart, requires `ShaderVariantCacheDiskLookup::Hit` for every derived `ShaderVariantCacheDiskKey::from_variant_key(...)`, and creates WGPU shader modules from the read-back WGSL under a validation error scope. Status: `render_plan08_builtin_material_staged_prewarm_cache_hit_wgpu_module_passed_renderdoc_deferred`.

Runtime Base mesh staged prewarm cache hit closes the next consumer-level step. `runtime_base_mesh_pipeline_uses_staged_prewarm_without_compile_miss` writes the builtin fallback/standard material staged manifest into a temporary staged root, injects `ShaderVariantCacheDisk::with_fallback_roots(runtime_root, [staged_root])` into `MeshPipelineCache`, and calls `ensure_pipeline_for_variant(...)` on a real offscreen WGPU device. It then requires `ShaderVariantMissReport.disk_hit_count == 1` and `compile_miss_count == 0` while the Base mesh render pipeline is created under WGPU validation scope. Status: `render_plan08_runtime_base_mesh_staged_prewarm_cache_hit_wgpu_pipeline_passed_renderdoc_deferred`.

Product Base mesh second-launch staged prewarm closes the product-facing Base/Opaque slice. `graphics/tests/render_product_mesh_cache/staged_prewarm/mod.rs::render_product_base_mesh_second_launch_uses_staged_prewarm_without_compile_miss` writes the staged manifest once, then creates two fresh `WgpuRenderFramework` instances with `ShaderVariantCacheDisk::with_fallback_roots(runtime_root, [staged_root])` injected through the test-only `replace_shader_variant_disk_cache_for_tests(...)` seam. The product pipeline registers `mesh.opaque` with side effects, uses `DisplayMode::Shaded` to force BaseScenePass replay, and asserts both launches report shader-variant requests, staged disk hits, `compile_miss_count == 0`, no runtime cache writes/errors, mesh replay state changes, skinned draws, and executed `mesh.opaque` evidence. `runtime_15_product_base_mesh_staged_prewarm_is_wired` keeps the product child owner and status/docs anchors locked. Status: `render_plan08_product_base_mesh_second_launch_staged_prewarm_passed_renderdoc_deferred`.

Runtime mesh variant geometry-source key wiring extends the live Base mesh path beyond the static wrapper. `MeshDrawGeometrySource::shader_geometry_source_id()` is the bridge from draw-source classification to shader `GeometrySourceId`: prepared and dynamic CPU-side batches remain static mesh, while `DynamicGpuSkinningSource` resolves to skinned mesh. Direct CPU-morphed draw-source metadata keeps CPU-baked morph draws explicit: active morph weights on direct mesh snapshots create `PendingMeshGeometry::CpuMorphed`, `pending_mesh_geometry_source(...)` and the command cache extract/plan map it to `DynamicCpuMorphedSource`, `uses_cpu_morphed_source()` exposes the shared CPU-morphed classification, and the shader id remains `GEOMETRY_SOURCE_ID_STATIC_MESH`. CPU-morphed GPU-skinning draw-source metadata keeps the skinned counterpart explicit too: `PendingSkinnedGpuSource::CpuMorphed` maps to `DynamicCpuMorphedGpuSkinningSource`, `uses_cpu_morphed_gpu_skinning_source()` exposes the GPU-skinning classification, prepared queue stats still count it as dynamic geometry, and `MeshPassBuildContext` resolves it to `GEOMETRY_SOURCE_ID_SKINNED_MESH` through the queue profile. Those conservative shader ids remain intentional for CPU-baked fallback draws, but payload-backed direct morph draws now select the real morphed shader ids: `DynamicGpuMorphedSource` resolves to `GEOMETRY_SOURCE_ID_MORPHED_MESH`, and `DynamicGpuSkinnedMorphedSource` resolves to `GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH`. The previous morph-weight velocity follow-up keeps that same geometry-source selection live even when current weights return to zero, because previous-only payloads still need the Morphed shader's `fetch_prev_position(...)` path. Status: `render_plan08_runtime_mesh_variant_geometry_source_key_wiring_static_passed_cargo_deferred_implementation_cadence`; CPU-morphed follow-up statuses: `render_plan08_direct_cpu_morphed_draw_source_metadata_check_passed_wgpu_deferred` and `render_plan08_cpu_morphed_gpu_skinning_draw_source_metadata_static_passed_cargo_deferred_active_lanes`; payload-backed/product observability statuses: `render_plan08_morph_geometry_source_selection_static_passed_wgpu_deferred`, `render_plan08_morph_gpu_source_product_guard_wgpu_passed_renderdoc_deferred`, `render_plan08_morph_gpu_cpu_product_parity_wgpu_passed_renderdoc_deferred`, and `render_plan08_morph_previous_weights_velocity_check_passed_product_deferred`.

Morph storage buffers upload now closes the first GPUScene data-owner handoff for those reserved morphed-geometry bindings. `GpuMorphDelta` and `GpuMorphWeight` define the storage row ABI, GPUScene group3 reserves bindings 7 and 8 beside the existing palette and VirtualGeometry slots, `GpuScene::upload_morph_buffers(...)` owns fallback/live buffer recreation, CPU shadows, queue writes, and bind group rebuild, and `zr_gpu_scene.wgsl` exposes guarded `zr_gpu_scene_morph_delta(...)` / `zr_gpu_scene_morph_weight(...)` helpers consumed by the morphed geometry includes. Status: `render_plan08_morph_storage_buffers_upload_check_passed_wgpu_deferred`; `render_gpu_scene_uploads_morph_storage_buffers` and `runtime_15_morph_storage_buffers_upload_is_wired` lock the ABI, bindings, upload owner, WGSL helper route, docs/status, and session anchors. This deliberately does not switch CPU-baked morph draws to morphed shader ids, and it does not close asset-to-GPU morph payload projection, product WGPU capture, or product miss=0 acceptance.

## Current Limits

This module is not a full Bevy `ShaderPlugin`, `ShaderCache`, or `PipelineCache`. It does not parse WGSL imports, resolve shader include graphs, apply shader definitions to Naga composition, validate Naga modules, track dependent pipelines, deduplicate bind group layouts, or support async pipeline creation states.

Runtime Base mesh template source selection now distinguishes static, skinned, morphed, and skinned-morphed geometry where the draw queue exposes the matching source. Asset-root builtin standard-material prewarm and pure built-in fallback prewarm now emit template requests for each requested built-in geometry source and for Forward, GBuffer, DepthPrepass, Shadow, and Velocity pass types that the source owner can assemble. Direct CPU-morphed draw-source metadata is preserved by `PendingMeshGeometry::CpuMorphed`, `DynamicCpuMorphedSource`, and `uses_cpu_morphed_source()`, but it deliberately resolves to `GEOMETRY_SOURCE_ID_STATIC_MESH`; CPU-morphed GPU-skinning metadata is preserved by `DynamicCpuMorphedGpuSkinningSource` and `uses_cpu_morphed_gpu_skinning_source()`, but it deliberately resolves to `GEOMETRY_SOURCE_ID_SKINNED_MESH`. GPUScene now owns morph bindings 7/8 plus payload binding 11, payload-backed direct morph selection uses the morphed shader geometry ids, product stats/diagnostics can prove the selected source, and previous morph weights are code-wired for Velocity through a second weight block. Direct and skinned pixel GPU-vs-CPU morph parity now have product guards below; RenderDoc/product velocity capture remains pending.

Asset-level shader readiness is intentionally narrower than renderer readiness. It can report missing runtime WGSL, invalid entry-point stage tokens, duplicate or empty shader definitions, source-only versus redirected import rows, and copied validation diagnostics, but it does not decide whether a concrete device can create a module or pipeline.

The layout descriptor is serialized intent, not reflection. It does not yet derive bind groups from WGSL, validate binding type compatibility, model dynamic offsets, express texture sample types, or map push constants to backend feature gates. Future shader milestones should add those checks below the framework DTO layer so `.zshader` authoring and renderer preparation continue to share one stable contract.

Asset-root prewarm scanning is still intentionally conservative, but it no longer hardcodes the geometry-source dimension or the built-in standard-material pass to Forward. It defaults to static-mesh requests for compatibility, and explicit `--geometry-source` / `--shader-geometry-source` values can expand built-in static, skinned, morphed, and skinned+morphed requests across the pass dimension from `.zshader` entry-point stages plus material-instance alpha-test, double-sided, built-in shading-model variants, explicit custom shading-model plugin ids, explicit custom geometry-source plugin ids, alpha-blend Forward-only filtering, selected quality tiers, source-hash-derived edit revisions, explicit resource-registry revision overlays, and project/plugin shader permutation registry overlays. Builtin standard-material `.zmaterial` references and pure `--builtin-fallback` requests now use the same requested geometry source list and emit Forward, GBuffer, DepthPrepass, Shadow, and Velocity template requests, while custom `.zshader` and standalone `.wgsl` requests still remain raw scanned source payloads. Runtime draw submission can carry a non-Medium `ShaderQualityTier` into `ShaderVariantKey.quality`, build staging can prewarm matching quality tiers, built-in geometry sources, explicit plugin geometry-source ids, pass-specific standard-material templates, explicit custom shading-model ids, project/plugin permutation registry ids, asset-root edit revisions, explicit project shader asset roots, selected plugin asset roots, and exported live shader resource revisions, and the base mesh WGPU cache path now consumes that same quality-aware key. The template assembler can now produce deterministic WGSL/hash inputs for those built-in geometry sources, has a standard material surface source owner, has a Naga validation helper, carries uv1/tangent interpolation through pass templates, aligns `ZrVertexInput` with runtime mesh vertex attributes, applies runtime scene/GPUScene world-to-clip transform in `zr_build_vertex_output(...)`, exposes generic runtime `vs_main`/`fs_main` aliases over `zr_vs_main_impl`/`zr_fs_main_impl`, samples the standard normal map, and has template-level alpha clip behavior for Forward/GBuffer/DepthPrepass/Shadow/Velocity when alpha-test is enabled. The Base mesh fallback/missing-shader runtime source now consumes that standard material Forward template output and feeds include/source hashes into the runtime disk/module cache keys, Velocity consumes template source with previous-position input and source-hash module identity, TAA reactive mask consumes its auxiliary template source with source-hash module identity, Shadow consumes template source with source-hash module identity, current runtime DepthPrepass consumes normal-target template source, Deferred GBuffer consumes albedo/material template source, and the built-in fallback plus asset-root builtin standard material prewarm manifests write matching pass-specific source/hash/revision payloads. Builtin standard material staged prewarm now has focused write, restart cache-hit, and WGPU shader-module validation evidence under `render_plan08_builtin_material_staged_prewarm_cache_hit_wgpu_module_passed_renderdoc_deferred`; runtime Base mesh now has staged fallback root hit, WGPU pipeline creation, and `compile_miss_count == 0` evidence under `render_plan08_runtime_base_mesh_staged_prewarm_cache_hit_wgpu_pipeline_passed_renderdoc_deferred`; Product Base mesh second-launch staged prewarm has two fresh product submits with staged disk hits and zero compile misses under `render_plan08_product_base_mesh_second_launch_staged_prewarm_passed_renderdoc_deferred`; asset-root shader edit revision export is locked under `render_plan08_asset_root_shader_edit_revision_export_passed_cargo_renderdoc_deferred`; explicit custom geometry-source id prewarm is wired under `render_plan08_asset_root_custom_geometry_source_id_prewarm_typecheck_passed_test_timeout_no_result`; project shader asset roots auto-export is wired under `render_plan08_project_shader_asset_roots_auto_export_python_static_passed_cargo_deferred`; VirtualGeometry page/cluster shader bindings are wired under `render_plan08_virtual_geometry_page_cluster_shader_bindings_static_passed_cargo_deferred`; VirtualGeometry resident buffer upload is wired under `render_plan08_virtual_geometry_resident_buffers_upload_static_passed_cargo_deferred`; report-level prewarm dimension diagnostics are wired under `render_plan08_prewarm_dimension_diagnostics_typecheck_passed_test_timeout_no_result`; explicit resource registry overlay is wired under `render_plan08_asset_root_resource_registry_revision_overlay_typecheck_passed_test_timeout_no_result`; and Shader permutation registry overlay is wired under `render_plan08_shader_permutation_registry_overlay_focused_tests_passed_renderdoc_deferred`. Asset-root custom shader scanning still does not invoke the assembler or validator. RenderDoc/product capture remains pending. The asset-root prewarm tool can now consume project/plugin permutation registry JSON from explicit `--shader-permutation-registry` paths or asset-root `shader_permutation_registry.json`, and `tools/zircon_build.py --shader-asset-root <path>` can add project roots to the same staged `--asset-root` / automatic `shader_resource_records.json` export path locked by `test_zircon_build_resolves_project_shader_asset_roots_for_prewarm` and `runtime_15_shader_prewarm_project_asset_roots_auto_export_is_wired`; however, it does not yet automatically generate the full project/plugin shader, shading-model, and geometry-source registry export. Base shader-source requests also remain conservative when no material instance narrows the pass set. Asset/importer page payload decode and meshlet vertex ordinal now have separate static closeout anchors; staged-cache compile acceptance plus product VG draw evidence are still needed before long-lived edited projects can claim the same product-level acceptance breadth as the focused staged-cache test.

Build-tool prewarm report consumption now reads `shader_variants_report.json` after the staged prewarm process returns, prints the `dimension_summary` groups as a compact log summary, and then propagates any non-zero exit code. Status: `render_plan08_build_tool_prewarm_dimension_summary_python_tests_passed_cargo_deferred`.

Shader permutation registry overlay now lets that same staged path consume external `shader_permutation_registry.json` files for custom geometry-source and custom shading-model ids. Status: `render_plan08_shader_permutation_registry_overlay_focused_tests_passed_renderdoc_deferred`; `shader_permutation_registry_paths`, `shader_prewarm_permutation_registry_merges_custom_geometry_and_shading_ids`, and `runtime_15_shader_prewarm_permutation_registry_overlay_is_wired` lock the input seam. Full project/plugin registry generation, real Naga/WGPU prewarm compile, RenderDoc/product capture, and runtime pure-depth DepthPrepass remain open.

Shader permutation registry auto-export now lets the staged build tool generate the same overlay schema from known custom id inputs when no explicit registry path is supplied. `BuildConfig.shader_prewarm_permutation_registry_path` writes `ZirconEngine/cache/shader_permutation_registry.json`, `write_generated_shader_permutation_registry(...)` emits `geometry_source_ids` / `shading_model_ids`, and `shader_permutation_registry_paths_for_prewarm(...)` passes either the explicit override or the generated file to `zircon_shader_prewarm`. Status: `render_plan08_build_tool_shader_permutation_registry_auto_export_focused_tests_passed_renderdoc_deferred`; `test_write_generated_shader_permutation_registry_writes_json` and `runtime_15_shader_prewarm_permutation_registry_auto_export_is_wired` lock the handoff. This is still a build-tool export from `--shader-geometry-source-id` / `--shader-shading-model-id`, not full project/plugin shader, shading-model, or geometry-source discovery.

Plugin shader permutation registry auto-export now lets selected plugin package manifests contribute the same custom id records to staged prewarm without repeating them on the command line. `PluginPackageManifest.shader_permutation` owns the manifest schema, `virtual_geometry` declares `custom:virtual_geometry = 4` in both its descriptor and static `plugin.toml`, and the build helper merges selected plugin records with explicit CLI records before writing the generated `shader_permutation_registry.json`. Status: `render_plan08_plugin_shader_permutation_registry_auto_export_focused_tests_passed_renderdoc_deferred`; `test_zircon_build_discovers_plugin_shader_permutation_records`, `test_generated_shader_permutation_registry_document_merges_selected_plugin_ids`, `test_build_command_uses_generated_shader_permutation_registry_for_selected_plugin_ids`, and `runtime_15_shader_prewarm_plugin_permutation_registry_auto_export_is_wired` lock selected-only discovery and handoff. This still does not imply full project shader resource discovery, custom shading-model plugin descriptor registration, real Naga/WGPU prewarm compile, RenderDoc/product capture, or runtime pure-depth DepthPrepass migration.

Plugin shader permutation registry export contract now validates that generated handoff before the staged prewarm subprocess runs. `validate_shader_permutation_registry_export_contract(...)` reads the generated `ZirconEngine/cache/shader_permutation_registry.json` and requires the current selected-plugin plus explicit CLI geometry-source and shading-model id specs to appear in its `geometry_source_ids` / `shading_model_ids` arrays. Status: `render_plan08_plugin_shader_permutation_registry_export_contract_python_passed_cargo_deferred`; `test_validate_generated_registry_requires_selected_plugin_ids`, `test_prewarm_shaders_validates_generated_registry_before_run`, and `runtime_15_shader_prewarm_plugin_permutation_registry_auto_export_is_wired` lock the generated-registry acceptance gate. This closes the build-tool handoff contract only; real Naga/WGPU prewarm compile, RenderDoc/product capture, full project/plugin shader resource discovery, and miss=0 product acceptance remain open.

2026-07-01 shader prewarm source-hash helper support: the plugin shading-model prewarm manifest path now has a local `shader_prewarm_source_hash(...)` helper in `dynamic_api/shader_prewarm.rs`, used when `PluginShadingModelTemplateSource::from_template(...)` adds the assembled WGSL source hash to the include/content hash list. The helper uses `blake3::hash(source.as_bytes()).to_hex().to_string()`, matching the existing shader include and variant-cache hash convention. Validation: `cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never` passed with existing warnings, and the dependent `zircon_app --bin zircon_editor` build passed in the same external target directory. This is a compile-gate support fix, not product RenderDoc/prewarm acceptance.

Plugin shading-model descriptor registration now gives custom shading models the real descriptor owner that the earlier F11 review required before reintroducing plugin registration. `PluginPackageManifest.shading_models` serializes `ShadingModelDescriptor` rows, `RuntimeExtensionRegistry` tracks them by plugin owner for register/merge/revoke flows, and `graphics/material/shading_models/registry.rs::register_plugin_descriptor(...)` keeps plugin ids out of the built-in shading-model range. The build tool also derives selected-plugin prewarm `shader_shading_model_ids` from those `[[shading_models]]` rows, so a plugin does not have to duplicate the same id in `shader_permutation.shading_model_ids`. Status: `render_plan08_plugin_shading_model_descriptor_registration_typecheck_python_passed_libtest_blocked_by_ui_input_error`; `plugin_package_manifest_declares_custom_shading_model_descriptors`, `test_zircon_build_discovers_plugin_shading_model_descriptors_as_shader_ids`, and `runtime_15_shader_prewarm_plugin_shading_model_descriptor_registration_is_wired` lock the schema, runtime extension registry, graphics registry guard, build discovery, and docs anchors. This descriptor-owner slice did not close real Naga/WGPU prewarm compile, RenderDoc/product capture, or runtime pure-depth DepthPrepass migration.

Plugin shading-model descriptor registry export now carries those descriptor rows through the generated prewarm registry. `tools/zircon_build.py::PluginPackage.shader_shading_model_descriptors` preserves selected plugin descriptors, `tools/zircon_build_shader_prewarm.py::generated_shader_permutation_registry_document(...)` writes `shading_model_descriptors`, and `zircon_shader_prewarm/manifest/permutation_registry.rs` normalizes descriptor tokens/ids into the overlay while rejecting incompatible duplicate ids through `IncompatibleShadingModelDescriptor`. Status: `render_plan08_plugin_shading_model_descriptor_registry_export_static_passed_cargo_deferred`; `test_generated_shader_permutation_registry_document_exports_selected_plugin_shading_model_descriptors`, `shader_prewarm_permutation_registry_merges_custom_shading_model_descriptors`, and `runtime_15_shader_prewarm_plugin_shading_model_descriptor_registration_is_wired` lock the build helper, generated registry, overlay, run/error owner, docs, and session anchors. This is registry/overlay contract evidence only; descriptor-driven shader template dispatch and real custom shading-model Naga/WGPU compile remain open.

Descriptor-driven forward shading include dispatch closes the next static template handoff after that registry export. `MaterialShaderTemplateRequest::with_shading_model_descriptor(...)` lets Forward template assembly receive a `ShadingModelDescriptor`, `graphics/shader/template/include_registry.rs::shading_model_forward_include_for(...)` resolves `forward_include` to the built-in `zr_shading_standard_pbr.wgsl` include, and `ShaderTemplateAssemblyError::UnknownShadingInclude` rejects unknown plugin include tokens before assembly can silently fall back. Status: `render_plan08_descriptor_driven_forward_shading_include_dispatch_static_passed_cargo_deferred`; `render_shader_template_uses_shading_model_descriptor_forward_include`, `render_shader_template_rejects_unknown_shading_model_forward_include`, and `runtime_15_render_shader_template_assembly_is_folder_backed` lock the request field, include resolver, error contract, and docs anchors. This is Forward include selection only; plugin WGSL source export, GBuffer/deferred descriptor include dispatch, and real custom shading-model Naga/WGPU compile remain open.

Plugin geometry-source descriptor registration now gives custom geometry sources the same manifest/runtime owner path. `PluginPackageManifest.geometry_sources` serializes `GeometrySourceDescriptor` rows, `RuntimeExtensionRegistry` tracks them by plugin owner for register/merge/revoke flows, and selected plugin `[[geometry_sources]]` rows feed `tools/zircon_build.py::discover_plugins(...)` so staged prewarm derives `shader_geometry_source_ids` without requiring a duplicate `shader_permutation.geometry_source_ids` row. `virtual_geometry` declares `custom:virtual_geometry = 4` through both its runtime descriptor and static `plugin.toml`, while the legacy id row remains accepted as a compatibility input for staged registries. Status: `render_plan08_plugin_geometry_source_descriptor_registration_typecheck_python_cargo_check_passed_renderdoc_deferred`; `plugin_package_manifest_declares_custom_geometry_source_descriptors`, `test_zircon_build_discovers_plugin_geometry_source_descriptors_as_shader_ids`, and `runtime_15_shader_prewarm_plugin_geometry_source_descriptor_registration_is_wired` lock the schema, runtime extension registry, build discovery, and docs anchors. This descriptor-owner slice did not close real Naga/WGPU prewarm compile, RenderDoc/product capture, complete project shader resource discovery, or runtime pure-depth DepthPrepass migration.

Plugin geometry-source descriptor runtime WGPU prewarm closes the next handoff after descriptor registration. `tools/zircon_build.py::resolve_config(...)` now treats explicit `--plugins` as selected shader/geometry contribution input even when the build target is only `runtime`; only the `plugins` target triggers plugin binary packaging. The generated permutation registry now writes `geometry_source_descriptors` beside `geometry_source_ids`, `bin/zircon_shader_prewarm/manifest/permutation_registry.rs` merges those descriptors into the runtime overlay, and `dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor(...)` routes built-in standard-material fallback requests through `mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor(...)`. `zr_geometry_virtual_geometry.wgsl` is registered through `graphics/shader/template/include_registry.rs` so the selected plugin descriptor can assemble all six material pass templates under WGPU module validation. Status: `render_plan08_plugin_geometry_descriptor_runtime_wgpu_prewarm_passed_product_deferred`; the live staged command wrote 12/12 cache variants and validated 12/12 WGPU modules for geometry source ids `0` and `4`. This proves descriptor-to-WGPU prewarm connectivity, not the full VirtualGeometry page/cluster runtime fetch path, RenderDoc/product capture, full live registry export, or second-launch miss=0 acceptance.

VirtualGeometry page/cluster shader bindings close the shader-side part of that remaining fetch gap. `GeometrySourceBindingKind` now includes `VirtualGeometryClusters`; the virtual_geometry runtime descriptor and static `plugin.toml` require both `virtual_geometry.pages` and `virtual_geometry.clusters`; and GPUScene group3 reserves storage bindings 9 and 10 for those buffers without colliding with the morph storage slots at 7 and 8. `zr_gpu_scene.wgsl` exposes `zr_virtual_geometry_pages` and `zr_virtual_geometry_clusters`; `zr_geometry_virtual_geometry.wgsl` reads primitive `payload_slot`, page table cluster base and vertex count, then fetches position, normal, and tangent words with bounds checks and vertex-input fallback. Status: `render_plan08_virtual_geometry_page_cluster_shader_bindings_cargo_wrapper_wgpu_layout_passed_renderdoc_deferred`; direct-binary status `render_plan08_virtual_geometry_page_cluster_shader_bindings_direct_binary_wgpu_layout_passed_renderdoc_deferred` and original static status `render_plan08_virtual_geometry_page_cluster_shader_bindings_static_passed_cargo_deferred` remain as historical guard anchors. `mesh_pipeline_virtual_geometry_template_source_declares_page_cluster_fetch_bindings`, `render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings`, and `runtime_15_virtual_geometry_page_cluster_shader_bindings_are_wired` lock the descriptor, manifest, GPUScene slot, WGSL fetch, prewarm fixture, docs/status, and session anchors. The 2026-07-02 direct-binary backfill passed the shader-source and WGPU-layout focused tests 1/1 each; the 2026-07-03 no-default Cargo-wrapper backfill passed the same full-path tests 1/1 with 5984 and 5992 filtered; the default-feature Cargo-wrapper backfill passed shader-source, GPUScene layout, and structure-guard tests 1/1 under `render_plan08_virtual_geometry_page_cluster_shader_bindings_default_features_cargo_wrapper_wgpu_layout_passed_renderdoc_deferred`. A follow-up structure-guard closeout repaired current-source compile drift in material defaults, disabled-pass test seeds, command-cache disabled-pass keys, and the shader-template `module_registry.rs` hard-cut owner; `runtime_15_virtual_geometry_page_cluster_shader_bindings_are_wired` then passed 1/1 with 6028 filtered under status `render_plan08_virtual_geometry_page_cluster_shader_bindings_structure_guard_compile_drift_cargo_passed_renderdoc_deferred`. Product VG draw, RenderDoc/product capture, full live registry export, and product miss=0 remain open.

VirtualGeometry cluster payload upload now closes the first explicit resident payload projection into those shader bindings. `RenderVirtualGeometryPagePayload` and `RenderVirtualGeometryPagePayloadVertex` are exported through the debug snapshot surface as `resident_page_payloads`; the production snapshot builder receives the decoded payload sidecar from `FrameSubmissionContext::virtual_geometry_resident_page_payloads()` and keeps mesh-build as the only owner that encodes payload vertices into cluster words. The mesh-build resident upload owner converts each payload vertex into four `GpuVirtualGeometryClusterWord` rows (position, normal, tangent, pad) using `GPU_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX`, shares cluster word storage across repeated resident submissions for the same page, and leaves pages without payload at `vertex_count == 0` so WGSL keeps falling back safely. Status: `render_plan08_virtual_geometry_cluster_payload_upload_static_passed_cargo_deferred`; `virtual_geometry_cluster_words_follow_resident_page_payloads` and `runtime_15_virtual_geometry_cluster_payload_upload_is_wired` lock the DTO, re-export, GPU ABI constant, upload projection, docs/status, and session anchors. Asset payload decode and meshlet vertex ordinal now have follow-up anchors; product VG draw, RenderDoc/product capture, full live registry export, and product miss=0 remain open.

A 2026-07-02 direct-binary WGPU backfill upgrades the focused evidence for that projection to status `render_plan08_virtual_geometry_cluster_payload_upload_direct_binary_wgpu_passed_renderdoc_deferred`. The existing `virtual_geometry_cluster_words_follow_resident_page_payloads` test passed 1/1 from the no-default generated `zircon_runtime` lib-test binary, proving the resident payload rows, shared cluster-word storage, and per-vertex position/normal/tangent/pad encoding without changing production renderer code. The later no-default Cargo-wrapper rerun passed under `render_plan08_virtual_geometry_resident_cluster_upload_cargo_wrapper_passed_renderdoc_deferred`, and the default-feature Cargo-wrapper rerun passed under `render_plan08_virtual_geometry_resident_cluster_upload_default_features_cargo_wrapper_passed_renderdoc_deferred`. RenderDoc/product capture, workspace/full CI, full live registry export, and broader product miss=0 remain open.

VirtualGeometry asset payload decode now feeds that resident payload sidecar from the first-party `virtual_geometry` runtime plugin. `nanite/page_payload.rs` parses cooked `ZVG0` page payload items, maps each `triangle_start` / `triangle_count` range through `ModelPrimitiveAsset` source indices and vertices, and emits `RenderVirtualGeometryPagePayloadVertex { position, normal, tangent }` after local page ids are remapped to global resident page ids. `VirtualGeometryAutomaticExtractInstance::from_model_primitive(...)` preserves primitive vertices and indices, `VirtualGeometryRuntimeExtractOutput` carries `resident_page_payloads`, `FrameSubmissionContext` gates them behind the existing VirtualGeometry enable flag, and `build_virtual_geometry_debug_snapshot(...)` passes them into `RenderVirtualGeometryDebugSnapshot.resident_page_payloads`. Status: `render_plan08_virtual_geometry_asset_payload_decode_static_passed_cargo_deferred`; `render_page_payloads_decode_cooked_triangle_vertices_with_global_page_ids`, the imported cooked model extract assertions, and `runtime_15_virtual_geometry_asset_payload_decode_is_wired` lock the decode module, sidecar chain, snapshot consumer, docs/status, and session anchors. Scoped rustfmt and static anchors passed; `zircon_plugin_virtual_geometry_runtime --lib` check timed out after about 304 seconds and is not counted as passed. This closes cooked asset page payload decode to the render debug snapshot sidecar; product VG draw, RenderDoc/product capture, full live registry export, and product miss=0 remain separate gates.

VirtualGeometry meshlet vertex ordinal now closes the asset-to-shader ordinal handoff that the page/cluster fetch path depends on. `ModelPrimitiveAsset::assign_virtual_geometry_vertex_ordinals()` packs each VG source vertex index into `MeshVertex.joint_indices[0]` and `[1]` as a 16+16 bit value, while non-VG primitives keep their authored joint channels untouched. OBJ/GLTF primitive import, `.model.toml` VG backfill, `MeshAsset::from_model_primitive(...)`, and `MeshAsset::to_model_primitive(...)` all use that standardization path so root model primitives and labeled mesh subassets feed the same ordinal into the vertex buffer. `zr_geometry_virtual_geometry.wgsl` unpacks `v.joints.x | (v.joints.y << 16)` before reading `zr_virtual_geometry_clusters`, and `mesh_pipeline_virtual_geometry_template_source_declares_page_cluster_fetch_bindings` checks the unpack expression. Status: `render_plan08_virtual_geometry_meshlet_vertex_ordinal_direct_binary_asset_shader_passed_renderdoc_deferred`; the original static status remains recorded as `render_plan08_virtual_geometry_meshlet_vertex_ordinal_static_passed_cargo_deferred` for older generated guard binaries. Direct generated-binary runs now pass the model ordinal filter 2/2 plus the focused MeshAsset conversion, OBJ import, `.model.toml` backfill, two glTF import paths, and shader-source unpack tests 1/1 each. The broader ProjectAssetManager `.model.toml` test exposed a stale all-zero-joint expected fixture in the old generated binary; the current source fixture now normalizes expected VG primitives through `assign_virtual_geometry_vertex_ordinals()`, but the Cargo-wrapper rerun is deferred while other Cargo lanes are active. This closes focused direct-binary asset/importer/shader evidence, not the ProjectAssetManager Cargo rerun, RenderDoc/product capture, default features, workspace/full CI, full live registry export, or product miss=0.

VirtualGeometry product draw-source now has focused no-default and default-feature Cargo/WGPU evidence for the product-test automatic draw-source seam after the resident buffer, payload decode, and meshlet ordinal contracts. The frame submission context only calls `build_automatic_virtual_geometry_extract(...)` when VirtualGeometry is enabled and no authored `RenderVirtualGeometryExtract` was submitted, then uses the runtime provider `build_extract_from_meshes(...)` and `ProjectAssetManager::load_model_asset(...)` to build the effective VG extract from normal `RenderMeshSnapshot` model handles. The test provider is split into `plugin_render_feature_fixtures/virtual_geometry_provider.rs`, and `render_product_virtual_geometry_model_asset_uses_automatic_draw_source` registers a cooked VG `ModelAsset`, submits `GeometryExtract::from_meshes(...)`, and verifies `RenderVirtualGeometryPayloadSource::AutomaticFallback` with indirect execution stats. Status: `render_plan08_virtual_geometry_product_draw_source_cargo_wrapper_wgpu_passed_renderdoc_deferred`; default-feature follow-up status: `render_plan08_virtual_geometry_product_draw_source_default_features_wgpu_passed_renderdoc_deferred`. `runtime_15_virtual_geometry_product_draw_source_is_wired` locks submit-context/provider/product-test/docs/status/session anchors and file budgets. The short-name exact direct-binary attempt ran 0 tests and is not counted; the full-path no-default direct-binary run passed 1/1, the fresh no-default Cargo wrapper passed 1/1 with 5881 filtered, and the warmed default-feature Cargo wrapper passed 1/1 with 5933 filtered after the generated default-feature binary passed the same exact filter 1/1. This closes the focused Cargo/WGPU product draw-source proof for no-default and default features, not RenderDoc/product capture, workspace/full CI, full live registry export, or product miss=0.

VirtualGeometry page/cluster product execution now closes the next focused WGPU product layer under status `render_plan08_virtual_geometry_page_cluster_product_execution_wgpu_passed_renderdoc_deferred`. `render_product_virtual_geometry_page_cluster_bindings_drive_visible_frame` is anchored by `PAGE_CLUSTER_PRODUCT_STATUS`, reuses the automatic cooked `ModelAsset` path, and proves a visible product frame while checking public page/cluster execution stats instead of plugin-private provider table counters. The product fixture registers `custom:virtual_geometry` through `GeometrySourceDescriptor`, carries `GeometrySourceBindingKind::VirtualGeometryPages` and `GeometrySourceBindingKind::VirtualGeometryClusters`, emits `ZR_GEOMETRY_SOURCE_VIRTUAL_GEOMETRY`, and feeds the descriptor into the pluginized WGPU framework constructors. The same slice fixes fallback shader source selection so a `builtin://shader/pbr.wgsl` fallback key still uses the standard material template even when the streamer records that URI as a Surface shader. The focused page/cluster product test passed 1/1 with 6133 filtered, the automatic draw-source regression passed 1/1, and `builtin_fallback_shader_loaded_as_surface_still_uses_standard_material_template` passed 1/1; `runtime_15_virtual_geometry_product_draw_source_is_wired` locks the product fixture, tests, docs/status, and session anchors. The default-feature rerun for this exact page/cluster product test is now closed by `render_plan08_virtual_geometry_page_cluster_product_default_features_wgpu_passed_renderdoc_deferred`; RenderDoc/product capture, workspace/full CI, full live registry export, and broader product miss=0 remain open.

VirtualGeometry page/cluster product default-feature WGPU backfill now records that same page/cluster product path under default features. `render_product_virtual_geometry_page_cluster_bindings_drive_visible_frame` passed through the default-feature Cargo wrapper with 1/1 and 6152 filtered after a cold 25m36s build on `E:\cargo-targets\zircon-plan08-vg-product-default-0703`, with repository-existing warnings only. No production renderer, provider, descriptor, WGSL ABI, or fixture behavior changed for this backfill; it reuses the same automatic cooked `ModelAsset`, `custom:virtual_geometry` descriptor, page/cluster binding requirements, visible Unlit capture, and public WGPU execution stats. Status: `render_plan08_virtual_geometry_page_cluster_product_default_features_wgpu_passed_renderdoc_deferred`; guard: `runtime_15_virtual_geometry_product_draw_source_is_wired`, which passed its default-feature Cargo-wrapper guard 1/1 with 6166 filtered after a 9m34s build on the same target dir.

VirtualGeometry page/cluster product execution now also has focused product readback PNG evidence under `render_plan08_virtual_geometry_page_cluster_product_readback_png_passed_renderdoc_deferred`. The ignored export `export_virtual_geometry_page_cluster_product_png` reuses `render_product_virtual_geometry_page_cluster_bindings_drive_visible_frame`, automatic cooked `ModelAsset` VG, visible Unlit capture, `custom:virtual_geometry` page/cluster binding requirements, and the same public execution stats, then writes `docs/tests/runtime/render/runtime_render_plan08_virtual_geometry_page_cluster_product_20260703.png`. Direct generated-binary execution passed 1/1 with 6204 filtered and 7.97s, producing a 320x240 PNG, 2965 bytes, SHA256 `0322783567544681379085E0C944EF40DD2E6453EE4AE0CB5897F12EBBEBDDE6`; the same binary passed `runtime_15_virtual_geometry_product_draw_source_is_wired` 1/1 with 6204 filtered and 0.26s. RenderDoc/product capture, workspace/full CI, full live registry export, and broader product miss=0 remain open.

VirtualGeometry resident buffers upload now has direct-binary, no-default Cargo-wrapper, and default-feature Cargo-wrapper WGPU evidence for the first runtime data-owner handoff for those bindings. `GpuVirtualGeometryPage` and `GpuVirtualGeometryClusterWord` define 16-byte GPUScene page/cluster rows, `GpuScene::upload_virtual_geometry_resident_buffers(...)` owns CPU shadows, storage buffer recreation, queue writes, and bind group rebuild, and mesh draw building uploads resident page rows from `RenderVirtualGeometryDebugSnapshot.execution_segments` before GPUScene draw sync. The draw sync path writes the VirtualGeometry indirect `submission_slot` into primitive and instance `payload_slot`, and execution projection uses `VirtualGeometrySubmissionDetail::payload_slot()` for the same semantic slot. Status: `render_plan08_virtual_geometry_resident_buffers_upload_direct_binary_wgpu_passed_renderdoc_deferred`; no-default Cargo-wrapper status: `render_plan08_virtual_geometry_resident_cluster_upload_cargo_wrapper_passed_renderdoc_deferred`; default-feature Cargo-wrapper status: `render_plan08_virtual_geometry_resident_cluster_upload_default_features_cargo_wrapper_passed_renderdoc_deferred`. `render_gpu_scene_uploads_virtual_geometry_resident_buffers`, `virtual_geometry_page_rows_follow_submission_slots`, and `runtime_15_virtual_geometry_resident_buffers_upload_is_wired` lock the ABI, upload owner, mesh-build handoff, payload slot, docs/status, and session anchors. RenderDoc/product capture, workspace/full CI, full live registry export, and product miss=0 remain open.

Plugin shader asset roots auto-export now makes selected plugin package assets part of the staged prewarm input set. `tools/zircon_build.py::discover_plugins(...)` resolves existing plugin `asset_roots`, the default `assets` root, and legacy `[distribution] assets = ["assets/**"]` roots into `PluginPackage.asset_roots`; `tools/zircon_build_shader_prewarm.py::shader_asset_root_paths_for_prewarm(...)` appends those roots after staged `ZirconEngine/assets`. Because `zircon_shader_prewarm::run` already exports shader resource records for every `--asset-root`, selected plugin WGSL payloads participate in the same `--export-resource-registry` pass as engine assets. Status: `render_plan08_plugin_shader_asset_roots_auto_export_focused_tests_passed_cargo_deferred_renderdoc_deferred`; `test_build_command_includes_selected_plugin_asset_roots` and `runtime_15_shader_prewarm_plugin_asset_roots_auto_export_is_wired` lock the build command, discovery inputs, docs anchors, and owner budget. This closes selected-plugin asset-root participation, but not full live project/plugin shader resource registry export, real Naga/WGPU prewarm compile, or RenderDoc/product capture.

Runtime pure-depth DepthPrepass product migration moves the runtime mesh DepthPrepass consumer from the temporary normal-target contract to the existing depth-only pass template. `MeshPassPipelineKind::DepthPrepass` now maps to `ShaderPassType::DepthPrepass`, `mesh_pipeline_depth_prepass_template_source_for_geometry(...)` selects `zr_template_depth.wgsl` / `zr_template_depth_alpha.wgsl`, and `create_depth_prepass_mesh_pipeline(...)` no longer imports or declares `NORMAL_FORMAT`. Opaque depth prepass uses no fragment stage; alpha-test depth prepass keeps `fs_main` for discard but uses `targets: &[]`, so the WGPU pipeline writes only `DEPTH_FORMAT`. Status: `render_plan08_runtime_depth_prepass_pure_depth_product_migration_static_passed_cargo_check_renderdoc_deferred`; `mesh_pipeline_depth_prepass_template_source_uses_depth_only_template`, `mesh_pipeline_variant_registry_maps_depth_prepass_to_depth_prepass_pass_type`, and `runtime_15_depth_prepass_pure_depth_product_migration_is_wired` lock the source, variant identity, WGPU descriptor, docs anchors, and owner budgets. This closes runtime pure-depth DepthPrepass product migration; real staged prewarm Naga/WGPU compile, full live project/plugin registry export, RenderDoc capture, and broader product acceptance remain separate Plan 08 work.

TAA reactive shader pass identity now has the same cache/prewarm dimension discipline as the other mesh material passes. `ShaderPassType::TaaReactiveMask` serializes and reports as `taa_reactive_mask`, `MeshPipelineVariantRegistry` maps both `TaaReactiveMask` and `TaaReactiveMaterialMask` mesh kinds to that pass, and `taa_reactive_mask_mesh_shader_key(...)` includes `|pass=taa_reactive_mask|` in the module key through `ShaderVariantKey::canonical_string()`. Built-in fallback and asset-root full-material prewarm enumerate six material passes, so staged cache reports can show TAA reactive entries instead of folding them into Forward. Status: `render_plan08_taa_reactive_shader_pass_identity_static_passed_cargo_deferred`; `render_shader_pass_type_names_taa_reactive_mask_separately_from_forward`, `mesh_pipeline_variant_registry_maps_taa_reactive_to_taa_reactive_pass_type`, `taa_reactive_mask_shader_key_includes_shader_variant_identity_and_source_hash`, and `runtime_15_taa_reactive_shader_pass_identity_is_wired` lock this behavior. The slice passed rustfmt, source/docs anchor scans, stale pass-pattern scan, and diff-check on 2026-06-28; Cargo check was deferred because unrelated runtime text/editor layout compile lanes were active. This closes only the TAA reactive pass identity/cache-prewarm dimension, not real runtime WGPU execution, RenderDoc capture, full registry export, or miss=0 product acceptance.

Prewarm opt-in WGPU shader-module validation gives staged prewarm a real module creation gate without changing the default cache-write path. `prewarm_shader_variants_to_disk_with_module_validation(...)` still runs the existing Naga WGSL validator first, then invokes an injected module validator before `ShaderVariantCacheDisk::write(...)`; validation failure records `WGPU shader module validation failed` in `ShaderVariantPrewarmReport` and leaves the disk cache untouched. `prewarm_shader_variants_with_wgpu_module_validation(...)` is the dynamic API owner for the real WGPU path: it creates an offscreen backend, pushes a validation error scope, calls `device.create_shader_module(...)` with the request WGSL, and maps setup or validation failures back into the same prewarm report. `zircon_shader_prewarm --validate-wgpu-modules` and `tools/zircon_build.py --validate-wgpu-shaders` are opt-in switches, so existing staged prewarm users keep the current Naga-only write path unless they request WGPU module validation. Status: `render_plan08_prewarm_wgpu_module_validation_gate_python_cargo_check_passed_runtime_run_timeout_deferred`; `render_shader_variant_prewarm_rejects_wgpu_module_validation_failure_before_disk_write`, `test_build_command_forwards_wgpu_shader_module_validation`, and `runtime_15_shader_prewarm_wgpu_module_validation_is_wired` lock the failure-before-write behavior, CLI/build-tool handoff, docs anchors, and owner budgets. Python and scoped Cargo checks passed; the actual `cargo run ... --validate-wgpu-modules` attempt timed out while compiling on Windows and is not accepted as runtime execution evidence.

Prewarm WGPU validation report summary makes that opt-in gate observable in the report artifact. `ShaderVariantPrewarmReport` carries `wgpu_module_validation.enabled`, `requested_count`, `validated_count`, `failed_count`, and `skipped_count`; the prewarm write path increments those counters when module validation passes, fails, or is skipped because WGSL validation failed first. The dynamic setup failure path records the same WGPU failure counts when an offscreen backend cannot be created. `tools/zircon_build_shader_prewarm.py` prints the summary line and reads both older `requested`/`written`/`failed` rows and Rust's actual `requested_count`/`written_count`/`failed_count` rows for dimension summaries. Status: `render_plan08_prewarm_wgpu_validation_report_summary_python_passed_cargo_deferred`; `test_dimension_summary_lines_format_wgpu_module_validation_counts`, `test_dimension_summary_lines_accept_rust_count_field_names`, `render_shader_variant_prewarm_records_wgpu_module_validation_success`, and `runtime_15_shader_prewarm_wgpu_validation_report_summary_is_wired` lock the report field, log summary, tests, docs anchors, and owner budgets. This does not count as real staged WGPU execution evidence.

Build-tool WGPU validation report contract makes the report summary a required success condition when the staged build asks for WGPU module validation. `tools/zircon_build.py::prewarm_shaders(...)` calls `validate_shader_prewarm_report_contract(...)` only after a zero exit code, and the helper requires the report to confirm `wgpu_module_validation.enabled`, a positive requested count, `validated_count == requested_count`, and zero failed/skipped variants. Non-zero prewarm exits still print the report summary and propagate the process failure without running the contract check. Status: `render_plan08_build_tool_wgpu_report_contract_python_passed_cargo_deferred`; `test_prewarm_shaders_validates_wgpu_report_after_success`, `test_validate_report_contract_requires_wgpu_validation_when_requested`, `test_validate_report_contract_accepts_wgpu_validation_counts`, and `runtime_15_shader_prewarm_wgpu_report_contract_is_wired` lock the build-tool gate. This is still Python/static evidence; real staged WGPU execution remains a later acceptance item.

Build-tool WGPU validation totals match contract makes that gate count-complete against the top-level prewarm report. `validate_shader_prewarm_report_contract(...)` now rejects successful reports where `wgpu_module_validation.requested_count`, `validated_count`, or `failed_count` disagree with top-level `requested_count`, `written_count`, or `failed_count`. The dedicated Python owner is `tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py`; `test_validate_report_contract_rejects_wgpu_validation_total_mismatch` locks the mismatch case, and `runtime_15_shader_prewarm_wgpu_report_contract_is_wired` asserts the WGPU report-contract regressions stay in that owner instead of returning to the general prewarm tests. Status: `render_plan08_build_tool_wgpu_validation_totals_match_python_passed_cargo_deferred`; the general prewarm owner is now 653 lines and the WGPU report owner is 87 lines. This still does not count as real `zircon_shader_prewarm --validate-wgpu-modules` runtime evidence.

Shader prewarm source provenance summary closes the report-level provenance gap for staged prewarm artifacts. `ShaderVariantPrewarmSource.source_label` records the asset scan stable label or `builtin://shader/pbr.wgsl`, while each request refers to that immutable source by id; the prewarm write path records successful and failed requests through `record_written_request(...)` / `record_failure_request(...)`; and `ShaderVariantPrewarmReport.source_provenance` groups each source/template payload by id, label, WGSL source hash, include hashes, template revision, Naga version, WGPU version, and requested/written/failed counts. `tools/zircon_build_shader_prewarm.py` prints a compact `source provenance:` line so build logs can identify which source payload produced report entries without printing full WGSL. Status: `render_plan08_shader_prewarm_source_provenance_summary_python_passed_cargo_deferred`; `test_dimension_summary_lines_format_source_provenance`, `render_shader_variant_prewarm_report_groups_written_and_failed_dimensions`, `shader_prewarm_asset_root_manifest_reads_compound_zshader_package`, and `runtime_15_shader_prewarm_source_provenance_summary_is_wired` lock the DTO/report/manifest/build-helper wiring. This remains Python/static/rustfmt evidence; Cargo guard, real `zircon_shader_prewarm --validate-wgpu-modules`, RenderDoc/product capture, full registry export, and product miss=0 are not closed by this row.

Build-tool source provenance report contract makes that provenance field a staged build success condition. `tools/zircon_build.py::prewarm_shaders(...)` now passes `require_source_provenance=True` after a zero prewarm exit, and `validate_shader_prewarm_report_contract(...)` requires a non-empty `source_provenance.sources` map, matching source count, variant count coverage for the report requested count, and per-source `source_label`, `source_hash`, `template_revision`, and closed requested/written/failed counts. Status: `render_plan08_build_tool_source_provenance_report_contract_python_passed_cargo_deferred`; `test_validate_report_contract_requires_source_provenance_when_requested`, `test_validate_report_contract_accepts_source_provenance_counts`, the expanded `test_prewarm_shaders_validates_wgpu_report_after_success`, and `runtime_15_shader_prewarm_source_provenance_report_contract_is_wired` lock the build-tool gate. This still does not close Cargo guard, real staged WGPU execution, RenderDoc/product capture, full registry export, or miss=0 product acceptance.

Build-tool shader resource registry export contract makes the automatic staged
`shader_resource_records.json` export a parseable build-tool product. The helper
accepts the same registry container shapes consumed by `zircon_shader_prewarm`:
a raw `ResourceRecord` array, `{ resources: [...] }`, or `{ records: [...] }`.
Empty arrays are valid, while missing files, invalid JSON, non-array containers,
and non-object records fail before the staged build can claim a successful
auto-export. Status:
`render_plan08_build_tool_resource_registry_export_contract_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_requires_resource_records`,
the expanded `test_prewarm_shaders_validates_wgpu_report_after_success`, and
`runtime_15_shader_prewarm_resource_registry_export_contract_is_wired` lock this
build-tool gate. This still does not close Cargo guard, real staged WGPU
execution, RenderDoc/product capture, full live registry export, or miss=0
product acceptance. A focused Cargo guard attempt was blocked before compile
because `Cargo.lock` would need update under `--locked`; no Rust diagnostics
were produced and that run is not counted as passed.

Build-tool shader resource registry report correlation now requires exported
resource records to match report source provenance for staged shader assets.
`validate_shader_resource_registry_export_contract(..., report_path=...)`
extracts `res://` source labels from `source_provenance.sources` and requires
matching `primary_locator` or `artifact_locator` entries in the exported
`ResourceRecord` container. Builtin shader sources and raw path-like source
labels are intentionally ignored because they are not emitted by `.zmeta`
resource registry export. Status:
`render_plan08_build_tool_resource_registry_report_correlation_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_missing_report_source_locator`,
`test_validate_registry_export_contract_accepts_report_source_locator`,
`test_validate_registry_export_contract_ignores_builtin_report_sources`, and
`runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired` lock
this staged prewarm gate. It still does not close Cargo guard, real staged WGPU
execution, RenderDoc/product capture, full live registry export, or miss=0
product acceptance.

The registry/report correlation now also covers actual written variant source
labels. `_report_resource_source_labels(...)` merges `res://` labels from
`source_provenance.sources[*].source_label` and
`written_variants[].source_label` before checking exported resource record
locators. Status:
`render_plan08_build_tool_resource_registry_written_source_label_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_missing_written_variant_locator`
and `runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired`
lock this extension. It is still Python/build-helper evidence, not real WGPU
execution, RenderDoc/product capture, full live registry export, or product
miss=0 acceptance.

The same gate now checks that a matched registry row is usable by shader
revision overlay code. `_usable_shader_resource_record_locators(...)` filters
records to `kind=Shader` and positive `revision` before accepting a `res://`
report source as covered, matching the runtime export/overlay filtering.
Status:
`render_plan08_build_tool_resource_registry_usable_shader_revision_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_non_shader_report_source_record`,
`test_validate_registry_export_contract_rejects_zero_revision_report_source_record`,
and `runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired`
lock this extension. It remains build-helper evidence only; real WGPU
execution, RenderDoc/product capture, full live registry export, and product
miss=0 acceptance remain open gates.

Resource registry ready shader revision contract now closes the state part of
that filter. The build helper only treats a report source as covered when the
matched exported `ResourceRecord` is `kind=Shader`, `state=Ready`, and has a
positive revision; `ShaderPrewarmResourceRegistryOverlay::from_records(...)`
mirrors the same `ResourceState::Ready` gate before recording a material
revision. Status:
`render_plan08_resource_registry_ready_shader_revision_contract_python_static_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_non_ready_report_source_record`,
`shader_prewarm_resource_registry_overlay_uses_ready_shader_revisions_only`,
`runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired`, and
`runtime_15_shader_prewarm_resource_registry_revision_overlay_is_wired` lock
the Python helper, Rust overlay owner, docs/status anchors, and child test
ownership. This remains Python/static evidence only; real staged WGPU
execution, full live project/plugin registry export, RenderDoc/product capture,
fallback-root hit, and miss=0 acceptance remain open gates.

Build-tool explicit registry exact revision acceptance applies the same
report/registry correlation to caller-provided live or project registry inputs.
`validate_staged_shader_prewarm_acceptance_contract(...)` now validates
`config.shader_resource_registry` when supplied, otherwise the automatic staged
`shader_prewarm_resource_registry_path`, always passing the successful prewarm
report path into the registry validator. Explicit registries are still not
treated as auto-export artifacts, but they can no longer bypass the
`usable shader ResourceRecord revisions` requirement for report-visible source
labels. Status:
`render_plan08_build_tool_explicit_registry_exact_revision_acceptance_python_passed_cargo_deferred`;
`test_acceptance_contract_validates_explicit_registry_against_report`,
`test_acceptance_contract_rejects_explicit_registry_without_ready_revision`, and
`runtime_15_shader_prewarm_acceptance_contract_is_wired` lock this gate. This
narrows the full live project/plugin registry export gap, but does not close the
actual production export, RenderDoc/product capture, real VG page/cluster
fetch/bindings, or product miss=0 gates.

The registry contract regressions now have a dedicated Python owner,
`tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py`.
Status:
`render_plan08_build_tool_resource_registry_contract_tests_owner_split_python_passed_cargo_deferred`;
the general build-helper owner is 540 lines after the split, while the registry
owner is 234 lines. New registry/report correlation tests should stay in that
dedicated owner.

Registry export validation now has its own build-tool module:
`tools/zircon_build_shader_resource_registry.py`. The prewarm helper keeps the
public `validate_shader_resource_registry_export_contract(...)` handoff, but
the ResourceRecord JSON checks live beside the registry/report correlation
logic. The contract requires Rust-deserializable ResourceRecord wire fields:
UUID `id` and `dependency_ids`, known `kind`/`state`, `primary_locator`,
present nullable `artifact_locator`, non-negative `revision` and
`importer_version`, `diagnostics` severity/message records, and
`source_hash/importer_id/config_hash`. Status:
`render_plan08_build_tool_resource_registry_record_shape_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_incomplete_resource_record`
and `runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired`
lock this contract. This is still build-helper evidence, not a real WGPU run,
RenderDoc/product capture, live registry export, or product miss=0 acceptance.

Because `ResourceKind` and `ResourceState` serialize as serde unit enums, the
same gate now requires `kind` and `state` to be string enum values, not tagged
objects. Status:
`render_plan08_build_tool_resource_registry_enum_wire_shape_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_tagged_enum_resource_record`
locks the negative case.

Numeric ResourceRecord fields now match Rust widths: `revision` must fit `u64`
and `importer_version` must fit `u32`. Status:
`render_plan08_build_tool_resource_registry_numeric_width_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_u64_revision_overflow` and
`test_validate_registry_export_contract_rejects_u32_importer_version_overflow`
lock the overflow cases.

ResourceRecord locator fields now mirror `ResourceLocator::parse(...)` instead
of accepting any string with `://`. The build-tool gate allows only
`res/lib/package/builtin/mem`, rejects empty/root/drive-prefixed/escaping paths
and empty labels, and makes `package://` carry both a package id and a
package-local path. Status:
`render_plan08_build_tool_resource_registry_locator_wire_shape_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_rejects_invalid_locator_wire_shape`,
`test_validate_registry_export_contract_accepts_locator_wire_shape_variants`,
and `test_validate_registry_export_contract_rejects_invalid_artifact_locator`
lock the primary and artifact locator cases.

Registry/report correlation now treats all project registry-backed locator
schemes as source labels, not just `res://`. `_RESOURCE_REGISTRY_BACKED_LOCATOR_SCHEMES`
keeps `res`, `lib`, `package`, and `mem` in the report correlation set after the
same `ResourceLocator::parse(...)`-shaped validation, while `builtin://` remains
an internal shader source and is skipped by staged project registry validation.
Status:
`render_plan08_build_tool_resource_registry_backed_locator_correlation_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_accepts_registry_backed_source_locators`,
`test_validate_registry_export_contract_rejects_missing_registry_backed_source_locator`,
and `runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired`
lock the extension. This is still build-helper evidence only; real WGPU
execution, RenderDoc/product capture, full live registry export, and product
miss=0 acceptance remain open gates.

Build-tool shader prewarm report dimension contract now requires successful
staged reports to cover the pass, quality, and geometry dimensions requested by
the build command. `prewarm_shaders(...)` passes `expected_pass_types`,
`expected_quality_tiers=config.shader_quality_tiers`, and
`expected_geometry_sources=config.shader_geometry_sources` into
`validate_shader_prewarm_report_contract(...)`; the helper checks
`dimension_summary.pass_types`, `dimension_summary.quality_tiers`, and
`dimension_summary.geometry_source_ids` for positive requested counts. Status:
`render_plan08_build_tool_report_dimension_contract_python_passed_cargo_deferred`;
`test_validate_report_contract_requires_requested_pass_types`,
`test_validate_report_contract_requires_requested_quality_tiers`,
`test_validate_report_contract_requires_requested_geometry_sources`,
`test_validate_report_contract_accepts_requested_dimensions`, and
`runtime_15_shader_prewarm_report_dimension_contract_is_wired` lock this staged
prewarm gate. This closes the build-tool requested-dimension check but still
does not close Cargo guard, real staged WGPU execution, RenderDoc/product
capture, full live registry export, or miss=0 product acceptance.

Build-tool shader prewarm report dimension complete-count contract tightens the
same report gate so requested dimensions must be complete, not merely present.
The report helper keeps missing-requested errors separate, then rejects any
expected pass, quality tier, built-in geometry source, geometry-source id, or
shading-model id with `requested_count > 0` but `written_count != requested_count`
or `failed_count != 0`. Status:
`render_plan08_build_tool_report_dimension_complete_counts_python_passed_cargo_deferred`;
`test_validate_report_contract_rejects_incomplete_requested_dimension_counts`
locks the `forward requested=6 written=5 failed=1` case, and
`runtime_15_shader_prewarm_report_dimension_contract_is_wired` anchors the
helper and docs. This remains Python/static evidence until live WGPU/product
validation runs.

Build-tool shader prewarm report dimension totals match contract closes the next
report-summary hole: each present dimension group must now sum to the same
`requested_count`, `written_count`, and `failed_count` values as the top-level
report. `_validate_dimension_summary_totals_match_report(...)` rejects reports
where an individual dimension is complete but the group total drifts, for
example `requested=6/7 written=6/7 failed=0/0`. Status:
`render_plan08_build_tool_report_dimension_totals_match_python_passed_cargo_deferred`;
`test_validate_report_contract_rejects_dimension_count_total_mismatch` and
`runtime_15_shader_prewarm_report_dimension_contract_is_wired` lock the helper,
negative report, docs anchors, and status record. This remains Python/static
evidence until live WGPU/product validation runs.

Build-tool product Base pass acceptance contract ties the build-helper success
bundle to the product Base/Opaque consumer. That earlier slice first routed the
`forward` pass into the report and cache validators, so a staged report had to
request Forward and the written cache identities had to include `pass=forward`.
Status:
`render_plan08_build_tool_product_base_pass_acceptance_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_requested_pass_types`,
`test_validate_cache_artifact_contract_accepts_requested_pass_types`, and the
report/cache/acceptance structure guards lock this handoff. It prevents a
Shadow-only or otherwise non-Base cache bundle from satisfying the build-helper
contract, but it still does not claim a new real WGPU/product run.

Build-tool product material mesh pass acceptance contract extends that same
handoff to the full product material mesh pass set. The staged acceptance helper
now owns `_PRODUCT_MATERIAL_MESH_PASS_TYPES = ("forward", "gbuffer",
"depth_prepass", "shadow", "velocity", "taa_reactive_mask")` and passes it to
both report and cache validators, so a forward-only staged report/cache bundle
cannot satisfy full product material cache acceptance. Status:
`render_plan08_build_tool_product_material_mesh_pass_acceptance_contract_python_passed_cargo_deferred`;
`test_acceptance_contract_rejects_forward_only_staged_pass_report`, the
acceptance handoff assertion for `expected_pass_types`, and
`runtime_15_shader_prewarm_acceptance_contract_is_wired` lock this expanded
contract. It remains build-helper/static evidence until a real staged WGPU run,
RenderDoc/product capture, full live registry export, and product miss=0 pass.

Build-tool cache quality/geometry identity contract extends written cache
identity checks to the dimensions already required in the report. The
acceptance helper passes `config.shader_quality_tiers` and
`config.shader_geometry_sources` into the cache validator, and the cache
validator checks `written_variants[].canonical_string` for the requested
`quality=<tier>` and built-in `geometry=<id>` values. Status:
`render_plan08_build_tool_cache_quality_geometry_identity_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_requested_quality_tiers`,
`test_validate_cache_artifact_contract_requires_requested_geometry_sources`, and
`test_validate_cache_artifact_contract_accepts_requested_quality_and_geometry`
lock the gate. It keeps report dimensions and actual cache keys aligned, but it
still does not claim a new real WGPU/product run.

Build-tool cache dimension combination contract closes the next staged-cache
hole after independent quality/geometry checks. The cache validator now calls
`_validate_expected_written_variant_combinations(...)`, parses canonical strings
with `_canonical_dimension_values(...)`, and requires each requested `pass x
quality x built-in geometry` combination to be present in the same written
variant identity. Status:
`render_plan08_build_tool_cache_dimension_combination_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_requested_dimension_combinations`
locks the case where `forward/high/static` plus `shadow/medium/skinned` cannot
satisfy `forward/high/skinned`. It keeps the build helper aligned with the
runtime manifest product expansion, but it still does not claim a new real
WGPU/product run.

Build-tool cache custom id combination contract closes the same combination
hole for selected custom geometry and shading ids. The cache validator now calls
`_validate_expected_written_custom_id_combinations(...)` after the individual
custom id checks, and requires custom geometry id plus custom shading id to
share one written canonical identity; requested pass/quality dimensions are part
of the same match when present. Status:
`render_plan08_build_tool_cache_custom_id_combination_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_requested_custom_id_combinations`
locks the case where `forward/high/geometry=0/shading=0` plus
`shadow/medium/geometry=4/shading=16` cannot satisfy
`forward/high/geometry=4/shading=16`. It keeps plugin/CLI id cache evidence
aligned with runtime product expansion, but it still does not claim a new real
WGPU/product run.

Build-tool written variant uniqueness contract closes the next report/cache
identity hole after field completeness and count checks. The shared
`tools/zircon_build_shader_prewarm_written_variants.py` owner now parses
`written_variants`, validates BLAKE3 cache-hash shape, keeps source-label
provenance correlation in one place, and rejects duplicate written identity
rows through `validate_unique_written_variant_identity(...)`. Status:
`render_plan08_build_tool_written_variant_uniqueness_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_rejects_duplicate_written_variant_identity`
and `test_acceptance_contract_rejects_duplicate_written_variant_identity` lock
the cache-helper and acceptance-helper entry points. This prevents duplicate
`cache_hash` or duplicate `canonical_string` rows from satisfying
`written_count`, but it still does not claim a new real WGPU/product run.

Build-tool cache metadata field type contract closes a lower-level artifact
wire-shape gap. The cache artifact helper now rejects `.meta` files whose
`schema_version` or `created_unix_seconds` are bool/string values instead of
non-bool integers, and rejects non-string `canonical_string`,
`template_revision`, `naga_version`, or `wgpu_version` fields before variant
matching runs. Status:
`render_plan08_build_tool_cache_metadata_field_type_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_rejects_invalid_metadata_field_types`
locks the cache-helper entry point. This keeps stale or malformed metadata from
being counted as runtime-cache-ready evidence, but it still does not claim a new
real WGPU/product run.

Build-tool staged prewarm written cache-hash shape acceptance moves the same
BLAKE3 key-shape rule into the success precheck. The acceptance helper now calls
`validate_cache_hash_shape(...)` for each `written_variants[].cache_hash`
before lower report/cache/resource-registry validators run. Status:
`render_plan08_build_tool_staged_prewarm_written_cache_hash_shape_python_passed_cargo_deferred`;
`test_acceptance_contract_rejects_invalid_written_variant_cache_hash_shape`
locks the acceptance entry point. This prevents malformed written identity from
leaving the early success bundle, but it still does not claim a new real
WGPU/product run.

Build-tool source-label nonblank contract keeps source provenance and written
variant source identity from being satisfied by whitespace-only strings. The
report contract now checks `source_label`, `source_hash`, and
`template_revision` with `_is_nonblank_string(...)`; the written-variant helper
and staged acceptance precheck also reject blank `written_variants[].source_label`.
Status:
`render_plan08_build_tool_source_label_nonblank_contract_python_passed_cargo_deferred`;
`test_validate_report_contract_rejects_blank_source_provenance_strings` and
`test_acceptance_contract_rejects_blank_written_variant_source_label` lock the
report and acceptance entry points. Final build-helper aggregate validation
passed 99/99, with py_compile, rustfmt, anchor/conflict/trailing-whitespace and
line-budget scans, and scoped diff-check passing with only LF/CRLF warnings.
This prevents malformed provenance from
skipping registry-backed locator correlation, but it still does not claim a new
real WGPU/product run.

Build-tool source-label trim contract tightens the same source evidence one step
further: source provenance `source_label`, `source_hash`, and
`template_revision` must already be trim-clean, and written variant
`source_label` must not carry leading or trailing whitespace. Status:
`render_plan08_build_tool_source_label_trim_contract_python_passed_cargo_deferred`;
`test_validate_report_contract_rejects_untrimmed_source_provenance_strings`,
`test_acceptance_contract_rejects_untrimmed_written_variant_source_label`, and
`test_validate_cache_artifact_contract_rejects_untrimmed_written_variant_source_label`
lock the report, acceptance, and cache-parser entry points. The source,
acceptance, and cache helper suite passed 46/46, and the build-helper aggregate
passed 102/102. This is still Python/static build-helper evidence, not a live
WGPU/product run.

Build-tool explicit registry exact revision acceptance closes the staged
acceptance bypass for caller-provided shader registries. The acceptance helper
now validates `config.shader_resource_registry` when present, otherwise the
automatic staged `shader_prewarm_resource_registry_path`, and always supplies
`report_path=config.shader_prewarm_report_path` so explicit live/project records
must match report-visible shader sources with usable shader ResourceRecord
revisions. Status:
`render_plan08_build_tool_explicit_registry_exact_revision_acceptance_python_passed_cargo_deferred`;
`test_acceptance_contract_validates_explicit_registry_against_report` and
`test_acceptance_contract_rejects_explicit_registry_without_ready_revision` lock
the positive handoff and zero-revision failure. This is still build-helper
acceptance evidence, not a completed full live project/plugin registry export or
RenderDoc/product run.

Build-tool shader permutation id report dimension contract extends that same
gate to selected plugin and explicit CLI custom ids. `prewarm_shaders(...)`
passes `expected_geometry_source_ids=shader_geometry_source_id_specs(config)`
and `expected_shading_model_ids=shader_shading_model_id_specs(config)` into the
report contract, and `validate_shader_prewarm_report_contract(...)` requires
`dimension_summary.geometry_source_ids` / `dimension_summary.shading_model_ids`
to contain positive requested counts for the parsed numeric ids. Status:
`render_plan08_build_tool_permutation_id_report_dimension_contract_python_passed_cargo_deferred`;
`test_validate_report_contract_requires_requested_geometry_source_ids`,
`test_validate_report_contract_requires_requested_shading_model_ids`,
`test_prewarm_shaders_passes_selected_custom_ids_to_report_contract`, and
`runtime_15_shader_prewarm_report_dimension_contract_is_wired` lock this staged
prewarm gate. This does not close real staged WGPU execution, RenderDoc/product
capture, full live registry export, or miss=0 product acceptance.

Build-tool shader prewarm cache artifact contract now requires successful
staged reports with positive `written_count` to be backed by cache files under
the staged cache root. After the report contract passes, `prewarm_shaders(...)`
calls `validate_shader_prewarm_cache_artifact_contract(...)`; the helper reads
the report and counts `.wgsl.zst` artifacts that have same-hash `.meta`
siblings with parseable runtime cache metadata, including matching `hash`,
`schema_version`, `canonical_string`, template revision, and Naga/WGPU version
fields. Status:
`render_plan08_build_tool_cache_artifact_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_written_cache_pairs`,
`test_validate_cache_artifact_contract_rejects_orphan_wgsl_artifacts`,
`test_validate_cache_artifact_contract_rejects_invalid_metadata`,
`test_validate_cache_artifact_contract_rejects_metadata_hash_mismatch`,
`test_validate_cache_artifact_contract_accepts_written_cache_pairs`, and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` lock this staged
prewarm gate. This closes build-tool cache artifact presence but still does not
close runtime key lookup, real staged WGPU execution, RenderDoc/product capture,
full live registry export, or miss=0 product acceptance.

Prewarm report cache identity contract now records the exact cache key written
by the Rust prewarm path. `ShaderVariantPrewarmReport.written_variants` carries
`ShaderVariantPrewarmWrittenVariant` entries for the successful writes, and
`graphics/shader/variant_cache/prewarm.rs` fills each entry from the
`ShaderVariantCacheDiskKey` returned before `ShaderVariantCacheDisk::write(...)`.
The build helper consumes that identity when present and requires the staged
metadata hash, canonical string, template revision, and Naga/WGPU versions to
match the report exactly. Status:
`render_plan08_prewarm_report_cache_identity_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_report_written_variants`,
`test_validate_cache_artifact_contract_rejects_partial_written_variant_report`,
`test_validate_cache_artifact_contract_rejects_wrong_canonical_variant`, and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` lock this exact
identity gate. This closes the build-tool report-to-cache identity check but
does not close runtime key lookup, real staged WGPU execution,
RenderDoc/product capture, full live registry export, or miss=0 product
acceptance.

Prewarm cache runtime layout contract makes that build-tool check mirror the
disk cache path used by runtime lookup. The helper now rejects staged `.wgsl.zst`
files outside `<cache_root>/v1/<hash[0..2]>/<hash>.wgsl.zst` and rejects `.meta`
files whose `schema_version` is not `1`, matching the current
`ShaderVariantCacheDisk` schema and shard convention. Status:
`render_plan08_prewarm_cache_runtime_layout_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_rejects_non_runtime_cache_layout`,
`test_validate_cache_artifact_contract_rejects_schema_version_mismatch`, and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` lock this runtime
layout gate. This closes another build acceptance gap before true fallback-root
lookup evidence, but it still does not close real staged WGPU execution,
RenderDoc/product capture, full live registry export, or miss=0 product
acceptance.

Prewarm cache hash shape contract makes the same build-tool gate reject cache
keys that cannot come from the runtime BLAKE3 disk key path. The helper now
requires staged artifact names and report `written_variants.cache_hash` values to
be 64-character lowercase hex strings before accepting the artifact as a
runtime-addressable cache entry. Status:
`render_plan08_prewarm_cache_hash_shape_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_rejects_non_blake3_hex_cache_hash` and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` lock this shape
gate. This closes another static acceptance gap but still does not close real
staged WGPU execution, RenderDoc/product capture, full live registry export, or
miss=0 product acceptance.

Prewarm cache custom id correlation contract tightens that same staged-cache
gate from "the report requested the custom dimensions" to "the written cache keys
actually carry those dimensions." After successful prewarm,
`prewarm_shaders(...)` passes `shader_geometry_source_id_specs(config)` and
`shader_shading_model_id_specs(config)` into
`validate_shader_prewarm_cache_artifact_contract(...)`. The helper parses each
selected plugin / explicit CLI `custom:name=ID` record and checks
`written_variants[].canonical_string`, whose stable format comes from
`ShaderVariantKey::canonical_string()`, for `geometry=<id>` and `shading=<id>`.
Status:
`render_plan08_prewarm_cache_custom_id_correlation_contract_python_passed_cargo_deferred`;
`test_validate_cache_artifact_contract_requires_requested_custom_ids`,
`test_validate_cache_artifact_contract_requires_requested_shading_ids`,
`test_validate_cache_artifact_contract_accepts_requested_custom_ids`, and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` lock this exact-key
custom-id gate. This still does not close real staged WGPU execution,
RenderDoc/product capture, full live registry export, runtime fallback-root hit,
or miss=0 product acceptance.

Runtime prewarm custom id cache lookup contract fixes the corresponding Rust
owner expectation for plugin-range ids. `graphics/shader/variant_cache/prewarm.rs`
now has `render_shader_variant_prewarm_custom_ids_survive_disk_lookup`, which
builds a prewarm request with `GeometrySourceId::new(4)` and
`ShadingModelId::new(16)`, writes it through `prewarm_shader_variants_to_disk`,
and verifies `ShaderVariantCacheDisk::new(&root).lookup(&disk_key)` hits with
the same canonical string containing `geometry=4` and `shading=16`. Status:
`render_plan08_runtime_prewarm_custom_id_cache_lookup_static_passed_cargo_deferred`;
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` locks that this
runtime lookup coverage stays beside the prewarm/cache owner. This is still a
static Rust owner contract; focused Cargo, live WGPU module validation,
RenderDoc/product capture, full live registry export, and miss=0 product
acceptance remain separate Plan 08 gates.

Runtime custom id staged fallback lookup contract fixes the next fallback-root
expectation for those same plugin-range ids. `render_shader_variant_prewarm_custom_ids_hit_staged_fallback_root`
writes a `GeometrySourceId::new(4)` / `ShadingModelId::new(16)` request into a
staged `cache/shader_variants` root, verifies the empty runtime root misses with
`ShaderVariantCacheDisk::new(&runtime_root).lookup(&disk_key)`, then verifies
`ShaderVariantCacheDisk::with_fallback_roots(&runtime_root, [&staged_root])`
hits the same canonical key without creating the writable runtime root. Status:
`render_plan08_runtime_custom_id_staged_fallback_lookup_static_passed_cargo_deferred`;
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` locks the fallback
lookup owner and docs anchors. This is still static Rust coverage; live WGPU
validation, RenderDoc/product capture, full live registry export, and miss=0
product acceptance remain separate gates.

Material custom shading-model runtime registry now connects those selected plugin descriptors to material runtime consumption. `RuntimeModuleExtensionInputs` collects `RuntimeExtensionRegistry::shading_models()`, `GraphicsModule` and the WGPU framework pass the descriptor list through `SceneRenderer::new_with_plugin_render_extensions_and_shading_models(...)`, and `ResourceStreamer::new_with_plugin_shading_models(...)` builds one `ShadingModelRegistry` for both material preparation and material capture seeds. Status: `render_plan08_material_custom_shading_model_runtime_registry_material_test_static_guard_passed_cargo_guard_timeout_renderdoc_deferred`; `render_plan08_selected_plugin_shading_model_registration_inputs_static_guard_cargo_deferred` adds `plugin_registration_inputs_collect_shading_model_descriptors`, proving a selected plugin registration report carries its `RuntimeExtensionRegistry::register_shading_model(...)` descriptor into `RuntimeModuleRegistrationInputs::shading_models()` before graphics module assembly. The direct lib-test binary backfill status `render_plan08_selected_plugin_shading_model_registration_inputs_direct_binary_passed_cargo_wrapper_deferred` records that focused filter passing 1/1, and Selected plugin/source-registry guard Cargo-wrapper backfill status `render_plan08_selected_plugin_source_registry_guards_cargo_wrapper_passed_renderdoc_deferred` records the same guard passing 1/1 with 5839 filtered. `render_product_streamer_projects_plugin_custom_shading_model_into_pipeline_key` and `runtime_15_material_custom_shading_model_runtime_registry_is_wired` lock that a `.zmaterial` custom lighting model such as `custom:subsurface` reaches `PipelineKey.shading_model_id` through the plugin descriptor registry rather than falling back to StandardPBR. This closes source-level selected-plugin descriptor handoff and runtime custom lighting-model resolution at focused Cargo-wrapper level, but real Naga/WGPU prewarm compile, RenderDoc/product capture, complete project shader resource discovery, and automatic project/plugin shader/shading/geometry registry export remain open.

Custom forward shading include source dispatch now covers the template-side source injection needed after descriptor registry export. `MaterialShaderTemplateRequest::with_shading_model_forward_include_source(...)` stores request-local `ShaderTemplateInclude` rows with dynamic token/source strings, and `include_registry.rs::shading_model_forward_include_for(..., source_includes)` resolves the descriptor `forward_include` through built-in includes first, then those supplied sources. `render_shader_template_uses_custom_shading_model_forward_include_source` proves a `custom:toon` descriptor can inject `zr_shading_toon.wgsl` through `CUSTOM_TOON_FORWARD_INCLUDE` without also injecting `zr_shading_standard_pbr.wgsl`; the no-source path still fails with `UnknownShadingInclude`. Status: `render_plan08_custom_forward_shading_include_source_static_passed_cargo_deferred`. This closes request-level Forward custom source dispatch; project/plugin source export now flows through `ShadingModelIncludeSourceSet`, while runtime handoff, real custom shading-model Naga/WGPU compile, RenderDoc/product capture, and broader miss=0 remain open.

Deferred GBuffer shading include source dispatch covers the matching template-side source injection for `ShadingModelDescriptor.gbuffer_encode_include`. `DeferredGBufferShaderTemplateRequest::with_shading_model_descriptor(...)` and `with_shading_model_gbuffer_include_source(...)` feed the dedicated GBuffer assembler, while `include_registry.rs::shading_model_gbuffer_include_for(..., source_includes)` resolves the built-in `zr_gbuffer_encode_standard_pbr.wgsl` first and then request-local plugin sources. `zr_template_deferred_gbuffer.wgsl` now delegates packing to `encode_gbuffer(surface, zr_build_shading_context(input))`, and `render_deferred_gbuffer_template_uses_custom_shading_model_gbuffer_include_source` proves a `custom:toon` descriptor can inject `zr_gbuffer_encode_toon.wgsl` without also injecting the standard encode include. Status: `render_plan08_deferred_gbuffer_shading_include_source_static_passed_cargo_deferred`. This closes request-level deferred GBuffer encode source dispatch; project/plugin source export now flows through `ShadingModelIncludeSourceSet`, while runtime handoff, real custom shading-model Naga/WGPU compile, RenderDoc/product capture, and broader miss=0 remain open.

Deferred lighting include source dispatch covers the matching source injection for `ShadingModelDescriptor.deferred_include`. `DeferredLightingShaderSourceRequest::with_shading_model_descriptor(...)` and `with_shading_model_deferred_include_source(...)` feed `assemble_deferred_lighting_shader_source(...)`, which keeps built-in GPUScene/light-grid/shadow chunks and built-in deferred leaf includes, then inserts descriptor-backed custom dispatch at `zr-deferred-lighting-custom-shading-model-dispatch`. `deferred_lighting_shader_uses_custom_shading_model_deferred_include_source` proves a `custom:toon` descriptor can inject `zr_shade_deferred_toon.wgsl`, while the no-source path returns `UnknownDeferredInclude`. Status: `render_plan08_deferred_lighting_include_source_dispatch_static_passed_cargo_deferred`. This closes request-level deferred lighting source dispatch; project/plugin source export now flows through `ShadingModelIncludeSourceSet`, while runtime pipeline descriptor/source connection, real custom shading-model Naga/WGPU compile, RenderDoc/product capture, and broader miss=0 remain open.

Project/plugin shading-model include source set now bridges the live project asset shader registry into those request-level include lanes. `ShadingModelIncludeSourceSet::from_project_asset_manager(...)` reads Ready `ResourceKind::Shader` records from `ProjectAssetManager`, matches plugin descriptor include tokens against shader locators, extracts `ShaderAsset::runtime_wgsl_source()`, and reports `MissingInclude`, `DuplicateIncludeToken`, `MissingRuntimeSource`, or `LoadShader` instead of silently falling back. `MaterialShaderTemplateRequest::with_shading_model_forward_include_sources(...)`, `DeferredGBufferShaderTemplateRequest::with_shading_model_gbuffer_include_sources(...)`, and `DeferredLightingShaderSourceRequest::with_shading_model_deferred_include_sources(...)` consume the same source set. Status: `render_plan08_shading_model_include_source_set_static_passed_cargo_deferred`; direct-binary validation status `render_plan08_shading_model_include_source_set_direct_binary_passed_cargo_wrapper_deferred` records `exported_include_source_set_feeds_forward_and_gbuffer_template_requests` passing 1/1, and grouped Cargo-wrapper status `render_plan08_selected_plugin_source_registry_guards_cargo_wrapper_passed_renderdoc_deferred` records the same guard passing 1/1 with 5839 filtered. This closes project/plugin WGSL source export to template/deferred source assembly at focused Cargo-wrapper level; runtime SceneRenderer pipeline handoff, real custom shading-model Naga/WGPU compile, RenderDoc/product capture, and broader miss=0 remain open.
After docs sync, `runtime_15_material_custom_shading_model_runtime_registry_is_wired` passed 1/1 with 5842 filtered for the grouped status anchors.

Runtime shading-model include source handoff now carries those request lanes into SceneRenderer pipeline source assembly under status `render_plan08_shading_model_include_source_runtime_handoff_static_passed_cargo_deferred`. `ResourceStreamer` owns the live descriptor/source lookup for a `PipelineKey`; Forward/Base mesh template creation applies `with_runtime_shading_model_sources(...)`; deferred GBuffer creation accepts `ResourceStreamer` through graph execution and calls the GBuffer source-set builder; deferred lighting pipeline creation assembles dynamic WGSL with plugin descriptors and `DeferredLightingShaderSourceRequest::with_shading_model_deferred_include_sources(...)` before WGPU pipeline creation. This closes the static/runtime descriptor-plus-source handoff for plugin shading models; WGPU module proof for Forward/GBuffer and the deferred lighting custom include WGPU pipeline validation are tracked by the following statuses.

Custom shading-model runtime WGPU module validation now closes the Forward/GBuffer authored-plugin module proof under status `render_plan08_custom_shading_model_runtime_wgpu_module_passed_product_renderdoc_deferred`. `runtime_custom_shading_model_sources_compile_as_wgpu_modules` uses Ready shader records plus `ResourceStreamer::new_for_test_with_plugin_shading_models(...)` to assemble plugin `custom:toon` Forward and deferred GBuffer sources, verifies the custom include constants are present, and creates WGPU shader modules for both outputs under validation error scope. Deferred lighting custom include WGPU pipeline validation now closes under status `render_plan08_deferred_lighting_custom_include_wgpu_pipeline_passed_product_renderdoc_deferred`: `custom_shading_model_deferred_lighting_pipeline_creates_with_project_include_source` uses Ready plugin include sources, product lighting/GPUScene layouts, and `create_lighting_pipeline(...)` to create the deferred lighting render pipeline under a WGPU validation scope. Product scene/readback evidence, RenderDoc/product capture, broader miss=0/product sweeps, and full CI remain open.

Plan 08 three shading-model forward/deferred product parity is now closed at the focused WGPU product layer under status `render_plan08_three_shading_models_forward_deferred_parity_wgpu_passed_light_grid_fallback_renderdoc_deferred`. `graphics/tests/render_product_mesh_cache/shading_model_parity.rs::render_product_three_shading_models_forward_deferred_parity` renders PBR/Blinn-Phong/Unlit material swatches through the default Forward + Deferred product pipelines, disables optional post/lighting features that would hide material-path differences, requires `mesh.opaque` on Forward and `deferred.gbuffer` plus `lighting.deferred` on Deferred, then compares RGBA output with `assert_rgba_frames_nearly_equal(...)`. The first WGPU pass exposed that disabling clustered lighting left mesh/deferred light-grid reads without external buffers, so `bind_execution_owned_graph_resources.rs::bind_light_grid_external_buffers` now binds `LightGridParams::disabled()`, empty z-bin/tile-mask buffers, and `:light-grid-execution-fallback` aliases before graph validation/execution. `runtime_15_render_plan08_three_shading_models_forward_deferred_parity_is_wired` locks the product child, parent mount, light-grid fallback, docs anchors, and file budgets. The existing `deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path` follow-up is now closed under status `render_plan08_deferred_project_shader_gbuffer_probe_wgpu_passed_renderdoc_deferred`: its fixture uses pure green `write_flat_color_wgsl(..., [0.0, 1.0, 0.0])` and `average_channel_in_region(...)` center sampling so Forward proves authored project WGSL while Deferred proves GBuffer material/base-color decode. RenderDoc/product capture remains open.

Plan 08 three shading-model forward/deferred product parity default-feature WGPU backfill is now recorded under status `render_plan08_three_shading_models_forward_deferred_parity_default_features_wgpu_passed_renderdoc_deferred`. The existing product test passed through default features with 1/1, 5876 filtered, and 11.81s; the only code unblock was a test-scope `graphics/scene/scene_renderer/ui/text.rs` import that fixes the `SdfAtlasRect` path when default-feature lib tests compile. The refreshed `runtime_15_render_plan08_three_shading_models_forward_deferred_parity_is_wired` status/docs guard passed by direct generated-binary rerun with 1/1, 5873 filtered, and 0.41s after its Cargo wrapper timed out during compile/link. This closes the focused PBR/Blinn-Phong/Unlit Forward + Deferred default-feature rerun gap, while RenderDoc/product capture, workspace/full CI, and broader product coverage remain open.

Live ResourceManager shader registry export is now wired at the resource/prewarm seam. `ResourceManager::ready_records_for_kind(ResourceKind::Shader)` exports deterministic ready shader `ResourceRecord` rows with non-zero live revisions, and `shader_resource_records_from_manager(&manager)` feeds those rows into `ShaderPrewarmResourceRegistryOverlay` so asset-root `.zmeta` shader scans can use the live `material_revision` instead of a fallback source hash. Status: `render_plan08_live_resource_manager_shader_registry_export_focused_tests_passed_renderdoc_deferred`. `shader_prewarm_resource_registry_overlay_uses_live_resource_manager_shader_revisions` and `runtime_15_shader_prewarm_live_resource_manager_registry_export_is_wired` lock the handoff. `shader_reimport_exports_updated_revision_for_prewarm_registry` now covers the edited project shader path by reimporting `res://shaders/pbr.wgsl` through `ProjectAssetManager` and requiring the exported Ready Shader record to carry the updated revision; status is `render_plan08_edited_shader_revision_export_static_guard_cargo_deferred`, with direct-binary validation status `render_plan08_edited_shader_revision_export_direct_binary_passed_cargo_wrapper_deferred` after the focused filter passed 1/1. Grouped Cargo-wrapper status `render_plan08_selected_plugin_source_registry_guards_cargo_wrapper_passed_renderdoc_deferred` records the same edited-revision guard passing 1/1 with 5839 filtered. This does not yet automatically enumerate full project/plugin shader, shading-model, or geometry-source registries, and it does not close real Naga/WGPU prewarm compile, RenderDoc/product capture, or runtime pure-depth DepthPrepass migration.

## 2026-07-03 SH02-SH04 shader/material contract status

The SH02 contract is now represented in `core/framework/render/shader/material_property_layout.rs`. `MaterialPropertyLayout` is the single source for property slots, texture bindings, packed size, and `layout_hash`; `MaterialOptionTable` owns bool/enum option bit packing and converts selected material values into `RenderShaderDefinitionValue` entries such as `ZR_OPT_*`. The asset-side generator in `asset/assets/shader/property_layout.rs` writes the generated material WGSL that the graphics template registers as `self::material`.

The SH03 import contract is represented in `core/framework/render/shader/module_import.rs`. It scans line-leading `#include <...>` directives, strips authored include directives before final template assembly, and classifies `self::` generated modules versus built-in `zr_*` modules. `graphics/shader/template/module_registry.rs` is the implementation owner for built-in and request-local module resolution; it topologically orders transitive module dependencies, injects each token once, reports cycles/unknown modules, and contributes content hashes to template output.

The SH04 material contract has the material renderer path closed through L2, the asset-root sparse prewarm path wired, and the first compute/fullscreen neutral contracts represented in code. `.zmaterial` is hard-switched to v2 with `parent`, `options`, and `queue`; material loading folds parent chains, validates property/texture/option/queue values against the shader artifact, and computes material option bits for `ShaderVariantKey`. `ShaderVariantKey` and `PipelineKey` now include `material_layout_hash`, and renderer-side material pass source assembly receives generated material WGSL, module include sources, and option defines from `ResourceStreamer`. The renderer material ABI now matches generated material WGSL: group 2 binding 0 is the material uniform and bindings 1..10 are the standard texture/sampler pairs. `MaterialPropertyOverrideBlock` now flows from `MeshRenderer` through `GeometryExtract.material_property_overrides` into per-draw binding0 uniform payloads, while static batches and static command-cache extraction skip override draws. `zircon_shader_prewarm` now derives the same material layout hash and option table from `.zshader` documents and writes only `.zmaterial` actual option selections into `ShaderVariantKey.material_option_bits`; duplicate material option selections dedupe by canonical key. `ComputeDispatchBuilder` emits the SH04 compute ABI group0 binding0 params plus binding1.. named resources, validates shader kind/kernel/resource binding mismatches, produces a neutral compute pipeline cache key, and can feed `RenderGraphComputeWorkload`. `FullscreenPassBuilder` emits the fullscreen ABI group0 frame, group1 pass inputs, group2 params, validates fragment entry/resource mismatches, and feeds `RenderFeaturePassDescriptor`. Clustered lighting now consumes a compute dispatch plan, motion-vector tile max consumes a fullscreen pass plan, and streamed builtin fallback shaders stay on the standard material template instead of entering the generated surface-material path. The focused project/plugin material-pass second-launch staged-prewarm gate now has fresh `compile_miss_count == 0` product evidence; broader L2 render_product parity, HZB/particles/more postprocess executor migration, real compute pipeline disk cache, wider product/perf sweeps, and RenderDoc/product capture remain open.

Validation status: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh02-m2 --message-format short --color never` passed on 2026-07-03 after the module registry hard cutover, renderer pass source wiring, material ABI migration, and SH04-M3 contract wiring. `cargo check -p zircon_runtime --bin zircon_shader_prewarm --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh02-m2 --message-format short --color never` passed after sparse prewarm wiring. The generated bin-test binary passed `manifest::tests::shader_prewarm_asset_root_manifest_uses_sparse_material_option_keys`, two adjacent V2 fixture tests, and the full `manifest::tests` group 21/21. The generated lib-test binary passed the broad `compute` filter 36/36, the `fullscreen` filter 3/3, `runtime_base_mesh_pipeline_keeps_builtin_fallback_on_standard_template_after_shader_stream` 1/1, and `runtime_custom_geometry_descriptor_non_base_pipelines_use_staged_prewarm_without_compile_miss` 1/1. The Cargo-wrapper bridge regression `graphics::tests::render_framework_bridge::pipeline_profiles::headless_wgpu_server_falls_back_async_compute_passes_to_graphics` passed 1/1 with graph-derived compute workload expectations. `cargo test -p zircon_runtime --lib graphics::tests::project_render::project_scenes::export_example_vampire_scene_png --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh02-m2 --message-format short --color never -- --ignored --exact --nocapture --test-threads=1` passed 1/1 and the accepted image is `docs/tests/runtime/shader/runtime_shader_material_vampire_offscreen_20260703.png` (1280x720, 54403 bytes, SHA256 `1526BE245965025596FA6098C495D85DCCBDA90E295C1B80489F4649740B5CE0`), with same-name target/cargo-targets scans returning no matches. Direct generated lib-test binary `E:\cargo-targets\zircon-runtime-03-gates-0704\debug\deps\zircon_runtime-33095b46939b64fc.exe` passed the exact `render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss` product test 1/1 with 6400 filtered in 10.59s. The short-name Cargo exact prewarm run filtered 0 tests and is not counted. Older focused template/layout unit tests timed out during the Windows lib-test harness compile and are not counted as passed.

## Test Coverage

`render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts` proves runtime WGSL selection, WGSL fallback source selection, non-WGSL missing-source rejection, entry-point stage projection, dependency projection, typed variant-key projection, and serialized pipeline layout projection.

`render_product_assets_shader_defs_accept_legacy_flags_and_typed_values`, `zshader_typed_shader_definition_rows_validate_kind_and_value`, and the compound `.zshader` import regression cover the typed shader-definition contract. Legacy `shader_defs = ["FEATURE"]` remains accepted as bool-true flags, while typed rows preserve bool, signed integer, and unsigned integer values through `ShaderAsset`, readiness reporting, and `RenderShaderVariantKey`.

`render_shader_template_assembles_standard_material_surface_source` checks that the standard material template source projects alpha-test, receive-shadows, and double-sided features, then assembles into renamed `zr_material_surface` WGSL with the expected material binding contract. `runtime_15_render_shader_template_assembly_is_folder_backed` locks `graphics/shader/template/material_surface.rs`, `standard_material_surface_source`, and the Plan 08 status anchors alongside the original template assembly guard.

`render_shader_template_validates_standard_material_wgsl_with_naga` records the intended Naga validation path for assembled standard material WGSL. The focused SH02/SH05 rerun now covers this template lane: `cargo test -p zircon_runtime --lib render_shader_template` first exposed a status-support mirror gap, then passed 18/18 after the render shader template assembly support anchors were restored.

The standard material template test also checks uv1/tangent interpolation strings and runtime mesh vertex input locations so template pass edits cannot silently drop `fetch_tangent(v, instance_index)`, `fetch_uv1(v)`, or the `input.uv1` material-source path.

`render_shader_template_clips_alpha_for_masked_standard_material_passes` checks the masked StandardPBR cutoff path across DepthPrepass and Shadow alpha templates. It asserts the alpha-only template tokens, `ZR_STANDARD_MATERIAL_ALPHA_CUTOFF`, `standard_material_alpha_cutoff()`, `standard_material_properties.data8.z`, `surface.alpha_cutoff`, and `zr_apply_alpha_clip(surface)` so alpha-test semantics cannot remain declared only as feature bits.

It also checks `standard_material_sampled_normal`, `standard_material_normal_tex`, and `input.tangent_handedness` so normal-map sampling cannot regress back to an unused binding while the runtime cutover is still pending.

`mesh_pipeline_standard_material_template_source_assembles_forward_base_source`, `mesh_pipeline_standard_material_template_source_uses_requested_geometry_source`, `mesh_pipeline_standard_material_shader_pass_source_keeps_depth_only_contract`, `mesh_pipeline_velocity_template_source_uses_previous_position_vertex_input`, `mesh_pipeline_taa_reactive_mask_template_source_uses_material_surface_without_lighting`, `mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked`, `mesh_pipeline_template_source_hashes_include_template_revision`, and `mesh_pipeline_template_source_hashes_feed_disk_and_module_keys` check the Base mesh runtime source cutover, the standard material prewarm pass source owner, the Velocity source-cache cutover, the TAA reactive source-cache cutover, the Shadow source-cache cutover, and the source owner split: fallback/missing shader source is generated by `mesh_pipeline_cache/shader_source.rs`, contains runtime `vs_main`/`fs_main`, carries Forward light-grid/shadow includes only for the Forward path, projects `ShaderFeatureBits`, records template revision separately from raw WGSL revision, can select the requested skinned geometry include, assembles pure depth-only DepthPrepass prewarm without material fragment code for opaque variants, assembles alpha-test DepthPrepass prewarm with `zr_template_depth_alpha.wgsl` and alpha clip but without normal-target output, assembles Velocity with `@location(8) previous_position` and alpha discard when needed, assembles TAA reactive mask without Forward light/shadow includes while preserving `fs_taa_reactive_mask` and `fs_taa_reactive_material_mask`, assembles ShadowDepth without a fragment entry for opaque variants, assembles ShadowDepthAlphaMask with `fs_main` and material alpha discard, and feeds both include hashes and final source hash into disk/module cache identity.

`mesh_pipeline_variant_registry_separates_geometry_sources` and `render_mesh_draw_processor_uses_batch_geometry_source_for_pipeline_variant_key` check that runtime batch geometry source reaches `ShaderVariantKey.geometry_source` before Base pipeline creation. `runtime_15_mesh_pass_processors_are_folder_backed` keeps that processor coverage in the child test owner, and `runtime_15_render_shader_template_assembly_is_folder_backed` locks the source owner and `ensure_pipeline.rs` delegation.

`velocity_mesh_shader_key_includes_shader_variant_identity_and_source_hash`, `taa_reactive_mask_shader_key_includes_shader_variant_identity_and_source_hash`, and `shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash` check the non-Base pass shader-module key helpers. Velocity, TAA, and Shadow cover the template source hash in module identity. `velocity_mesh_pipeline_declares_template_entries_and_previous_position_vertex_slot` checks the WGPU descriptor uses template `vs_main`/`fs_main` while keeping the previous-position vertex slot, `taa_reactive_mask_pipeline_declares_template_entries_and_static_vertex_layout` checks TAA reactive mask descriptors use `vs_main`, `fs_taa_reactive_mask`, and `fs_taa_reactive_material_mask` while keeping the static mesh vertex layout, and `shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias` checks Shadow descriptors use template `vs_main`/`fs_main`, keep static mesh vertex layout, and retain depth bias. `velocity_mesh_pipeline_creates_on_wgpu_device_with_template_shader` and `taa_reactive_mask_mesh_pipeline_creates_on_wgpu_device_with_template_shader` now cover the device-creation entry points with the shared `mesh_pipeline/test_support.rs` fixture, but current WSL direct execution segfaults in the shared offscreen path and is not accepted as passing evidence. `runtime_15_render_shader_template_assembly_is_folder_backed` also locks that the Velocity/TAA/Shadow pipeline maps are keyed by `MeshPipelineVariantId`, that old `pipeline_key_for_variant(...)` does not return as a narrower lookup, and that neither Velocity nor TAA reactive source cache paths import `FALLBACK_MESH_SHADER` while Shadow no longer mounts the deleted inline shadow shader path.

`builtin_fallback_shader_prewarm_manifest_uses_mesh_template_source`, `shader_prewarm_builtin_fallback_manifest_expands_requested_geometry_sources`, `runtime_base_mesh_pipeline_uses_staged_prewarm_without_compile_miss`, and `runtime_15_builtin_fallback_prewarm_uses_template_source` check that the dynamic API's built-in fallback prewarm manifest no longer writes `FALLBACK_MESH_SHADER`, but instead emits the same template source, content hashes, and template revision that Base mesh runtime cache consumes; that the CLI manifest path expands pure fallback requests across requested built-in geometry sources and the standard-material pass list; and that a staged fallback root hit can create the runtime Base mesh WGPU pipeline with `disk_hit_count == 1` and `compile_miss_count == 0`.

`render_product_base_mesh_second_launch_uses_staged_prewarm_without_compile_miss` and `runtime_15_product_base_mesh_staged_prewarm_is_wired` extend that evidence to the product path. The test lives in `graphics/tests/render_product_mesh_cache/staged_prewarm/mod.rs`, writes the staged manifest once, runs first and second product launches through fresh `WgpuRenderFramework` instances, and asserts staged disk hits plus `compile_miss_count == 0` without runtime cache writes/errors while `mesh.opaque` and skinned Base replay evidence are visible.

`render_product_three_shading_models_forward_deferred_parity`, `light_grid_external_fallback_buffers_satisfy_materialization_report`, `deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path`, and `runtime_15_render_plan08_three_shading_models_forward_deferred_parity_is_wired` lock the Plan 08 three shading-model forward/deferred product parity seam plus the related project-shader/GBuffer split probe. The product test uses PBR/Blinn-Phong/Unlit materials, runs Forward + Deferred default pipelines in separate `WgpuRenderFramework` captures, requires visible red/green/blue material swatches, and compares RGBA output. The light-grid fallback test proves disabled clustered-lighting graphs can bind required external `LIGHT_GRID_PARAMS`, `LIGHT_ZBINS`, and `LIGHT_TILE_MASKS` buffers with `:light-grid-execution-fallback` aliases. The project-shader probe uses `average_channel_in_region(...)` to sample the covered center region and separates the pure-green project shader from the red GBuffer material path. Focused no-default Cargo/WGPU passed for the product/fallback tests and the project-shader probe; the focused product parity test also passed under default features with status `render_plan08_three_shading_models_forward_deferred_parity_default_features_wgpu_passed_renderdoc_deferred` (1/1, 5876 filtered, 11.81s). RenderDoc/product capture, broader product coverage, and full CI remain open.

`builtin_standard_material_shader_prewarm_manifest_projects_material_features`, `builtin_standard_material_shader_prewarm_manifest_projects_geometry_source`, `builtin_standard_material_prewarm_writes_restart_hits_and_wgpu_modules`, `shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source`, and `runtime_15_shader_prewarm_builtin_standard_material_template_source_is_wired` check that an asset-root `.zmaterial` using `builtin://shader/pbr.wgsl` emits the same standard-material template source via `dynamic_api::builtin_standard_material_shader_prewarm_manifest(...)` / `dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry(...)`, preserves alpha cutoff and `ShaderFeatureBits::RECEIVE_SHADOWS`, writes requested `ShaderVariantKey.geometry_source`, emits static and skinned geometry includes, expands Forward/GBuffer/DepthPrepass/Shadow/Velocity/TAA reactive mask pass requests, keeps DepthPrepass prewarm on the pure depth/depth-alpha source contract, keeps TAA reactive prewarm on `zr_template_taa_reactive_mask.wgsl`, writes staged cache entries that hit after a simulated restart, and creates WGPU shader modules from the read-back WGSL without validation errors. The tests also ensure custom scanned WGSL sources are not wrapped by the builtin standard-material template path.

`render_shader_pass_type_names_taa_reactive_mask_separately_from_forward`, `mesh_pipeline_variant_registry_maps_taa_reactive_to_taa_reactive_pass_type`, and `runtime_15_taa_reactive_shader_pass_identity_is_wired` specifically cover the TAA reactive pass identity cutover. They lock the `taa_reactive_mask` token, registry mapping, TAA shader-module key, six-pass prewarm enumeration, documentation anchors, stale-pattern scan expectations, and owner file budgets.

`tools.tests.test_zircon_build_shader_prewarm` checks the build helper that consumes prewarm reports: it formats the four `dimension_summary` groups, ignores older reports without that field, skips malformed dimension rows without hiding valid counts, and verifies the build tool prints the summary before raising a non-zero prewarm exit.

`render_shader_variant_prewarm_rejects_wgpu_module_validation_failure_before_disk_write`, `shader_prewarm_args_parse_wgpu_module_validation_flag`, `test_build_command_forwards_wgpu_shader_module_validation`, and `runtime_15_shader_prewarm_wgpu_module_validation_is_wired` cover the opt-in WGPU shader-module validation gate. They lock that module validation runs after Naga WGSL validation but before disk write, that failed module validation records a prewarm failure and leaves the cache empty, that the CLI/build-tool switches are explicit opt-ins, and that the docs/status anchors stay synchronized. The current validation evidence is Python plus scoped Cargo check; a real `cargo run ... --validate-wgpu-modules` attempt timed out in Windows compilation and is not counted as runtime WGPU execution evidence.

`test_dimension_summary_lines_accept_rust_count_field_names`, `test_dimension_summary_lines_format_wgpu_module_validation_counts`, `render_shader_variant_prewarm_records_wgpu_module_validation_success`, and `runtime_15_shader_prewarm_wgpu_validation_report_summary_is_wired` cover the Prewarm WGPU validation report summary. They lock that Rust report JSON count fields are readable by the build helper, that opt-in WGPU module validation logs requested/validated/failed/skipped counts, and that the report DTO plus write/setup failure paths remain documented. Cargo execution for the Rust tests is deferred; this row is not real WGPU runtime evidence.

`test_prewarm_shaders_validates_wgpu_report_after_success`, `test_validate_report_contract_requires_wgpu_validation_when_requested`, `test_validate_report_contract_accepts_wgpu_validation_counts`, and `runtime_15_shader_prewarm_wgpu_report_contract_is_wired` cover the Build-tool WGPU validation report contract. They lock that `--validate-wgpu-shaders` requires the report to prove every requested WGPU module was validated, that missing report confirmation fails, and that non-zero prewarm exits keep the original process-error path instead of running a second contract check. Cargo/WGPU execution remains deferred to the staged runtime acceptance lane.

`test_validate_report_contract_rejects_wgpu_validation_total_mismatch` extends the Build-tool WGPU validation report contract to top-level totals. The report helper now compares WGPU validation requested/validated/failed counts with report requested/written/failed counts and raises `shader prewarm WGPU module validation counts did not match report totals` when a zero-failure report leaves a requested module unvalidated. `tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py` owns this regression with the existing WGPU report positive/required cases; the general prewarm test owner continues to cover the staged build orchestration path. Cargo/WGPU execution remains deferred to the staged runtime acceptance lane.

`test_dimension_summary_lines_format_source_provenance`, `render_shader_variant_prewarm_report_groups_written_and_failed_dimensions`, `shader_prewarm_asset_root_manifest_reads_compound_zshader_package`, and `runtime_15_shader_prewarm_source_provenance_summary_is_wired` cover the Shader prewarm source provenance summary. They lock that the build helper can print provenance-only reports, that report entries count written and failed request outcomes by source/template payload, and that asset/builtin manifest producers fill human-readable `source_label` values. Cargo execution for the Rust tests is deferred; this is not WGPU runtime evidence.

`test_validate_report_contract_requires_source_provenance_when_requested`, `test_validate_report_contract_accepts_source_provenance_counts`, the expanded `test_prewarm_shaders_validates_wgpu_report_after_success`, and `runtime_15_shader_prewarm_source_provenance_report_contract_is_wired` cover the Build-tool source provenance report contract. They lock that successful staged prewarm must produce a provenance summary, that malformed or missing provenance fails the build helper contract, and that this contract composes with the existing WGPU module validation contract. Cargo/WGPU execution remains deferred to the staged runtime acceptance lane.

`test_validate_report_contract_rejects_source_provenance_count_mismatch` extends
that source provenance contract to top-level totals. The report helper now sums
`source_provenance.sources[*].requested/written/failed` and requires those
values to match report-level `requested_count`, `written_count`, and
`failed_count`, so a report cannot claim top-level success while a source entry
records a failed variant. Status:
`render_plan08_build_tool_source_provenance_totals_match_python_passed_cargo_deferred`;
closeout verification passed the build-helper Python combo 66/66. This remains
Python/static report-contract evidence, not live WGPU execution or product
miss=0 acceptance.

The source provenance report-contract tests now have a dedicated Python owner:
`tools/tests/test_zircon_build_shader_prewarm_source_provenance_contract.py`.
`runtime_15_shader_prewarm_source_provenance_report_contract_is_wired` reads
that owner directly and asserts the mismatch regression does not return to the
general `test_zircon_build_shader_prewarm.py` owner. Status:
`render_plan08_build_tool_source_provenance_report_tests_owner_split_python_passed_cargo_deferred`;
the split reduced the general owner to 694 lines and left the dedicated owner
at 101 lines.

`test_validate_registry_export_contract_requires_resource_records`,
`test_validate_registry_export_contract_accepts_wrapped_resources`,
`test_validate_registry_export_contract_accepts_raw_array`,
the expanded `test_prewarm_shaders_validates_wgpu_report_after_success`, and
`runtime_15_shader_prewarm_resource_registry_export_contract_is_wired` cover the
Build-tool shader resource registry export contract. They lock that successful
automatic export must leave a parseable `ResourceRecord` container and that
this contract composes with the report and source-provenance contracts.
The focused Cargo guard is not counted as passed because it was blocked before
compile by `Cargo.lock` needing update under `--locked`; WGPU execution remains
deferred to the staged runtime acceptance lane.

`test_validate_registry_export_contract_rejects_missing_report_source_locator`,
`test_validate_registry_export_contract_accepts_report_source_locator`,
`test_validate_registry_export_contract_ignores_builtin_report_sources`, the
expanded `test_prewarm_shaders_validates_wgpu_report_after_success`, and
`runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired` cover
the Build-tool shader resource registry report correlation. They lock that
successful auto-exported registry evidence must include every report-visible
`res://` shader source locator, while builtin and raw source labels remain
outside the registry export contract.

`test_validate_report_contract_requires_requested_quality_tiers`,
`test_validate_report_contract_requires_requested_geometry_sources`,
`test_validate_report_contract_accepts_requested_dimensions`, the expanded
`test_prewarm_shaders_validates_wgpu_report_after_success`, and
`runtime_15_shader_prewarm_report_dimension_contract_is_wired` cover the
Build-tool shader prewarm report dimension contract. They lock that successful
staged reports must include positive requested counts and complete
written/failed counts for every pass, quality tier, and built-in geometry source
requested by the build command, so default `medium/static`, non-Forward, or
partially failed dimension summaries cannot satisfy broader prewarm requests
silently. `test_validate_report_contract_rejects_dimension_count_total_mismatch`
also locks that dimension-summary group totals must match the top-level report
counts, so a complete `forward` entry cannot hide an over-counted pass group.

`test_validate_report_contract_requires_requested_pass_types`,
`test_validate_cache_artifact_contract_requires_requested_pass_types`,
`test_validate_cache_artifact_contract_accepts_requested_pass_types`, the
acceptance handoff assertion for `expected_pass_types`, and the three structure
guards cover the Build-tool product Base pass acceptance contract. They lock
that the staged acceptance bundle routes `("forward",)` into both report and
cache validators before the product Base/Opaque path is treated as build-ready.

`test_acceptance_contract_rejects_forward_only_staged_pass_report`, the updated
`test_acceptance_contract_validates_report_cache_and_exported_registry` handoff
assertion, and `runtime_15_shader_prewarm_acceptance_contract_is_wired` cover
the Build-tool product material mesh pass acceptance contract. They lock that
the same staged acceptance bundle now routes `("forward", "gbuffer",
"depth_prepass", "shadow", "velocity", "taa_reactive_mask")` into both report
and cache validators before full product material cache acceptance is treated
as build-ready.

`test_validate_cache_artifact_contract_requires_written_cache_pairs`,
`test_validate_cache_artifact_contract_rejects_orphan_wgsl_artifacts`,
`test_validate_cache_artifact_contract_rejects_invalid_metadata`,
`test_validate_cache_artifact_contract_rejects_metadata_hash_mismatch`,
`test_validate_cache_artifact_contract_accepts_written_cache_pairs`, the
expanded prewarm success-path handoff tests, and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` cover the
Build-tool shader prewarm cache artifact contract. They lock that successful
staged reports with written variants must have `.wgsl.zst` cache artifacts and
matching parseable `.meta` files before the build is accepted.

`test_validate_cache_artifact_contract_requires_report_written_variants`,
`test_validate_cache_artifact_contract_rejects_partial_written_variant_report`,
`test_validate_cache_artifact_contract_rejects_wrong_canonical_variant`, and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` extend the cache
artifact coverage to exact report identity. They lock that a successful staged
report with `written_variants` must name cache hashes whose `.meta`
`canonical_string`, template revision, and Naga/WGPU versions match the report,
not merely the number of written variants.

`test_validate_cache_artifact_contract_requires_written_variant_source_labels_in_provenance`
and `runtime_15_shader_prewarm_cache_artifact_contract_is_wired` extend that
same cache identity handoff to source provenance. When a report contains
`source_provenance.sources`, every `written_variants[].source_label` must be
present and must match a provenance source label before the staged cache bundle
is accepted. This prevents a cache artifact with matching `.meta` identity from
being attributed to an untracked shader source.

`test_acceptance_contract_requires_written_variant_source_label_identity` and
`runtime_15_shader_prewarm_acceptance_contract_is_wired` move the source-label
requirement into the staged acceptance precheck. Successful reports must include
`source_label` with every written cache identity row before the acceptance helper
delegates to report, cache, source-provenance, or registry validators.

`test_validate_cache_artifact_contract_rejects_duplicate_written_variant_identity`,
`test_acceptance_contract_rejects_duplicate_written_variant_identity`, and both
shader prewarm structure guards cover the Build-tool written variant uniqueness
contract. They lock that a success report cannot duplicate a `cache_hash` or
`canonical_string` row to make `written_count` look complete while another
requested product variant is missing.

`test_acceptance_contract_validates_report_cache_and_exported_registry`,
`test_acceptance_contract_validates_explicit_registry_against_report`,
`test_acceptance_contract_rejects_explicit_registry_without_ready_revision`,
`test_prewarm_shaders_runs_acceptance_bundle_after_success`, the updated
`test_prewarm_shaders_validates_staged_acceptance_after_success`, and
`runtime_15_shader_prewarm_acceptance_contract_is_wired` cover the Build-tool
staged prewarm acceptance contract. They lock that the zero-exit build path
calls one acceptance helper that composes WGPU/report/source-provenance
dimension checks, staged cache artifact checks, and automatic
resource-registry/report correlation, including caller-supplied explicit
registries when present. `test_acceptance_contract_rejects_runtime_fallback_layout_drift`
and `test_acceptance_contract_accepts_runtime_fallback_layout` also lock the
runtime fallback root layout: staged prewarm must write cache artifacts under
`ZirconEngine/cache/shader_variants`, keep the report at
`ZirconEngine/cache/shader_variants_report.json`, and keep automatic
resource-registry exports beside them before any report JSON is read. This
aligns build output with `ShaderVariantCacheDisk::with_fallback_roots(...)`.
`test_acceptance_contract_rejects_empty_success_report` and
`test_acceptance_contract_rejects_failed_success_report` then lock that the
same acceptance helper rejects a zero-exit report unless it has
`requested_count > 0`, `written_count > 0`, and `failed_count == 0`, so an empty
or partially failed staged prewarm cannot pass through to the cache artifact
validator. Status:
`render_plan08_build_tool_staged_prewarm_nonempty_success_report_python_passed_cargo_deferred`;
closeout verification passed the build-helper Python combo 62/62.
`test_acceptance_contract_requires_written_variant_identity` and
`test_acceptance_contract_rejects_incomplete_written_variant_identity` then lock
that accepted reports must also carry `written_variants` entries matching
`written_count`, with `cache_hash`, `canonical_string`, template revision, Naga
version, and WGPU version fields present before the lower cache helper checks
the staged `.meta` files. Status:
`render_plan08_build_tool_staged_prewarm_written_variant_identity_python_passed_cargo_deferred`;
closeout verification passed the build-helper Python combo 64/64.
`test_acceptance_contract_rejects_partial_written_success_report` then locks
the final count relation for this helper: even when `failed_count == 0` and
`written_variants` identity rows are present, `written_count` must equal
`requested_count` before lower source/cache validators run. Status:
`render_plan08_build_tool_staged_prewarm_complete_written_count_python_passed_cargo_deferred`;
closeout verification passed the build-helper Python combo 65/65.
This is still Python/static gate evidence; live WGPU prewarm, RenderDoc/product
capture, full live registry export, and second-launch miss=0 remain separate
Plan 08 gates.

`test_validate_cache_artifact_contract_rejects_non_runtime_cache_layout`,
`test_validate_cache_artifact_contract_rejects_schema_version_mismatch`, and
`runtime_15_shader_prewarm_cache_artifact_contract_is_wired` extend the cache
artifact coverage to runtime lookup layout. They lock that staged cache
artifacts use the same `v1/<hash[0..2]>` shard and schema version expected by
`ShaderVariantCacheDisk`.

`runtime_15_non_base_mesh_variant_cache_owner_is_wired` locks the Non-Base mesh variant-aware cache owner
path. GBuffer, DepthPrepass, Shadow, Velocity, and TAA reactive mesh pipelines
must resolve `MeshPipelineVariantId` through
`pipeline_and_shader_key_for_variant(...)`, cache render pipelines by variant id,
and build shader module keys from `ShaderVariantKey::canonical_string()` plus
the assembled template source hash. Status:
`render_plan08_non_base_mesh_variant_cache_owner_static_passed_cargo_deferred`.

`tools.tests.test_zircon_build_plugin_carriers` checks selected plugin carrier discovery for legacy permutation id rows plus descriptor-owned shading-model and geometry-source ids. The geometry-source regression `test_zircon_build_discovers_plugin_geometry_source_descriptors_as_shader_ids` proves a selected plugin `[[geometry_sources]]` row is enough to feed staged `shader_geometry_source_ids`. The selected-plugin asset-root regressions `test_zircon_build_discovers_plugin_asset_roots_for_shader_prewarm`, `test_zircon_build_discovers_distribution_assets_as_plugin_asset_roots`, and `test_zircon_build_uses_existing_default_plugin_assets_root` prove top-level, legacy distribution, and default package asset roots can feed staged prewarm. `test_build_command_includes_selected_plugin_asset_roots` proves those roots are forwarded as additional `--asset-root` values.

`test_zircon_build_selects_plugin_contributions_for_runtime_prewarm` covers the runtime-only selected-plugin contribution path. It locks that `--targets runtime --plugins virtual_geometry --prewarm-shaders` carries `PluginPackage.shader_geometry_source_ids` and descriptors into the prewarm config without requiring the `plugins` build target. The live staged WGPU closeout command `python -u tools\zircon_build.py --targets runtime --plugins virtual_geometry --out target\codex-plan08-live-wgpu-prewarm-0629 --mode debug --prewarm-shaders --validate-wgpu-shaders` wrote 12 cache variants, validated 12 WGPU shader modules, and reported geometry source ids `0` and `4` across the six material pass types.

`runtime_custom_geometry_descriptor_pipeline_uses_staged_prewarm_without_compile_miss` covers the intended runtime side of that descriptor handoff. It prewarms a `custom:virtual_geometry` descriptor with `GeometrySourceId::new(4)`, configures a fresh runtime cache with the staged fallback root, registers the same descriptor in `MeshPipelineCache`, and creates the Base mesh WGPU pipeline through `ensure_pipeline_for_variant(...)`. The focused default-feature lib-test now passes and asserts the runtime shader variant miss report stays on the staged hit path: one request, one disk hit, zero compile misses, zero disk writes, zero disk errors, and the geometry id `4` dimension mirrors the same hit/no-miss counts. Product VG draw-source/page-cluster fetch, RenderDoc/product capture, and product second-launch miss=0 remain later Plan 08 gates.

`runtime_custom_geometry_descriptor_non_base_pipelines_use_staged_prewarm_without_compile_miss` covers the non-Base side of the same descriptor handoff. It uses the staged fallback root to create GBuffer, DepthPrepass, Shadow, Velocity, and TAA reactive WGPU pipelines for `custom:virtual_geometry=4`, then asserts five requests are five disk hits with zero compile misses, zero disk writes, and zero disk errors. The first focused run failed at four hits out of five because GBuffer prewarm used the generic material-pass source while runtime used the deferred-gbuffer template; `mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor(...)` now dispatches GBuffer and TAA reactive to their pass-specific template owners so prewarm and runtime source hashes stay aligned. Status: `render_plan08_runtime_custom_geometry_descriptor_non_base_staged_cache_hit_wgpu_pipelines_passed_product_deferred`. Product VG draw-source/page-cluster fetch, full live registry export, RenderDoc/product capture, and product second-launch miss=0 remain later Plan 08 gates.

`render_product_material_mesh_passes_second_launch_use_staged_prewarm_without_compile_miss` covers the product material-mesh side of staged cache handoff from `staged_prewarm/material_passes.rs`. It builds a product pipeline with DepthPrepass, Shadow, Deferred GBuffer, Lighting, PostProcess, and terminal AA fallback, then launches two fresh `WgpuRenderFramework` instances against the same staged fallback root. The first frame must request the staged depth and GBuffer variants as disk hits, the repeated frame must stay on memory-hit/no-compile-miss behavior, and the product stats must show shadow, light-grid, deferred-lighting, post-process, and material mesh work without runtime cache writes or errors. Status: `render_plan08_product_material_mesh_second_launch_staged_prewarm_passed_renderdoc_deferred`.

Product material mesh staged prewarm owner split + Velocity runtime contract is documented by status `render_plan08_product_material_mesh_staged_prewarm_owner_split_velocity_static_passed_cargo_deferred_active_lanes`. The previous single `staged_prewarm.rs` owner is now split into `staged_prewarm/mod.rs` for Base mesh coverage and `staged_prewarm/material_passes.rs` for the material mesh multi-pass product chain. The material-pass velocity frame now requires a previous velocity transform, zero missing velocity transforms, `temporal.velocity-object`, and a runtime `velocity pass` dimension with compile miss=0. Scoped static verification passed for the two new owners and `runtime_15_product_base_mesh_staged_prewarm_is_wired`; source/docs anchors, line budgets, old-file absence, conflict/trailing-whitespace scans, and scoped diff-check passed, with only LF/CRLF warnings from Git. Cargo/WGPU remains deferred behind active external lanes.

The same slice locks the runtime resource contract for deferred geometry. `DepthPrepass` is now pure depth and writes only `SCENE_DEPTH`; `deferred.gbuffer` owns `GBUFFER_ALBEDO`, `GBUFFER_NORMAL`, and `GBUFFER_MATERIAL`; `zr_template_deferred_gbuffer.wgsl`, `record_gbuffer_geometry(...)`, and `create_gbuffer_mesh_pipeline(...)` all declare the same three color-target layout. This closes the focused product graph contract, but broader RenderDoc/product capture, full live registry export, and real VirtualGeometry page/cluster fetch remain outside this slice.

`mesh_pipeline_virtual_geometry_template_source_declares_page_cluster_fetch_bindings` covers the shader assembly side of the VirtualGeometry fetch contract. It assembles a descriptor requiring `VirtualGeometryPages` and `VirtualGeometryClusters`, then checks the generated source includes group3 binding 9/10 declarations, `zr_virtual_geometry_vertex_word_index(...)`, and primitive `payload_slot` usage. `render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings` covers the WGPU-facing GPUScene layout side of the same contract. The structure guard `runtime_15_virtual_geometry_page_cluster_shader_bindings_are_wired` covers the wider static seam: descriptor enum, plugin runtime/static manifest, GPUScene fallback storage slots, WGSL guarded fetch helpers, prewarm registry fixtures, docs/status anchors, and line budgets. The focused direct-binary backfill passed both tests 1/1 under status `render_plan08_virtual_geometry_page_cluster_shader_bindings_direct_binary_wgpu_layout_passed_renderdoc_deferred`, the fresh no-default Cargo-wrapper backfill passed the same two full-path tests 1/1 under status `render_plan08_virtual_geometry_page_cluster_shader_bindings_cargo_wrapper_wgpu_layout_passed_renderdoc_deferred`, and the default-feature Cargo-wrapper backfill passed the shader-source, GPUScene layout, and structure-guard tests 1/1 under status `render_plan08_virtual_geometry_page_cluster_shader_bindings_default_features_cargo_wrapper_wgpu_layout_passed_renderdoc_deferred`. The 2026-07-03 current-source structure-guard rerun also passed 1/1 after the material/default disabled-pass and shader-template `module_registry.rs` hard-cut compile drift was repaired.

`virtual_geometry_cluster_words_follow_resident_page_payloads` covers the resident payload projection from `RenderVirtualGeometryDebugSnapshot.resident_page_payloads` into `GpuVirtualGeometryClusterWord` rows: page rows get a nonzero vertex count, repeated resident submissions for the same page share cluster word storage, and each vertex contributes position, normal, tangent, and pad words matching the WGSL fetch contract. The structure guard `runtime_15_virtual_geometry_cluster_payload_upload_is_wired` covers the wider static seam: debug snapshot DTO/re-export, production snapshot sidecar handoff from `FrameSubmissionContext`, GPUScene words-per-vertex ABI constant, resident upload projection, docs/status anchors, and file budgets. The focused direct-binary WGPU backfill passed 1/1 under status `render_plan08_virtual_geometry_cluster_payload_upload_direct_binary_wgpu_passed_renderdoc_deferred`; the later no-default Cargo-wrapper backfill passed the same full-path cluster payload test 1/1 under status `render_plan08_virtual_geometry_resident_cluster_upload_cargo_wrapper_passed_renderdoc_deferred`; the default-feature Cargo-wrapper backfill passed the same full-path cluster payload test 1/1 under status `render_plan08_virtual_geometry_resident_cluster_upload_default_features_cargo_wrapper_passed_renderdoc_deferred`.

`render_page_payloads_decode_cooked_triangle_vertices_with_global_page_ids` covers cooked `ZVG0` page payload decode from payload item triangle ranges into source mesh vertex payloads after local-to-global page remap. The imported cooked model extract assertions cover the end-to-end plugin sidecar from `ModelPrimitiveAsset` source vertices/indices to `VirtualGeometryAutomaticExtractOutput.resident_page_payloads`, while `runtime_15_virtual_geometry_asset_payload_decode_is_wired` locks the runtime output, frame submission context, production debug snapshot handoff, plugin decode owner, docs/status anchors, and file budgets. Status: `render_plan08_virtual_geometry_asset_payload_decode_static_passed_cargo_deferred`.

`virtual_geometry_vertex_ordinals_pack_into_joint_index_slots` covers the asset contract for VG source vertex ordinals: VG primitives pack source vertex index into the first two `joint_indices` slots, non-VG primitives remain untouched, and the decode helper round-trips a value above 16 bits. Mesh conversion and importer assertions cover root model primitives and labeled `MeshAsset` subassets carrying the same ordinal stream, while `runtime_15_virtual_geometry_meshlet_vertex_ordinal_is_wired` locks the primitive helper, OBJ/GLTF/model backfill, `MeshAsset` conversion, WGSL unpack expression, shader-source assembly check, docs/status anchors, and file budgets. Status: `render_plan08_virtual_geometry_meshlet_vertex_ordinal_direct_binary_asset_shader_passed_renderdoc_deferred`; the older static status `render_plan08_virtual_geometry_meshlet_vertex_ordinal_static_passed_cargo_deferred` remains a historical guard anchor. Follow-up status `render_plan08_virtual_geometry_project_asset_manager_fixture_source_guarded_cargo_rerun_deferred` additionally locks `asset_manager_imports_model_toml_with_virtual_geometry_payload` and the source-side `expected_model.primitives[0].assign_virtual_geometry_vertex_ordinals()` call. A stale direct-binary audit against `zircon_runtime-770562bad16f99eb.exe` timestamped `2026-07-02 05:27:25 +08:00` failed with the old all-zero expected payload; current source then passed the latest generated binary and fresh no-default Cargo-wrapper under `render_plan08_virtual_geometry_project_asset_manager_fixture_cargo_wrapper_passed_renderdoc_deferred`.

`render_product_virtual_geometry_model_asset_uses_automatic_draw_source` covers the product automatic draw-source path for VG: it registers a cooked `ModelAsset`, submits normal `GeometryExtract::from_meshes(...)` model snapshots instead of an authored `RenderVirtualGeometryExtract`, and asserts the frame uses `RenderVirtualGeometryPayloadSource::AutomaticFallback` with visible indirect execution stats. The structure guard `runtime_15_virtual_geometry_product_draw_source_is_wired` covers the wider seam: frame submission context authored-extract gating, runtime provider `build_extract_from_meshes(...)`, `ProjectAssetManager::load_model_asset(...)`, the folder-backed test provider, product fixture, docs/status anchors, file budgets, and the `render_plan08_virtual_geometry_product_draw_source_cargo_wrapper_wgpu_passed_renderdoc_deferred` status after direct-binary 1/1 plus no-default Cargo-wrapper 1/1 evidence.

VirtualGeometry product draw-source readback fixture status `render_plan08_virtual_geometry_product_draw_source_readback_passed_targeted_cargo` extends that product child owner so the same test registers an explicit Unlit material, keeps the visible camera on Perspective/Core3d with black clear and Shaded display mode, calls `capture_frame(viewport)`, and checks `assert_virtual_geometry_capture_visible(...)` after the automatic draw-source stats. The focused no-default Cargo/WGPU rerun passed 1/1; RenderDoc/product capture, default features, workspace/full CI, live registry export, and broader miss=0 remain open.

VirtualGeometry product draw-source default-feature WGPU backfill status `render_plan08_virtual_geometry_product_draw_source_default_features_wgpu_passed_renderdoc_deferred` reruns the same readback fixture through the default runtime feature set. The first Cargo wrapper exceeded the 1204s tool window while compiling/linking but produced the default-feature lib-test binary and left no target-dir residual processes; direct generated-binary execution passed 1/1 with 5928 filtered, then the warmed Cargo wrapper passed 1/1 with 5933 filtered and repository-existing warnings only. RenderDoc/product capture, workspace/full CI, live registry export, and broader miss=0 remain open.

`render_gpu_scene_uploads_virtual_geometry_resident_buffers` covers the GPUScene resident upload owner: typed page/cluster shadows, storage buffer rebuild, write-buffer byte reporting, and same-size reupload without bind group rebuild. `virtual_geometry_page_rows_follow_submission_slots` covers the mesh-build projection from execution segments to resident page rows, including pending/missing and missing-slot rejection. Both focused owners passed by direct no-default lib-test binary evidence under status `render_plan08_virtual_geometry_resident_buffers_upload_direct_binary_wgpu_passed_renderdoc_deferred`; the later no-default Cargo-wrapper backfill passed the resident GPUScene upload and page-row full-path tests 1/1 under status `render_plan08_virtual_geometry_resident_cluster_upload_cargo_wrapper_passed_renderdoc_deferred`; the default-feature Cargo-wrapper backfill passed the same resident GPUScene upload and page-row full-path tests 1/1 under status `render_plan08_virtual_geometry_resident_cluster_upload_default_features_cargo_wrapper_passed_renderdoc_deferred`. The structure guard `runtime_15_virtual_geometry_resident_buffers_upload_is_wired` covers the wider seam: GPUScene typed ABI, upload owner, mesh-build handoff, payload-slot assignment, docs/status anchors, and file budgets.

`render_gpu_scene_uploads_morph_storage_buffers` covers the GPUScene morph upload owner: typed delta/weight shadows, storage buffer rebuild, write-buffer byte reporting, and same-size reupload without bind group rebuild. The structure guard `runtime_15_morph_storage_buffers_upload_is_wired` covers the wider static seam: GPUScene typed ABI, group3 binding 7/8 layout entries, upload owner, WGSL helper declarations, morphed include helper consumption, docs/status anchors, and file budgets. Status: `render_plan08_morph_storage_buffers_upload_check_passed_wgpu_deferred`.

Morph payload projection is covered by `morph_payload_projection_keeps_active_position_deltas_and_weights` and `morph_payload_collection_deduplicates_shared_draw_payloads`. The mesh-build child owner `morph_payload_upload.rs` reads direct mesh `MESH_ATTRIBUTE_POSITION` morph deltas, encodes row-aligned `GpuMorphDelta` / `GpuMorphWeight` payloads, deduplicates shared pending payloads, and calls `GpuScene::upload_morph_buffers(...)`; `PendingMeshDraw` only carries an optional shared payload, and build aggregation adds the uploaded bytes to the existing GPUScene report. The current-weight block starts at `GpuMorphPayload.weight_base`; the previous-weight block used by Velocity starts at `weight_base + target_count` and keeps previous-only targets alive when current weights are zero. The structure guard `runtime_15_morph_payload_projection_is_wired` covers the earlier position projection status `render_plan08_morph_payload_projection_check_passed_wgpu_deferred`, and the previous-weight follow-up is recorded as `render_plan08_morph_previous_weights_velocity_check_passed_product_deferred`.

Morph payload slot indexing extends that handoff from storage rows to shader-addressable payloads. `morph_payload_projection_keeps_normal_tangent_and_color_delta_rows`, `morph_payload_collection_deduplicates_shared_draw_payloads`, and `render_shader_template_validates_morphed_geometry_sources_with_payload_slots` cover the new surface: `GpuMorphPayload` header rows live at GPUScene binding 11, `GpuInstanceData.morph_payload_slot` selects a payload per draw, mesh-build upload writes position/normal/tangent/color rows plus pending draw slots, and morphed/skinned-morphed WGSL uses `@builtin(vertex_index)` with payload metadata to fetch the correct delta rows. The structure guard `runtime_15_morph_payload_slot_indexing_is_wired` locks the wider ABI/WGSL/docs seam. Status: `render_plan08_morph_payload_slot_indexing_check_passed_wgpu_deferred`.

Morph geometry-source selection is now the production draw-level consumer for those payload slots. `build_mesh_draws/build/geometry_source_selection.rs` owns the shared classification used by final draw assembly, prepared-queue stats, and pending command-cache profiles. Payload-backed direct non-skinned morphs use `PendingMeshGeometry::GpuMorphed` and `DynamicGpuMorphedSource`, which resolves to `GEOMETRY_SOURCE_ID_MORPHED_MESH`; payload-backed skinned morphs keep the original prepared source and use `DynamicGpuSkinnedMorphedSource`, which resolves to `GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH`. CPU-baked direct morph fallbacks and CPU-morphed GPU-skinning fallbacks remain `DynamicCpuMorphedSource` / `DynamicCpuMorphedGpuSkinningSource` so the shader does not apply morph twice. `runtime_15_morph_geometry_source_selection_is_wired` locks the source, queue, mesh-pass, docs, and status anchors. Status: `render_plan08_morph_geometry_source_selection_static_passed_wgpu_deferred`. GPU-vs-CPU product parity and RenderDoc/product capture remain separate gates.

Morph GPU-source product observability extends that selection into `RenderStats` and DiagnosticStore. `PreparedMeshQueueStats` now separates `gpu_morphed_source_draw_count` and `gpu_skinned_morphed_source_draw_count`; submit stats expose `last_mesh_gpu_morphed_source_draw_count` and `last_mesh_gpu_skinned_morphed_source_draw_count`; diagnostics mirror `render.mesh.queue.gpu_morphed_source_draw_count` and `render.mesh.queue.gpu_skinned_morphed_source_draw_count`. The product guard `render_product_direct_mesh_active_morph_weights_use_gpu_morphed_source` submits a direct morph mesh through WGPU and requires the GPU Morphed source counter to increment. Status: `render_plan08_morph_gpu_source_product_guard_wgpu_passed_renderdoc_deferred`. This proves source selection, not pixel parity.

Morph GPU-vs-CPU product parity now closes the direct non-skinned pixel/readback gate. `render_product_direct_mesh_gpu_morph_matches_cpu_baked_reference_pixels` uses the same direct mesh/morph fixture but renders one frame through active GPU morph weights and a second through CPU-baked positions with no active morph weights. Both frames use an unlit material, matching camera and quality profile, and the product test compares captured primary-surface RGBA while checking the GPU path reports one GPU Morphed source draw and the CPU reference reports none. Status: `render_plan08_morph_gpu_cpu_product_parity_wgpu_passed_renderdoc_deferred`. The skinned counterpart is closed by the following status; previous morph weights are code-wired for velocity under `render_plan08_morph_previous_weights_velocity_check_passed_product_deferred`, while RenderDoc/product velocity capture remains a separate gate.

Skinned morph GPU-vs-CPU product parity now closes the skinned-morphed pixel/readback gate. `render_product_skinned_mesh_gpu_morph_matches_cpu_baked_reference_pixels` registers a skinned mesh, skeleton asset, and pose sideband, then compares a GPU SkinnedMorphed source frame against a CPU-baked morph reference that still exercises shader skinning. The GPU path must report one skinned-morphed source draw and one shader-skinning draw; the CPU reference must keep skinned-morphed source draws at zero while preserving shader skinning. Status: `render_plan08_skinned_morph_gpu_cpu_product_parity_wgpu_passed_renderdoc_deferred`. The previous morph-weight velocity code path is wired under `render_plan08_morph_previous_weights_velocity_check_passed_product_deferred`; RenderDoc/product velocity capture remains a separate gate.

`shader_resource_records_from_asset_roots_deduplicates_duplicate_shader_records` covers duplicate shader `.zmeta` metadata across `engine_assets` and `plugin_assets`. `shader_resource_records_from_asset_roots_rejects_id_locator_conflicts` and `shader_resource_records_from_asset_roots_rejects_locator_id_conflicts` cover the two conflict paths where dedupe must fail instead of creating an ambiguous overlay. `runtime_15_shader_prewarm_resource_registry_multi_root_dedupe_is_wired` locks the staged shader resource registry multi-root dedupe owner, docs/status anchors, and line budget.

`test_prewarm_plan_lists_asset_roots_for_registry_export` covers dry-run plan output for the same engine/plugin asset roots consumed by staged registry export. `test_prewarm_plan_lists_runtime_fallback_handoff_paths` extends that dry-run coverage to `shader prewarm cache root`, `shader prewarm report`, and `shader runtime fallback root`, matching the runtime fallback root path audited by staged acceptance. `test_build_command_auto_export_registry_scans_all_asset_roots` covers the final command's `--asset-root` sequence and default `--export-resource-registry` path. `runtime_15_shader_prewarm_asset_root_plan_visibility_is_wired` locks the Build-tool shader asset-root plan visibility status, docs anchors, and file budgets.

`test_acceptance_contract_requires_usable_records_for_project_plugin_auto_export`
and `runtime_15_shader_prewarm_project_plugin_registry_auto_export_is_wired`
cover the Project/plugin registry auto-export nonempty acceptance status
`render_plan08_project_plugin_registry_auto_export_nonempty_python_passed_cargo_deferred`.
When staged prewarm is using automatic `shader_resource_records.json` export and
the build config carries project `shader_asset_roots` or selected plugin
`asset_roots`, acceptance now passes `require_usable_shader_records=True` into
the registry export contract. `test_validate_registry_export_contract_requires_usable_shader_records_when_requested`
and `test_validate_registry_export_contract_accepts_usable_shader_records_when_requested`
lock the lower helper: the exported registry must contain at least one usable
Ready Shader `ResourceRecord` with a positive revision. This is build-helper
Python/static evidence only; full live project/plugin registry export, product
WGPU execution, RenderDoc/product capture, and broader second-launch miss=0 stay
as separate Plan 08 gates.

`test_acceptance_contract_requires_registry_source_for_project_plugin_auto_export`,
`test_acceptance_contract_accepts_registry_source_for_project_plugin_auto_export`,
and `runtime_15_shader_prewarm_project_plugin_registry_report_source_is_wired`
cover the Project/plugin registry report-source acceptance status
`render_plan08_project_plugin_registry_report_source_python_passed_cargo_deferred`.
When automatic registry export is active for project/plugin roots, acceptance
now also passes `require_report_registry_backed_sources=True`; the registry
contract rejects builtin-only reports and requires `report_path` when that switch
is enabled. Direct registry tests cover the required report path plus
positive/negative registry-backed report sources. This remains Python/static
evidence only; full live project/plugin registry export, product WGPU execution,
RenderDoc/product capture, and broader second-launch miss=0 remain open gates.

`shader_resource_records_from_project_and_plugin_asset_roots_export_distinct_shader_sources`,
`shader_prewarm_project_and_plugin_asset_roots_use_exported_registry_revisions`,
and `runtime_15_shader_prewarm_project_plugin_registry_live_asset_roots_are_wired`
cover the Project/plugin registry live asset-root export status
`render_plan08_project_plugin_registry_live_asset_roots_static_passed_cargo_deferred`.
The registry fixture writes separate project and plugin `.zmeta` shader roots,
then proves `shader_resource_records_from_asset_roots(...)` exports distinct
Ready Shader records for `res://project/shaders/project` and
`package://virtual_geometry/shaders/plugin`. The manifest fixture builds a
`ShaderPrewarmResourceRegistryOverlay` from those records, merges the project and
plugin manifests, and verifies each source label emits six requests whose
`material_revision` matches the exported registry revision. This remains a
static/focused live asset-root fixture; full staged WGPU/product execution,
RenderDoc/product capture, and broader second-launch miss=0 remain open gates.

`shader_prewarm_project_and_plugin_asset_roots_export_wrapped_resource_registry_file`
and `runtime_15_shader_prewarm_project_plugin_registry_export_file_is_wired`
cover the Project/plugin registry export file handoff status
`render_plan08_project_plugin_registry_export_file_static_passed_cargo_deferred`.
The run-path fixture calls `export_shader_resource_registry_for_asset_roots(...)`
with project and plugin `.zmeta` roots plus a staged
`ZirconEngine/cache/shader_resource_records.json` path, then verifies the written
JSON keeps the wrapped `{ "resources": [...] }` shape expected by staged
acceptance. The returned records are checked against the file records, and a
`ShaderPrewarmResourceRegistryOverlay` built from the exported file records must
resolve the exported revisions by locator. This closes only the run-path file
handoff; full staged WGPU/product execution, RenderDoc/product capture, and
broader second-launch miss=0 remain open gates.

`runtime_15_shader_prewarm_project_plugin_registry_production_fixture_is_wired`
covers the Project/plugin registry production fixture prewarm status
`render_plan08_project_plugin_registry_production_fixture_static_passed_cargo_timeout_no_result`.
The real `native_dynamic_fixture` package exposes `distribution.assets =
["assets/**"]`, and its shader asset now has a `.zmeta` sidecar at
`zircon_plugins/native_dynamic_fixture/assets/shader.wgsl.zmeta` for
`package://native_dynamic_fixture/shaders/shader`. This locks the selected
plugin asset-root path used by `collect_plugin_asset_roots(...)` and
`shader_asset_root_paths_for_prewarm(...)` without adding another production
facade. A staged project+plugin `zircon_shader_prewarm --validate-wgpu-modules
--export-resource-registry` attempt produced no accepted report or registry
before the Cargo lane ended without a usable result, so this slice remains
static fixture evidence only; full staged WGPU/product execution,
RenderDoc/product capture, and broader miss=0 remain open gates.

`test_build_command_auto_export_registry_uses_native_dynamic_fixture_assets`
and
`runtime_15_shader_prewarm_project_plugin_registry_production_command_is_wired`
cover the Project/plugin registry production command handoff status
`render_plan08_project_plugin_registry_production_command_python_passed_cargo_deferred`.
The Python regression discovers the real repository `native_dynamic_fixture`
package through `zircon_build.discover_plugins(...)`, verifies that its selected
plugin asset root contains `shader.wgsl.zmeta`, and then confirms
`build_shader_prewarm_command(...)` emits that root as an `--asset-root` while
keeping automatic `--export-resource-registry`. This proves the real plugin
fixture reaches the staged prewarm command without relying on a fake plugin
object. Cargo/WGPU/product execution and RenderDoc capture remain separate Plan
08 gates.

`test_cli_selects_native_dynamic_fixture_assets_for_prewarm_command` and
`runtime_15_shader_prewarm_project_plugin_registry_production_cli_selection_is_wired`
cover the Project/plugin registry production CLI selection handoff status
`render_plan08_project_plugin_registry_production_cli_selection_python_passed_cargo_deferred`.
This regression parses the public `zircon_build.py --targets runtime --plugins
native_dynamic_fixture --prewarm-shaders` path, resolves the repository plugin
catalog into a selected `native_dynamic_fixture` package, verifies the package's
production `assets` root still contains `shader.wgsl.zmeta`, and then checks the
prewarm command for the same root under `--asset-root`. It also verifies the
command uses automatic `--export-resource-registry` rather than explicit
`--resource-registry`, so the public CLI path keeps feeding the project/plugin
registry auto-export lane. Cargo/WGPU/product execution and RenderDoc capture
remain separate Plan 08 gates.

`test_cli_dry_run_prints_native_dynamic_fixture_prewarm_command` and
`runtime_15_shader_prewarm_project_plugin_registry_production_cli_dry_run_is_wired`
cover the Project/plugin registry production CLI dry-run handoff status
`render_plan08_project_plugin_registry_production_cli_dry_run_python_passed_cargo_deferred`.
This regression calls `zircon_build.main(...)` with the public runtime target,
selected `native_dynamic_fixture` plugin, shader prewarm enabled, and
`--dry-run`. The captured output must include the generated
`zircon_shader_prewarm` dry-run command, the real plugin `assets` root, and
automatic `--export-resource-registry` while avoiding explicit
`--resource-registry`. That gives Plan 08 a no-Cargo public CLI proof for the
same selected-plugin asset root before the full staged WGPU/product run is
attempted.

`runtime_project_plugin_registry_shader_keys_use_staged_prewarm_without_compile_miss`
and
`runtime_15_shader_prewarm_project_plugin_registry_runtime_staged_cache_hit_is_wired`
document the Project/plugin registry runtime staged-cache hit status
`render_plan08_project_plugin_registry_runtime_staged_cache_hit_static_passed_cargo_deferred`.
The focused runtime test constructs staged prewarm requests for
`res://project/shaders/project_shader` and
`package://native_dynamic_fixture/shaders/shader`, rewrites
`ShaderVariantKey.material_shader` and `material_revision` to the registry
`ResourceId`/revision rather than relying on source labels, then creates
matching `PipelineKey` values and injects
`ShaderVariantCacheDisk::with_fallback_roots(...)` into `MeshPipelineCache`.
The test contract requires registry provenance, disk hits for both sources,
and compile miss=0 with zero writes/errors when the Base pipeline is created.
Cargo/WGPU execution deferred while other build lanes are active; this is
static/source guard evidence until the milestone testing lane runs.

`render_product_project_plugin_registry_materials_use_staged_prewarm_without_compile_miss`
and
`runtime_15_shader_prewarm_project_plugin_registry_product_staged_cache_is_wired`
document the Project/plugin registry product staged-cache miss=0 status
`render_plan08_project_plugin_registry_product_staged_cache_static_passed_cargo_timeout_no_result`.
The product child test registers project and selected-plugin shader assets for
`res://project/shaders/project_shader` and
`package://native_dynamic_fixture/shaders/shader`, then overwrites the
exported Ready Shader revisions with `register_record(exported_record)` before
submitting registry-backed materials through `WgpuRenderFramework`. The staged
manifest rewrites `material_shader`, `material_revision`, and raw-WGSL
`include_content_hashes` so `ShaderVariantCacheDisk::with_fallback_roots(...)`
can match the runtime product lookup key. The product contract requires
`mesh.opaque` executor visibility, disk hit accounting, compile miss=0, and
zero runtime cache writes/errors. Cargo/WGPU execution timed out after about
904 seconds with no test result, so this row is static/source evidence until a
quiet no-proxy WGPU lane can rerun it.

`render_product_project_plugin_registry_material_passes_use_staged_prewarm_without_compile_miss`
and
`runtime_15_shader_prewarm_project_plugin_registry_material_passes_staged_cache_is_wired`
document the Project/plugin registry material-pass staged-cache miss=0 status
`render_plan08_project_plugin_registry_material_passes_staged_cache_static_passed_cargo_timeout_no_result`.
The product child test rewrites project/plugin registry shader id/revision into
standard-material staged prewarm requests while preserving each pass template's
content hashes, includes built-in fallback variants, registers a TAA-reactive
registry material, and submits through a product `WgpuRenderFramework` pipeline.
It covers DepthPrepass/GBuffer/Shadow/TAA reactive mask plus lighting/post and
requires runtime compile miss=0 with zero writes/errors. Cargo/WGPU execution
timed out after about 904 seconds with no test result, so this is static/source
guard evidence until the focused product lane can rerun.

`render_plan08_project_plugin_registry_material_passes_velocity_runtime_contract_static_passed_cargo_deferred_active_lanes`
documents the Project/plugin registry material-pass Velocity runtime contract.
The velocity frame assertion now requires a previous velocity transform,
rejects missing previous velocity transforms, requires
`temporal.velocity-object`, and checks that the runtime miss report contains a
`velocity pass` dimension with compile miss=0. This keeps the staged-cache proof
aligned with the Velocity entry in `REGISTRY_MATERIAL_PASS_TYPES` instead of
leaving Velocity as prewarm-only coverage while Cargo/WGPU remains deferred for
active external lanes.

`render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss`
and
`runtime_15_shader_prewarm_project_plugin_registry_material_passes_staged_cache_is_wired`
now also document the Project/plugin registry material-pass second-launch
contract under status
`render_plan08_project_plugin_registry_material_passes_second_launch_static_passed_wgpu_timeout_no_result`.
The new `second_launch.rs` child writes the staged manifest once and then
creates two independent `WgpuRenderFramework` launch cycles per registry-backed
project/plugin shader. Both launches reuse
`ShaderVariantCacheDisk::with_fallback_roots(runtime_root, [staged_root])` and
must satisfy the existing first-frame and velocity-frame miss-report assertions;
the additional `assert_runtime_shader_cache_root_empty(...)` check proves the
runtime cache root stays empty, so the second launch cannot pass by reading a
first-launch runtime write. Scoped rustfmt, source anchors, and line budgets
passed. The focused WGPU product cargo command timed out after about 1204
seconds without a test result, and follow-up bounded reruns at 2026-07-01 04:56,
05:03, and 05:15 +08:00 still stopped during dependency/runtime compile or link.
The later direct-binary backfill reused
`E:\cargo-targets\zircon-plan08-custom-shading-second-launch-guard-0701\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe`
and passed
`render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss`
1/1 with 5806 filtered in 14.95s under status
`render_plan08_project_plugin_registry_material_passes_second_launch_direct_binary_wgpu_passed_renderdoc_deferred`.
After the docs/status sync, the same binary also passed
`runtime_15_shader_prewarm_project_plugin_registry_material_passes_staged_cache_is_wired`
1/1 with 5806 filtered in 0.44s.
A later Project/plugin registry material-pass second-launch Cargo-wrapper WGPU
backfill closed the wrapper rerun under status
`render_plan08_project_plugin_registry_material_passes_second_launch_cargo_wrapper_wgpu_passed_renderdoc_deferred`:
`cargo test -p zircon_runtime --lib render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701 --message-format short --color never -- --nocapture --test-threads=1`
passed 1/1 with 5842 filtered in 12.86s. After docs sync,
`runtime_15_shader_prewarm_project_plugin_registry_material_passes_staged_cache_is_wired`
also passed 1/1 with 5848 filtered. RenderDoc/product capture and full CI remain
separate gates; the default-feature gap is closed by the follow-up below.

Project/plugin registry material-pass second-launch default-feature WGPU
backfill is recorded under status
`render_plan08_project_plugin_registry_material_passes_second_launch_default_features_wgpu_passed_renderdoc_deferred`.
The first default-feature Cargo-wrapper run exposed a real product fixture
blocker: the standard registry shader source was a fully assembled Forward pass
template registered as a `Surface` shader, so Deferred/GBuffer tried to wrap a
pass template as a material surface and failed before resolving the mesh
pipeline. The fixture now uses
`registry_material_pass_runtime_surface_source`, a standard PBR surface source
with the runtime `fn zr_material_surface(` entry contract. The default-feature
rerun passed 1/1 with 6177 filtered in 14.96s on
`E:\cargo-targets\zircon-plan08-vg-product-default-0703`; the no-default
regression passed 1/1 with 6171 filtered in 12.75s; and the helper contract
test passed 1/1 with 6171 filtered. RenderDoc/product capture,
workspace/full CI, and broader product cases outside this focused filter remain
open. The custom shading-model product group 6/12 follow-up is closed by
`render_plan08_custom_shading_model_product_group_default_features_wgpu_refresh_passed_renderdoc_deferred`.

`render_product_custom_shading_model_registry_material_passes_use_staged_prewarm_without_compile_miss`
documents the focused custom shading-model product material-pass staged-cache
status
`render_plan08_custom_shading_model_product_material_pass_staged_cache_wgpu_passed_renderdoc_deferred`.
The new `custom_shading_model.rs` child registers a plugin-range `custom:toon`
descriptor plus Ready Forward/GBuffer/deferred include WGSL, builds the six-pass
standard-material manifest through
`builtin_standard_material_shader_prewarm_manifest_for_geometry_with_plugin_shading_models(...)`,
prewarms it with WGPU pipeline validation, and then submits a product material
whose authored `lighting_model` is `custom:toon`. Runtime assertions now accept
the expected plugin `ShadingModelId` and require first-frame plus Velocity-frame
staged disk hits, compile miss=0, and zero runtime writes/errors. The same slice
added `with_deferred_lighting_renderer(...)` so Lighting graph execution can
provide deferred resources and mesh draw lists without pretending it has the
GBuffer-only streamer path. Focused no-default-features Cargo passed 1/1 for the
custom product test and 1/1 for the StandardPBR material-pass regression; only
repository warnings were reported. RenderDoc/product capture, broader product
sweep, default features, and full CI remain open after the follow-up
second-launch rerun below.

Custom shading-model second-launch staged-cache WGPU validation is covered by
`render_product_custom_shading_model_second_launch_uses_staged_prewarm_without_compile_miss`
and closes the focused custom second-launch product rerun under status
`render_plan08_custom_shading_model_second_launch_staged_cache_wgpu_passed_renderdoc_deferred`.
The `custom_second_launch.rs` child reuses the `custom:toon` descriptor/source
fixture and WGPU pipeline validation prewarm, then constructs two fresh product
frameworks against the same staged fallback root. Both launches must report the
plugin shading-model dimension, staged disk hits, compile miss=0, no runtime
writes/errors, Velocity previous-transform coverage, and an empty runtime cache
root. RenderDoc/product capture, broader product sweep, default features, and
full CI remain open.

Custom shading-model deferred-lighting product readback is covered by
`render_product_custom_shading_model_deferred_lighting_readback_uses_project_include`
under status
`render_plan08_custom_shading_model_deferred_lighting_product_readback_wgpu_passed_renderdoc_deferred`.
The existing `custom_shading_model.rs` product child now captures the first and
Velocity frames through
`submit_registry_material_passes_with_plugin_shading_model_capture(...)` and
checks that the authored project/plugin deferred include's `max(0.65, ...)`
green signature reaches the final product frame. The shared fixture uses
`select_visible_registry_material_pass_camera(...)` plus a fixed product
viewport so black-frame readbacks do not hide shader execution. Focused
no-default-features Cargo passed 1/1 with 5831 filtered; RenderDoc/product
capture, default features, workspace/full CI, and product cases outside the
focused filter remain open.

Custom shading-model product group direct-binary sweep is recorded under status
`render_plan08_custom_shading_model_product_group_direct_binary_wgpu_passed_renderdoc_deferred`.
The no-default lib-test binary was reused with the
`render_product_custom_shading_model` filter after `--list` confirmed the group
contains exactly the staged-cache, second-launch, and deferred-lighting readback
product tests. The direct run passed 3/3 with 5830 filtered, and the
`runtime_15_material_custom_shading_model_runtime_registry_is_wired` status guard
passed 1/1 with 5838 filtered after docs sync. This is product WGPU
group evidence only; RenderDoc/product capture, default features,
workspace/full CI, and product cases outside the custom filter remain open.

Custom shading-model product group Cargo-wrapper WGPU backfill is recorded under
status
`render_plan08_custom_shading_model_product_group_cargo_wrapper_wgpu_passed_renderdoc_deferred`.
The fresh no-default Cargo wrapper ran the same
`render_product_custom_shading_model` filter instead of reusing a generated test
binary, passed the three custom product tests with 5850 filtered in 19.04s, and
emitted only repository-existing warnings. After docs sync,
`runtime_15_material_custom_shading_model_runtime_registry_is_wired` passed 1/1
through the status/docs guard. This closes the Cargo-wrapper rerun gap and
status/docs guard for the focused custom product group while RenderDoc/product capture,
default features, workspace/full CI, and product cases outside the custom filter
remain open.

Custom shading-model product group default-feature WGPU backfill is recorded under
status
`render_plan08_custom_shading_model_product_group_default_features_wgpu_passed_renderdoc_deferred`.
The default-feature rerun used the same `render_product_custom_shading_model`
filter. The first Cargo-wrapper attempt timed out after 1204.4s while compiling
and linking, produced `zircon_runtime-770562bad16f99eb.exe`, and left no
target-dir cargo/rustc/link processes. The direct generated binary then passed
3/3 with 5932 filtered in 20.20s, and the warmed Cargo-wrapper rerun passed 3/3
with 5932 filtered; its test body finished in 21.45s after a 4m00s build. This
closes the focused default-feature WGPU rerun gate for the custom product group
while RenderDoc/product capture, workspace/full CI, and product cases outside
the custom filter remain open.

Custom shading-model product group default-feature WGPU refresh is recorded
under status
`render_plan08_custom_shading_model_product_group_default_features_wgpu_refresh_passed_renderdoc_deferred`.
This refresh documents the current code path after the standard registry
material-pass surface-source fix exposed a new 6/12 prewarm-validation failure.
`prewarm_pipeline_validation.rs` now validates the material bind-group ABI as
group2 binding0 uniform plus binding1..10 texture/sampler pairs, matching the
runtime material layout. The product fixture keeps runtime `Surface` shader
assets on `registry_material_pass_runtime_surface_source()` and records custom
Surface shader metadata from the plugin descriptor token, so `custom:toon`
stays on the custom shading model instead of requesting `standard_pbr`.
Validation passed lower-layer WGPU prewarm pipeline validation 1/1 with 6186
filtered, custom staged-cache 1/1 with 6187 filtered, custom deferred-lighting
readback 1/1 with 6191 filtered, and custom second-launch 1/1 with 6195 filtered
on `E:\cargo-targets\zircon-plan08-vg-product-default-0703`. This
supersedes the transient current-source 6/12 note; RenderDoc/product capture,
workspace/full CI, and product cases outside the custom filter remain open.

Plan 08 staged-prewarm product sweep is now recorded under status
`render_plan08_staged_prewarm_product_sweep_wgpu_passed_renderdoc_deferred`.
The sweep runs the `staged_prewarm_without_compile_miss` filter across the
  current staged-cache runtime and product tests, including Base mesh, custom
  geometry, project/plugin registry handoff, material-pass second launch, custom
  shading-model product and second launch, and material-mesh deferred second
  launch. The direct binary run reported 11 passed / 0 failed. Guard
  `runtime_15_render_plan08_staged_prewarm_product_sweep_is_wired` keeps that
  test set and this status synchronized; a fresh no-default Cargo wrapper run
  passed it 1/1 with 5825 filtered. RenderDoc/product capture, default
  features, workspace/full CI, and future tests outside the filter remain open.

Plan 08 staged-prewarm product sweep default-feature direct-binary WGPU backfill
is recorded under status
`render_plan08_staged_prewarm_product_sweep_default_features_direct_binary_wgpu_passed_renderdoc_deferred`.
The default-feature `staged_prewarm_without_compile_miss --list` run enumerated
the same 11 runtime/product staged-cache tests, and the direct generated-binary
run passed 11/11 with 5924 filtered in 56.74s. This closes only the focused
default-feature direct-binary WGPU sweep for the existing staged-prewarm filter;
Cargo-wrapper rerun, RenderDoc/product capture, workspace/full CI, and future
tests outside the filter remain open.

Plan 08 staged-prewarm product sweep default-feature Cargo-wrapper WGPU backfill
is recorded under status
`render_plan08_staged_prewarm_product_sweep_default_features_cargo_wrapper_wgpu_passed_renderdoc_deferred`.
The default-feature Cargo wrapper ran the same `staged_prewarm_without_compile_miss`
filter and passed 11/11 with 5925 filtered in 56.99s after a 10m 04s build,
with repository-existing warnings only. This closes the focused default-feature
Cargo-wrapper WGPU sweep for the existing staged-prewarm filter while
RenderDoc/product capture, workspace/full CI, and future tests outside the
filter remain open.

Plan 08 staged-prewarm product sweep default-feature current WGPU refresh is
recorded under status
`render_plan08_staged_prewarm_product_sweep_default_features_current_wgpu_refresh_passed_renderdoc_deferred`.
The current-source default-feature list still enumerated 11 tests for the
`staged_prewarm_without_compile_miss` filter. The first current sweep failed
10/11 at the raw project/plugin registry product because that fixture still
registered assembled Forward-pass WGSL as a runtime Surface shader. The test
support fix moves `registry_staged_cache_runtime_surface_source()` to the
mesh-cache product parent, shares the runtime `fn zr_material_surface(` source
with material-pass registry fixtures, and keeps the raw registry prewarm
manifest on the standard material template hashes/revision. The focused raw
registry product rerun passed 1/1 with 6197 filtered in 5.65s after a 10m 04s
build, and the final Cargo-wrapper sweep passed 11/11 with 6187 filtered in
48.90s after a 7m 36s build. `runtime_15_render_plan08_staged_prewarm_product_sweep_is_wired`
tracks this current refresh; RenderDoc/product capture, workspace/full CI, and
future tests outside the filter remain open.

`render_plan08_project_plugin_registry_material_passes_owner_split_static_passed_cargo_deferred_active_lanes`
documents the Project/plugin registry material-pass owner split. The material
pass product proof now lives in the folder-backed owner
`project_plugin_registry_material_passes_staged_cache/`: `mod.rs` owns the test
entry, `case.rs` owns registry locators and revisions, `manifest.rs` owns
staged request rewriting, `fixture.rs` owns product framework/material setup,
`pipeline.rs` owns the product render-pipeline descriptors, `assertions.rs`
owns prewarm/runtime miss-report checks, `custom_shading_model.rs` owns the
focused plugin shading-model product staged-cache proof, and `second_launch.rs`
owns the broader second-launch acceptance contract. The old single-file owner removed
constraint is guarded so future DepthPrepass/GBuffer/Shadow/TAA reactive mask
coverage cannot return to one near-budget test file. Cargo/WGPU deferred due
active editor validation lanes for this structure-only row.

`runtime_15_shader_prewarm_project_plugin_registry_production_live_wgpu_is_wired`
documents the Project/plugin registry production direct WGPU export status
`render_plan08_project_plugin_registry_production_direct_wgpu_export_passed_product_renderdoc_deferred`.
The live run reused an already-built `zircon_shader_prewarm.exe` instead of
starting Cargo while other build lanes were active, paired a temporary project
shader `.zmeta` root with the real `native_dynamic_fixture/assets` selected
plugin root, exported `shader_resource_records.json`, and immediately consumed
those Ready Shader records in the same prewarm run. The report closed
18/18 requested/written variants with 0 failures, WGPU module validation closed
18/18 validated modules, the runtime cache wrote 18 `.wgsl.zst` artifacts plus
18 `.meta` files, and the registry contained Ready Shader records for
`res://project/shaders/project_shader` and
`package://native_dynamic_fixture/shaders/shader`. The same staged root passed
`validate_staged_shader_prewarm_acceptance_contract(...)`, including WGPU,
cache artifact, usable Ready Shader record, and report-visible registry-backed
source checks. Public Cargo wrapper execution, RenderDoc/product capture, and
broader product miss=0 remain separate Plan 08 gates.

`test_public_runtime_wrapper_exports_project_plugin_registry_with_live_wgpu` and
`runtime_15_shader_prewarm_project_plugin_registry_wrapper_orchestration_is_wired`
document the Project/plugin registry production wrapper orchestration status
`render_plan08_project_plugin_registry_production_wrapper_orchestration_passed_cargo_proxy_product_renderdoc_deferred`.
The Python regression enters through `zircon_build.main(...)`, selects the real
`native_dynamic_fixture` plugin, stages runtime artifacts through a test-local
cargo proxy, and forwards the prewarm `cargo run ... --` payload to the existing
`zircon_shader_prewarm.exe`. The real prewarm run still exports and consumes the
project/plugin `shader_resource_records.json`, closes 18/18 requested/written
variants with 0 failures, validates 18/18 WGPU modules, and records Ready Shader
entries for `res://project/shaders/project_shader` and
`package://native_dynamic_fixture/shaders/shader`. The same public-wrapper
staged root passes `validate_staged_shader_prewarm_acceptance_contract(...)`.
This closes wrapper orchestration through a cargo proxy; a real Cargo-wrapper
build/run, RenderDoc/product capture, and broader product miss=0 remain separate
Plan 08 gates.

`test_runtime_server_wrapper_uses_client_features_for_preview_binary` and
`runtime_15_shader_prewarm_project_plugin_registry_wrapper_no_proxy_is_wired`
document the Project/plugin registry production wrapper no-proxy WGPU run status
`render_plan08_project_plugin_registry_production_wrapper_no_proxy_wgpu_passed_product_renderdoc_deferred`.
The no-proxy public wrapper path now separates target features by artifact:
runtime lib and `zircon_shader_prewarm` use the requested `target-server`
runtime feature set, while the preview executable is rebuilt with
`target-client` through `BuildConfig.runtime_preview_feature_arg`. The focused
target-server `zircon_runtime` lib-test guard passed for this source/docs/status
contract. The real
public command with `native_dynamic_fixture`, a temporary project shader asset
root, `--prewarm-shaders`, and `--validate-wgpu-shaders` completed without a
cargo proxy, staged runtime artifacts and engine assets, exported
`shader_resource_records.json`, then consumed the project/plugin Ready Shader
records in the same run. The resulting report closed 18/18 requested/written
variants with 0 failures and 18/18 WGPU module validation for
`res://project/shaders/project_shader` and
`package://native_dynamic_fixture/shaders/shader`. This closes the public
no-proxy staged build/prewarm/export wrapper gate; RenderDoc/product capture and
broader product miss=0 remain separate Plan 08 gates. This run used WGPU module
validation, not the stricter render-pipeline validation flag.

SH05 import-path/material-sphere evidence is recorded under status
`runtime_shader_sh05_import_path_self_material_material_sphere_wgpu_passed_followups_open`.
`core/framework/render/shader/module_import.rs` now owns deterministic shader
import-path derivation from project namespace plus `assets/shaders/...`, including
same-name directory/file folding and reserved namespace checks. The compound
shader importer consumes that derivation for missing surface/include
`import_path`, emits a warning for redundant explicit values, and rejects
`self::...`/`zr_*::...` overrides; project scan injects the project namespace
into the shader import settings and reports duplicate import-path conflicts in
the same scan batch. The template path uses `self::material` as the generated
material module id, and the explicit IDE anchor is byte-identical to automatic
injection.

`ShaderIdeModuleSource` lives in the `core/framework/render/shader/ide_env.rs` child owner and is
re-exported through the top-level render facade for runtime graphics code that needs the built-in
shader IDE module source list. The facade export is a current shader contract surface, not a
compatibility alias: `graphics/shader/mod.rs` can assemble built-in IDE module sources without
reaching into `shader::ide_env` or a removed template include registry path.

SH05-M2 adds the concrete IDE environment generator on top of that contract.
`zircon_runtime/src/graphics/shader/ide_env_generation.rs` is the shared
generator owner used by the CLI and editor import refresh path. It writes builtin module stubs, asset
`import_path` stubs, per-surface `self::material` generated stubs, and a
`module_map.json` under `.zircon/cache/shader_ide/v1` by default. The module map
entries include schema version, source URI, optional `self::material` scope URI,
stub path, kind, source files, content hash, and generated/source distinction.
`zircon_runtime/src/bin/zircon_shader_ide_env/run.rs` stays a thin shell for
argument parsing, project scanning, and report serialization; editor sync calls
the same generator after import when a project has Ready shader records, so the
IDE artifact remains a single source of truth.
`--variants` still writes the default preview artifact set: one
`preview/<shader>.default.wgsl` plus `preview/<shader>.default.segments.json`
per surface shader. The default variant is intentionally narrow: static mesh,
Forward pass, and all material option bits disabled. Explicit
`--variant <pass[:options=bits]>` requests now flow through
`ShaderIdePreviewVariant`, so the same generator can write non-default artifacts
such as `preview/<shader>.gbuffer_options_0x00000001.wgsl` with a matching
segment map. `graphics/shader/ide_preview.rs` assembles every requested preview
through the same template owner used by runtime material passes, recursively
collects imported include shader sources from asset import redirects, injects
the requested material option defines, and serializes
defines/include/generated-material/user-surface/pass-template segments into
`ShaderIdePreviewSegment` rows. Duplicate preview variant names are rejected
before file generation so report counts cannot drift from managed paths.
Validation passed the focused bin check and 5/5 bin tests in
`E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview`; the one-byte module
diff gate now proves only the changed module stub and `module_map.json` are
rewritten while unchanged stubs preserve content and mtime. The follow-up Naga
gate adds `graphics/shader/ide_validation.rs` and makes the generator parse every
generated stub with Naga before writing, fully validate each default preview
WGSL module, and report `naga_parsed_stub_count` plus
`naga_validated_preview_count`. Standalone stub semantic validation remains
scoped out because stubs may intentionally refer to other generated/imported
modules; semantic gates live on composed preview variants. Editor refresh hook
typecheck now passes, and the editor path still requests only the default
Forward/0bits preview during import refresh. Product-level non-default preview
matrix enumeration remains a later SH05 follow-up.

SH05-M3 closes the diagnostic line-remap main path for material template
assembly. `graphics/shader/template/assemble.rs` now returns
`ShaderAssemblySegment` data for defines, includes, generated material,
user-surface, and pass-template chunks; `validation.rs` consumes Naga source
locations and appends the remapped `module_id:local_line:column` coordinate to
parse/validation diagnostics. Runtime material pass assembly now carries the
surface shader `import_path` from `ResourceStreamer` into Forward, GBuffer, and
TAA template requests, with `self::surface` as the fallback module id. The
focused SH05-M3 gate passed `cargo check -p zircon_runtime --lib` and
`cargo test -p zircon_runtime --lib shader_template_` 20/20 under
`E:\cargo-targets\zircon-runtime-shader-sh05-m3`.

The offscreen product screenshot test now builds a real compound surface shader
package, imports it as `ShaderAssetKind::Surface`, renders a material sphere
through `SceneRenderer`, and writes the accepted image to
`docs/tests/runtime/shader/runtime_shader_material_sphere_offscreen_20260703.png`.
The PNG is 1024x1024, SHA256
`927A652BFB6486145C9F6CDBD2E5EE49ED132DAA5290013422FD0C2E9769B794`, sampled
137956 visible pixels, 1710 unique colors, and 48.66 luma range; same-name
searches under repo `target` and `E:\cargo-targets\zircon-runtime-shader-sh05-m1`
returned no matches. Validation passed
`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m1 --message-format short --color never`
and
`cargo test -p zircon_runtime --lib export_runtime_shader_material_sphere_png --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-m1 -- --ignored --nocapture`.
Default preview variant files, segment JSON, and the content-aware one-byte
module diff gate are now owned by the library generator tests, while the bin
keeps only CLI argument coverage after the structure split. The stub
parse/default-preview validation gate now composes a validation-only context for
Naga parse: builtin WGSL dependencies, default feature defines,
`self::material` generated stubs, and the current stub are checked together,
without changing the emitted stub files. `zr_surface_types.wgsl` is the shared
surface/GBuffer contract owner for `ZrSurfaceInput`, `zr_surface_default(...)`,
`ZrDeferredGBufferOutput`, and deferred material flag encoding; the Deferred
GBuffer template consumes that contract instead of redeclaring the output ABI.
The post-split `shader_ide_env` library test now passes 6/6, the editor refresh
hook typecheck passes under
`E:\cargo-targets\zircon-runtime-shader-sh05-editor-refresh`, the
`shader_template_` regression passes 20/20 under
`E:\cargo-targets\zircon-runtime-shader-sh05-m4-preview`, and the focused
`shader_module` gate passes 6/6 under
`E:\cargo-targets\zircon-runtime-shader-sh05-shader-module`. The explicit
non-default preview slice passes the focused runtime bin check, editor lib
check, `shader_ide_env_writes_non_default_preview_variants_with_option_bits`,
`shader_ide_env_rejects_duplicate_preview_variant_names`, and both
`zircon_shader_ide_env` parser tests under
`E:\cargo-targets\zircon-runtime-shader-sh05-nondefault-preview`. The accepted
visual evidence remains outside target in
`docs/tests/runtime/shader/runtime_shader_material_sphere_offscreen_20260703.png`
and `docs/tests/runtime/shader/runtime_shader_material_vampire_offscreen_20260703.png`.
Product-level preview matrix coverage is now closed by the 2026-07-04 focused
filter, and focused material-pass second-launch product miss=0 is closed by the
SH04 direct-binary rerun. Broader product/RenderDoc gates and wider product/perf
sweeps remain open.
The 2026-07-04 preview-matrix backfill adds
`shader_ide_env_batches_preview_matrix_for_all_surface_shaders`, covering two
surface shaders across default, GBuffer option bits, DepthPrepass, Shadow,
Velocity, and TAA reactive-mask preview variants, plus stale preview cleanup
when the request shrinks back to default-only. The test code compiles under
`cargo check -p zircon_runtime --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-shader-sh05-preview-matrix-check --message-format short --color never`;
the later exact lib-test execution passed 1/1 under
`F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b`.
The same follow-up lane also closed the SH02 focused property/template checks:
`cargo test -p zircon_runtime --lib property_layout` passed 4/4 and
`cargo test -p zircon_runtime --lib render_shader_template` passed 18/18 on
`E:\cargo-targets\zircon-runtime-shader-sh05-nondefault-preview` after fixing
the status mirror anchor gap. Broader `shader`/`material` filters are still
separate product gates.

## 2026-07-04 SH05 Preview Matrix and TAA Material-Mask ABI

Related code: `zircon_runtime/src/graphics/shader/ide_env_generation.rs`, `zircon_runtime/src/graphics/shader/ide_env_generation/tests.rs`, `zircon_runtime/src/graphics/shader/wgsl/zr_template_taa_reactive_mask.wgsl`, `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs`, `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs`, and `zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/wgsl_contracts.rs`.

The IDE preview generator validates generated material dependencies by shader scope. Generated material stubs still publish the local `self::material` import path, but validation dependency selection now also requires the generated stub `scope_uri` to match the current shader source URI. This prevents multi-surface projects from appending another shader's generated `ZrMaterialProperties` block into the current stub validation source while leaving written module paths and preview output unchanged.

The TAA reactive material-mask template keeps the existing `fs_taa_reactive_material_mask` entry point for the mesh pipeline cache, but the body now calls `zr_material_surface(input)` and reads `surface.custom0.x`. Custom surface previews no longer require the old `standard_material_properties` binding just to parse and validate a TAA preview. Standard material still projects its authored TAA strength into `surface.custom0` through the standard material surface owner.

Evidence: `cargo test -p zircon_runtime --lib shader_ide_env_batches_preview_matrix_for_all_surface_shaders --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b --message-format short --color never -- --nocapture --test-threads=1` passed 1/1, covering two surface shaders across default, GBuffer option bits, DepthPrepass, Shadow, Velocity, TAA preview variants, and stale cleanup. `cargo test -p zircon_runtime --lib taa_reactive_mask --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b --message-format short --color never -- --nocapture --test-threads=1` passed 13/13. Both runs emitted only existing warnings. RenderDoc/product capture and wider product/perf sweeps remain separate acceptance gates; focused material-pass second-launch miss=0 is covered by the SH04 2026-07-04 direct-binary rerun.

The broader `render_product_assets` filter and `cargo check -p zircon_runtime --lib --tests --locked` remain the milestone-level compile/test gates for this surface.

## 2026-07-04 SH03 Asset-Root Include Module Revision Propagation

Related code: `zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs`, `zircon_runtime/src/bin/zircon_shader_prewarm/manifest/module_dependencies.rs`, `zircon_runtime/src/bin/zircon_shader_prewarm/manifest/revision.rs`, `zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs`, and `zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests/module_dependencies.rs`.

Asset-root prewarm scanning now keeps shader kind, `import_path`, and declared imports on each scanned `.zshader` source. Before variants are emitted, `manifest/module_dependencies.rs` builds a same-root include-module map and resolves imported include modules transitively. Referencing surface shaders receive the include modules' content hashes in `include_content_hashes`, and `manifest/revision.rs` mixes those dependency hashes into the surface shader `material_revision`. Sources without include dependencies keep their previous revision behavior, so unrelated shaders do not churn when a module they never import changes.

Evidence: `cargo test -p zircon_runtime --bin zircon_shader_prewarm shader_prewarm_asset_root_manifest_tracks_imported_include_module_revisions --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b --message-format short --color never -- --nocapture --test-threads=1` passed 1/1. The broader asset-root prewarm filter `cargo test -p zircon_runtime --bin zircon_shader_prewarm shader_prewarm_asset_root_manifest --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-shader-sh03-readiness-0704b --message-format short --color never -- --nocapture --test-threads=1` passed 10/10 with existing warnings only. Static structure equivalence passed for the new child-owner split: `manifest.rs` 762 lines, `manifest/tests.rs` 738 lines, `manifest/module_dependencies.rs` 77 lines, `manifest/tests/module_dependencies.rs` 97 lines, parent manifest test count 10, and docs/status mirrors include both new child owners. The direct Runtime 15 lib-test guard `runtime_15_shader_prewarm_manifest_tests_are_folder_backed` did not produce countable Cargo evidence in this session because Windows lib-test compilation timed out and the later log-file wrapper stopped making progress without a rustc/test child. Plugin manifest module registration and source-only live import dependency propagation are now focused-test closed; editor project-sync refresh has source-level regression coverage but its exact editor Cargo run is still waiting on a counted result. RenderDoc/product capture and broader product/perf sweeps remain open.

## 2026-07-04 SH03 Plugin Shader Module Registry Export

Related code: `zircon_runtime/src/plugin/package_manifest/plugin_shader_permutation_manifest.rs`, `zircon_runtime/src/plugin/package_manifest/constructors.rs`, `zircon_runtime/src/bin/zircon_shader_prewarm/manifest/permutation_registry.rs`, `zircon_runtime/src/bin/zircon_shader_prewarm/manifest/module_dependencies.rs`, `tools/zircon_build_plugin_shader_descriptors.py`, `tools/zircon_build_shader_prewarm.py`, and `tools/plugin_structure_audits/manifest_schema_geometry_sources.py`.

Plugin manifests can now declare shader modules with `[[shader_permutation.shader_modules]]` rows. Each row uses an `import_path` plus a package-relative `source` ending in `.zshader` or `.wgsl`. The Rust package manifest exposes the same contract as `PluginShaderModuleManifest` and `PluginPackageManifest::with_shader_module(...)`, so generated manifests and hand-written fixture manifests roundtrip through the same API.

The Python build path collects selected plugin modules into `PluginPackage.shader_modules`, hashes the source bytes, and emits registry records under `shader_modules`. The generated registry is now created when selected plugins contribute shader modules even if they do not contribute custom geometry-source or shading-model ids. The export contract validates selected plugin module import paths and hashes, and the plugin structure audit rejects unknown fields, non-namespace import paths, non-package-relative sources, backslash paths, unsupported suffixes, and duplicate import paths.

`zircon_shader_prewarm` now merges registry `shader_modules` into an external include-module hash table. Asset-root scanning still prefers a local include module with the same `import_path`, but a project shader can reference a plugin module exported through the registry and receive that external module hash in its `include_content_hashes` and mixed `material_revision`. This closes the cross-asset-root prewarm dependency gap without forcing project and plugin assets into one scan batch.

Evidence: Python regression coverage passed 52/52 with `python -m unittest tools.tests.test_zircon_build_plugin_carriers tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_permutation_registry_contract tools.tests.test_plugin_structure_audit_manifest_schema_geometry_sources`; `python -m py_compile` passed for the touched build/audit modules; Rust formatting passed for the touched prewarm files; and `cargo check -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir E:\cargo-targets\zircon-shader-plugin-modules-check --jobs 1 --message-format short --color never` passed with existing warnings. The direct generated test binary under `E:\cargo-targets\zircon-shader-plugin-modules-check\debug\deps\` passed `manifest::permutation_registry::tests::shader_prewarm_permutation_registry_merges_shader_modules` 1/1 and `manifest::tests::module_dependencies::shader_prewarm_asset_root_manifest_tracks_registry_shader_module_revisions` 1/1 after `--list` confirmed both exact tests were present. The accepted material sphere and vampire screenshots remain under `docs/tests/runtime/shader`, and same-name scans under target/cargo-target roots returned no matches.

## 2026-07-04 SH03 Source-Only Import Reload Dependencies

Related code: `zircon_runtime/src/asset/project/manager/scan_and_import.rs`, `zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies.rs`, and `zircon_runtime/tests/shader_import_dependency_contract.rs`.

Project import now has a runtime-facing dependency path for source-only shader module imports. After scan/import has written artifact records, `shader_import_dependencies.rs` indexes unique project include shaders by `ShaderAsset.import_path`, then appends matching include module locators to the referencing shader `ResourceRecord.dependency_ids`. Redirect imports keep the existing explicit dependency path, built-in `zr_*` and generated `self::*` tokens are skipped, and duplicate include import paths are not auto-bound because the scanner's duplicate diagnostics must resolve that ambiguity first.

Evidence: rustfmt passed for the touched scan/import files and the integration contract; scoped diff-check passed with only existing LF/CRLF warnings; `cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-live-import-deps-check --message-format short --color never` passed with existing warnings; and `cargo test -p zircon_runtime --test shader_import_dependency_contract --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-live-import-deps-check --message-format short --color never -- --nocapture --test-threads=1` passed 1/1. The broader lib-test lane for the same behavior is not counted because unrelated test-only drift blocked compilation in render-product and Runtime 15 status-support tests.

## 2026-07-04 SH03 Redirect Import Readiness Diagnostics

Related code: `zircon_runtime/src/asset/assets/material/material_asset.rs`, `zircon_runtime/src/asset/tests/assets/material/shader_readiness.rs`, and `zircon_runtime/tests/material_shader_redirect_dependency_contract.rs`.

Material readiness now checks the shader contract's redirect import dependencies as shader dependencies. When a surface shader imports an include module through a redirect and that redirected module cannot be resolved, `MaterialAsset::readiness_report_with_shader_contract(...)` adds `RenderMaterialValidationError::UnresolvedShaderReference` and a shader fallback usage to the material readiness report. This makes the redirect chain visible at the material/runtime diagnostic boundary instead of waiting for module template assembly or pipeline creation to fail later.

Evidence: rustfmt passed for the touched material readiness files and integration test; `cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-redirect-diagnostics-check --message-format short --color never` passed with existing warnings; and `cargo test -p zircon_runtime --test material_shader_redirect_dependency_contract --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-redirect-diagnostics-check --message-format short --color never -- --nocapture --test-threads=1` passed 2/2, covering both the synthetic dependency report and a `ProjectManager::scan_and_import` project that loads real imported shader/material artifacts before resolving the redirect dependency diagnostic. The equivalent lib-test wrapper is not counted because compiling the full runtime lib-test harness timed out in this Windows lane. RenderDoc/product capture and broader product miss=0 sweeps remain separate acceptance gates.

## 2026-07-04 SH03 Product Redirect Import Diagnostics

Related code: `zircon_runtime/src/graphics/scene/render_product_streamer_tests/readiness_diagnostics.rs` and `zircon_runtime/src/graphics/scene/render_product_streamer_tests/readiness_diagnostics/shader_redirect.rs`.

The product ResourceStreamer boundary now has focused redirect-import diagnostics coverage. The resolved case prepares the redirected include shader recursively and verifies `shader_module_include_sources(...)` exposes the include token and WGSL source for template assembly. The missing case verifies `ensure_material(...)` stores a non-ready material report with `UnresolvedShaderReference` and shader fallback usage for the missing redirected include without blocking the product diagnostic path. The parent readiness diagnostics test owner remains at 651 lines; the redirect child owner holds the new 148-line coverage.

Evidence: touched-file rustfmt passed for the parent and child readiness diagnostics files. The Cargo wrapper `cargo test -p zircon_runtime --lib render_product_streamer_ --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shader-redirect-product-streamer --message-format short --color never -- --nocapture --test-threads=1` exceeded the 1204s tool window and is not counted, but it produced `E:\cargo-targets\zircon-shader-redirect-product-streamer\debug\deps\zircon_runtime-6bef7a696c15c9a5.exe`; direct execution with `shader_redirect --nocapture --test-threads=1` passed 2/2 with 6399 filtered in 2.46s. RenderDoc/product capture and broader product/perf sweeps remain separate acceptance gates.

## 2026-07-04 SH04 Material-Pass Second-Launch Miss=0 Refresh

Related code: `zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/second_launch.rs` and `zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_material_passes_staged_cache/assertions.rs`.

The focused product material-pass staged-prewarm gate now has current direct-binary evidence for second-launch cache behavior. The test writes the staged material-pass manifest once, runs two fresh product launches per registry shader through the same staged fallback root, and asserts first-frame and Velocity-frame requests stay on staged/cache hit paths with `compile_miss_count == 0`, zero runtime cache writes/errors, and an empty runtime shader cache root.

Evidence: `E:\cargo-targets\zircon-runtime-03-gates-0704\debug\deps\zircon_runtime-33095b46939b64fc.exe graphics::tests::render_product_mesh_cache::project_plugin_registry_material_passes_staged_cache::second_launch::render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss --exact --nocapture --test-threads=1` passed 1/1 with 6400 filtered in 10.59s. This closes the focused material-pass second-launch miss=0 proof for the current shader/material slice only; broader staged-prewarm/product/perf sweeps and RenderDoc/product capture remain separate gates.

## 2026-07-03 Deferred Project-Shader GBuffer Default-Feature WGPU Refresh

Plan 08 three shading-model forward/deferred product parity now includes a default-feature refresh for the non-custom Deferred/GBuffer project-shader probe: `render_plan08_deferred_project_shader_gbuffer_probe_default_features_wgpu_refresh_passed_renderdoc_deferred`. This is mirrored beside `STATUS`, `DEFAULT_FEATURES_STATUS`, `DEFERRED_PROBE_STATUS`, and `DEFERRED_PROBE_DEFAULT_STATUS`; the product chain remains `render_product_three_shading_models_forward_deferred_parity`, `runtime_15_render_plan08_three_shading_models_forward_deferred_parity_is_wired`, PBR/Blinn-Phong/Unlit, Forward + Deferred, `light_grid_external_fallback_buffers_satisfy_materialization_report`, `deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path`, and `average_channel_in_region`.

The runtime shader contract now distinguishes Surface metadata from runtime material-surface source. A project `.wgsl` with `vs_main` / `fs_main` may still be a Surface asset for material/pass participation, but the mesh template path only wraps it when `shader_uses_material_surface_source` finds `fn zr_material_surface`. `runtime_surface_shader_with_full_pass_entry_points_uses_raw_wgsl_source` guards the raw full-pass branch so the Base pass resolves cache-backed pipeline variants instead of treating full-pass WGSL as a material function. Evidence: the default-feature parity backfill remains 5876 filtered and 11.81s; this probe passed 1/1 with 6202 filtered and 3.56s. Remaining gates: RenderDoc/product capture, workspace/full CI, and broader product coverage outside focused filters.

## 2026-07-04 Custom Shading-Model Deferred-Lighting Product Readback PNG

Custom shading-model deferred-lighting product readback PNG now includes focused product artifact evidence under `render_plan08_custom_shading_model_deferred_lighting_product_readback_png_passed_renderdoc_deferred`. The ignored export `export_custom_shading_model_deferred_lighting_product_png` reuses the `custom:toon` registry material-pass manifest, WGPU pipeline validation, staged-cache first-frame/Velocity-frame hit assertions, read-only runtime cache checks, and `render_product_custom_shading_model_deferred_lighting_readback_uses_project_include`, then writes `docs/tests/runtime/render/runtime_render_plan08_custom_shading_model_deferred_lighting_20260704.png` through `project_plugin_registry_material_passes_staged_cache/custom_product_png.rs`.

The code path remains test-only and folder-backed: `custom_product_png.rs` owns the ignored export, `custom_shading_model.rs` exposes the existing dominant-green `max(0.65, ...)` assertion to its sibling, and `product_png.rs` owns only generic visible-frame and side-by-side PNG helpers. Evidence: the first Cargo-wrapper command exceeded the 908s tool window and is not counted; direct generated-binary execution passed 1/1 with 6309 filtered and 9.66s, producing a 641x240 PNG, 3871 bytes, SHA256 `21188825B3FCEC7089BC198CDF89B53527332583FFAF5B3755317BF11EAD66F2`, with 4794 non-black pixels, 4554 dominant green pixels, and 240 magenta separator pixels. RenderDoc/product capture, workspace/full CI, full live project/plugin registry export beyond focused fixtures, and broader product miss=0 remain open.

## 2026-07-04 Project/Plugin Registry Material-Pass Product Readback PNG

Plan 08 project/plugin registry material-pass product readback PNG now includes focused product readback PNG evidence under `render_plan08_project_plugin_registry_material_passes_product_readback_png_passed_renderdoc_deferred`. The ignored export `export_project_plugin_registry_material_passes_product_png` reuses the existing registry material-pass manifest, staged-cache first-frame/Velocity-frame hit assertions, and read-only runtime cache root checks, then writes `docs/tests/runtime/render/runtime_render_plan08_project_plugin_registry_material_passes_20260703.png` through `project_plugin_registry_material_passes_staged_cache/product_png.rs`.

The code path remains test-only and folder-backed: `fixture.rs::submit_registry_material_passes_with_staged_cache_capture(...)` only forwards the existing capture flag, while `product_png.rs` owns visible-frame checks, side-by-side RGBA copy, separator pixels, and PNG writing. Evidence: the first Cargo-wrapper command exceeded the 904.5s tool window and is not counted; direct generated-binary execution passed 1/1 with 6290 filtered and 6.58s, producing a 641x240 PNG, 3871 bytes, SHA256 `2FF919F50FDFFBAEB1544CAD9C14B7748FA8234C784175195AF3E550FB6151BB`, with 4794 non-black pixels. RenderDoc/product capture, workspace/full CI, full live project/plugin registry export beyond the focused fixture, and broader product miss=0 remain open.

## 2026-07-04 Project/Plugin Registry Material-Pass Live Registry Source-Label Product Bridge

Project/plugin registry material-pass live registry source-label product bridge now records `render_plan08_project_plugin_registry_material_passes_live_registry_source_label_product_wgpu_passed_renderdoc_deferred`. `render_product_project_plugin_registry_material_passes_live_registry_source_labels_hit_staged_cache` lives in `project_plugin_registry_material_passes_staged_cache/live_registry_bridge.rs`, rewrites project/plugin prewarm `source_label` values to live registry locators, and guards that `live registry source label should not depend on test-only pass suffixes` while pass identity remains in `ShaderVariantKey`.

Evidence: the `PluginShaderModuleManifest` root export was added in `zircon_runtime/src/plugin/mod.rs`, unblocking the `package_manifest_declarations.rs` import shape. `rustfmt --edition 2021 --check zircon_runtime/src/plugin/mod.rs` and the static root-export scan passed. After an initial timeout and unlaunchable generated binary on `E:\cargo-targets\zircon-plan08-live-registry-label-0704`, the focused no-default/target-server Cargo rerun passed 1/1 with 6358 filtered, test body 10.44s, after a 9m55s build. RenderDoc/product capture and broader validation remain open.

## 2026-07-04 Project/Plugin Registry Material-Pass Asset-Root ResourceRecord Product Bridge

Project/plugin registry material-pass asset-root ResourceRecord product bridge now records `render_plan08_project_plugin_registry_material_passes_asset_root_records_wgpu_passed_renderdoc_deferred`. `render_product_project_plugin_registry_material_passes_asset_root_records_hit_staged_cache` lives in `project_plugin_registry_material_passes_staged_cache/live_registry_records.rs` and proves shader identity can be derived from Ready Shader `ResourceRecord`s scanned from asset roots rather than fixture-only revisions.

The bridge writes a temporary project shader `.wgsl/.zmeta`, reads the selected plugin record from `native_dynamic_fixture/assets/shader.wgsl.zmeta`, applies the asset-scan source-hash revision rule, and converts those records into product registry cases through `registry_shader_cases_from_live_records`. The product path then uses the shared live-source-label prewarm helper and submits first-frame plus Velocity-frame WGPU product passes against staged fallback roots. Evidence: touched-file rustfmt passed, and the focused no-default/target-server Cargo command passed 1/1 with 6373 filtered and 10.71s on `E:\cargo-targets\zircon-plan08-live-registry-label-0704`. RenderDoc/product capture, workspace/full CI, and broader live project/plugin registry export acceptance remain open.

## 2026-07-04 Project/Plugin Registry Shared ResourceRecord Export Product Bridge

Project/plugin registry shared ResourceRecord export product bridge records `render_plan08_project_plugin_registry_shared_resource_record_export_product_wgpu_passed_renderdoc_deferred`. The shared scanner now lives in `zircon_runtime/src/asset/project/shader_resource_records.rs` and is re-exported from `asset/project/mod.rs`; `bin/zircon_shader_prewarm/manifest/resource_registry.rs` delegates to it, maps `ShaderResourceRecordExportError` back into the prewarm tool's typed registry error surface, and mirrors the shared multi-root export as `project_shader_resource_records_from_asset_roots`. The material-pass product bridge now calls `shader_resource_records_from_asset_roots(...)`, so the WGPU staged-cache product proof consumes the same Ready Shader record export path as the prewarm CLI.

The shared owner covers `.zmeta` shader discovery, root and subasset shader records, duplicate id/locator rejection, stable ordering, Ready state, and source-hash revision projection. Evidence so far: touched-file rustfmt passed for the shared owner, prewarm wrapper/tests, product bridge, and Runtime 15 guards; the focused `zircon_shader_prewarm` Cargo check passed after 4m48s; `runtime_15_lock_poison_status_row_data_status_mirror_children_are_child_owned` passed 1/1 with 6394 filtered after the lower status-output support fix; the focused WGPU product rerun passed 1/1 with 6395 filtered, test body 10.29s, after an 8m40s build; the refreshed structure guard passed 1/1 with 6402 filtered; and the shader-prewarm bin duplicate-id/locator regression passed 1/1 with 54 filtered. RenderDoc/product capture, workspace/full CI, and broader live registry export acceptance remain open.

## 2026-07-03 Three Shading-Model Product Readback PNG

Plan 08 three shading-model forward/deferred product parity now includes focused product readback PNG evidence under `render_plan08_three_shading_models_forward_deferred_product_readback_png_passed_renderdoc_deferred`. The ignored export `export_three_shading_models_forward_deferred_product_png` reuses `render_product_three_shading_models_forward_deferred_parity`, PBR/Blinn-Phong/Unlit, Forward + Deferred, `light_grid_external_fallback_buffers_satisfy_materialization_report`, `deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path`, and `average_channel_in_region`, then writes `docs/tests/runtime/render/runtime_render_plan08_three_shading_models_forward_deferred_product_20260703.png`.

Evidence: the first Cargo-wrapper attempt failed at MSVC `link.exe` exit `0xc0000142` and is not counted. The warmed generated test binary `zircon_runtime-33095b46939b64fc.exe` passed `export_three_shading_models_forward_deferred_product_png` 1/1 with 6212 filtered and 14.04s, producing a 641x240 PNG, 3998 bytes, SHA256 `D493C941CBCF418A2C66F84F663881E9DA6B0984B023BAACEC254F094AB483B6`. The same binary passed `runtime_15_render_plan08_three_shading_models_forward_deferred_parity_is_wired` 1/1 with 6212 filtered and 0.86s, confirming the module-doc anchors. RenderDoc/product capture, workspace/full CI, and broader product coverage remain open.

## 2026-07-03 Morph Velocity Scene-Velocity Readback PNG

Plan 08 morph velocity now includes focused scene-velocity PNG evidence under `render_plan08_morph_weight_velocity_product_png_passed_renderdoc_deferred`. The ignored exports `export_direct_morph_weight_velocity_product_png` and `export_skinned_morph_weight_velocity_product_png` reuse the existing direct/skinned 0.0 -> 1.0 morph velocity product paths, then encode test-only `scene-velocity` RG16Float readback bytes through `graphics/tests/render_product_mesh_cache/morph/velocity_png.rs` as sign-masked half-float payload visualization.

The readback bytes remain a test-only diagnostic route: `RenderGraphExecutionRecord` keeps them behind `#[cfg(test)]`, and `WgpuRenderFramework::last_scene_velocity_readback_rg16_float_bytes_for_tests` exposes them only to crate tests. The shared `RenderSceneVelocityReadbackReport` now ignores signed-zero RG16Float pixels, covered by `scene_velocity_readback_ignores_signed_zero_half_float_pixels`, so stats and debug output agree on real nonzero payloads. Evidence: the first Cargo-wrapper attempt exceeded the 1213.4s tool window and is not counted; generated binary `zircon_runtime-0a7825d39d44b0c4.exe` passed the regular direct/skinned product tests 1/1 each with 6207 filtered (3.84s direct, 3.60s skinned), the signed-zero regression 1/1 with 6207 filtered, direct/skinned exports 1/1 each with 6207 filtered (3.36s direct, 3.42s skinned), and the fully qualified `runtime_15_render_product_mesh_cache_morph_tests_are_child_owners` 1/1 with 6207 filtered and 0.25s. Both output PNGs are 128x128, 723 bytes, SHA256 `7B40FA8BA6EA60F7F24F3C5465F3C802B4119E26A8965DACECA08187FF665DE7`, with 508 non-black pixels. RenderDoc/product capture, workspace/full CI, and broader product miss=0/second-launch acceptance remain open.

## 2026-07-04 Procedural Skybox PBR Environment Matrix

Plan 11 EL-M1 now has a formal environment extract path and a shader-visible procedural skybox source for the standard material path. `core/framework/render/environment` owns `EnvironmentExtract`, `SkyboxSettings`, `ProceduralSkyParams`, and `IblBakeKey`; render-frame and viewport packets carry that extract, and preview sky settings project into the same contract instead of leaving sky policy in the overlay pass.

The renderer side adds `graphics/scene/scene_renderer/environment` and `skybox_procedural.wgsl` for the visible procedural skybox. `SceneUniform` now publishes sky colors and environment flags, while `graphics/shader/wgsl/zr_environment.wgsl` provides the shared `zr_environment_pbr_indirect(...)` helper. Forward standard PBR, fallback mesh shading, and deferred lighting all consume that helper so metallic and smoothness change the reflected procedural environment in the same material contract.

The product evidence is the ignored export `graphics::tests::project_render::project_scenes::export_runtime_shader_pbr_metallic_smoothness_matrix_png`. It builds a temporary project with 64 standard PBR `.zmaterial` spheres: columns sweep metallic `0.0..1.0`, rows sweep smoothness `0.0..1.0`, and the camera frames them inside the procedural skybox. The accepted artifact is `docs/tests/runtime/shader/runtime_shader_pbr_metallic_smoothness_matrix_skybox_20260704.png`, 1280x960, 109556 bytes, SHA256 `E883A3BDF657025EAD16A7F39B1F8BE5D7FFCDA1FDEF0243A8636A05C217030D`; the same-name scan under repo `target`, `E:\cargo-targets`, `F:\cargo-targets`, and `D:\cargo-targets` returned zero hits.

This section does not mark cubemap import/prefiltering, reflection probe capture/blending, lightmaps, light probe grids, analytic fog, RenderDoc capture, or a broad product/perf sweep complete. Those remain Plan 11 later milestones.

## 2026-07-04 Retired Real HDRI Sampled Environment Reflection

This section is historical evidence for the rejected sampled-equirect bridge. The current real HDRI validation path is the source-cubemap GGX PMREM path documented in [source-cubemap.md](/E:/Git/ZirconEngine/docs/zircon_runtime/core/framework/render/environment/source-cubemap.md), and the runtime render contract no longer exposes the sampled-equirect environment type.

The source asset is Poly Haven `lakes` 1K HDR, stored as `docs/tests/runtime/shader/assets/polyhaven_lakes_1k.hdr` with MD5 `B615491D315A3D4E23BB09C2C96C9E03` and SHA256 `FAF3ECE79216E568A29F0D8FC176A795C66EB9C312C3CF3EE18D9AC04A71DECB`. The retired test/export path decoded that HDR, reduced it to a fixed 16x8 equirectangular sample table, and sent it through the old environment transport.

The accepted product evidence is `graphics::tests::project_render::project_scenes::export_runtime_shader_pbr_real_hdri_reflection_png`. It builds the same 8x8 standard PBR metallic/smoothness matrix, but uses `SkyboxMode::SampledEquirectangular` so the skybox and PBR indirect reflection read the real HDR-derived table. The accepted artifact is `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png`, 1280x960, 132232 bytes, SHA256 `958E3B200EC56BCA16BF9596B1F05D872179F51CEB9A64925E10FC2D41792DEE`; the same-name scan under repo `target`, `E:\cargo-targets`, `F:\cargo-targets`, and `D:\cargo-targets` returned zero hits.

This was intentionally a sampled equirectangular environment proof, not the final cubemap pipeline. It is superseded by the source-cubemap screenshots and does not close Plan 13 cubemap asset import, production RGBA16F PMREM storage, GPU/offline compute, reflection probe capture/blending, or SH irradiance bake.

## 2026-07-05 Source Cubemap SH9 Diffuse Bridge

Plan 06 EC-M2b now routes source-cubemap diffuse ambient through SH9 instead of sampling a rough low mip. `SourceCubemapMipChain` stores nine cosine-lobe-premultiplied coefficients projected from the regular source mip closest to 32x32 faces, with the same exact cubemap solid-angle weighting used by the PMREM CPU reference. `SceneUniform.environment_sh9` carries those coefficients to WGSL, and `zr_environment_sh9_eval(...)` evaluates the y-up basis before applying environment intensity.

This row was closed before the EC-M2c float bridge below. Current source-cubemap specular now uses the CPU GGX PMREM/RGBA16F runtime path documented in the next section, while GPU/offline SH9/PMREM/BRDF baking, IEM, derived caches, seam tests, and quantitative 8x8 acceptance remain open. Evidence for the original SH9 row: `runtime_environment_source_cubemap_contract` passed 5/5, and the ignored HDRI export refreshed `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_ggx_pmrem_reflection_20260705.png` at 1280x960, 862851 bytes, SHA256 `D3A8077CAD3A7F0CBD83634F33EBBFA62422E3C75E2B65B34AA85A66FEDF0029`; same-name target/cargo-target scan returned zero hits.

## 2026-07-06 Source Cubemap IEM CPU Bridge

Plan 06 EC-M2d adds [source_irradiance_cubemap.rs](/E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/environment/source_irradiance_cubemap.rs) as the optional IEM CPU reference owner. It builds a fixed 32x32x6 face-major diffuse irradiance cube by direct cosine convolution over the source cubemap mip selected for diffuse irradiance, using the same exact solid-angle weights and cross-face sampling discipline as the SH9/source-cubemap path.

This does not yet change runtime WGSL diffuse lighting: source-cubemap diffuse still consumes SH9 today. The IEM bridge exists so later GPU/offline bake, derived-cache, upload/binding, and shader-option work can compare against a concrete CPU artifact without adding more code to the large `source_cubemap.rs` file.

Evidence: `rustfmt` passed for the new module, environment facades, and focused test. The first focused Cargo run failed on missing root `core::framework::render` facade exports for the new IEM symbols; after adding those exports, `CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --test runtime_environment_source_irradiance_cubemap_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-iem-contract-0706 --message-format short --color never -- --nocapture --test-threads=1` passed 2/2. A later Cargo wrapper recheck timed out at the tool boundary after 904s and is not counted as pass evidence; direct execution of the already-built test binary then passed 2/2 in 15.85s. This slice generated no screenshot.

## 2026-07-05 HDR Source/PMREM Float Bridge

Plan 06 EC-M2c separates sky/source sampling from specular reflection sampling. `SourceCubemapMipChain::source_texels()` now preserves the regular source mip pyramid for skybox and source lookups, while `SourceCubemapMipChain::texels()` remains the GGX PMREM output consumed by standard PBR reflections. The scene bind group carries both cube textures: binding 1 is the source cube, binding 4 is the specular PMREM cube, and binding 3 is an RG16F BRDF LUT for split-sum specular scale/bias.

The runtime upload path now uses RGBA16F for both cube textures, with shared FP16 packing in `scene_renderer_core/half_float.rs`. The real-HDRI export helper preserves exposed linear HDR samples before cubemap construction instead of Reinhard tone-mapping them into 0..1, so bright HDR values can influence PMREM and BRDF-LUT reflection results. This section still leaves GPU/offline PMREM baking, derived artifact cache reuse, IEM, seam quantization, and full EC-M3 8x8 numerical acceptance open.

Evidence: `cargo check -p zircon_runtime --lib --no-default-features --features core-min` passed; `runtime_environment_source_cubemap_contract` passed 6/6; `runtime_environment_brdf_lut_contract` passed 2/2; and the ignored HDRI export passed 1/1 after the shadow-map scene bind group fallback was brought to the same 5-binding ABI. The accepted artifact is `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_hdr_pmrem_reflection_20260705.png`, 1280x960, 845069 bytes, SHA256 `64D5873A09DDC348C15A2444221DF07961A47A34D5FB281B73F7122FEAB0782E`, 74903 unique colors; same-name target/cargo-target scan returned zero hits.

## 2026-07-05 HDRI PBR Matrix Quantitative Guard

Plan 06 EC-M3a now adds whole-matrix quantitative checks to the real-HDRI PBR export. The test still writes the accepted PNG under `docs/tests/runtime/shader`, but after saving it also samples all 64 metallic/smoothness cells and asserts broad luma response, smooth metal versus dielectric separation, metallic and smoothness group deltas, row/column response coverage, and absence of legacy 16x8 sampled-equirect grid boundaries in the skybox region.

The same slice fixed the shared scene layout owner so `scene_bind_group_layout_entries()` is visible across the `scene_renderer` subtree while remaining private to that subsystem. This keeps renderer construction, prewarm validation, and focused pipeline tests on the same 5-binding scene layout: uniform, source cube, sampler, BRDF LUT, and specular PMREM cube.

Evidence: `CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-hdr-pmrem-export-it-0705 --message-format short --color never -- --ignored --exact export_runtime_shader_pbr_real_hdri_reflection_png --nocapture --test-threads=1` passed 1/1. The PNG remained 1280x960, 845069 bytes, SHA256 `64D5873A09DDC348C15A2444221DF07961A47A34D5FB281B73F7122FEAB0782E`, with 74903 unique colors; same-name target/cargo-target scan returned zero hits. The current matrix metrics were luma `124.501..233.256`, endpoint/group deltas `85.106` / `32.130` / `59.064`, responsive rows/columns `8/8` / `6/8`, and legacy grid vertical/horizontal means `0.02491` / `0.02711`. Full Plan 06 EC-M3 remains open for SSIM source-reference, strict high-frequency roughness monotonicity, cube seam quantization, and GPU/offline PMREM artifact validation.

## 2026-07-05 PMREM Mip Progressive Blur Contract

Plan 06 EC-M3b strengthens the source-cubemap PMREM contract directly. `runtime_environment_source_cubemap_contract.rs` now builds a 64-face high-frequency checker plus bright-spot HDR environment, measures luma variance across every PMREM mip, and asserts that rougher mips progressively reduce high-frequency energy. It also promotes the cmft final-mip discipline into a public contract by checking that the final 1x1 PMREM mip is averaged across all six faces.

Evidence: `rustfmt --edition 2021 zircon_runtime\tests\runtime_environment_source_cubemap_contract.rs` passed. The first focused Cargo run timed out at the tool boundary during Windows cold compile while cargo/rustc continued in the background; after those processes naturally exited, the same command passed: `cargo test -p zircon_runtime --test runtime_environment_source_cubemap_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never -- --nocapture --test-threads=1` ran 8/8 tests successfully, with a 3.56s test body and 7m53s total warmed run. This slice generated no screenshot; seam quantization, SSIM reference comparison, screenshot-level roughness monotonicity, and GPU/offline PMREM artifacts remain open.

## 2026-07-05 PMREM Cube Seam Quantization Guard

Plan 06 EC-M3c adds source PMREM cube-edge seam quantization. The focused contract test now derives adjacent faces through the public cubemap projection helpers, samples all four edges at mip0, an intermediate rough mip, and the rough PMREM mip, and asserts that mean and worst luma seam energy drop as the PMREM mip gets rougher. This keeps the seam test aligned with the actual face order and projection API instead of depending on a separate hand-written adjacency table.

Evidence: `rustfmt --edition 2021 zircon_runtime\tests\runtime_environment_source_cubemap_contract.rs` passed. `CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --test runtime_environment_source_cubemap_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-cubemap-projection-check-0705 --message-format short --color never -- --nocapture --test-threads=1` passed 9/9, with a 4.74s test body and 7m18s total run. This slice generated no screenshot. Source CPU PMREM seam behavior is now covered; GPU/offline PMREM artifact seam comparison, SSIM reference comparison, and screenshot-level roughness monotonicity remain open.

## 2026-07-05 Source Cubemap Cross-Face PMREM 2K HDRI

Plan 06 EC-M3d responds to the visible blockiness in the 1K lakes HDRI cubemap and the rough reflection mip chain. The source sky remains a full-resolution source cube, while the reflection path uses the separate specular PMREM cube. CPU mip generation now resolves bilinear taps that cross cube-face edges by projecting the out-of-face tap direction back onto the neighboring face instead of clamping on the current face. This follows the cmft/cmftStudio neighbor/edge-fixup discipline while keeping the existing UE GGX roughness-to-mip, pdf, and solid-angle source-mip formulas.

The PMREM bridge also raises high-roughness filtered importance sampling to 128 samples and treats repeated `roughness == 1.0` mips as a saturated tail: later saturated mips downsample from the previous PMREM level rather than independently resampling the source pyramid. This prevents high mip levels from reintroducing source-face blocks after the roughness mapping has already collapsed to the roughest lobe.

Evidence: `source_cubemap_linear_sampling_bleeds_across_face_edges` passed 1/1; `runtime_environment_source_cubemap_contract` passed 9/9 after the cross-face/high-roughness changes; and `export_runtime_shader_pbr_real_hdri_2k_reflection_png` passed 1/1. The source asset is `docs/tests/runtime/shader/assets/polyhaven_lakes_2k.hdr`, 5,918,432 bytes, SHA256 `B2506E0EE912C4C599FF013566FBD3ECAAC2F4B176319D450CCE0DE5758FED98`. The accepted PNG is `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_2k_hdr_pmrem_reflection_20260705.png`, 1280x960, 1,009,731 bytes, SHA256 `920A028DC6B0BB64A45F1798E89BF5E0FBE2BABF3A90BED22FFBA842DD1714F0`, 80,333 unique colors; same-name target and `E:\cargo-targets` scans returned zero hits. GPU/offline PMREM artifacts, derived cache, IEM, probe capture/blending, SSIM, RenderDoc/product capture, and 4K/16K offline bake acceptance remain open.

## 2026-07-06 Saved HDRI PNG Metrics Regression

Plan 06 EC-M3e adds a non-ignored regression for the accepted 2K HDRI PBR matrix screenshot. `runtime_shader_pbr_hdri_export.rs` now delegates reusable screenshot checks to `runtime_shader_pbr_hdri_export/hdri_metrics.rs`, which reads either a live `ViewportFrame` or a saved PNG byte buffer and runs the same sky variation, 64-cell metallic/smoothness matrix response, and legacy 16x8 grid-seam assertions. The new `runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics` test reads `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_2k_hdr_pmrem_reflection_20260705.png` directly, so the accepted artifact no longer depends only on the ignored manual export path for coverage.

Evidence: `rustfmt --edition 2021 zircon_runtime\tests\runtime_shader_pbr_hdri_export.rs zircon_runtime\tests\runtime_shader_pbr_hdri_export\hdri_metrics.rs` passed. `CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --test runtime_shader_pbr_hdri_export --locked --jobs 1 --target-dir E:\cargo-targets\zircon-hdri-png-metrics-0706 --message-format short --color never runtime_shader_pbr_real_hdri_2k_reflection_png_matches_plan06_metrics -- --exact --nocapture --test-threads=1` passed 1/1 with 0 ignored and 2 filtered tests; the test body was 0.23s after an 8m43s build, and stderr contained only existing workspace warnings. The PNG stayed in `docs/tests/runtime/shader`, 1,009,731 bytes, SHA256 `920A028DC6B0BB64A45F1798E89BF5E0FBE2BABF3A90BED22FFBA842DD1714F0`; same-name scans under `target` and `E:\cargo-targets` returned zero hits. This closes the saved-PNG regression gap, while strict high-frequency/roughness monotonicity, SSIM, RenderDoc/product capture, and GPU/offline PMREM artifacts remain open.

## 2026-07-10 Deferred HDR Emissive GBuffer ABI

Deferred material output now preserves authored emissive energy through an independent fourth GBuffer
target. `PostProcessGraphResourceNames::GBUFFER_EMISSIVE` names the graph resource, the fixed offscreen
target retains an `Rgba16Float` texture/view, and the built-in deferred geometry and lighting
descriptors declare the write/read edge. `GBUFFER_EMISSIVE_FORMAT` is the single WGPU format owner.

`ZrDeferredGBufferOutput` is a four-target ABI:

1. location 0: albedo
2. location 1: encoded world normal
3. location 2: metallic, roughness, occlusion, and shading flags
4. location 3: HDR emissive

The GBuffer mesh pipeline declares matching color targets, and `record_gbuffer_geometry(...)` records
all four attachments in the same render pass. Standard PBR writes non-negative `surface.emissive`.
Plugin shading-model GBuffer includes must also construct the fourth field; all repository-owned plugin
fixtures were hard-cut to this ABI. There is no three-target compatibility constructor or re-export.

Deferred lighting binds the emissive texture at group 1 binding 5. The final dispatch adds emissive
after built-in Unlit, Blinn-Phong, Standard PBR, or generated plugin shading-model evaluation. Keeping
the add at the common dispatch boundary prevents shading models from implementing divergent emissive
rules and matches Forward, where emissive is added after direct/environment lighting.

The change was driven by the Hybrid GI dynamic-light product RED: Forward warm emissive center RGB was
`48.66,25.07,15.71`, while Deferred was `14.71,14.20,14.16`. After the shared GBuffer fix, Forward and
Deferred both produce `48.66,25.07,15.71`; directional, point, and spot columns are also exact matches.
The real WGPU 2x4 product is
`docs/tests/runtime/render/plan18_hybrid_gi_dynamic_light_matrix_forward_deferred_wgpu_20260710.png`
with SHA-256 `1F4CC3565B9E3B7C3F8B46D7B6B792E12EAABDF49D11F0A16AFD8D537F3970F6`.

Plan sources: `.codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md`,
`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`,
`docs/plans/engine-code-structure-convention.md`, and
`docs/plans/engine-code-review-findings-2026-06.md`.

Validation: the target-client plugin `cargo check` passed; the no-debug plugin test binary passed six
wired HGI contracts and the ignored dynamic-light product 1/1. The product itself creates the four
target GBuffer pipeline and the lighting bind group on WGPU, so format/layout drift fails at device
validation. The first root runtime lib-test link exceeded 15 minutes under concurrent compiler memory
pressure, but a warmed no-debug build subsequently completed the current 7,439-test binary. Eight
focused Deferred/emissive tests passed from that binary: graph descriptor write/read, generated WGSL
emissive behavior and Naga validation, standard/plugin GBuffer source ABI, and built-in/custom WGPU
pipeline creation. The broad default Deferred graph-order test remains red only because its expected
Bloom/Exposure ordering is stale relative to concurrent post-process work; this slice does not claim
full runtime/full-workspace green.

## Runtime 15 / Plan 08 Anchor Mirrors

This section is an explicit cross-document anchor mirror for structure guards. Shadow WGPU device pipeline validation is tracked by `render_plan08_shadow_wgpu_device_pipeline_validation_implemented_validation_not_closed` and `shadow_mesh_pipeline_creates_on_wgpu_device_with_template_shader`. The current skinned geometry template source retains `zr_skinned_joint_matrix(v.joints.x)` while storage palette migration remains MS-M2 work.

Plan 08 shader prewarm mirrors current structure guard anchors: `Plugin shader permutation registry auto-export`, `render_plan08_plugin_shader_permutation_registry_auto_export_focused_tests_passed_renderdoc_deferred`, `Plugin shader permutation registry export contract`, `render_plan08_plugin_shader_permutation_registry_export_contract_python_passed_cargo_deferred`, `test_zircon_build_discovers_plugin_shader_permutation_records`, `test_validate_generated_registry_requires_selected_plugin_ids`, `runtime_15_shader_prewarm_plugin_permutation_registry_auto_export_is_wired`, `Build-tool staged WGPU handoff command contract`, `render_plan08_build_tool_staged_wgpu_handoff_command_contract_python_passed_cargo_deferred`, `test_full_staged_wgpu_handoff_keeps_generated_registries_and_roots`, `runtime_15_shader_prewarm_staged_wgpu_handoff_command_contract_is_wired`, `Plugin shading-model descriptor registration`, `render_plan08_plugin_shading_model_descriptor_registration_typecheck_python_passed_libtest_blocked_by_ui_input_error`, `test_zircon_build_discovers_plugin_shading_model_descriptors_as_shader_ids`, `Plugin shading-model descriptor registry export`, `render_plan08_plugin_shading_model_descriptor_registry_export_static_passed_cargo_deferred`, `shading_model_descriptors`, `test_generated_shader_permutation_registry_document_exports_selected_plugin_shading_model_descriptors`, `shader_prewarm_permutation_registry_merges_custom_shading_model_descriptors`, `runtime_15_shader_prewarm_plugin_shading_model_descriptor_registration_is_wired`, and `Project shader asset roots auto-export`.

2026-07-02 shader prewarm structure guard mirror: `Shader prewarm geometry-source enumeration`, `render_plan08_shader_prewarm_geometry_source_enumeration_static_passed_cargo_deferred_implementation_cadence`, `Asset-root custom geometry-source id prewarm`, `render_plan08_asset_root_custom_geometry_source_id_prewarm_typecheck_passed_test_timeout_no_result`, `Asset-root custom shading-model id prewarm`, `render_plan08_asset_root_custom_shading_model_id_prewarm_static_passed_cargo_deferred_implementation_cadence`, `bin/zircon_shader_prewarm/args.rs`, `bin/zircon_shader_prewarm/manifest.rs`, `tools/zircon_build.py`, `tools/zircon_build_shader_prewarm.py`, `shader_prewarm_asset_root_manifest_expands_requested_geometry_sources`, `shader_prewarm_asset_root_manifest_expands_custom_geometry_source_plugin_ids`, `shader_prewarm_asset_root_manifest_maps_custom_shading_model_plugin_ids`, `runtime_15_shader_prewarm_geometry_source_enumeration_is_wired`, `runtime_15_shader_prewarm_custom_geometry_source_id_is_wired`, `runtime_15_shader_prewarm_custom_shading_model_id_is_wired`, `Build-tool staged prewarm acceptance contract`, `render_plan08_build_tool_staged_prewarm_acceptance_contract_python_passed_cargo_deferred`, `Build-tool staged prewarm nonempty success report acceptance`, `render_plan08_build_tool_staged_prewarm_nonempty_success_report_python_passed_cargo_deferred`, `Build-tool staged prewarm written variant identity acceptance`, `render_plan08_build_tool_staged_prewarm_written_variant_identity_python_passed_cargo_deferred`, `Build-tool staged prewarm written source-label identity acceptance`, `render_plan08_build_tool_staged_prewarm_written_source_label_identity_python_passed_cargo_deferred`, `Build-tool staged prewarm complete written count acceptance`, `render_plan08_build_tool_staged_prewarm_complete_written_count_python_passed_cargo_deferred`, `Build-tool product Base pass acceptance contract`, `render_plan08_build_tool_product_base_pass_acceptance_contract_python_passed_cargo_deferred`, `Build-tool product material mesh pass acceptance contract`, `render_plan08_build_tool_product_material_mesh_pass_acceptance_contract_python_passed_cargo_deferred`, `Build-tool written variant uniqueness contract`, `render_plan08_build_tool_written_variant_uniqueness_contract_python_passed_cargo_deferred`, `Build-tool staged prewarm written cache-hash shape acceptance`, `render_plan08_build_tool_staged_prewarm_written_cache_hash_shape_python_passed_cargo_deferred`, `Build-tool source-label nonblank contract`, `render_plan08_build_tool_source_label_nonblank_contract_python_passed_cargo_deferred`, `Build-tool source-label trim contract`, `render_plan08_build_tool_source_label_trim_contract_python_passed_cargo_deferred`, `Build-tool explicit registry exact revision acceptance`, `render_plan08_build_tool_explicit_registry_exact_revision_acceptance_python_passed_cargo_deferred`, `expected_pass_types`, `taa_reactive_mask`, `test_acceptance_contract_validates_report_cache_and_exported_registry`, `test_acceptance_contract_validates_explicit_registry_against_report`, `test_acceptance_contract_rejects_explicit_registry_without_ready_revision`, `test_acceptance_contract_rejects_forward_only_staged_pass_report`, `test_acceptance_contract_rejects_duplicate_written_variant_identity`, `test_validate_cache_artifact_contract_requires_requested_pass_types`, `test_validate_cache_artifact_contract_accepts_requested_pass_types`, `test_validate_cache_artifact_contract_requires_requested_quality_tiers`, `test_validate_cache_artifact_contract_requires_requested_geometry_sources`, `test_validate_cache_artifact_contract_accepts_requested_quality_and_geometry`, `tools/zircon_build_shader_prewarm_written_variants.py`, `duplicate written cache variant identity`, `runtime fallback root`, `usable shader ResourceRecord revisions`, `Build-tool shader prewarm cache artifact contract`, `Prewarm report cache identity contract`, `Prewarm cache runtime layout contract`, `Prewarm cache hash shape contract`, `Prewarm cache custom id correlation contract`, `Runtime prewarm custom id cache lookup contract`, `Runtime custom id staged fallback lookup contract`, `render_plan08_runtime_custom_id_staged_fallback_lookup_static_passed_cargo_deferred`, `Build-tool cache quality/geometry identity contract`, `render_plan08_build_tool_cache_quality_geometry_identity_contract_python_passed_cargo_deferred`, `Build-tool cache dimension combination contract`, `render_plan08_build_tool_cache_dimension_combination_contract_python_passed_cargo_deferred`, `Build-tool cache custom id combination contract`, `render_plan08_build_tool_cache_custom_id_combination_contract_python_passed_cargo_deferred`, `Build-tool cache source-label provenance correlation contract`, `render_plan08_build_tool_cache_source_label_provenance_contract_python_passed_cargo_deferred`, `Build-tool cache metadata field type contract`, `render_plan08_build_tool_cache_metadata_field_type_contract_python_passed_cargo_deferred`, `test_validate_report_contract_rejects_untrimmed_source_provenance_strings`, `runtime_15_shader_prewarm_acceptance_contract_is_wired`, `runtime_15_shader_prewarm_cache_artifact_contract_is_wired`, `Asset-root resource registry revision overlay`, `render_plan08_asset_root_resource_registry_revision_overlay_typecheck_passed_test_timeout_no_result`, `render_plan08_resource_registry_ready_shader_revision_contract_python_static_passed_cargo_deferred`, `bin/zircon_shader_prewarm/manifest/resource_registry.rs`, `shader_prewarm_asset_root_manifest_uses_resource_registry_revision_overlay`, `shader_prewarm_resource_registry_overlay_uses_ready_shader_revisions_only`, `runtime_15_shader_prewarm_resource_registry_revision_overlay_is_wired`, `Prewarm WGPU validation report summary`, `render_plan08_prewarm_wgpu_validation_report_summary_python_passed_cargo_deferred`, `Build-tool WGPU validation report contract`, `render_plan08_build_tool_wgpu_report_contract_python_passed_cargo_deferred`, `Build-tool WGPU validation totals match contract`, `render_plan08_build_tool_wgpu_validation_totals_match_python_passed_cargo_deferred`, `test_zircon_build_shader_prewarm_wgpu_report_contract.py`, `test_dimension_summary_lines_format_wgpu_module_validation_counts`, `test_validate_report_contract_requires_wgpu_validation_when_requested`, `test_validate_report_contract_rejects_wgpu_validation_total_mismatch`, `runtime_15_shader_prewarm_wgpu_validation_report_summary_is_wired`, and `runtime_15_shader_prewarm_wgpu_report_contract_is_wired`.
