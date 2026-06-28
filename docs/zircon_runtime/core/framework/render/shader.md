---
related_code:
  - dev/bevy/crates/bevy_shader/src/lib.rs
  - dev/bevy/crates/bevy_shader/src/shader.rs
  - dev/bevy/crates/bevy_shader/src/shader_cache.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_render/src/render_resource/bind_group_layout.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/shader/mod.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/geometry_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/queue_profile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/graphics/shader/template/mod.rs
  - zircon_runtime/src/graphics/shader/template/assemble.rs
  - zircon_runtime/src/graphics/shader/template/include_registry.rs
  - zircon_runtime/src/graphics/shader/template/material_surface.rs
  - zircon_runtime/src/graphics/shader/template/pass_specialization.rs
  - zircon_runtime/src/graphics/shader/template/taa_reactive_mask.rs
  - zircon_runtime/src/graphics/shader/template/validation.rs
  - zircon_runtime/src/graphics/shader/template/tests.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_surface_types.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_scene_runtime.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_static.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_forward.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_gbuffer.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_depth.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_depth_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_taa_reactive_mask.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/test_support.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/staged_prewarm.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly.rs
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/paths.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/permutation_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/revision.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - zircon_runtime/src/core/resource/manager/registry_export.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_shader_permutation_manifest.rs
  - zircon_plugins/virtual_geometry/runtime/src/plugin.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/geometry_source.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_live_resource_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_permutation_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_permutation_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_permutation_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_geometry_source_descriptor.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_revision.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/depth_prepass_pure_depth_product_migration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_module_validation.rs
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
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_mesh_pass_processors.rs
  - tools/zircon_build.py
  - tools/zircon_build_shader_prewarm.py
  - tools/zircon_build_shader_resource_registry.py
  - tools/zircon_build_shader_prewarm_report_contract.py
  - tools/zircon_build_shader_prewarm_cache_artifacts.py
  - tools/zircon_build_shader_prewarm_acceptance.py
  - tools/zircon_build_shader_prewarm_written_variants.py
  - tools/tests/test_zircon_build_shader_prewarm.py
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
  - zircon_runtime/src/core/framework/render/shader/mod.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/geometry_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/graphics/shader/template/mod.rs
  - zircon_runtime/src/graphics/shader/template/assemble.rs
  - zircon_runtime/src/graphics/shader/template/include_registry.rs
  - zircon_runtime/src/graphics/shader/template/material_surface.rs
  - zircon_runtime/src/graphics/shader/template/pass_specialization.rs
  - zircon_runtime/src/graphics/shader/template/taa_reactive_mask.rs
  - zircon_runtime/src/graphics/shader/template/validation.rs
  - zircon_runtime/src/graphics/shader/template/tests.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_surface_types.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_scene_runtime.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_static.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_forward.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_gbuffer.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_depth.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_depth_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_taa_reactive_mask.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/test_support.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly.rs
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/paths.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/permutation_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/revision.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - zircon_runtime/src/core/resource/manager/registry_export.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_shader_permutation_manifest.rs
  - zircon_plugins/virtual_geometry/runtime/src/plugin.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest/geometry_source.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_live_resource_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_permutation_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_permutation_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_asset_roots_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/depth_prepass_pure_depth_product_migration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_module_validation.rs
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
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_acceptance_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_mesh_pass_processors.rs
  - tools/zircon_build.py
  - tools/zircon_build_shader_prewarm.py
  - tools/zircon_build_shader_prewarm_report_contract.py
  - tools/zircon_build_shader_prewarm_cache_artifacts.py
  - tools/zircon_build_shader_prewarm_acceptance.py
  - tools/zircon_build_shader_prewarm_written_variants.py
  - tools/tests/test_zircon_build_shader_prewarm.py
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
tests:
  - rustfmt --edition 2021 zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/depth_prepass_pure_depth_product_migration.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-27 Runtime pure-depth DepthPrepass product migration: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-depth-prepass-pure-depth-check --message-format short --color never (2026-06-27 Runtime pure-depth DepthPrepass product migration: passed with existing warnings)
  - cargo test -p zircon_runtime --lib depth_prepass_mesh_pipeline_creates_on_wgpu_device_with_template_shader --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-depth-prepass-pure-depth-check --message-format short --color never -- --test-threads=1 --nocapture (2026-06-27 Runtime pure-depth DepthPrepass product migration: timed out after 15 minutes in Windows lib-test link, no test result)
  - rustfmt --edition 2021 zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs zircon_runtime/src/graphics/shader/variant_cache/mod.rs zircon_runtime/src/graphics/shader/mod.rs zircon_runtime/src/dynamic_api/shader_prewarm.rs zircon_runtime/src/dynamic_api/mod.rs zircon_runtime/src/bin/zircon_shader_prewarm/args.rs zircon_runtime/src/bin/zircon_shader_prewarm/run.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_module_validation.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-27 Prewarm opt-in WGPU shader-module validation: passed)
  - python -m py_compile tools/zircon_build.py tools/zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm.py (2026-06-27 Prewarm opt-in WGPU shader-module validation: passed)
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm (2026-06-27 Prewarm opt-in WGPU shader-module validation: passed, 13 tests)
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-wgpu-module-prewarm-check --message-format short --color never (2026-06-27 Prewarm opt-in WGPU shader-module validation: passed with existing warnings)
  - cargo check -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-wgpu-module-prewarm-check --message-format short --color never (2026-06-27 Prewarm opt-in WGPU shader-module validation: passed with existing warnings)
  - cargo run -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --locked --jobs 1 --target-dir target/codex-plan08-wgpu-module-prewarm-run-build --message-format short --color never -- --project-root . --cache-dir target/codex-plan08-wgpu-module-prewarm-run/cache --report target/codex-plan08-wgpu-module-prewarm-run/report.json --builtin-fallback --validate-wgpu-modules --pretty (2026-06-27 Prewarm opt-in WGPU shader-module validation: timed out after 604s in Windows compile/run setup, no report, not counted as passed)
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
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_skips_export_contract_for_explicit_registry tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_requires_resource_records tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_accepts_wrapped_resources tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_validate_registry_export_contract_accepts_raw_array (2026-06-28 Build-tool shader resource registry export contract: RED then passed, 5 tests)
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
  - python -m unittest tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_validates_wgpu_report_after_success tools.tests.test_zircon_build_shader_prewarm.ZirconBuildShaderPrewarmTests.test_prewarm_shaders_skips_export_contract_for_explicit_registry tools.tests.test_zircon_build_shader_prewarm_cache_contract (2026-06-28 Build-tool shader prewarm cache artifact contract: RED then passed, 5 tests)
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
  - rustfmt --edition 2021 zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs zircon_runtime/src/core/framework/render/shader/mod.rs zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/core/runtime/diagnostics/render_stats_store/shader_variant.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/queue_profile.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs zircon_runtime/src/graphics/tests/render_product_mesh_cache/staged_prewarm.rs (2026-06-27 runtime shader variant dimension correlation: passed)
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
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs::shader_prewarm_asset_root_manifest_uses_zmeta_source_hash_revision
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

`RenderShaderEntryPointDescriptor` records the public entry-point name plus its `RenderShaderStage`. Asset-side parsing accepts authoring aliases such as `vert`, `vs`, `frag`, `fs`, `comp`, and `cs`, but the framework contract only exposes canonical stage values.

`RenderShaderDependency` records a `ResourceKind` and `AssetReference`. Dependencies are explicit serialized authoring data in the current milestone; they are not inferred from WGSL import syntax by the framework layer.

`RenderShaderDefinitionValue` records Bevy-style shader definition inputs as bool, signed integer, or unsigned integer values. `From<&str>` and `From<String>` create bool-true flag definitions so legacy authoring paths and small tests can stay concise while the runtime contract is no longer string-only.

`RenderShaderVariantKey` records an optional entry point, optional stage, and typed definition list. It is a neutral key for material or pipeline specialization diagnostics and single-module compile requests, not the full material pipeline-cache key.

`GeometrySourceId` is the geometry-source dimension for the material shader variant space. Built-in ids are reserved as `0 = StaticMesh`, `1 = SkinnedMesh`, `2 = MorphedMesh`, and `3 = SkinnedMorphed`; plugin geometry sources start at `GEOMETRY_SOURCE_PLUGIN_ID_START`. This keeps VertexFactory-style geometry source selection in the framework contract without pulling WGPU vertex-buffer declarations into the neutral layer.

The 2026-06-24 GeometrySource descriptor contract foundation extends that owner beyond ids. `GeometrySourceDescriptor` now records the stable token, WGSL include token, vertex attributes, backend-neutral required bindings, and typed shader defines for each geometry source. Built-in descriptor helpers cover static, skinned, morphed, and skinned+morphed meshes; all require the GPUScene instance binding, while skinned descriptors add skinning palette storage and morphed descriptors add morph weight/target storage. This contract intentionally stops at serializable framework data: no `wgpu` types, no pipeline descriptors, and no concrete bind group creation live in this module. The guard `runtime_15_render_shader_geometry_source_descriptor_contract_is_complete` locks the shape under status `render_plan08_geometry_source_descriptor_contract_static_passed_cargo_deferred_implementation_cadence`.

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

`ShaderVariantPrewarmManifest`, `ShaderVariantPrewarmRequest`, and `ShaderVariantPrewarmReport` are the neutral offline-cache DTOs. The manifest version-gates a list of requests; each request carries the final `ShaderVariantKey`, WGSL source, include/source hashes, and template/compiler version strings. The report records requested, written, and failed counts, per-variant failures, and a `dimension_summary` grouped by pass type, geometry source id, shading model id, and quality tier. Geometry and shading dimensions use stable numeric id strings so report readers do not need renderer-private registries to diagnose gaps. These DTOs let build tooling and headless runtime code populate `graphics::shader::variant_cache` without depending on WGPU objects. The `zircon_shader_prewarm` tool can read an authored manifest, emit the built-in fallback manifest, or scan asset roots for `.zmeta` compound shader packages, `.zshader` files, standalone `.wgsl` files, and `.zmaterial` material instances. Automatically generated built-in and asset-root requests can be expanded with repeated `--quality-tier low|medium|high|ultra` or `--quality-tier all`; no explicit tier still defaults to Medium so existing staging size stays stable. Authored manifest files keep their serialized quality keys unchanged.

The 2026-06-24 Shader prewarm WGSL validation gate keeps the DTO unchanged but changes the graphics-side write behavior: `graphics/shader/variant_cache/prewarm.rs` now calls `validate_shader_variant_prewarm_wgsl(...)` before writing a request to `ShaderVariantCacheDisk`. Invalid WGSL increments `ShaderVariantPrewarmReport.failed_count`, stores the variant index and error, and leaves the disk cache at miss. The tool-level invalid manifest probe returned exit 2 with requested=1, written=0, failed=1, and cache_files=0, matching the report path. This uses the existing Naga validation owner under `graphics/shader/template/validation.rs`; it is not WGPU shader-module or render-pipeline creation evidence. Status: `render_plan08_shader_prewarm_wgsl_validation_check_passed_test_compile_timeout_no_result`.

The 2026-06-27 prewarm dimension diagnostics slice makes the staged-cache report show which variant dimensions actually wrote or failed. `graphics/shader/variant_cache/prewarm.rs` now records successful writes and WGSL/write failures through the variant-aware report methods, while schema-level failures can still remain top-level failures without a dimension key. `ShaderVariantPrewarmReport.dimension_summary` is serde-defaulted for older report JSON and exposes `pass_types`, `geometry_source_ids`, `shading_model_ids`, and `quality_tiers`, each with requested/written/failed counters. The focused regression is `render_shader_variant_prewarm_report_groups_written_and_failed_dimensions`; current status is `render_plan08_prewarm_dimension_diagnostics_typecheck_passed_test_timeout_no_result`.

The 2026-06-24 Shader prewarm geometry-source enumeration slice adds the geometry dimension to generated prewarm manifests. Repeated `--geometry-source static|skinned|morphed|skinned-morphed|all` expands asset-root requests across built-in `GeometrySourceId` values while preserving static as the default. `asset_root_manifest_for_quality_tiers_and_geometry_sources(...)` now forms the pass x quality x geometry-source product before writing `ShaderVariantKey.geometry_source`, and the older `asset_root_manifest_for_quality_tiers(...)` remains a static-default compatibility wrapper. `tools/zircon_build.py --prewarm-shaders` forwards quality tiers through `--shader-quality-tier` and geometry sources through `--shader-geometry-source`, which the staged cache command maps back to repeated `--geometry-source` arguments. The guard `runtime_15_shader_prewarm_geometry_source_enumeration_is_wired` locks the CLI parser, run forwarding, manifest product expansion, build-script forwarding, docs/status anchors, and the focused test `shader_prewarm_asset_root_manifest_expands_requested_geometry_sources` under status `render_plan08_shader_prewarm_geometry_source_enumeration_static_passed_cargo_deferred_implementation_cadence`.

Asset-root builtin standard material template prewarm is the follow-up that connects `.zmaterial` references to the runtime template source without changing custom shader scan semantics. When a material shader locator is exactly `builtin://shader/pbr.wgsl`, `zircon_shader_prewarm` calls `dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry(...)` for each requested built-in `GeometrySourceId` instead of looking for an asset-root shader file. The old `dynamic_api::builtin_standard_material_shader_prewarm_manifest(...)` remains the static wrapper. Both builders project `ShaderFeatureBits`, `ShadingModelId`, alpha cutoff, and quality tiers into a `PipelineKey`, then reuse `mesh_pipeline_standard_material_template_source(...)` for static or `mesh_pipeline_standard_material_template_source_for_geometry(...)` for explicit geometry so the request carries matching WGSL, include hashes, source hash, template revision, and `ShaderVariantKey.geometry_source`. Custom `.zshader` and standalone `.wgsl` files remain raw scanned WGSL requests. `shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source` now checks static and skinned builtin template requests, alpha cutoff constant, `ShaderFeatureBits::RECEIVE_SHADOWS`, and geometry-specific includes; `runtime_15_shader_prewarm_builtin_standard_material_template_source_is_wired` locks the dynamic API export, scene facade, manifest expansion, docs/status anchors, and line budgets. Status: `render_plan08_asset_root_builtin_standard_material_template_prewarm_static_passed_cargo_deferred_implementation_cadence`; multi-geometry follow-up status: `render_plan08_asset_root_builtin_standard_material_multi_geometry_prewarm_static_passed_cargo_deferred_implementation_cadence`.

Asset-root custom shading-model id prewarm adds an explicit project/plugin id map without reintroducing a dead plugin registry surface. `zircon_shader_prewarm --shading-model-id custom:subsurface=16` and `tools/zircon_build.py --shader-shading-model-id custom:subsurface=16` normalize the custom token, reject ids below `SHADING_MODEL_PLUGIN_ID_START`, and forward the map into `asset_root_manifest_for_quality_tiers_geometry_sources_and_shading_model_ids(...)`. The staging command assembly lives in `tools/zircon_build_shader_prewarm.py` so `tools/zircon_build.py` remains the build orchestrator instead of accumulating more shader-prewarm detail. A `.zmaterial` with `lighting_model = "custom:subsurface"` then writes `ShaderVariantKey.shading_model = ShadingModelId::new(16)` for builtin standard-material template requests and raw scanned material requests. Unknown custom models still fall back to StandardPBR, keeping the staged cache conservative until a real project/插件 registry exporter exists. The focused guards are `shader_prewarm_args_parse_custom_shading_model_plugin_ids`, `shader_prewarm_asset_root_manifest_maps_custom_shading_model_plugin_ids`, and `runtime_15_shader_prewarm_custom_shading_model_id_is_wired`. Status: `render_plan08_asset_root_custom_shading_model_id_prewarm_static_passed_cargo_deferred_implementation_cadence`.

Asset-root custom geometry-source id prewarm adds the equivalent explicit id path for geometry dimensions. `zircon_shader_prewarm --geometry-source-id custom:gpu-driven=4` and `tools/zircon_build.py --shader-geometry-source-id custom:gpu-driven=4` normalize the custom token, reject ids below `GEOMETRY_SOURCE_PLUGIN_ID_START`, and append the resulting `GeometrySourceId` to the manifest geometry-source list. The manifest path already accepts arbitrary `GeometrySourceId` values, so no renderer-private WGPU descriptors or plugin registry surface are introduced for this slice. A raw asset-root shader can therefore write five pass-specific `ShaderVariantKey.geometry_source = GeometrySourceId::new(4)` requests under the explicit CLI/build-tool input. The focused guards are `shader_prewarm_args_parse_custom_geometry_source_plugin_ids`, `shader_prewarm_asset_root_manifest_expands_custom_geometry_source_plugin_ids`, and `runtime_15_shader_prewarm_custom_geometry_source_id_is_wired`. Status: `render_plan08_asset_root_custom_geometry_source_id_prewarm_typecheck_passed_test_timeout_no_result`.

Asset-root shader edit revision export closes the staged-cache edit-key gap without pretending the offline scan owns the live `ResourceManager` counter. `manifest/revision.rs` turns `.zmeta.source_hash` into a stable non-zero `ShaderVariantKey.material_revision`; raw `.wgsl` and `.zshader` sources without `.zmeta` derive the revision from their include/source content hash list. `shader_source_from_zmeta(...)` and fallback `shader_prewarm_source(...)` now consume those values, so changing a shader package source hash or raw WGSL content produces a new prewarm key instead of reusing revision `1`. `shader_prewarm_asset_root_manifest_uses_zmeta_source_hash_revision` and `shader_prewarm_asset_root_manifest_uses_raw_source_hash_revision` lock the edit boundary, and `runtime_15_shader_prewarm_asset_revision_export_is_wired` locks the module split and docs/status anchors. Status: `render_plan08_asset_root_shader_edit_revision_export_passed_cargo_renderdoc_deferred`.

Asset-root resource registry revision overlay is the explicit handoff point for projects that can already export live `ResourceRecord` data, narrowing the older live project registry exact revision overlay gap to a caller-supplied JSON input. `zircon_shader_prewarm --resource-registry <records.json>` and `tools/zircon_build.py --shader-resource-registry <records.json>` accept a `ResourceRecord` array or a JSON object containing `resources`/`records`; `manifest/resource_registry.rs` filters to shader records with non-zero revisions and indexes them by `ResourceId`, primary locator, and artifact locator. During `.zmeta` shader scanning, `asset_root_manifest_with_resource_registry_revisions(...)` uses the matching `ResourceRecord.revision` for `ShaderVariantKey.material_revision`; unmatched `.zmeta` sources still use `source_hash`, and raw sources without `.zmeta` still use content-hash revision. The focused guards are `shader_prewarm_args_parse_resource_registry_path`, `shader_prewarm_asset_root_manifest_uses_resource_registry_revision_overlay`, and `runtime_15_shader_prewarm_resource_registry_revision_overlay_is_wired`. Status: `render_plan08_asset_root_resource_registry_revision_overlay_typecheck_passed_test_timeout_no_result`.

Staged shader resource registry auto-export closes the build-tool default path between explicit revision overlay and asset-root manifest generation. When `tools/zircon_build.py --prewarm-shaders` has no explicit `--shader-resource-registry`, `tools/zircon_build_shader_prewarm.py` now forwards `--export-resource-registry ZirconEngine/cache/shader_resource_records.json`; explicit registries still use `--resource-registry`. `bin/zircon_shader_prewarm/manifest/resource_registry.rs::shader_resource_records_from_asset_root(...)` reads staged `.zmeta` files, exports shader-only ready `ResourceRecord` rows with source-hash-derived staged revisions, and `run.rs` feeds those rows into `ShaderPrewarmResourceRegistryOverlay::from_records(...)` for the same scan. The manifest scan skips a raw `.wgsl`/`.zshader` file when a sidecar `.zmeta` owns that single-file shader, so generated registry revisions are not mixed with duplicate content-hash fallback variants. `shader_prewarm_asset_root_exports_shader_resource_records` and `runtime_15_shader_prewarm_registry_auto_export_is_wired` guard the handoff. Status: `render_plan08_shader_resource_registry_auto_export_focused_tests_passed_renderdoc_deferred`.

Staged shader resource registry multi-root dedupe keeps that auto-export deterministic when engine and selected plugin asset roots contain the same shader metadata. `shader_resource_records_from_asset_roots(...)` gathers all requested roots and calls `deduplicate_shader_resource_records(...)` before `run.rs` writes `shader_resource_records.json`; exact duplicate `ResourceRecord` rows collapse to one entry, while conflicting id-to-locator or locator-to-id mappings fail the prewarm command instead of creating a last-writer overlay. Status: `render_plan08_shader_resource_registry_multi_root_dedupe_static_passed_cargo_deferred`; `shader_resource_records_from_asset_roots_deduplicates_duplicate_shader_records` and `runtime_15_shader_prewarm_resource_registry_multi_root_dedupe_is_wired` lock the code owner, docs anchors, and 800-line guard. This is not counted as real WGPU runtime execution, RenderDoc capture, full live registry export, or miss=0 product acceptance.

Build-tool shader asset-root plan visibility makes the same root set visible in staged build dry-run output. `print_shader_prewarm_plan(...)` now prints `shader asset roots: ...` through `shader_asset_root_paths_for_prewarm(config)`, so the plan output and `build_shader_prewarm_command(...)` share the engine/plugin root owner before `--export-resource-registry` runs. The same plan visibility owner now also prints `shader prewarm cache root`, `shader prewarm report`, and `shader runtime fallback root`, so dry-run output exposes the staged cache/report handoff path before the acceptance helper reads a real report. Status: `render_plan08_build_tool_shader_asset_root_plan_visibility_python_passed_cargo_deferred`; `test_prewarm_plan_lists_asset_roots_for_registry_export`, `test_prewarm_plan_lists_runtime_fallback_handoff_paths`, `test_build_command_auto_export_registry_scans_all_asset_roots`, and `runtime_15_shader_prewarm_asset_root_plan_visibility_is_wired` lock the plan text, command handoff, docs anchors, and line budget. Closeout verification passed build-helper Python combo 60/60 plus py_compile, rustfmt, anchors, whitespace/conflict, line-budget, and scoped diff-check. This is not counted as real WGPU runtime execution, RenderDoc capture, full live registry export, or miss=0 product acceptance.

Shader permutation registry overlay is the next explicit project/plugin input layer for staged prewarm. `zircon_shader_prewarm --shader-permutation-registry <registry.json>` merges a registry file, and `shader_permutation_registry_paths` also discovers `shader_permutation_registry.json` below each asset root. The registry document's `geometry_source_ids` and `shading_model_ids` records are normalized to custom tokens, range-checked against plugin id starts, and merged before asset-root manifests expand, so external project/plugin ids can populate `ShaderVariantKey.geometry_source` and `ShaderVariantKey.shading_model` without hand-written CLI ids for every build. `tools/zircon_build.py --shader-permutation-registry <path>` forwards explicit registries while staged asset roots keep the sidecar discovery path. The focused guards are `shader_prewarm_permutation_registry_merges_custom_geometry_and_shading_ids`, `shader_prewarm_permutation_registry_discovers_asset_root_registry`, and `runtime_15_shader_prewarm_permutation_registry_overlay_is_wired`. Status: `render_plan08_shader_permutation_registry_overlay_focused_tests_passed_renderdoc_deferred`.

The staged auto-export is intentionally narrower than live project registry export: it records asset-root `.zmeta` shader rows for build-time prewarm, not the running `ResourceManager` revision counter or custom plugin registry ids.

The scan path mirrors the shader package importer by reading `.zshader` `wgsl_files` in order and combining those files into the runtime WGSL payload before writing disk-cache entries. `.zshader` entry-point stages drive StandardPBR pass expansion for each selected geometry source: vertex+fragment sources emit Forward, GBuffer, DepthPrepass, Shadow, and Velocity; vertex-only sources emit DepthPrepass, Shadow, and Velocity; fragment-only sources emit Forward and GBuffer; compute-only sources do not enter the material-variant prewarm space. Standalone `.wgsl` sources default to the full material pass set because they do not carry serialized stage metadata. Scanned shader requests now use source-hash-derived material revisions for edit invalidation while source/include hashes remain part of the disk-cache key payload for stale-entry validation. `.zmaterial` files are parsed through `MaterialAsset`, joined back to scanned shader sources by shader `AssetReference` URL or resource id, and expanded into deduplicated material-dimension variants. The feature mapping matches runtime `PipelineKey`: `AlphaMode::Mask` sets `ShaderFeatureBits::ALPHA_TEST`, `double_sided = true` sets `ShaderFeatureBits::DOUBLE_SIDED`, and runtime `PipelineKey.receive_shadows` now sets `ShaderFeatureBits::RECEIVE_SHADOWS`. Built-in material lighting models also enter the prewarm key through `ShadingModelId::from_lighting_model`: PBR maps to StandardPBR, BlinnPhong maps to BlinnPhong, and Unlit maps to Unlit. `AlphaMode::Blend` material-instance requests are filtered to the Forward pass so transparent materials align with the current runtime transparent queue instead of prewarming unused G-buffer, depth, shadow, or velocity variants for that material instance. Custom lighting models can be mapped through explicit `--shading-model-id` / `--shader-shading-model-id` plugin ids; unknown custom models continue to fall back to StandardPBR until a project shading-model registry exporter can provide those ids automatically.

## Runtime 15 M3 shader prewarm manifest test folder split

状态：`runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred`。

Runtime 15 R4.1/M3 的结构切片先移动 shader prewarm manifest 的测试 owner，后续 Plan 08 geometry-source 枚举、builtin standard material template prewarm、custom shading-model id prewarm、asset-root shader edit revision export 与 resource registry revision overlay 切片继续在同一 folder-backed owner 内扩展覆盖。`bin/zircon_shader_prewarm/manifest.rs` 当前 705 行，父文件只保留生产扫描/manifest 逻辑和 `#[cfg(test)] mod tests;` 挂载；`bin/zircon_shader_prewarm/manifest/revision.rs` 承接 source-hash/content-hash revision projection，`bin/zircon_shader_prewarm/manifest/paths.rs` 承接路径扫描 helper，`bin/zircon_shader_prewarm/manifest/resource_registry.rs` 承接 exported `ResourceRecord` revision overlay；原内联测试迁入 `bin/zircon_shader_prewarm/manifest/tests.rs`，测试子文件当前 563 行。

子文件保留 `shader_prewarm_asset_root_manifest_reads_compound_zshader_package`，继续覆盖 compound `.zshader`、`.zmaterial` feature bits、BlinnPhong/Unlit shading model 映射、material revision 与 alpha-blend Forward-only pass filtering；新增 `shader_prewarm_asset_root_manifest_expands_requested_geometry_sources`，覆盖 asset-root manifest 按 static+skinned geometry source 展开 10 个 pass x geometry 请求；`shader_prewarm_asset_root_manifest_expands_custom_geometry_source_plugin_ids` 覆盖显式 plugin-range geometry-source id 展开；`shader_prewarm_builtin_fallback_manifest_expands_requested_geometry_sources` 覆盖纯 builtin fallback 的多 geometry 展开；`shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source` 覆盖 builtin standard material `.zmaterial` 生成 static+skinned Forward template source；`shader_prewarm_asset_root_manifest_maps_custom_shading_model_plugin_ids` 覆盖 custom lighting-model token 到 plugin-range shading id 的显式 map；`shader_prewarm_asset_root_manifest_uses_zmeta_source_hash_revision` 和 `shader_prewarm_asset_root_manifest_uses_raw_source_hash_revision` 覆盖 source_hash/raw content 编辑后 revision 变化。`structure_convention/test_file_budget/shader_prewarm_manifest.rs::runtime_15_shader_prewarm_manifest_tests_are_folder_backed` 现在锁定 7 个 manifest 测试保留与 revision child owner；`runtime_15_shader_prewarm_geometry_source_enumeration_is_wired` 额外锁定 `--geometry-source` CLI、manifest geometry-source product expansion、`tools/zircon_build.py --shader-geometry-source` 转发和 docs/status anchors；`runtime_15_shader_prewarm_custom_geometry_source_id_is_wired` 锁定 `--geometry-source-id`、`tools/zircon_build.py --shader-geometry-source-id`、manifest plugin-range geometry dimension 和状态锚点 `render_plan08_asset_root_custom_geometry_source_id_prewarm_typecheck_passed_test_timeout_no_result`；`runtime_15_shader_prewarm_builtin_standard_material_template_source_is_wired` 锁定 `dynamic_api::builtin_standard_material_shader_prewarm_manifest(...)`、`dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry(...)`、builtin URI routing、`ShaderFeatureBits::RECEIVE_SHADOWS` 和 statuses `render_plan08_asset_root_builtin_standard_material_template_prewarm_static_passed_cargo_deferred_implementation_cadence` / `render_plan08_asset_root_builtin_standard_material_multi_geometry_prewarm_static_passed_cargo_deferred_implementation_cadence`；`runtime_15_shader_prewarm_custom_shading_model_id_is_wired` 锁定 `--shading-model-id`、`tools/zircon_build.py --shader-shading-model-id`、manifest explicit id map 和状态锚点 `render_plan08_asset_root_custom_shading_model_id_prewarm_static_passed_cargo_deferred_implementation_cadence`；`runtime_15_shader_prewarm_asset_revision_export_is_wired` 锁定 revision child owner、测试和状态锚点 `render_plan08_asset_root_shader_edit_revision_export_passed_cargo_renderdoc_deferred`。

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

Builtin fallback prewarm template source alignment makes the staged built-in fallback cache use that same source DTO. `dynamic_api/shader_prewarm.rs::builtin_fallback_shader_prewarm_manifest()` now calls `mesh_pipeline_standard_material_template_source(...)` through the crate-visible scene facade and writes the returned template WGSL, include/source hashes, and `zr-material-template-v1` revision into `ShaderVariantPrewarmRequest`. It no longer imports or writes `FALLBACK_MESH_SHADER`; if the controlled template assembly fails, the built-in manifest is empty rather than producing a stale wrong cache entry. `bin/zircon_shader_prewarm/manifest.rs::builtin_fallback_manifest_for_quality_tiers_and_geometry_sources(...)` extends the pure `--builtin-fallback` path across requested built-in geometry sources by reusing `dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry(...)`, while the older quality-only wrapper remains static by default. The focused tests are `builtin_fallback_shader_prewarm_manifest_uses_mesh_template_source`, `shader_prewarm_builtin_fallback_manifest_expands_requested_geometry_sources`, and `runtime_15_builtin_fallback_prewarm_uses_template_source`; statuses: `render_plan08_builtin_fallback_prewarm_template_source_static_passed_cargo_deferred_implementation_cadence` and `render_plan08_builtin_fallback_multi_geometry_prewarm_static_passed_cargo_no_result`.

Asset-root builtin standard material template prewarm extends that DTO path to material-authored builtin references and now emits five pass templates for each requested quality/geometry pair. `dynamic_api::builtin_standard_material_shader_prewarm_manifest(...)` keeps the static default, while `dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry(...)` takes the material-derived feature bits, shading model id, optional alpha cutoff, explicit `GeometrySourceId`, and quality tier list. Both route through `mesh_pipeline_standard_material_template_source_for_shader_pass(...)` so Forward, GBuffer, DepthPrepass, Shadow, and Velocity prewarm entries use the same mesh source owner and source-hash inputs as runtime template consumers. DepthPrepass prewarm is intentionally pure depth-only: opaque variants select `zr_template_depth.wgsl` without material fragment code, and alpha-test variants select `zr_template_depth_alpha.wgsl` with alpha clip but no normal-target encode. `bin/zircon_shader_prewarm/manifest.rs` uses the explicit builder for each requested geometry source when `.zmaterial` references `builtin://shader/pbr.wgsl`; raw asset-root `.zshader` and standalone `.wgsl` sources continue through the scanned-source path. The focused tests are `builtin_standard_material_shader_prewarm_manifest_projects_material_features`, `builtin_standard_material_shader_prewarm_manifest_projects_geometry_source`, `shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source`, `mesh_pipeline_standard_material_shader_pass_source_keeps_depth_only_contract`, and `runtime_15_shader_prewarm_builtin_standard_material_template_source_is_wired`; statuses: `render_plan08_asset_root_builtin_standard_material_template_prewarm_static_passed_cargo_deferred_implementation_cadence`, `render_plan08_asset_root_builtin_standard_material_multi_geometry_prewarm_static_passed_cargo_deferred_implementation_cadence`, and `render_plan08_builtin_material_multi_pass_depth_only_prewarm_tests_passed_renderdoc_deferred`.

The staged-cache acceptance slice now covers the next step for builtin standard material requests. `builtin_standard_material_prewarm_writes_restart_hits_and_wgpu_modules` writes the generated static and skinned five-pass manifests through `prewarm_shader_variants(...)`, reopens `ShaderVariantCacheDisk` to simulate a restart, requires `ShaderVariantCacheDiskLookup::Hit` for every derived `ShaderVariantCacheDiskKey::from_variant_key(...)`, and creates WGPU shader modules from the read-back WGSL under a validation error scope. Status: `render_plan08_builtin_material_staged_prewarm_cache_hit_wgpu_module_passed_renderdoc_deferred`.

Runtime Base mesh staged prewarm cache hit closes the next consumer-level step. `runtime_base_mesh_pipeline_uses_staged_prewarm_without_compile_miss` writes the builtin fallback/standard material staged manifest into a temporary staged root, injects `ShaderVariantCacheDisk::with_fallback_roots(runtime_root, [staged_root])` into `MeshPipelineCache`, and calls `ensure_pipeline_for_variant(...)` on a real offscreen WGPU device. It then requires `ShaderVariantMissReport.disk_hit_count == 1` and `compile_miss_count == 0` while the Base mesh render pipeline is created under WGPU validation scope. Status: `render_plan08_runtime_base_mesh_staged_prewarm_cache_hit_wgpu_pipeline_passed_renderdoc_deferred`.

Product Base mesh second-launch staged prewarm closes the product-facing Base/Opaque slice. `graphics/tests/render_product_mesh_cache/staged_prewarm.rs::render_product_base_mesh_second_launch_uses_staged_prewarm_without_compile_miss` writes the staged manifest once, then creates two fresh `WgpuRenderFramework` instances with `ShaderVariantCacheDisk::with_fallback_roots(runtime_root, [staged_root])` injected through the test-only `replace_shader_variant_disk_cache_for_tests(...)` seam. The product pipeline registers `mesh.opaque` with side effects, uses `DisplayMode::Shaded` to force BaseScenePass replay, and asserts both launches report shader-variant requests, staged disk hits, `compile_miss_count == 0`, no runtime cache writes/errors, mesh replay state changes, skinned draws, and executed `mesh.opaque` evidence. `runtime_15_product_base_mesh_staged_prewarm_is_wired` keeps the product child owner and status/docs anchors locked. Status: `render_plan08_product_base_mesh_second_launch_staged_prewarm_passed_renderdoc_deferred`.

Runtime mesh variant geometry-source key wiring extends the live Base mesh path beyond the static wrapper. `MeshDrawGeometrySource::shader_geometry_source_id()` is the only current bridge from draw-source classification to shader `GeometrySourceId`: prepared and dynamic CPU-side batches remain static mesh, while `DynamicGpuSkinningSource` resolves to skinned mesh. `MeshPassBuildContext` now derives that id from the batch queue profile before variant resolution, `MeshPipelineVariantRegistry` stores it in the registry-owned `ShaderVariantKey`, and `ensure_pipeline.rs` forwards it to the source owner. This is intentionally conservative: morphed and skinned+morphed variants still need finer draw metadata before they can be selected at runtime. Status: `render_plan08_runtime_mesh_variant_geometry_source_key_wiring_static_passed_cargo_deferred_implementation_cadence`.

## Current Limits

This module is not a full Bevy `ShaderPlugin`, `ShaderCache`, or `PipelineCache`. It does not parse WGSL imports, resolve shader include graphs, apply shader definitions to Naga composition, validate Naga modules, track dependent pipelines, deduplicate bind group layouts, or support async pipeline creation states.

Runtime Base mesh template source selection now distinguishes static and skinned geometry where the draw queue exposes a GPU skinning source. Asset-root builtin standard-material prewarm and pure built-in fallback prewarm now emit template requests for each requested built-in geometry source and for Forward, GBuffer, DepthPrepass, Shadow, and Velocity pass types that the source owner can assemble. Morphed/skinned-morphed runtime selection still needs richer draw-source metadata.

Asset-level shader readiness is intentionally narrower than renderer readiness. It can report missing runtime WGSL, invalid entry-point stage tokens, duplicate or empty shader definitions, source-only versus redirected import rows, and copied validation diagnostics, but it does not decide whether a concrete device can create a module or pipeline.

The layout descriptor is serialized intent, not reflection. It does not yet derive bind groups from WGSL, validate binding type compatibility, model dynamic offsets, express texture sample types, or map push constants to backend feature gates. Future shader milestones should add those checks below the framework DTO layer so `.zshader` authoring and renderer preparation continue to share one stable contract.

Asset-root prewarm scanning is still intentionally conservative, but it no longer hardcodes the geometry-source dimension or the built-in standard-material pass to Forward. It defaults to static-mesh requests for compatibility, and explicit `--geometry-source` / `--shader-geometry-source` values can expand built-in static, skinned, morphed, and skinned+morphed requests across the pass dimension from `.zshader` entry-point stages plus material-instance alpha-test, double-sided, built-in shading-model variants, explicit custom shading-model plugin ids, explicit custom geometry-source plugin ids, alpha-blend Forward-only filtering, selected quality tiers, source-hash-derived edit revisions, explicit resource-registry revision overlays, and project/plugin shader permutation registry overlays. Builtin standard-material `.zmaterial` references and pure `--builtin-fallback` requests now use the same requested geometry source list and emit Forward, GBuffer, DepthPrepass, Shadow, and Velocity template requests, while custom `.zshader` and standalone `.wgsl` requests still remain raw scanned source payloads. Runtime draw submission can carry a non-Medium `ShaderQualityTier` into `ShaderVariantKey.quality`, build staging can prewarm matching quality tiers, built-in geometry sources, explicit plugin geometry-source ids, pass-specific standard-material templates, explicit custom shading-model ids, project/plugin permutation registry ids, asset-root edit revisions, and exported live shader resource revisions, and the base mesh WGPU cache path now consumes that same quality-aware key. The template assembler can now produce deterministic WGSL/hash inputs for those built-in geometry sources, has a standard material surface source owner, has a Naga validation helper, carries uv1/tangent interpolation through pass templates, aligns `ZrVertexInput` with runtime mesh vertex attributes, applies runtime scene/GPUScene world-to-clip transform in `zr_build_vertex_output(...)`, exposes generic runtime `vs_main`/`fs_main` aliases over `zr_vs_main_impl`/`zr_fs_main_impl`, samples the standard normal map, and has template-level alpha clip behavior for Forward/GBuffer/DepthPrepass/Shadow/Velocity when alpha-test is enabled. The Base mesh fallback/missing-shader runtime source now consumes that standard material Forward template output and feeds include/source hashes into the runtime disk/module cache keys, Velocity consumes template source with previous-position input and source-hash module identity, TAA reactive mask consumes its auxiliary template source with source-hash module identity, Shadow consumes template source with source-hash module identity, current runtime DepthPrepass consumes normal-target template source, Deferred GBuffer consumes albedo/material template source, and the built-in fallback plus asset-root builtin standard material prewarm manifests write matching pass-specific source/hash/revision payloads. Builtin standard material staged prewarm now has focused write, restart cache-hit, and WGPU shader-module validation evidence under `render_plan08_builtin_material_staged_prewarm_cache_hit_wgpu_module_passed_renderdoc_deferred`; runtime Base mesh now has staged fallback root hit, WGPU pipeline creation, and `compile_miss_count == 0` evidence under `render_plan08_runtime_base_mesh_staged_prewarm_cache_hit_wgpu_pipeline_passed_renderdoc_deferred`; Product Base mesh second-launch staged prewarm has two fresh product submits with staged disk hits and zero compile misses under `render_plan08_product_base_mesh_second_launch_staged_prewarm_passed_renderdoc_deferred`; asset-root shader edit revision export is locked under `render_plan08_asset_root_shader_edit_revision_export_passed_cargo_renderdoc_deferred`; explicit custom geometry-source id prewarm is wired under `render_plan08_asset_root_custom_geometry_source_id_prewarm_typecheck_passed_test_timeout_no_result`; report-level prewarm dimension diagnostics are wired under `render_plan08_prewarm_dimension_diagnostics_typecheck_passed_test_timeout_no_result`; explicit resource registry overlay is wired under `render_plan08_asset_root_resource_registry_revision_overlay_typecheck_passed_test_timeout_no_result`; and Shader permutation registry overlay is wired under `render_plan08_shader_permutation_registry_overlay_focused_tests_passed_renderdoc_deferred`. Asset-root custom shader scanning still does not invoke the assembler or validator. RenderDoc/product capture remains pending. The asset-root prewarm tool can now consume project/plugin permutation registry JSON from explicit `--shader-permutation-registry` paths or asset-root `shader_permutation_registry.json`, but it does not yet automatically generate the full project/plugin shader, shading-model, and geometry-source registry export. Base shader-source requests also remain conservative when no material instance narrows the pass set. Staged-cache compile acceptance is still needed before long-lived edited projects can claim the same product-level acceptance breadth as the focused staged-cache test.

Build-tool prewarm report consumption now reads `shader_variants_report.json` after the staged prewarm process returns, prints the `dimension_summary` groups as a compact log summary, and then propagates any non-zero exit code. Status: `render_plan08_build_tool_prewarm_dimension_summary_python_tests_passed_cargo_deferred`.

Shader permutation registry overlay now lets that same staged path consume external `shader_permutation_registry.json` files for custom geometry-source and custom shading-model ids. Status: `render_plan08_shader_permutation_registry_overlay_focused_tests_passed_renderdoc_deferred`; `shader_permutation_registry_paths`, `shader_prewarm_permutation_registry_merges_custom_geometry_and_shading_ids`, and `runtime_15_shader_prewarm_permutation_registry_overlay_is_wired` lock the input seam. Full project/plugin registry generation, real Naga/WGPU prewarm compile, RenderDoc/product capture, and runtime pure-depth DepthPrepass remain open.

Shader permutation registry auto-export now lets the staged build tool generate the same overlay schema from known custom id inputs when no explicit registry path is supplied. `BuildConfig.shader_prewarm_permutation_registry_path` writes `ZirconEngine/cache/shader_permutation_registry.json`, `write_generated_shader_permutation_registry(...)` emits `geometry_source_ids` / `shading_model_ids`, and `shader_permutation_registry_paths_for_prewarm(...)` passes either the explicit override or the generated file to `zircon_shader_prewarm`. Status: `render_plan08_build_tool_shader_permutation_registry_auto_export_focused_tests_passed_renderdoc_deferred`; `test_write_generated_shader_permutation_registry_writes_json` and `runtime_15_shader_prewarm_permutation_registry_auto_export_is_wired` lock the handoff. This is still a build-tool export from `--shader-geometry-source-id` / `--shader-shading-model-id`, not full project/plugin shader, shading-model, or geometry-source discovery.

Plugin shader permutation registry auto-export now lets selected plugin package manifests contribute the same custom id records to staged prewarm without repeating them on the command line. `PluginPackageManifest.shader_permutation` owns the manifest schema, `virtual_geometry` declares `custom:virtual_geometry = 4` in both its descriptor and static `plugin.toml`, and the build helper merges selected plugin records with explicit CLI records before writing the generated `shader_permutation_registry.json`. Status: `render_plan08_plugin_shader_permutation_registry_auto_export_focused_tests_passed_renderdoc_deferred`; `test_zircon_build_discovers_plugin_shader_permutation_records`, `test_generated_shader_permutation_registry_document_merges_selected_plugin_ids`, `test_build_command_uses_generated_shader_permutation_registry_for_selected_plugin_ids`, and `runtime_15_shader_prewarm_plugin_permutation_registry_auto_export_is_wired` lock selected-only discovery and handoff. This still does not imply full project shader resource discovery, custom shading-model plugin descriptor registration, real Naga/WGPU prewarm compile, RenderDoc/product capture, or runtime pure-depth DepthPrepass migration.

Plugin shader permutation registry export contract now validates that generated handoff before the staged prewarm subprocess runs. `validate_shader_permutation_registry_export_contract(...)` reads the generated `ZirconEngine/cache/shader_permutation_registry.json` and requires the current selected-plugin plus explicit CLI geometry-source and shading-model id specs to appear in its `geometry_source_ids` / `shading_model_ids` arrays. Status: `render_plan08_plugin_shader_permutation_registry_export_contract_python_passed_cargo_deferred`; `test_validate_generated_registry_requires_selected_plugin_ids`, `test_prewarm_shaders_validates_generated_registry_before_run`, and `runtime_15_shader_prewarm_plugin_permutation_registry_auto_export_is_wired` lock the generated-registry acceptance gate. This closes the build-tool handoff contract only; real Naga/WGPU prewarm compile, RenderDoc/product capture, full project/plugin shader resource discovery, and miss=0 product acceptance remain open.

Plugin shading-model descriptor registration now gives custom shading models the real descriptor owner that the earlier F11 review required before reintroducing plugin registration. `PluginPackageManifest.shading_models` serializes `ShadingModelDescriptor` rows, `RuntimeExtensionRegistry` tracks them by plugin owner for register/merge/revoke flows, and `graphics/material/shading_models/registry.rs::register_plugin_descriptor(...)` keeps plugin ids out of the built-in shading-model range. The build tool also derives selected-plugin prewarm `shader_shading_model_ids` from those `[[shading_models]]` rows, so a plugin does not have to duplicate the same id in `shader_permutation.shading_model_ids`. Status: `render_plan08_plugin_shading_model_descriptor_registration_typecheck_python_passed_libtest_blocked_by_ui_input_error`; `plugin_package_manifest_declares_custom_shading_model_descriptors`, `test_zircon_build_discovers_plugin_shading_model_descriptors_as_shader_ids`, and `runtime_15_shader_prewarm_plugin_shading_model_descriptor_registration_is_wired` lock the schema, runtime extension registry, graphics registry guard, build discovery, and docs anchors. This descriptor-owner slice did not close real Naga/WGPU prewarm compile, RenderDoc/product capture, or runtime pure-depth DepthPrepass migration.

Plugin geometry-source descriptor registration now gives custom geometry sources the same manifest/runtime owner path. `PluginPackageManifest.geometry_sources` serializes `GeometrySourceDescriptor` rows, `RuntimeExtensionRegistry` tracks them by plugin owner for register/merge/revoke flows, and selected plugin `[[geometry_sources]]` rows feed `tools/zircon_build.py::discover_plugins(...)` so staged prewarm derives `shader_geometry_source_ids` without requiring a duplicate `shader_permutation.geometry_source_ids` row. `virtual_geometry` declares `custom:virtual_geometry = 4` through both its runtime descriptor and static `plugin.toml`, while the legacy id row remains accepted as a compatibility input for staged registries. Status: `render_plan08_plugin_geometry_source_descriptor_registration_typecheck_python_cargo_check_passed_renderdoc_deferred`; `plugin_package_manifest_declares_custom_geometry_source_descriptors`, `test_zircon_build_discovers_plugin_geometry_source_descriptors_as_shader_ids`, and `runtime_15_shader_prewarm_plugin_geometry_source_descriptor_registration_is_wired` lock the schema, runtime extension registry, build discovery, and docs anchors. This descriptor-owner slice did not close real Naga/WGPU prewarm compile, RenderDoc/product capture, complete project shader resource discovery, or runtime pure-depth DepthPrepass migration.

Plugin shader asset roots auto-export now makes selected plugin package assets part of the staged prewarm input set. `tools/zircon_build.py::discover_plugins(...)` resolves existing plugin `asset_roots`, the default `assets` root, and legacy `[distribution] assets = ["assets/**"]` roots into `PluginPackage.asset_roots`; `tools/zircon_build_shader_prewarm.py::shader_asset_root_paths_for_prewarm(...)` appends those roots after staged `ZirconEngine/assets`. Because `zircon_shader_prewarm::run` already exports shader resource records for every `--asset-root`, selected plugin WGSL payloads participate in the same `--export-resource-registry` pass as engine assets. Status: `render_plan08_plugin_shader_asset_roots_auto_export_focused_tests_passed_cargo_deferred_renderdoc_deferred`; `test_build_command_includes_selected_plugin_asset_roots` and `runtime_15_shader_prewarm_plugin_asset_roots_auto_export_is_wired` lock the build command, discovery inputs, docs anchors, and owner budget. This closes selected-plugin asset-root participation, but not full live project/plugin shader resource registry export, real Naga/WGPU prewarm compile, or RenderDoc/product capture.

Runtime pure-depth DepthPrepass product migration moves the runtime mesh DepthPrepass consumer from the temporary normal-target contract to the existing depth-only pass template. `MeshPassPipelineKind::DepthPrepass` now maps to `ShaderPassType::DepthPrepass`, `mesh_pipeline_depth_prepass_template_source_for_geometry(...)` selects `zr_template_depth.wgsl` / `zr_template_depth_alpha.wgsl`, and `create_depth_prepass_mesh_pipeline(...)` no longer imports or declares `NORMAL_FORMAT`. Opaque depth prepass uses no fragment stage; alpha-test depth prepass keeps `fs_main` for discard but uses `targets: &[]`, so the WGPU pipeline writes only `DEPTH_FORMAT`. Status: `render_plan08_runtime_depth_prepass_pure_depth_product_migration_static_passed_cargo_check_renderdoc_deferred`; `mesh_pipeline_depth_prepass_template_source_uses_depth_only_template`, `mesh_pipeline_variant_registry_maps_depth_prepass_to_depth_prepass_pass_type`, and `runtime_15_depth_prepass_pure_depth_product_migration_is_wired` lock the source, variant identity, WGPU descriptor, docs anchors, and owner budgets. This closes runtime pure-depth DepthPrepass product migration; real staged prewarm Naga/WGPU compile, full live project/plugin registry export, RenderDoc capture, and broader product acceptance remain separate Plan 08 work.

TAA reactive shader pass identity now has the same cache/prewarm dimension discipline as the other mesh material passes. `ShaderPassType::TaaReactiveMask` serializes and reports as `taa_reactive_mask`, `MeshPipelineVariantRegistry` maps both `TaaReactiveMask` and `TaaReactiveMaterialMask` mesh kinds to that pass, and `taa_reactive_mask_mesh_shader_key(...)` includes `|pass=taa_reactive_mask|` in the module key through `ShaderVariantKey::canonical_string()`. Built-in fallback and asset-root full-material prewarm enumerate six material passes, so staged cache reports can show TAA reactive entries instead of folding them into Forward. Status: `render_plan08_taa_reactive_shader_pass_identity_static_passed_cargo_deferred`; `render_shader_pass_type_names_taa_reactive_mask_separately_from_forward`, `mesh_pipeline_variant_registry_maps_taa_reactive_to_taa_reactive_pass_type`, `taa_reactive_mask_shader_key_includes_shader_variant_identity_and_source_hash`, and `runtime_15_taa_reactive_shader_pass_identity_is_wired` lock this behavior. The slice passed rustfmt, source/docs anchor scans, stale pass-pattern scan, and diff-check on 2026-06-28; Cargo check was deferred because unrelated runtime text/editor layout compile lanes were active. This closes only the TAA reactive pass identity/cache-prewarm dimension, not real runtime WGPU execution, RenderDoc capture, full registry export, or miss=0 product acceptance.

Prewarm opt-in WGPU shader-module validation gives staged prewarm a real module creation gate without changing the default cache-write path. `prewarm_shader_variants_to_disk_with_module_validation(...)` still runs the existing Naga WGSL validator first, then invokes an injected module validator before `ShaderVariantCacheDisk::write(...)`; validation failure records `WGPU shader module validation failed` in `ShaderVariantPrewarmReport` and leaves the disk cache untouched. `prewarm_shader_variants_with_wgpu_module_validation(...)` is the dynamic API owner for the real WGPU path: it creates an offscreen backend, pushes a validation error scope, calls `device.create_shader_module(...)` with the request WGSL, and maps setup or validation failures back into the same prewarm report. `zircon_shader_prewarm --validate-wgpu-modules` and `tools/zircon_build.py --validate-wgpu-shaders` are opt-in switches, so existing staged prewarm users keep the current Naga-only write path unless they request WGPU module validation. Status: `render_plan08_prewarm_wgpu_module_validation_gate_python_cargo_check_passed_runtime_run_timeout_deferred`; `render_shader_variant_prewarm_rejects_wgpu_module_validation_failure_before_disk_write`, `test_build_command_forwards_wgpu_shader_module_validation`, and `runtime_15_shader_prewarm_wgpu_module_validation_is_wired` lock the failure-before-write behavior, CLI/build-tool handoff, docs anchors, and owner budgets. Python and scoped Cargo checks passed; the actual `cargo run ... --validate-wgpu-modules` attempt timed out while compiling on Windows and is not accepted as runtime execution evidence.

Prewarm WGPU validation report summary makes that opt-in gate observable in the report artifact. `ShaderVariantPrewarmReport` carries `wgpu_module_validation.enabled`, `requested_count`, `validated_count`, `failed_count`, and `skipped_count`; the prewarm write path increments those counters when module validation passes, fails, or is skipped because WGSL validation failed first. The dynamic setup failure path records the same WGPU failure counts when an offscreen backend cannot be created. `tools/zircon_build_shader_prewarm.py` prints the summary line and reads both older `requested`/`written`/`failed` rows and Rust's actual `requested_count`/`written_count`/`failed_count` rows for dimension summaries. Status: `render_plan08_prewarm_wgpu_validation_report_summary_python_passed_cargo_deferred`; `test_dimension_summary_lines_format_wgpu_module_validation_counts`, `test_dimension_summary_lines_accept_rust_count_field_names`, `render_shader_variant_prewarm_records_wgpu_module_validation_success`, and `runtime_15_shader_prewarm_wgpu_validation_report_summary_is_wired` lock the report field, log summary, tests, docs anchors, and owner budgets. This does not count as real staged WGPU execution evidence.

Build-tool WGPU validation report contract makes the report summary a required success condition when the staged build asks for WGPU module validation. `tools/zircon_build.py::prewarm_shaders(...)` calls `validate_shader_prewarm_report_contract(...)` only after a zero exit code, and the helper requires the report to confirm `wgpu_module_validation.enabled`, a positive requested count, `validated_count == requested_count`, and zero failed/skipped variants. Non-zero prewarm exits still print the report summary and propagate the process failure without running the contract check. Status: `render_plan08_build_tool_wgpu_report_contract_python_passed_cargo_deferred`; `test_prewarm_shaders_validates_wgpu_report_after_success`, `test_validate_report_contract_requires_wgpu_validation_when_requested`, `test_validate_report_contract_accepts_wgpu_validation_counts`, and `runtime_15_shader_prewarm_wgpu_report_contract_is_wired` lock the build-tool gate. This is still Python/static evidence; real staged WGPU execution remains a later acceptance item.

Build-tool WGPU validation totals match contract makes that gate count-complete against the top-level prewarm report. `validate_shader_prewarm_report_contract(...)` now rejects successful reports where `wgpu_module_validation.requested_count`, `validated_count`, or `failed_count` disagree with top-level `requested_count`, `written_count`, or `failed_count`. The dedicated Python owner is `tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py`; `test_validate_report_contract_rejects_wgpu_validation_total_mismatch` locks the mismatch case, and `runtime_15_shader_prewarm_wgpu_report_contract_is_wired` asserts the WGPU report-contract regressions stay in that owner instead of returning to the general prewarm tests. Status: `render_plan08_build_tool_wgpu_validation_totals_match_python_passed_cargo_deferred`; the general prewarm owner is now 653 lines and the WGPU report owner is 87 lines. This still does not count as real `zircon_shader_prewarm --validate-wgpu-modules` runtime evidence.

Shader prewarm source provenance summary closes the report-level provenance gap for staged prewarm artifacts. `ShaderVariantPrewarmRequest.source_label` records the asset scan stable label or `builtin://shader/pbr.wgsl`; the prewarm write path records successful and failed requests through `record_written_request(...)` / `record_failure_request(...)`; and `ShaderVariantPrewarmReport.source_provenance` groups each source/template payload by label, WGSL source hash, include hashes, template revision, Naga version, WGPU version, and requested/written/failed counts. `tools/zircon_build_shader_prewarm.py` prints a compact `source provenance:` line so build logs can identify which source payload produced report entries without printing full WGSL. Status: `render_plan08_shader_prewarm_source_provenance_summary_python_passed_cargo_deferred`; `test_dimension_summary_lines_format_source_provenance`, `render_shader_variant_prewarm_report_groups_written_and_failed_dimensions`, `shader_prewarm_asset_root_manifest_reads_compound_zshader_package`, and `runtime_15_shader_prewarm_source_provenance_summary_is_wired` lock the DTO/report/manifest/build-helper wiring. This remains Python/static/rustfmt evidence; Cargo guard, real `zircon_shader_prewarm --validate-wgpu-modules`, RenderDoc/product capture, full registry export, and product miss=0 are not closed by this row.

Build-tool source provenance report contract makes that provenance field a staged build success condition. `tools/zircon_build.py::prewarm_shaders(...)` now passes `require_source_provenance=True` after a zero prewarm exit, and `validate_shader_prewarm_report_contract(...)` requires a non-empty `source_provenance.sources` map, matching source count, variant count coverage for the report requested count, and per-source `source_label`, `source_hash`, `template_revision`, and closed requested/written/failed counts. Status: `render_plan08_build_tool_source_provenance_report_contract_python_passed_cargo_deferred`; `test_validate_report_contract_requires_source_provenance_when_requested`, `test_validate_report_contract_accepts_source_provenance_counts`, the expanded `test_prewarm_shaders_validates_wgpu_report_after_success`, and `runtime_15_shader_prewarm_source_provenance_report_contract_is_wired` lock the build-tool gate. This still does not close Cargo guard, real staged WGPU execution, RenderDoc/product capture, full registry export, or miss=0 product acceptance.

Build-tool shader resource registry export contract makes the automatic staged
`shader_resource_records.json` export an explicit build-tool success condition.
After a zero prewarm exit, `tools/zircon_build.py::prewarm_shaders(...)` calls
`validate_shader_resource_registry_export_contract(...)` when
`--shader-resource-registry` was not supplied. The helper accepts the same
registry container shapes consumed by `zircon_shader_prewarm`: a raw
`ResourceRecord` array, `{ resources: [...] }`, or `{ records: [...] }`. Empty
arrays are valid, while missing files, invalid JSON, non-array containers, and
non-object records fail before the staged build can claim a successful
auto-export. Status:
`render_plan08_build_tool_resource_registry_export_contract_python_passed_cargo_deferred`;
`test_validate_registry_export_contract_requires_resource_records`,
`test_prewarm_shaders_skips_export_contract_for_explicit_registry`, the expanded
`test_prewarm_shaders_validates_wgpu_report_after_success`, and
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

Material custom shading-model runtime registry now connects those selected plugin descriptors to material runtime consumption. `RuntimeModuleExtensionInputs` collects `RuntimeExtensionRegistry::shading_models()`, `GraphicsModule` and the WGPU framework pass the descriptor list through `SceneRenderer::new_with_plugin_render_extensions_and_shading_models(...)`, and `ResourceStreamer::new_with_plugin_shading_models(...)` builds one `ShadingModelRegistry` for both material preparation and material capture seeds. Status: `render_plan08_material_custom_shading_model_runtime_registry_material_test_static_guard_passed_cargo_guard_timeout_renderdoc_deferred`; `render_product_streamer_projects_plugin_custom_shading_model_into_pipeline_key` and `runtime_15_material_custom_shading_model_runtime_registry_is_wired` lock that a `.zmaterial` custom lighting model such as `custom:subsurface` reaches `PipelineKey.shading_model_id` through the plugin descriptor registry rather than falling back to StandardPBR. This closes runtime custom lighting-model resolution, but it still does not close real Naga/WGPU prewarm compile, RenderDoc/product capture, complete project shader resource discovery, or automatic project/plugin shader/shading/geometry registry export.

Live ResourceManager shader registry export is now wired at the resource/prewarm seam. `ResourceManager::ready_records_for_kind(ResourceKind::Shader)` exports deterministic ready shader `ResourceRecord` rows with non-zero live revisions, and `shader_resource_records_from_manager(&manager)` feeds those rows into `ShaderPrewarmResourceRegistryOverlay` so asset-root `.zmeta` shader scans can use the live `material_revision` instead of a fallback source hash. Status: `render_plan08_live_resource_manager_shader_registry_export_focused_tests_passed_renderdoc_deferred`. `shader_prewarm_resource_registry_overlay_uses_live_resource_manager_shader_revisions` and `runtime_15_shader_prewarm_live_resource_manager_registry_export_is_wired` lock the handoff. This does not yet automatically enumerate full project/plugin shader, shading-model, or geometry-source registries, and it does not close real Naga/WGPU prewarm compile, RenderDoc/product capture, or runtime pure-depth DepthPrepass migration.

## Test Coverage

`render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts` proves runtime WGSL selection, WGSL fallback source selection, non-WGSL missing-source rejection, entry-point stage projection, dependency projection, typed variant-key projection, and serialized pipeline layout projection.

`render_product_assets_shader_defs_accept_legacy_flags_and_typed_values`, `zshader_typed_shader_definition_rows_validate_kind_and_value`, and the compound `.zshader` import regression cover the typed shader-definition contract. Legacy `shader_defs = ["FEATURE"]` remains accepted as bool-true flags, while typed rows preserve bool, signed integer, and unsigned integer values through `ShaderAsset`, readiness reporting, and `RenderShaderVariantKey`.

`render_shader_template_assembles_standard_material_surface_source` checks that the standard material template source projects alpha-test, receive-shadows, and double-sided features, then assembles into renamed `zr_material_surface` WGSL with the expected material binding contract. `runtime_15_render_shader_template_assembly_is_folder_backed` locks `graphics/shader/template/material_surface.rs`, `standard_material_surface_source`, and the Plan 08 status anchors alongside the original template assembly guard.

`render_shader_template_validates_standard_material_wgsl_with_naga` records the intended Naga validation path for assembled standard material WGSL. The test is part of the template contract, but the current slice only completed static validation; Cargo/Naga execution is deferred to the milestone testing lane.

The standard material template test also checks uv1/tangent interpolation strings and runtime mesh vertex input locations so template pass edits cannot silently drop `fetch_tangent(v, instance_index)`, `fetch_uv1(v)`, or the `input.uv1` material-source path.

`render_shader_template_clips_alpha_for_masked_standard_material_passes` checks the masked StandardPBR cutoff path across DepthPrepass and Shadow alpha templates. It asserts the alpha-only template tokens, `ZR_STANDARD_MATERIAL_ALPHA_CUTOFF`, `standard_material_alpha_cutoff()`, `standard_material_properties.data8.z`, `surface.alpha_cutoff`, and `zr_apply_alpha_clip(surface)` so alpha-test semantics cannot remain declared only as feature bits.

It also checks `standard_material_sampled_normal`, `standard_material_normal_tex`, and `input.tangent_handedness` so normal-map sampling cannot regress back to an unused binding while the runtime cutover is still pending.

`mesh_pipeline_standard_material_template_source_assembles_forward_base_source`, `mesh_pipeline_standard_material_template_source_uses_requested_geometry_source`, `mesh_pipeline_standard_material_shader_pass_source_keeps_depth_only_contract`, `mesh_pipeline_velocity_template_source_uses_previous_position_vertex_input`, `mesh_pipeline_taa_reactive_mask_template_source_uses_material_surface_without_lighting`, `mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked`, `mesh_pipeline_template_source_hashes_include_template_revision`, and `mesh_pipeline_template_source_hashes_feed_disk_and_module_keys` check the Base mesh runtime source cutover, the standard material prewarm pass source owner, the Velocity source-cache cutover, the TAA reactive source-cache cutover, the Shadow source-cache cutover, and the source owner split: fallback/missing shader source is generated by `mesh_pipeline_cache/shader_source.rs`, contains runtime `vs_main`/`fs_main`, carries Forward light-grid/shadow includes only for the Forward path, projects `ShaderFeatureBits`, records template revision separately from raw WGSL revision, can select the requested skinned geometry include, assembles pure depth-only DepthPrepass prewarm without material fragment code for opaque variants, assembles alpha-test DepthPrepass prewarm with `zr_template_depth_alpha.wgsl` and alpha clip but without normal-target output, assembles Velocity with `@location(8) previous_position` and alpha discard when needed, assembles TAA reactive mask without Forward light/shadow includes while preserving `fs_taa_reactive_mask` and `fs_taa_reactive_material_mask`, assembles ShadowDepth without a fragment entry for opaque variants, assembles ShadowDepthAlphaMask with `fs_main` and material alpha discard, and feeds both include hashes and final source hash into disk/module cache identity.

`mesh_pipeline_variant_registry_separates_geometry_sources` and `render_mesh_draw_processor_uses_batch_geometry_source_for_pipeline_variant_key` check that runtime batch geometry source reaches `ShaderVariantKey.geometry_source` before Base pipeline creation. `runtime_15_mesh_pass_processors_are_folder_backed` keeps that processor coverage in the child test owner, and `runtime_15_render_shader_template_assembly_is_folder_backed` locks the source owner and `ensure_pipeline.rs` delegation.

`velocity_mesh_shader_key_includes_shader_variant_identity_and_source_hash`, `taa_reactive_mask_shader_key_includes_shader_variant_identity_and_source_hash`, and `shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash` check the non-Base pass shader-module key helpers. Velocity, TAA, and Shadow cover the template source hash in module identity. `velocity_mesh_pipeline_declares_template_entries_and_previous_position_vertex_slot` checks the WGPU descriptor uses template `vs_main`/`fs_main` while keeping the previous-position vertex slot, `taa_reactive_mask_pipeline_declares_template_entries_and_static_vertex_layout` checks TAA reactive mask descriptors use `vs_main`, `fs_taa_reactive_mask`, and `fs_taa_reactive_material_mask` while keeping the static mesh vertex layout, and `shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias` checks Shadow descriptors use template `vs_main`/`fs_main`, keep static mesh vertex layout, and retain depth bias. `velocity_mesh_pipeline_creates_on_wgpu_device_with_template_shader` and `taa_reactive_mask_mesh_pipeline_creates_on_wgpu_device_with_template_shader` now cover the device-creation entry points with the shared `mesh_pipeline/test_support.rs` fixture, but current WSL direct execution segfaults in the shared offscreen path and is not accepted as passing evidence. `runtime_15_render_shader_template_assembly_is_folder_backed` also locks that the Velocity/TAA/Shadow pipeline maps are keyed by `MeshPipelineVariantId`, that old `pipeline_key_for_variant(...)` does not return as a narrower lookup, and that neither Velocity nor TAA reactive source cache paths import `FALLBACK_MESH_SHADER` while Shadow no longer mounts the deleted inline shadow shader path.

`builtin_fallback_shader_prewarm_manifest_uses_mesh_template_source`, `shader_prewarm_builtin_fallback_manifest_expands_requested_geometry_sources`, `runtime_base_mesh_pipeline_uses_staged_prewarm_without_compile_miss`, and `runtime_15_builtin_fallback_prewarm_uses_template_source` check that the dynamic API's built-in fallback prewarm manifest no longer writes `FALLBACK_MESH_SHADER`, but instead emits the same template source, content hashes, and template revision that Base mesh runtime cache consumes; that the CLI manifest path expands pure fallback requests across requested built-in geometry sources and the standard-material pass list; and that a staged fallback root hit can create the runtime Base mesh WGPU pipeline with `disk_hit_count == 1` and `compile_miss_count == 0`.

`render_product_base_mesh_second_launch_uses_staged_prewarm_without_compile_miss` and `runtime_15_product_base_mesh_staged_prewarm_is_wired` extend that evidence to the product path. The test lives in `graphics/tests/render_product_mesh_cache/staged_prewarm.rs`, writes the staged manifest once, runs first and second product launches through fresh `WgpuRenderFramework` instances, and asserts staged disk hits plus `compile_miss_count == 0` without runtime cache writes/errors while `mesh.opaque` and skinned Base replay evidence are visible.

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
`test_prewarm_shaders_skips_export_contract_for_explicit_registry`, the expanded
`test_prewarm_shaders_validates_wgpu_report_after_success`, and
`runtime_15_shader_prewarm_resource_registry_export_contract_is_wired` cover the
Build-tool shader resource registry export contract. They lock that successful
automatic export must leave a parseable `ResourceRecord` container, that
explicit registry inputs do not validate an export they did not produce, and
that this contract composes with the report and source-provenance contracts.
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
`test_acceptance_contract_skips_export_validation_for_explicit_registry`,
`test_prewarm_shaders_runs_acceptance_bundle_after_success`, the updated
`test_prewarm_shaders_validates_staged_acceptance_after_success`, and
`runtime_15_shader_prewarm_acceptance_contract_is_wired` cover the Build-tool
staged prewarm acceptance contract. They lock that the zero-exit build path
calls one acceptance helper that composes WGPU/report/source-provenance
dimension checks, staged cache artifact checks, and automatic
resource-registry/report correlation while skipping export validation for
explicit registry inputs. `test_acceptance_contract_rejects_runtime_fallback_layout_drift`
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

`shader_resource_records_from_asset_roots_deduplicates_duplicate_shader_records` covers duplicate shader `.zmeta` metadata across `engine_assets` and `plugin_assets`. `shader_resource_records_from_asset_roots_rejects_id_locator_conflicts` and `shader_resource_records_from_asset_roots_rejects_locator_id_conflicts` cover the two conflict paths where dedupe must fail instead of creating an ambiguous overlay. `runtime_15_shader_prewarm_resource_registry_multi_root_dedupe_is_wired` locks the staged shader resource registry multi-root dedupe owner, docs/status anchors, and line budget.

`test_prewarm_plan_lists_asset_roots_for_registry_export` covers dry-run plan output for the same engine/plugin asset roots consumed by staged registry export. `test_prewarm_plan_lists_runtime_fallback_handoff_paths` extends that dry-run coverage to `shader prewarm cache root`, `shader prewarm report`, and `shader runtime fallback root`, matching the runtime fallback root path audited by staged acceptance. `test_build_command_auto_export_registry_scans_all_asset_roots` covers the final command's `--asset-root` sequence and default `--export-resource-registry` path. `runtime_15_shader_prewarm_asset_root_plan_visibility_is_wired` locks the Build-tool shader asset-root plan visibility status, docs anchors, and file budgets.

The broader `render_product_assets` filter and `cargo check -p zircon_runtime --lib --tests --locked` remain the milestone-level compile/test gates for this surface.
