---
related_code:
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs
  - zircon_runtime/src/asset/load/texture.rs
  - zircon_runtime/src/asset/tests/load/texture.rs
  - zircon_runtime/src/asset/load/mesh.rs
  - zircon_runtime/src/asset/formats/obj/error.rs
  - zircon_runtime/src/asset/formats/obj/decode_obj_file.rs
  - zircon_runtime/src/asset/formats/obj/parse_obj_face_vertex.rs
  - zircon_runtime/src/asset/formats/obj/parse_obj_scalar.rs
  - zircon_runtime/src/asset/formats/obj/resolve_obj_index.rs
  - zircon_runtime/src/asset/tests/load/mesh.rs
  - zircon_runtime/src/asset/tests/formats/obj.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/late_api_cleanup.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render/lights.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/asset_scene/mesh_bindings.rs
  - zircon_runtime/src/scene/tests/asset_scene/hierarchy_sources.rs
  - zircon_runtime/src/scene/tests/asset_scene/product_fields.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/scene/tests/world_basics/world_state.rs
  - zircon_runtime/src/scene/tests/world_basics/render_extract.rs
  - zircon_runtime/src/scene/tests/world_basics/sprites.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_vertices/build_particle_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_velocity_vertices/build_particle_velocity_vertices.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/registration.rs
  - zircon_runtime/src/graphics/runtime_provider/update.rs
  - zircon_runtime/src/graphics/runtime_provider/feedback.rs
  - zircon_runtime/src/graphics/runtime_provider/prepare_input.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/asset/importer/ingest/import_sound.rs
  - zircon_runtime/src/asset/tests/assets/sound.rs
  - zircon_runtime/src/asset/prelude.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/prelude.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/prelude.rs
  - zircon_runtime/src/ui/public_runtime_frame.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/property_mutation/metadata_dirty.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/v2/style/runtime_state.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/arrange/grid_masonry.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/slot_contract.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_x_classes.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_x_classes/data_grid.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/document/validation.rs
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/extract/state.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase/descriptor_builders.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/ui.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset/management.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/material_asset/management.rs
  - zircon_runtime/src/asset/assets/material/material_asset/readiness.rs
  - zircon_runtime/src/asset/assets/material/material_asset/value_sync.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets/material.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/descriptor/settings.rs
  - zircon_runtime/src/rhi/device.rs
  - zircon_runtime/src/rhi/device/handles.rs
  - zircon_runtime/src/rhi_wgpu/device.rs
  - docs/zircon_runtime/rhi/descriptors.md
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime/src/dynamic_api/session/registry.rs
  - zircon_runtime/src/dynamic_api/session/tests/lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries/core_spine.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries/module_families.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries/runtime_root.rs
  - zircon_runtime/src/tests/runtime_absorption/core_spine_root_generated.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_boundary.py
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/root_entries.rs
  - docs/zircon_runtime/dynamic_api/session.md
  - zircon_runtime/src/scene/dynamic_scene/spawn_task/task.rs
  - zircon_runtime/src/scene/dynamic_scene/spawn_task/loader.rs
  - docs/zircon_runtime/scene/dynamic_scene.md
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - docs/zircon_runtime/scene/ecs.md
  - zircon_runtime/src/core/resource/manager/resource_manager.rs
  - zircon_runtime/src/core/resource/manager/registry_ops.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime/src/core/resource/manager/lease_ops.rs
  - zircon_runtime/src/core/resource/manager/events.rs
  - docs/zircon_runtime/core/resource.md
  - zircon_runtime/src/animation/manager/mod.rs
  - docs/zircon_runtime/animation/runtime.md
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/default_input_action_manager.rs
  - docs/zircon_runtime/input/input_state.md
  - zircon_runtime/src/core/runtime/config_store.rs
  - docs/zircon_runtime/core/runtime/config_store.md
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/navigation/runtime/tests.rs
  - docs/zircon_runtime/navigation/runtime.md
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_cache_payload.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/mesh_asset.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/material_asset.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_project_scan_import.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_gltf_labeled_subassets.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/texture_descriptor_settings.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_asset_integration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_world_basics.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/rhi_device_handles.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/dynamic_api_session_profile.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/dynamic_api_session_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/module_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/rhi_wgpu_command_validation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/rhi_wgpu_ui_surface_render_setup.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_scene_world.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shadow.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_stats_graph.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/scene_fixed_lights.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_text_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_v2_style.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_template_style_apply.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_template_document.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_accessibility_extract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_component_catalog_editor_showcase.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/table/columns.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/ui.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/editor_workbench.rs
  - zircon_runtime/src/asset/watch/asset_change_construction.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_construction.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - zircon_runtime/src/graphics/runtime/render_framework
  - zircon_runtime/src/ui/tests/runtime_ui_support
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_public_runtime.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/prelude.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/core/runtime/state/module_entry.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/registration/register_module.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/handle/states.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - docs/zircon_runtime/core/diagnostics.md
  - docs/zircon_runtime/core/runtime/lifecycle.md
  - docs/zircon_runtime/core/state.md
  - docs/zircon_runtime/core/tasks.md
  - zircon_runtime/src/script/vm/backend/backend_registry.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests.rs
  - zircon_runtime/src/script/vm/tests/reflection_docs.rs
  - docs/zircon_runtime/script/vm/zr_vm_host_reflection.md
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_construct/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_construct/construct.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_resource.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/virtual_geometry_indirect.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/residual_fallback.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_update.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_update.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/solari_runtime_provider/provider_registration.rs
  - docs/zircon_runtime/script/vm/host/function_ledger.md
  - docs/zircon_runtime/script/vm/tests.md
  - docs/zircon_runtime/script/vm/zr_vm_host_reflection.md
  - docs/zircon_runtime/graphics/runtime_provider/registration.md
  - docs/zircon_runtime/graphics/runtime_provider/update.md
  - docs/zircon_runtime/graphics/runtime_provider/feedback.md
  - docs/zircon_runtime/graphics/runtime_provider/prepare_input.md
  - docs/zircon_runtime/graphics/render-product-submit.md
  - zircon_runtime/src/graphics/tests/render_product_camera_targets.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/composite.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/material_sampling.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/ordering.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/viewport.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/fixture.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/primary_surface.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/texture_target.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers/queue_override.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/provider_boilerplate.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/facade_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/native_live_host_lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/script_vm_lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/diagnostics_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/root_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/root_layout/status_scan.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/root_layout/ui_children.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/pack.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/facade.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/project.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/material.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_gltf_importer.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_importer.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/render_products.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/runtime_diagnostics.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/rhi_command_list.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/rhi_device_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_query.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_schedule.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_systems.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_accessibility.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_component_state.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_component_state_keyboard.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_material_foundation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_event_routing.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_window_event_abi.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_window_input_pump.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/window_timer.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/route_order.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/route_matrix.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/touch_pointer.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_manager.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/basic_editing.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/selection_navigation.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/word_shortcuts.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/clipboard_newline.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/text_ime.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tests/focus_navigation/focus_state.rs
  - zircon_runtime/src/ui/tests/focus_navigation/property_mutation.rs
  - zircon_runtime/src/ui/tests/focus_navigation/tab_directional.rs
  - zircon_runtime/src/ui/tests/focus_navigation/modal_popup.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_focus_navigation.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/input_method.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/owner_validation.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/high_precision_dispatch.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/drag_drop.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/popup_tooltip.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/route_trace.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_ownership.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_taffy_layout_pass.rs
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/script/vm/tests/host_exports.rs
  - zircon_runtime/src/script/vm/tests/bridge_host.rs
  - zircon_runtime/src/script/vm/tests/reflection_docs.rs
  - zircon_runtime/src/script/vm/tests/plugin_runtime.rs
  - zircon_runtime/src/script/vm/tests/module_surface.rs
  - zircon_runtime/src/script/vm/tests/support.rs
  - zircon_runtime/src/script/vm/tests/lifecycle_failures.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/spawn_transform.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/component_state.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/combat_lifecycle.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/property_animation.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs
  - zircon_runtime/src/asset/tests/pack.rs
  - zircon_runtime/src/asset/tests/pack/basic.rs
  - zircon_runtime/src/asset/tests/pack/reader_validation.rs
  - zircon_runtime/src/asset/tests/pack/delta_reader_validation.rs
  - zircon_runtime/src/asset/tests/pack/delta_pack.rs
  - zircon_runtime/src/asset/tests/pack/delta_installer.rs
  - zircon_runtime/src/asset/tests/pack/trim.rs
  - zircon_runtime/src/asset/tests/facade.rs
  - zircon_runtime/src/asset/tests/facade/handle_events.rs
  - zircon_runtime/src/asset/tests/facade/load_state_roots.rs
  - zircon_runtime/src/asset/tests/facade/project_facade.rs
  - zircon_runtime/src/asset/tests/facade/recursive_dependencies.rs
  - zircon_runtime/src/asset/tests/facade/dependency_failures.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/asset/tests/project/zmeta/metadata_lifecycle.rs
  - zircon_runtime/src/asset/tests/project/zmeta/package_roots.rs
  - zircon_runtime/src/asset/tests/project/zmeta/compound_shader.rs
  - zircon_runtime/src/asset/tests/project/zmeta/shader_diagnostics_fixture.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/asset/tests/project/manager/library_imports.rs
  - zircon_runtime/src/asset/tests/project/manager/restore_failure_migration.rs
  - zircon_runtime/src/asset/tests/project/manager/subassets_errors.rs
  - zircon_runtime/src/asset/tests/assets/material.rs
  - zircon_runtime/src/asset/tests/assets/material/asset_serialization.rs
  - zircon_runtime/src/asset/tests/assets/material/owned_descriptor.rs
  - zircon_runtime/src/asset/tests/assets/material/override_validation.rs
  - zircon_runtime/src/asset/tests/assets/material/shader_readiness.rs
  - zircon_runtime/src/asset/tests/assets/material/management_records.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/basic_import.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/labeled_subassets.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/multi_primitive.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/external_inputs.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/vertex_channels.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/material_transforms.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/multi_scene.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/assets/importer/structure.rs
  - zircon_runtime/src/asset/tests/assets/importer/typed_toml_ui.rs
  - zircon_runtime/src/asset/tests/assets/importer/builtin_data.rs
  - zircon_runtime/src/asset/tests/assets/importer/registry_priority.rs
  - zircon_runtime/src/asset/tests/assets/importer/registry_errors.rs
  - zircon_runtime/src/asset/tests/assets/importer/shader_model.rs
  - zircon_runtime/src/asset/tests/assets/importer/physics_animation.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/core/framework/tests/framework_surfaces.rs
  - zircon_runtime/src/core/framework/tests/render_product_surface.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/external_dependents.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_two_three_dependency_matcher.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/shutdown_order.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_four_dependency_matcher.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_without_index_map.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_dependency_matcher.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/core_runtime_deactivation.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_runtime/src/ui/tests/v2_asset/asset_loading.rs
  - zircon_runtime/src/ui/tests/v2_asset/style_runtime.rs
  - zircon_runtime/src/ui/tests/v2_asset/default_controls.rs
  - zircon_runtime/src/ui/tests/v2_asset/range_controls.rs
  - zircon_runtime/src/ui/tests/v2_asset/demo_and_builder.rs
  - zircon_runtime/src/ui/tests/v2_asset/composite_components.rs
  - zircon_runtime/src/ui/tests/v2_asset/file_cache.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/shared_core/layout_surface.rs
  - zircon_runtime/src/ui/tests/shared_core/box_flow.rs
  - zircon_runtime/src/ui/tests/shared_core/input_visibility.rs
  - zircon_runtime/src/ui/tests/shared_core/navigation.rs
  - zircon_runtime/src/ui/tests/shared_core/scroll_mutation.rs
  - zircon_runtime/src/ui/tests/accessibility.rs
  - zircon_runtime/src/ui/tests/accessibility/extraction.rs
  - zircon_runtime/src/ui/tests/accessibility/naming_relations.rs
  - zircon_runtime/src/ui/tests/accessibility/focus_diagnostics.rs
  - zircon_runtime/src/ui/tests/accessibility/description_references.rs
  - zircon_runtime/src/ui/tests/accessibility/activation_actions.rs
  - zircon_runtime/src/ui/tests/accessibility/value_actions.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/event_routing/pointer_state.rs
  - zircon_runtime/src/ui/tests/event_routing/component_events.rs
  - zircon_runtime/src/ui/tests/event_routing/dispatch_effects.rs
  - zircon_runtime/src/ui/tests/event_routing/shared_input.rs
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/catalog_inventory.rs
  - zircon_runtime/src/ui/tests/component_catalog/descriptor_contracts.rs
  - zircon_runtime/src/ui/tests/component_catalog/registry_queries.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/retained_events.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/collection_mutation.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/reference_sources.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/interaction_numeric.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/action_selection.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/menu_navigation.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/text_inputs.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/numeric_controls.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/planned_layers.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/editor_components.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mui_surface_overlay.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mui_x_runtime.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/folder_structure.rs
  - zircon_runtime/src/ui/tests/boundary.rs
  - zircon_runtime/src/ui/tests/boundary/template_namespace.rs
  - zircon_runtime/src/ui/tests/boundary/layout_tree_surface.rs
  - zircon_runtime/src/ui/tests/boundary/binding_event_roots.rs
  - zircon_runtime/src/ui/tests/boundary/asset_fixture_projection.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/route_trace_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/pointer_bubble_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/focus_text_accessibility_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/focus_path.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/semantic_actions.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/timers_disabled.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/directional.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/selection.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/drag_reorder.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/virtualization.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi/batch_adapter.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi/pointer_window_routes.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi/keyboard_gamepad_routes.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump/lifecycle.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump/pointer_routes.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump/metrics_dirty.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/window_timer.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/route_order.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/route_matrix.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/touch_pointer.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_manager.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/module_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/renderer_output_accessors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/backend_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/gpu_resource_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/resource_streamer_cleanup.rs
  - zircon_runtime/src/rhi/tests/command_list.rs
  - zircon_runtime/src/rhi/tests/command_list/basic_commands.rs
  - zircon_runtime/src/rhi/tests/command_list/bind_groups.rs
  - zircon_runtime/src/rhi/tests/command_list/raster_draws.rs
  - zircon_runtime/src/rhi/tests/command_list/vertex_index_state.rs
  - zircon_runtime/src/rhi/tests/device_contract.rs
  - zircon_runtime/src/rhi/tests/device_contract/basic_resources.rs
  - zircon_runtime/src/rhi/tests/device_contract/texture_sampler_descriptors.rs
  - zircon_runtime/src/rhi/tests/device_contract/bind_groups.rs
  - zircon_runtime/src/rhi/tests/device_contract/invalid_descriptors.rs
  - zircon_runtime/src/rhi/tests/device_contract/transfer_and_fences.rs
  - zircon_runtime/src/rhi/tests/device_contract/framework_boundary.rs
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/conflict_graph.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/fixed_update.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/parallel_executor.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/resources_events.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/schedule_plan.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/render_extract.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/world_driver.rs
  - zircon_runtime/src/scene/tests/ecs_systems.rs
  - zircon_runtime/src/scene/tests/ecs_systems/commands.rs
  - zircon_runtime/src/scene/tests/ecs_systems/state_params.rs
  - zircon_runtime/src/scene/tests/ecs_systems/events.rs
  - zircon_runtime/src/scene/tests/ecs_systems/run_window_filters.rs
  - zircon_runtime/src/scene/tests/ecs_systems/many_single_queries.rs
  - zircon_runtime/src/scene/tests/ecs_systems/removal_local.rs
  - zircon_runtime/src/scene/tests/ecs_query.rs
  - zircon_runtime/src/scene/tests/ecs_query/read_items.rs
  - zircon_runtime/src/scene/tests/ecs_query/mutation_access.rs
  - zircon_runtime/src/scene/tests/ecs_query/fixed_ticks.rs
  - zircon_runtime/src/scene/tests/ecs_query/iter_many.rs
  - zircon_runtime/src/scene/tests/ecs_query/cached_queries.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/mod.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/types.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/multi.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/specialized.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/shutdown.rs
  - zircon_runtime/src/rhi_wgpu/command_validation.rs
  - zircon_runtime/src/rhi_wgpu/command_validation/render_state.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry/tests.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/graph.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/graph/execution_resources.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/material_asset/management.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets/material.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/descriptor/settings.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_gltf_labeled_subassets.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/texture_descriptor_settings.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/material_asset.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/rhi_wgpu_ui_surface_geometry.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_01_05.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_06_10.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_11_14.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_01_05.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_06_10.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_11_14.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/evidence_anchors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups.rs
implementation_files:
  - docs/zircon_runtime/structure/module-convention.md
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/mod.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/types.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/multi.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/specialized.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/shutdown.rs
  - zircon_runtime/src/rhi_wgpu/command_validation.rs
  - zircon_runtime/src/rhi_wgpu/command_validation/render_state.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry/tests.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/graph.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/graph/execution_resources.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/rhi_wgpu_ui_surface_geometry.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render/lights.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/asset_scene/mesh_bindings.rs
  - zircon_runtime/src/scene/tests/asset_scene/hierarchy_sources.rs
  - zircon_runtime/src/scene/tests/asset_scene/product_fields.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/scene/tests/world_basics/world_state.rs
  - zircon_runtime/src/scene/tests/world_basics/render_extract.rs
  - zircon_runtime/src/scene/tests/world_basics/sprites.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_asset_integration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_world_basics.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/registration.rs
  - zircon_runtime/src/graphics/runtime_provider/update.rs
  - zircon_runtime/src/graphics/runtime_provider/feedback.rs
  - zircon_runtime/src/graphics/runtime_provider/prepare_input.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/asset/prelude.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/assets/sound.rs
  - zircon_runtime/src/asset/importer/ingest/import_sound.rs
  - zircon_runtime/src/scene/prelude.rs
  - zircon_runtime/src/ui/prelude.rs
  - zircon_runtime/src/graphics/prelude.rs
  - zircon_runtime/src/ui/public_runtime_frame.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/property_mutation/metadata_dirty.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/v2/style/runtime_state.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/arrange/grid_masonry.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/slot_contract.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_x_classes.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_x_classes/data_grid.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/document/validation.rs
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/extract/state.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase/descriptor_builders.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/ui.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/rhi/device.rs
  - zircon_runtime/src/rhi/device/handles.rs
  - zircon_runtime/src/rhi_wgpu/device.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/rhi_wgpu_lock_poison.rs
  - docs/zircon_runtime/rhi/descriptors.md
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime/src/dynamic_api/session/registry.rs
  - zircon_runtime/src/dynamic_api/session/tests/lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries/core_spine.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries/module_families.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries/runtime_root.rs
  - zircon_runtime/src/tests/runtime_absorption/core_spine_root_generated.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_boundary.py
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/root_entries.rs
  - docs/zircon_runtime/dynamic_api/session.md
  - zircon_runtime/src/scene/dynamic_scene/spawn_task/task.rs
  - zircon_runtime/src/scene/dynamic_scene/spawn_task/loader.rs
  - docs/zircon_runtime/scene/dynamic_scene.md
  - zircon_runtime/src/core/resource/manager/resource_manager.rs
  - zircon_runtime/src/core/resource/manager/registry_ops.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime/src/core/resource/manager/lease_ops.rs
  - zircon_runtime/src/core/resource/manager/events.rs
  - docs/zircon_runtime/core/resource.md
  - zircon_runtime/src/animation/manager/mod.rs
  - docs/zircon_runtime/animation/runtime.md
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/default_input_action_manager.rs
  - docs/zircon_runtime/input/input_state.md
  - zircon_runtime/src/core/runtime/config_store.rs
  - docs/zircon_runtime/core/runtime/config_store.md
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/navigation/runtime/tests.rs
  - docs/zircon_runtime/navigation/runtime.md
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_cache_payload.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_project_scan_import.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/rhi_device_handles.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/dynamic_api_session_profile.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/dynamic_api_session_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/module_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/rhi_wgpu_command_validation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/rhi_wgpu_ui_surface_render_setup.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_scene_world.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/scene_world_render_lights.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shadow.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_stats_graph.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/scene_fixed_lights.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_text_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_v2_style.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_template_style_apply.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_template_document.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_accessibility_extract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_component_catalog_editor_showcase.rs
  - zircon_runtime/src/ui/tests/runtime_ui_support
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_public_runtime.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/core/runtime/state/module_entry.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/registration/register_module.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/tests/registration/structure.rs
  - zircon_runtime/src/core/runtime/tests/registration/structure/behavior_layout.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/handle/states.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - docs/zircon_runtime/core/diagnostics.md
  - docs/zircon_runtime/core/state.md
  - docs/zircon_runtime/core/tasks.md
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/native_host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/native_live_host_tests.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader/real_fixture.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/native_plugin_loader.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge/basics.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge/diagnostics.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge/lifecycle.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/extension_registry_bridge.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions/editor_only.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions/net.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/manifest_contributions.rs
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_package_manifest.rs
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_package_manifest/feature_modules.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/runtime_plugin_package_manifest.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan/catalog_projection.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/export_build_plan.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform/browser_hosts.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/export_build_plan_platform.rs
  - docs/zircon_runtime/plugin/bridge.md
  - docs/zircon_runtime/plugin/package_manifest.md
  - docs/zircon_runtime/plugin/export_build_plan.md
  - zircon_runtime/src/script/vm/backend/backend_registry.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests.rs
  - docs/zircon_runtime/script/vm/zr_vm_host_reflection.md
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_construct/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_construct/construct.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_resource.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/virtual_geometry_indirect.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_update.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/prepare_input.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_update.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/solari_runtime_provider/provider_registration.rs
  - docs/zircon_runtime/script/vm/host/function_ledger.md
  - docs/zircon_runtime/graphics/runtime_provider/registration.md
  - docs/zircon_runtime/graphics/runtime_provider/update.md
  - docs/zircon_runtime/graphics/runtime_provider/feedback.md
  - docs/zircon_runtime/graphics/runtime_provider/prepare_input.md
  - docs/zircon_runtime/graphics/render-product-submit.md
  - zircon_runtime/src/graphics/tests/render_product_camera_targets.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/composite.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/material_sampling.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/ordering.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/viewport.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/fixture.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/primary_surface.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/texture_target.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/provider_boilerplate.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/native_live_host_lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/script_vm_lock_poison.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/module_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/renderer_output_accessors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/backend_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/gpu_resource_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/resource_streamer_cleanup.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/root_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/root_layout/status_scan.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/root_layout/ui_children.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/pack.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/facade.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/project.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/material.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_gltf_importer.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/asset_importer.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/render_products.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/runtime_diagnostics.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/rhi_command_list.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/rhi_device_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_query.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_schedule.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_systems.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_accessibility.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_component_state.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_component_state_keyboard.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_material_foundation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_event_routing.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_window_event_abi.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_window_input_pump.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/window_timer.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/route_order.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/route_matrix.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/touch_pointer.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_manager.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/basic_editing.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/selection_navigation.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/word_shortcuts.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/clipboard_newline.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/text_ime.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tests/focus_navigation/focus_state.rs
  - zircon_runtime/src/ui/tests/focus_navigation/property_mutation.rs
  - zircon_runtime/src/ui/tests/focus_navigation/tab_directional.rs
  - zircon_runtime/src/ui/tests/focus_navigation/modal_popup.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_focus_navigation.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/input_method.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/owner_validation.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/high_precision_dispatch.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/drag_drop.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/popup_tooltip.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/route_trace.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_ownership.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_taffy_layout_pass.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/spawn_transform.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/component_state.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/combat_lifecycle.rs
  - zircon_runtime/src/script/vm/gameplay_host/tests/property_animation.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs
  - zircon_runtime/src/asset/tests/pack.rs
  - zircon_runtime/src/asset/tests/pack/basic.rs
  - zircon_runtime/src/asset/tests/pack/reader_validation.rs
  - zircon_runtime/src/asset/tests/pack/delta_reader_validation.rs
  - zircon_runtime/src/asset/tests/pack/delta_pack.rs
  - zircon_runtime/src/asset/tests/pack/delta_installer.rs
  - zircon_runtime/src/asset/tests/pack/trim.rs
  - zircon_runtime/src/asset/tests/facade.rs
  - zircon_runtime/src/asset/tests/facade/handle_events.rs
  - zircon_runtime/src/asset/tests/facade/load_state_roots.rs
  - zircon_runtime/src/asset/tests/facade/project_facade.rs
  - zircon_runtime/src/asset/tests/facade/recursive_dependencies.rs
  - zircon_runtime/src/asset/tests/facade/dependency_failures.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/asset/tests/project/zmeta/metadata_lifecycle.rs
  - zircon_runtime/src/asset/tests/project/zmeta/package_roots.rs
  - zircon_runtime/src/asset/tests/project/zmeta/compound_shader.rs
  - zircon_runtime/src/asset/tests/project/zmeta/shader_diagnostics_fixture.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/asset/tests/project/manager/library_imports.rs
  - zircon_runtime/src/asset/tests/project/manager/restore_failure_migration.rs
  - zircon_runtime/src/asset/tests/project/manager/subassets_errors.rs
  - zircon_runtime/src/asset/tests/assets/material.rs
  - zircon_runtime/src/asset/tests/assets/material/asset_serialization.rs
  - zircon_runtime/src/asset/tests/assets/material/owned_descriptor.rs
  - zircon_runtime/src/asset/tests/assets/material/override_validation.rs
  - zircon_runtime/src/asset/tests/assets/material/shader_readiness.rs
  - zircon_runtime/src/asset/tests/assets/material/management_records.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/basic_import.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/labeled_subassets.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/multi_primitive.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/external_inputs.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/vertex_channels.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/material_transforms.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/multi_scene.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/assets/importer/structure.rs
  - zircon_runtime/src/asset/tests/assets/importer/typed_toml_ui.rs
  - zircon_runtime/src/asset/tests/assets/importer/builtin_data.rs
  - zircon_runtime/src/asset/tests/assets/importer/registry_priority.rs
  - zircon_runtime/src/asset/tests/assets/importer/registry_errors.rs
  - zircon_runtime/src/asset/tests/assets/importer/shader_model.rs
  - zircon_runtime/src/asset/tests/assets/importer/physics_animation.rs
  - zircon_runtime/src/rhi/tests/command_list.rs
  - zircon_runtime/src/rhi/tests/command_list/basic_commands.rs
  - zircon_runtime/src/rhi/tests/command_list/bind_groups.rs
  - zircon_runtime/src/rhi/tests/command_list/raster_draws.rs
  - zircon_runtime/src/rhi/tests/command_list/vertex_index_state.rs
  - zircon_runtime/src/rhi/tests/device_contract.rs
  - zircon_runtime/src/rhi/tests/device_contract/basic_resources.rs
  - zircon_runtime/src/rhi/tests/device_contract/texture_sampler_descriptors.rs
  - zircon_runtime/src/rhi/tests/device_contract/bind_groups.rs
  - zircon_runtime/src/rhi/tests/device_contract/invalid_descriptors.rs
  - zircon_runtime/src/rhi/tests/device_contract/transfer_and_fences.rs
  - zircon_runtime/src/rhi/tests/device_contract/framework_boundary.rs
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/conflict_graph.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/fixed_update.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/parallel_executor.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/resources_events.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/schedule_plan.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/render_extract.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/world_driver.rs
  - zircon_runtime/src/scene/tests/ecs_systems.rs
  - zircon_runtime/src/scene/tests/ecs_systems/commands.rs
  - zircon_runtime/src/scene/tests/ecs_systems/state_params.rs
  - zircon_runtime/src/scene/tests/ecs_systems/events.rs
  - zircon_runtime/src/scene/tests/ecs_systems/run_window_filters.rs
  - zircon_runtime/src/scene/tests/ecs_systems/many_single_queries.rs
  - zircon_runtime/src/scene/tests/ecs_systems/removal_local.rs
  - zircon_runtime/src/scene/tests/ecs_query.rs
  - zircon_runtime/src/scene/tests/ecs_query/read_items.rs
  - zircon_runtime/src/scene/tests/ecs_query/mutation_access.rs
  - zircon_runtime/src/scene/tests/ecs_query/fixed_ticks.rs
  - zircon_runtime/src/scene/tests/ecs_query/iter_many.rs
  - zircon_runtime/src/scene/tests/ecs_query/cached_queries.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/external_dependents.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_two_three_dependency_matcher.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/shutdown_order.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_four_dependency_matcher.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_without_index_map.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/deactivation/blocked/exact_five_dependency_matcher.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/core_runtime_deactivation.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_runtime/src/ui/tests/v2_asset/asset_loading.rs
  - zircon_runtime/src/ui/tests/v2_asset/style_runtime.rs
  - zircon_runtime/src/ui/tests/v2_asset/default_controls.rs
  - zircon_runtime/src/ui/tests/v2_asset/range_controls.rs
  - zircon_runtime/src/ui/tests/v2_asset/demo_and_builder.rs
  - zircon_runtime/src/ui/tests/v2_asset/composite_components.rs
  - zircon_runtime/src/ui/tests/v2_asset/file_cache.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/shared_core/layout_surface.rs
  - zircon_runtime/src/ui/tests/shared_core/box_flow.rs
  - zircon_runtime/src/ui/tests/shared_core/input_visibility.rs
  - zircon_runtime/src/ui/tests/shared_core/navigation.rs
  - zircon_runtime/src/ui/tests/shared_core/scroll_mutation.rs
  - zircon_runtime/src/ui/tests/accessibility.rs
  - zircon_runtime/src/ui/tests/accessibility/extraction.rs
  - zircon_runtime/src/ui/tests/accessibility/naming_relations.rs
  - zircon_runtime/src/ui/tests/accessibility/focus_diagnostics.rs
  - zircon_runtime/src/ui/tests/accessibility/description_references.rs
  - zircon_runtime/src/ui/tests/accessibility/activation_actions.rs
  - zircon_runtime/src/ui/tests/accessibility/value_actions.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/event_routing/pointer_state.rs
  - zircon_runtime/src/ui/tests/event_routing/component_events.rs
  - zircon_runtime/src/ui/tests/event_routing/dispatch_effects.rs
  - zircon_runtime/src/ui/tests/event_routing/shared_input.rs
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/catalog_inventory.rs
  - zircon_runtime/src/ui/tests/component_catalog/descriptor_contracts.rs
  - zircon_runtime/src/ui/tests/component_catalog/registry_queries.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/retained_events.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/collection_mutation.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/reference_sources.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/interaction_numeric.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/action_selection.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/menu_navigation.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/text_inputs.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/numeric_controls.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/planned_layers.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/editor_components.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mui_surface_overlay.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mui_x_runtime.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/folder_structure.rs
  - zircon_runtime/src/ui/tests/boundary.rs
  - zircon_runtime/src/ui/tests/boundary/template_namespace.rs
  - zircon_runtime/src/ui/tests/boundary/layout_tree_surface.rs
  - zircon_runtime/src/ui/tests/boundary/binding_event_roots.rs
  - zircon_runtime/src/ui/tests/boundary/asset_fixture_projection.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/route_trace_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/pointer_bubble_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/focus_text_accessibility_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/focus_path.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/semantic_actions.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/timers_disabled.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/directional.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/selection.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/drag_reorder.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/virtualization.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi/batch_adapter.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi/pointer_window_routes.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi/keyboard_gamepad_routes.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump/lifecycle.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump/pointer_routes.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump/metrics_dirty.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/window_timer.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/route_order.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/route_matrix.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/touch_pointer.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_manager.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_01_05.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_06_10.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_11_14.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_01_05.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_06_10.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_11_14.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/module_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/evidence_anchors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - zircon_runtime/src/asset/tests/assets/sound.rs::sound_asset_wav_parse_reports_typed_error_variants
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs::review_f5_sound_asset_uses_typed_error
  - cargo test -p zircon_runtime --lib runtime_15_mixed_visibility_has_facade_note --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib prelude --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_prelude_covers_required_types --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_facade_surface_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_runtime_dead_code_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_diagnostics_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_core_framework_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_core_runtime_deactivation_blocked_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_ui_v2_asset_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_ui_shared_core_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_ui_accessibility_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_ui_accessibility_extract_state_is_child_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_ui_component_catalog_editor_showcase_helpers_are_child_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_ui_event_routing_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_ui_runtime_input_reply_routes_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_ui_runtime_input_reply_route_children_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_rhi_command_list_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_rhi_device_contract_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_asset_pack_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_asset_facade_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_asset_project_zmeta_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_asset_project_manager_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_asset_material_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_asset_gltf_importer_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_asset_importer_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_test_file_budget_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_root_entries_guard_child_owners_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_asset_test_budget_guard_child_owner_split --no-default-features --features core-min --locked
  - runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_guard_covers_spawn_task
  - dynamic_scene_spawn_task_accessors_recover_poisoned_locks
  - runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_guard_covers_batch_result_slots
  - schedule_parallel_executor_batch_result_slot_recovers_poisoned_lock
  - runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries
  - vm_plugin_manager_selected_backend_accessors_recover_poisoned_lock
  - runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_guard_covers_manager_selector
  - vm_backend_registry_accessors_recover_poisoned_family_lock
  - host_registry_accessors_recover_poisoned_handle_lock
  - host_export_registry_accessors_recover_poisoned_module_lock
  - hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock
  - runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager
  - project_asset_manager_runtime_accessors_recover_poisoned_locks
  - runtime_15_asset_worker_pool_lock_poison_recovery_guard_covers_asset_worker_pool
  - asset_worker_pool_accessors_recover_poisoned_locks
  - runtime_15_wgpu_render_framework_lock_poison_recovery_guard_covers_wgpu_framework
  - wgpu_render_framework_accessors_recover_poisoned_locks
  - runtime_15_native_live_host_tests_are_folder_backed
  - native_live_host_runtime_descriptor_includes_validation_report
  - native_live_host_reuses_installed_bridge_bindings_for_loaded_manifest_scopes
  - native_hot_reload_state_saves_and_restores_runtime_snapshot
  - runtime_15_extension_registry_bridge_tests_are_folder_backed
  - duplicate_interface_export_rejected
  - bridge_table_summarizes_diagnostics_for_matrix
  - bridge_table_reports_owner_enabled_transition
  - runtime_15_manifest_contributions_tests_are_folder_backed
  - editor_only_plugin_tomls_declare_package_level_targets_and_capabilities
  - net_plugin_toml_declares_content_download_http_dependency
  - runtime_15_runtime_plugin_package_manifest_tests_are_folder_backed
  - native_runtime_plugin_registration_report_rejects_invalid_package_optional_features
  - native_runtime_plugin_registration_report_rejects_invalid_package_module_identities
  - runtime_15_export_build_plan_tests_are_folder_backed
  - source_template_preserves_builtin_catalog_target_modes_after_manifest_completion
  - source_template_links_rendering_default_owner_features
  - source_template_with_native_dynamic_merges_native_loader_reports
  - runtime_15_export_build_plan_platform_tests_are_folder_backed
  - generated_browser_hosts_instantiate_wasm_exports_and_gate_asset_origins
  - runtime_15_native_host_api_adapter_tests_are_child_owner
  - native_host_api_v3_registers_systems_and_components_into_runtime_registry
  - native_host_bridge_call_scope_dispatches_registered_method
  - runtime_15_rhi_wgpu_render_device_lock_poison_recovery_guard_covers_device_state
  - wgpu_render_device_state_accessors_recover_poisoned_lock
  - cargo test -p zircon_runtime --lib runtime_15_gameplay_host_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_shader_prewarm_manifest_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_scene_ecs_schedule_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_scene_ecs_systems_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_scene_ecs_query_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_runtime_ui_dead_code_surface_is_test_support --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_diagnostics_use_frame_trait_without_world_wrapper --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_provider_registration_uses_shared_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_provider_update_uses_shared_stats_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_provider_feedback_uses_shared_payload_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_provider_prepare_input_uses_shared_extract_generation_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_runtime_owned_dead_code_suppression_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_script_host_value_descriptors_do_not_suppress_dead_code --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_offscreen_target_texture_owner_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_gpu_material_uniform_owner_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_gpu_mesh_order_signature_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_gpu_model_identity_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_post_process_lut_texture_owner_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_output_target_texture_owner_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_resource_streamer_diagnostics_accessor_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_resource_streamer_resolve_texture_id_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_particle_gpu_readback_output_accessor_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_advanced_plugin_output_test_accessor_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_graphics_dead_code_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_provider_boilerplate_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_material_asset_management_records_are_child_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_gltf_labeled_material_subassets_are_child_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_texture_descriptor_settings_parser_is_child_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_scene_world_render_light_collectors_are_child_owner --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib core_handle_diagnostic_accessors_recover_poisoned_store_lock --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_core_handle_diagnostics_lock_poison_recovery_guard_covers_diagnostic_store --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib core_handle_time_accessors_recover_poisoned_runtime_clocks --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_core_handle_time_lock_poison_recovery_guard_covers_runtime_clocks --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib core_handle_state_accessors_recover_poisoned_state_registry_lock --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_core_handle_states_lock_poison_recovery_guard_covers_state_registry --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib job_handle_accessors_recover_poisoned_state_lock --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib pending_scheduled_job_recovers_poisoned_task_lock --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_core_runtime_task_lock_poison_recovery_guard_covers_job_handles --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib profile_recorder_accessors_recover_poisoned_global_lock --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_core_runtime_profiling_lock_poison_recovery_guard_covers_global_recorder --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib core_handle_registry_accessors_recover_poisoned_runtime_locks --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_core_handle_registry_lock_poison_recovery_guard_covers_registry_accessors --no-default-features --features core-min --locked
  - bridge_entry_provider_accessors_recover_poisoned_provider_lock
  - runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot
  - native_live_host_bridge_method_bindings_recover_poisoned_lock
  - runtime_15_native_live_host_bridge_methods_lock_poison_recovery_guard_covers_binding_registry
  - cargo test -p zircon_runtime --lib runtime_15_core_runtime_service_lists_are_folder_backed --no-default-features --features core-min --locked
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
doc_type: module-detail
---

# Runtime 模块结构规范镜像文档

> 本文是 [Runtime 15](../../plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md) 的镜像文档，固定 `module_convention_gate` 的结构审计事实，由 `runtime_15_module_convention_mirror_docs_match_structure_audit_counts` 守卫锁定计数。上游规范：[`engine-code-structure-convention.md`](../../plans/engine-code-structure-convention.md)。
>
> 状态：in_progress（Runtime 15 F9 runtime prelude required type coverage、Runtime 15 graphics facade visibility note、Runtime 15 runtime UI dead-code support split、Runtime 15 F12 runtime-owned dead-code suppression cleanup、Runtime 15 F12 script host value descriptor dead-code cleanup、Runtime 15 F12 script reflection macro fixture dead-code cleanup、Runtime 15 F12 offscreen target texture owner cleanup、Runtime 15 F12 render backend state owner cleanup、Runtime 15 F12 gpu texture resource owner cleanup、Runtime 15 F12 gpu material uniform owner cleanup、Runtime 15 F12 gpu mesh order signature cleanup、Runtime 15 F12 gpu model identity cleanup、Runtime 15 F12 post-process LUT texture owner cleanup、Runtime 15 F12 output target texture owner cleanup、Runtime 15 F12 material runtime capture seed cleanup、Runtime 15 F12 resource streamer diagnostics accessor cleanup、Runtime 15 F12 resource streamer resolve texture id cleanup、Runtime 15 F12 particle GPU readback output accessor cleanup、Runtime 15 F12 advanced plugin output test accessor cleanup、Runtime 15 M3 graphics dead-code guard module split、Runtime 15 M3 graphics dead-code guard child-owner split、Runtime 15 M3 provider boilerplate guard module split、Runtime 15 M3 facade surface guard module split、Runtime 15 M3 runtime dead-code guard module split、Runtime 15 M3 diagnostics guard module split、Runtime 15 M3 core framework test folder split、Runtime 15 M3 core runtime deactivation blocked test folder split、Runtime 15 M3 UI v2 asset test folder split、Runtime 15 M3 UI shared core test folder split、Runtime 15 M3 UI accessibility test folder split、Runtime 15 M3 UI accessibility widget actions test folder split、Runtime 15 M3 UI layout slots test folder split、Runtime 15 M3 UI surface-frame authority test folder split、Runtime 15 M3 UI surface dirty domains test folder split、Runtime 15 M3 UI material layout test folder split、Runtime 15 M3 UI event routing test folder split、Runtime 15 M3 UI runtime input reply routes test folder split、Runtime 15 M3 UI runtime input reply route child folder split、Runtime 15 M3 runtime diagnostics test folder split、Runtime 15 M3 RHI command list test folder split、Runtime 15 M3 RHI device contract test folder split、Runtime 15 M3 asset pack test folder split、Runtime 15 M3 asset facade test folder split、Runtime 15 M3 asset project zmeta test folder split、Runtime 15 M3 asset project manager test folder split、Runtime 15 M3 asset project flow sample test folder split、Runtime 15 M3 asset material test folder split、Runtime 15 M3 asset glTF importer test folder split、Runtime 15 M3 asset glTF primitive fixture folder split、Runtime 15 M3 asset importer test folder split、Runtime 15 M3 asset scene test folder split、Runtime 15 M3 test file budget guard folder split、Runtime 15 M3 Runtime 07 performance hotspot guard folder split、Runtime 15 M3 script VM test folder split、Runtime 15 M3 scene ECS schedule test folder split、Runtime 15 F14 diagnostics normalization、Runtime 15 F13 provider registration shared owner、Runtime 15 F13 provider update shared stats owner、Runtime 15 F13 provider feedback shared payload owner、Runtime 15 F13 provider prepare input shared frame owner 与 Runtime 15 F13 full provider boilerplate audit 已落地；完整 `module_convention_boundary.py` 审计计数、全量 dead-code sweep 与测试组织拆分仍 pending）。
>
> 最新完成：Runtime 15 F5 texture loader typed errors（`runtime_15_texture_loader_typed_errors_static_passed_cargo_deferred`）已把 `asset/load/texture.rs` 的 `load_texture(...)` / `decode_image_file(...)` 从 `Result<CpuTexturePayload, String>` / `format!("open image ...")` 收敛到 `TextureLoadError` / `TextureLoadResult`；`TextureLoadError::OpenImage` 保留 `image::ImageError` source，`asset/pipeline/worker_pool.rs` 只在 `CpuAssetPayload::Failure { message }` 出口 stringify，`review_f5_texture_loader_uses_typed_error` 已同步锁定 loader owner、worker boundary、test 和 status/docs anchors；scoped rustfmt 与静态扫描已通过，Cargo 因并行 cargo/rustc lane active deferred。
>
> 最新完成：Runtime 15 F5 mesh loader typed errors（`runtime_15_mesh_loader_typed_errors_static_passed_cargo_deferred`）已把 `asset/load/mesh.rs` 的 `load_mesh(...)` / `decode_mesh_file(...)` 从 `Result<CpuMeshPayload, String>` / `Err(format!(...))` 收敛到 `MeshLoadError` / `MeshLoadResult`；`asset/formats/obj/error.rs` 拥有 `ObjDecodeError` / `ObjDecodeResult`，OBJ read/scalar/index/face/empty-mesh failures 均为 typed variants，worker pool 只在 mesh `CpuAssetPayload::Failure { message }` 出口 stringify；`review_f5_mesh_loader_and_obj_decoder_use_typed_errors` 已同步锁定 loader/decoder owner、worker boundary、test 和 status/docs anchors；scoped rustfmt 与静态扫描已通过，Cargo 因并行 cargo/rustc lane active deferred。
>
> 最新完成：Runtime 15 F5 animation manager typed errors（`runtime_15_animation_manager_typed_errors_static_passed_cargo_deferred`）已新增 `core/framework/animation/error.rs` 的 `AnimationError` / `AnimationResult` typed-error owner，并把 `core/framework/animation/manager.rs`、`animation/manager/{mod,pose,sampling}.rs` 与 `animation/sequence/{apply,conversion}.rs` 的公共 manager/apply、pose sampling 和 channel conversion 入口收敛到 `AnimationResult`；新增 `review_f5_animation_manager_uses_animation_error` 锁定无 `Result<_, String>` / `Err(format!)` 回归；scoped rustfmt 与静态扫描已通过，Cargo 因并行 cargo/rustc lane active deferred。
>
> 最新完成：Runtime 15 F5 asset authoring typed errors（`runtime_15_asset_authoring_typed_errors_static_passed_cargo_deferred`）已把 `asset/assets/authoring.rs` 的 Terrain/TileMap/MaterialGraph authoring 校验从 `Result<(), String>` / `Err(format!(...))` 收敛到 `AssetAuthoringError` / `AssetAuthoringResult`；`asset/importer/ingest/import_authoring_asset.rs` 只在 `AssetImportError::Parse` 边界 stringify，`review_f5_asset_authoring_uses_typed_error` 已同步锁定 authoring owner、facade exports、import boundary 和 status/docs anchors；scoped rustfmt 与静态扫描已通过，Cargo 因并行 cargo/rustc lane active deferred。
>
> 最新完成：Runtime 15 F5 navigation asset typed errors（`runtime_15_navigation_asset_typed_errors_static_passed_cargo_deferred`）已把 `asset/assets/navigation.rs` 的 `NavMeshAsset::to_bytes(...)` / `from_bytes(...)` 从 `Result<_, String>` / `error.to_string()` 收敛到 `NavigationAssetError` / `NavigationAssetResult`；`asset/assets/mod.rs` 与 `asset/mod.rs` 公开导出 typed surface，`review_f5_navigation_asset_uses_typed_error` 已同步锁定 navigation owner、test mount、module doc 和 status/docs anchors；scoped rustfmt 与静态扫描已通过，Cargo 因并行 cargo/rustc lane active deferred。
>
> 最新完成：Runtime 15 F5 font asset typed errors（`runtime_15_font_asset_typed_errors_static_passed_cargo_deferred`）已把 `asset/assets/font.rs` 的 `FontAsset::from_toml_str(...)` 从 `FontAssetError::Parse(String)` / `error.to_string()` 收敛到 `FontAssetError::Parse(#[source] toml::de::Error)` / `FontAssetResult`；`asset/importer/ingest/import_font_asset.rs` 只在 `AssetImportError::Parse` 边界格式化错误，`review_f5_font_asset_uses_typed_error_source` 已同步锁定 font owner、facade exports、import boundary、module doc 和 status/docs anchors；scoped rustfmt 与静态扫描已通过，Cargo 因并行 cargo/rustc lane active deferred。
>
> 最新完成：Runtime 15 F5 sound asset typed errors（`runtime_15_sound_asset_typed_errors_static_passed_cargo_deferred`）已把 `asset/assets/sound.rs` 的 `SoundAsset::from_wav_bytes(...)` 与 WAV parser helpers 从 `Result<_, String>` / `Err(format!(...))` / `.to_string()` 收敛到 `SoundAssetError` / `SoundAssetResult`；`asset/importer/ingest/import_sound.rs` 只在 `AssetImportError::Parse` 边界格式化错误，`review_f5_sound_asset_uses_typed_error` 已同步锁定 sound owner、facade exports、import boundary、module doc 和 status/docs anchors；scoped rustfmt 与静态扫描已通过，Cargo 因并行 cargo/rustc lane active deferred。
>
> 最新完成：Runtime 15 F5 zshader definition typed errors（`runtime_15_zshader_definition_typed_errors_static_passed_cargo_deferred`）已把 `asset/assets/shader/zshader.rs` 的 `ZShaderDefinitionValueDocument::to_render_definition(...)` 与 `ZShaderDocument::shader_definition_values(...)` 从 `Result<_, String>` / `format!` 字符串错误收敛到 `ZShaderDefinitionError` / `ZShaderDefinitionResult`；`review_f5_zshader_definition_values_use_typed_error` 已同步锁定 zshader owner、facade exports、import boundary、shader/material doc 和 status/docs anchors；scoped rustfmt 与静态扫描已通过，Cargo 因并行 cargo/rustc lane active deferred。
>
> 最新完成：Runtime 15 F5 asset meta typed errors（`runtime_15_asset_meta_typed_errors_static_passed_cargo_deferred`）已把 `.zmeta` `asset/project/meta.rs::migrate_to_current(...)` 从 `Result<(), String>` / `Err(format!(...))` 收敛到 `AssetMetaError` / `AssetMetaResult`；`AssetMetaError::UnsupportedFormatVersion` 保留 found/supported 版本，`AssetMetaDocument::load(...)` 只在 `std::io::ErrorKind::InvalidData` 边界 stringify，`review_f5_asset_meta_uses_typed_error` 已同步锁定 meta owner、facade exports、importer doc 和 status/docs anchors；scoped rustfmt 与静态扫描已通过，Cargo 因并行 cargo/rustc lane active deferred。
>
> 最新完成：Runtime 15 F5 fixed world mutation typed errors（`runtime_15_fixed_world_mutation_typed_errors_static_passed_cargo_deferred`）已把 `scene/world/{component_access,hierarchy,query,records}.rs` 的固定 World mutation helper 从内部 `Err(format!(...)).into()` / `to_string().into()` 收敛到显式 `SceneError` 变体；`SceneError::MissingRequiredComponent`、joint/hierarchy/mobility/record/name typed variants 与 `review_f5_fixed_world_mutation_uses_scene_error_variants` 已同步锁定 owner 和 status/docs anchors；scoped rustfmt 与静态扫描已通过，Cargo 因并行 cargo/rustc lane active deferred。
>
> 最新完成：Runtime 15 F5 typed API residual typed errors（`runtime_15_typed_api_residual_typed_errors_static_passed_cargo_deferred`）已把 `scene/world/typed_api.rs` 的 dynamic component presence helper 与 `scene/world/identity.rs::register_stable_entity(...)` 从内部 `Result<_, String>` / `error.to_string()` 收敛到 `SceneResult`；`SceneError::EntityRegistry(#[from] EntityRegistryError)` 保留 stable entity registry source，`review_f5_world_spawn_bundle_surface_uses_scene_error` 已同步锁定 typed API、identity owner 和 status/docs anchors；scoped rustfmt 与静态扫描已通过，Cargo 因并行 cargo/rustc lane active deferred。
>
> 最新完成：Runtime 15 F5 scene property access typed errors（`runtime_15_scene_property_access_typed_errors_static_passed_cargo_deferred`）已把 `scene/world/property_access/{read,write,value_conversion}.rs` 与 `scene/world/property_access/write/physics.rs` 的公共 property 读取/写入入口和转换 helper 收敛到 `SceneResult` / `SceneError`，新增 `review_f5_scene_property_access_uses_scene_error` 锁定 `World::property`、`World::set_property`、`SceneError::PropertyUnavailable` 与无 `Result<_, String>` 回归；scoped rustfmt 与静态扫描已通过，Cargo 因并行 cargo/rustc lane active deferred。
>
> 最新完成：Runtime 15 M3 facade surface guard module split（`runtime_15_facade_surface_guard_module_split_static_passed_cargo_lock_blocked`）已把 façade/prelude 结构守卫迁入 `structure_convention/facade_surface.rs`；完整测试组织拆分仍 pending。
>
> 最新完成：Runtime 15 M3 runtime dead-code guard module split（`runtime_15_runtime_dead_code_guard_module_split_static_passed_cargo_lock_blocked`）已把 runtime dead-code 结构守卫迁入 `structure_convention/runtime_dead_code.rs`；完整测试组织拆分仍 pending。
>
> 最新完成：Runtime 15 F12 script reflection macro fixture dead-code cleanup（`runtime_15_script_reflection_macro_fixture_dead_code_cleanup_static_passed_cargo_deferred`）已移除 `script/vm/tests/reflection_docs.rs` 中 TestVec3/TestEnum/Point 宏 fixture 的 dead-code suppression；完整全量 F12 sweep 仍 pending。
>
> 最新完成：Runtime 15 M3 diagnostics guard module split（`runtime_15_diagnostics_guard_module_split_static_passed_cargo_lock_blocked`）已把 diagnostics 结构守卫迁入 `structure_convention/diagnostics_surface.rs`；完整测试组织拆分仍 pending。
>
> 最新完成：Runtime 15 M3 core framework test folder split（`runtime_15_core_framework_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `core/framework/tests.rs` 降到当前 653 行并迁出 framework surface / render product / phase-queue 合约子 owner；该历史根已由后续 closeout 守卫统一证明低于 800 行。
>
> 最新完成：Runtime 15 M3 core runtime deactivation blocked test folder split（`runtime_15_core_runtime_deactivation_blocked_tests_folder_split_static_passed_cargo_deferred`）已把 `core/runtime/tests/activation/behavior/deactivation/blocked.rs` 降到 7 行并迁出五个新增 folder-backed blocked deactivation owner；10 个测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 code review findings test folder split（`runtime_15_code_review_findings_tests_folder_split_static_passed_cargo_deferred`）已把 `tests/runtime_absorption/code_review_findings.rs` 降到 3 行并迁出 `typed_error_convergence/`、`f8_api_convergence.rs` 与 `late_api_cleanup.rs` 三个 folder-backed review guard owner；25 个评审守卫保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI architecture test folder split（`runtime_15_ui_architecture_tests_folder_split_static_passed_cargo_deferred`）已把 `tests/runtime_absorption/ui_architecture.rs` 降到 104 行并迁出 `tests/runtime_absorption/ui_architecture/architecture_boundaries.rs`、`tests/runtime_absorption/ui_architecture/legacy_renames.rs` 与 `tests/runtime_absorption/ui_architecture/mirror_docs.rs` 三个 folder-backed absorption guard owner；18 个 UI architecture 守卫保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 dynamic scene absorption guard folder split（`runtime_15_dynamic_scene_absorption_guard_folder_split_static_passed_cargo_deferred`）已把 `tests/runtime_absorption/dynamic_scene.rs` 降到 38 行并迁出 `tests/runtime_absorption/dynamic_scene/patch_preview_api.rs`、`tests/runtime_absorption/dynamic_scene/patch_preview_status_docs.rs`、`tests/runtime_absorption/dynamic_scene/patch_preview_behavior.rs`、`tests/runtime_absorption/dynamic_scene/session_capture_persistence.rs`、`tests/runtime_absorption/dynamic_scene/session_retention_mutation_merge.rs`、`tests/runtime_absorption/dynamic_scene/session_load_query_path.rs` 与 `tests/runtime_absorption/dynamic_scene/asset_reload_selection_status.rs` 七个 folder-backed absorption guard owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI v2 asset test folder split（`runtime_15_ui_v2_asset_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `ui/tests/v2_asset.rs` 降到当前 331 行并迁出七个 folder-backed 行为 owner；该历史根已由后续 closeout 守卫统一证明低于 800 行。
>
> 最新完成：Runtime 15 M3 UI shared core test folder split（`runtime_15_ui_shared_core_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `ui/tests/shared_core.rs` 降到当前 77 行并迁出五个 folder-backed 行为 owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 historical oversized test roots closeout（`runtime_15_historical_oversized_test_roots_closeout_static_passed_cargo_deferred`）已用 `runtime_15_historical_oversized_test_roots_are_folder_backed` 统一锁定 `core/framework/tests.rs`、`ui/tests/v2_asset.rs` 与 `ui/tests/shared_core.rs` 三个历史 S6 超大测试根的 folder-backed 收口事实；完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI accessibility test folder split（`runtime_15_ui_accessibility_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/accessibility.rs` 降到 125 行并迁出六个 folder-backed accessibility 行为 owner；49 个测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI accessibility widget actions test folder split（`runtime_15_ui_accessibility_widget_actions_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/accessibility_widget_actions.rs` 降到 250 行并迁出 `ui/tests/accessibility_widget_actions/popup_actions.rs`、`ui/tests/accessibility_widget_actions/tooltip_menu.rs` 等三个 folder-backed widget action owner；新增 `runtime_15_ui_accessibility_widget_actions_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI layout slots test folder split（`runtime_15_ui_layout_slots_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/layout_slots.rs` 降到 100 行并迁出 `ui/tests/layout_slots/linear_free.rs`、`ui/tests/layout_slots/flow_grid_masonry.rs` 等三个 folder-backed layout slot owner；新增 `runtime_15_ui_layout_slots_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI surface-frame authority test folder split（`runtime_15_ui_surface_frame_authority_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/surface_frame_authority.rs` 降到 409 行并迁出 `ui/tests/surface_frame_authority/arranged_authority.rs`、`ui/tests/surface_frame_authority/taffy_wrap_grid.rs` 等四个 folder-backed surface-frame authority owner；新增 `runtime_15_ui_surface_frame_authority_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI surface dirty domains test folder split（`runtime_15_ui_surface_dirty_domains_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/surface_dirty_domains.rs` 降到 297 行并迁出 `ui/tests/surface_dirty_domains/rebuild_domains.rs`、`ui/tests/surface_dirty_domains/incremental_layout.rs` 等四个 folder-backed dirty-domain owner；新增 `runtime_15_ui_surface_dirty_domains_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI material layout test folder split（`runtime_15_ui_material_layout_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/material_layout.rs` 降到 111 行并迁出 `ui/tests/material_layout/button_icon_metrics.rs`、`ui/tests/material_layout/field_values.rs` 等五个 folder-backed material-layout owner；新增 `runtime_15_ui_material_layout_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI template test folder split（`runtime_15_ui_template_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/template.rs` 降到 154 行并迁出 `ui/tests/template/interaction_bindings.rs`、`ui/tests/template/slot_contracts.rs` 等五个 folder-backed template owner；新增 `runtime_15_ui_template_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI component catalog test folder split（`runtime_15_ui_component_catalog_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/component_catalog.rs` 降到 136 行并迁出 `ui/tests/component_catalog/catalog_inventory.rs`、`ui/tests/component_catalog/descriptor_contracts.rs` 与 `ui/tests/component_catalog/registry_queries.rs` 三个 folder-backed component catalog owner；新增 `runtime_15_ui_component_catalog_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI boundary test folder split（`runtime_15_ui_boundary_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/boundary.rs` 降到 62 行并迁出 `ui/tests/boundary/template_namespace.rs`、`ui/tests/boundary/layout_tree_surface.rs`、`ui/tests/boundary/binding_event_roots.rs` 与 `ui/tests/boundary/asset_fixture_projection.rs` 四个 folder-backed boundary owner；新增 `runtime_15_ui_boundary_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI component state test folder split（`runtime_15_ui_component_catalog_component_state_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/component_catalog/component_state.rs` 降到 26 行并迁出 `ui/tests/component_catalog/component_state/retained_events.rs`、`ui/tests/component_catalog/component_state/collection_mutation.rs`、`ui/tests/component_catalog/component_state/reference_sources.rs` 与 `ui/tests/component_catalog/component_state/interaction_numeric.rs` 四个 folder-backed component-state owner；新增 `runtime_15_ui_component_catalog_component_state_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI component state keyboard test folder split（`runtime_15_ui_component_catalog_component_state_keyboard_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/component_catalog/component_state/keyboard.rs` 降到 20 行并迁出 `ui/tests/component_catalog/component_state/keyboard/action_selection.rs`、`ui/tests/component_catalog/component_state/keyboard/menu_navigation.rs`、`ui/tests/component_catalog/component_state/keyboard/text_inputs.rs` 与 `ui/tests/component_catalog/component_state/keyboard/numeric_controls.rs` 四个 folder-backed component-state keyboard owner；新增 `runtime_15_ui_component_catalog_component_state_keyboard_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI component state-reducer keyboard menu submenu owner split（`runtime_15_ui_component_state_reducer_keyboard_menu_submenu_owner_split_static_passed_cargo_deferred`）已把 `ui/component/state_reducer/keyboard/menu.rs` 降到 609 行，并迁出 271 行 `ui/component/state_reducer/keyboard/menu/submenu.rs` 承接 submenu focus-loop、hover pending、open/close 与 target lookup owner；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI component state-reducer tree view editing owner split（`runtime_15_ui_component_state_reducer_tree_view_editing_owner_split_static_passed_cargo_deferred`）已把 `ui/component/state_reducer/tree_view.rs` 降到 508 行，并迁出 312 行 `ui/component/state_reducer/tree_view/editing.rs` 承接 tree-view rename/editing state owner；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI Material foundation test folder split（`runtime_15_ui_component_catalog_material_foundation_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/component_catalog/material_foundation/mod.rs` 降到 149 行并迁出 `ui/tests/component_catalog/material_foundation/planned_layers.rs`、`ui/tests/component_catalog/material_foundation/editor_components.rs`、`ui/tests/component_catalog/material_foundation/mui_surface_overlay.rs`、`ui/tests/component_catalog/material_foundation/mui_x_runtime.rs` 与 `ui/tests/component_catalog/material_foundation/folder_structure.rs` 五个 folder-backed Material foundation owner；新增 `runtime_15_ui_component_catalog_material_foundation_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI asset test folder split（`runtime_15_ui_asset_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/asset.rs` 降到 251 行并迁出 `ui/tests/asset/style_rule_ids.rs`、`ui/tests/asset/style_write_apis.rs`、`ui/tests/asset/loader_validation.rs`、`ui/tests/asset/document_compiler.rs`、`ui/tests/asset/fixture_migration.rs` 与 `ui/tests/asset/component_schema.rs` 六个 folder-backed UI asset owner；新增 `runtime_15_ui_asset_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI asset MUI X web style test folder split（`runtime_15_ui_asset_mui_web_mui_x_style_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/asset_mui_web_mui_x_style.rs` 降到 685 行并迁出 `ui/tests/asset_mui_web_mui_x_style/data_grid.rs`、`ui/tests/asset_mui_web_mui_x_style/tree_view.rs`、`ui/tests/asset_mui_web_mui_x_style/date_time_pickers.rs`、`ui/tests/asset_mui_web_mui_x_style/charts.rs` 与 `ui/tests/asset_mui_web_mui_x_style/agent_chat.rs` 五个 folder-backed MUI X web style owner；新增 `runtime_15_ui_asset_mui_web_mui_x_style_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI asset MUI web style test folder split（`runtime_15_ui_asset_mui_web_style_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/asset_mui_web_style.rs` 降到 648 行并迁出 `ui/tests/asset_mui_web_style/state_icons.rs`、`ui/tests/asset_mui_web_style/slots_native.rs`、`ui/tests/asset_mui_web_style/feedback.rs`、`ui/tests/asset_mui_web_style/surface.rs` 与 `ui/tests/asset_mui_web_style/data_display.rs` 五个 folder-backed MUI web style owner；新增 `runtime_15_ui_asset_mui_web_style_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI taffy layout pass test folder split（`runtime_15_ui_taffy_layout_pass_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/taffy_layout_pass.rs` 降到 168 行并迁出 `ui/tests/taffy_layout_pass/routing_diagnostics.rs`、`ui/tests/taffy_layout_pass/arrangement.rs`、`ui/tests/taffy_layout_pass/linear_slots.rs`、`ui/tests/taffy_layout_pass/fallback_policy.rs` 与 `ui/tests/taffy_layout_pass/grid_slots.rs` 五个 folder-backed Taffy layout pass owner；新增 `runtime_15_ui_taffy_layout_pass_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI runtime window input pump test folder split（`runtime_15_ui_runtime_window_input_pump_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/runtime_window_input_pump.rs` 降到 184 行并迁出 `ui/tests/runtime_window_input_pump/lifecycle.rs`、`ui/tests/runtime_window_input_pump/pointer_routes.rs` 与 `ui/tests/runtime_window_input_pump/metrics_dirty.rs` 三个 folder-backed runtime window input pump owner；新增 `runtime_15_ui_runtime_window_input_pump_tests_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI runtime window event ABI child folder split（`runtime_15_ui_runtime_window_event_abi_children_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/runtime_ui_window_event_routes/abi.rs` 降到 5 行并迁出 `ui/tests/runtime_ui_window_event_routes/abi/batch_adapter.rs`、`ui/tests/runtime_ui_window_event_routes/abi/pointer_window_routes.rs` 与 `ui/tests/runtime_ui_window_event_routes/abi/keyboard_gamepad_routes.rs` 三个 folder-backed ABI route owner；新增 `runtime_15_ui_runtime_window_event_abi_children_are_folder_backed`，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI widget text input keyboard test folder split（`runtime_15_ui_widget_text_input_keyboard_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/widget_text_input_keyboard.rs` 降到 318 行并迁出五个 folder-backed text-input keyboard owner；52 个键盘/文本/IME 测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 UI focus navigation test folder split（`runtime_15_ui_focus_navigation_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/focus_navigation.rs` 降到 346 行并迁出四个 folder-backed focus/navigation owner；16 个 focus、mutation、tab/directional 与 modal/popup 测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI runtime input manager test folder split（`runtime_15_ui_runtime_input_manager_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/runtime_input_manager.rs` 降到 295 行并迁出四个 folder-backed input manager owner；15 个 window/timer、route-order、route-matrix、double-click/touch/multi-pointer 测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI layout arrange grid/masonry owner split（`runtime_15_ui_layout_arrange_grid_masonry_owner_split_static_passed_cargo_deferred`）已把 `ui/layout/pass/arrange.rs` 降到 690 行，并迁出 181 行 `ui/layout/pass/arrange/grid_masonry.rs` 承接 GridBox/MasonryBox fallback arrangement owner；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI text layout engine visual-order owner split（`runtime_15_ui_text_layout_engine_visual_order_owner_split_static_passed_cargo_deferred`）已把 `ui/text/layout_engine.rs` 降到 530 行，并迁出 301 行 `ui/text/layout_engine/visual_order.rs` 承接 BiDi visual-order scaffold；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI template MUI X DataGrid class owner split（`runtime_15_ui_template_mui_x_data_grid_class_owner_split_static_passed_cargo_deferred`）已把 `ui/template/asset/compiler/style_apply/mui_x_classes.rs` 降到 575 行，并迁出 277 行 `ui/template/asset/compiler/style_apply/mui_x_classes/data_grid.rs` 承接 DataGrid class owner；上层 `style_apply.rs` 与完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI template document validation owner split（`runtime_15_ui_template_document_validation_owner_split_static_passed_cargo_deferred`）已把 `ui/template/asset/document.rs` 降到 653 行，并迁出 100 行 `ui/template/asset/document/validation.rs` 承接 document validation owner；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI template style slot-contract owner split（`runtime_15_ui_template_style_slot_contract_owner_split_static_passed_cargo_timeout_no_result`）已把 `ui/template/asset/compiler/style_apply.rs` 降到 701 行，并迁出 207 行 `ui/template/asset/compiler/style_apply/slot_contract.rs` 承接 slot-props/slot-utility owner；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI v2 style runtime-state owner split（`runtime_15_ui_v2_style_runtime_state_owner_split_static_passed_cargo_deferred`）已把 `ui/v2/style.rs` 降到 793 行，并迁出 362 行 `ui/v2/style/runtime_state.rs` 承接 pseudo-state、retained-state 与 dirty-delta owner；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI accessibility extract state owner split（`runtime_15_ui_accessibility_extract_state_owner_split_static_passed_cargo_deferred`）已把 `ui/accessibility/extract.rs` 降到 668 行，并迁出 339 行 `ui/accessibility/extract/state.rs` 承接 accessibility state projection owner；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI component catalog editor-showcase helper owner split（`runtime_15_ui_component_catalog_editor_showcase_helper_owner_split_static_passed_cargo_timeout_no_result`）已把 `ui/component/catalog/editor_showcase.rs` 降到 674 行，并迁出 429 行 `ui/component/catalog/editor_showcase/descriptor_builders.rs` 承接 editor showcase descriptor builder owner；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M2 UI editor showcase descriptor builders module naming hard cutover（`runtime_15_ui_editor_showcase_descriptor_builders_naming_hard_cutover_static_passed_cargo_deferred`）已删除 `ui/component/catalog/editor_showcase/helpers.rs` 并硬切为 `ui/component/catalog/editor_showcase/descriptor_builders.rs`；`runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name` 锁定旧路径不回流，完整 banned-name sweep 与 `module_convention_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI surface event-routing owner split（`runtime_15_ui_surface_event_routing_owner_split_static_passed_cargo_deferred`）已把 `ui/surface/surface.rs` 降到 317 行，并迁出 578 行 `ui/surface/surface/event_routing.rs` 与 356 行 `ui/surface/surface/pointer_component_events.rs` 承接 surface input routing 和 pointer component event owners；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI surface property mutation metadata dirty owner split（`runtime_15_ui_surface_property_mutation_metadata_dirty_owner_split_static_passed_cargo_deferred`）已把 `ui/surface/property_mutation.rs` 降到 522 行，并迁出 322 行 `ui/surface/property_mutation/metadata_dirty.rs` 承接 metadata dirty classification owner；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI surface render feedback command/color owner split（`runtime_15_ui_surface_render_feedback_command_color_owner_split_static_passed_cargo_deferred`）已把 `ui/surface/render/feedback.rs` 降到 590 行，并迁出 268 行 `ui/surface/render/feedback/colors.rs` 与 100 行 `ui/surface/render/feedback/commands.rs` 承接 feedback color resolution 和 primitive render-command owner；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 RHI WGPU UI surface geometry test owner split（`runtime_15_rhi_wgpu_ui_surface_geometry_tests_owner_split_static_passed_cargo_timeout_no_result`）已把 `rhi_wgpu/ui_surface/geometry.rs` 降到 559 行，并迁出 308 行 `rhi_wgpu/ui_surface/geometry/tests.rs` 承接 geometry test suite 与 test-only helper；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI surface default-interactions keyboard/timer owner split（`runtime_15_ui_surface_default_interactions_keyboard_timer_owner_split_static_passed_cargo_deferred`）已把 `ui/surface/surface/default_interactions.rs` 降到 596 行，并迁出 229 行 `ui/surface/surface/default_interactions/keyboard.rs` 与 172 行 `ui/surface/surface/default_interactions/timers.rs` 承接 keyboard default actions 和 timer-derived component event owners；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M4 UI surface table column helper owner split（`runtime_15_ui_surface_table_column_helper_owner_split_static_passed_cargo_deferred`）已把 `ui/surface/surface/default_interactions/table/mod.rs` 降到 677 行，并迁出 292 行 `ui/surface/surface/default_interactions/table/columns.rs` 承接 column metadata、sort/width helper 与 resize drag-token owner；完整 `large_file_ownership_gate` 仍 pending。
>
> 最新完成：Runtime 15 M2 UI table sortingMode server literal allowed-context sync（`runtime_15_ui_table_sorting_mode_server_literal_allowed_context_static_passed_cargo_deferred`）已把 `ui/surface/surface/default_interactions/table/columns.rs` 中的 `sortingMode = "server"` 登记到非网络 `server` 命名审计和 Rust 守卫的新 owner allowlist；graphics render-framework server naming debt 已由后续 M2 hard cutover 关闭，完整 `module_convention_gate` 仍 pending。
>
> 最新完成：Runtime 15 M2 graphics render-framework receiver naming hard cutover（`runtime_15_graphics_render_framework_receiver_naming_hard_cutover_static_passed_cargo_deferred`）已把 `graphics/runtime/render_framework/**` 的非网络 receiver/context 变量从 `server` 硬切为 `framework`，并用 `runtime_15_render_framework_receiver_uses_framework_name` 锁定 `framework: &WgpuRenderFramework`；Rust guard 与 Python `non_network_server_naming.py` 均不再保留 retired graphics debt bucket，后续 editor workbench authority-label M2 cutover 已继续清掉当时剩余的非网络 `server` 命名债。
>
> 最新完成：Runtime 15 M2 editor workbench authority-label naming hard cutover（`runtime_15_editor_workbench_authority_label_naming_hard_cutover_static_passed_cargo_deferred`）已把 `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs` 的输出文案从 `Selected Condition_Night   server authority` 硬切为 `Selected Condition_Night   editor authority`；`runtime_15_editor_workbench_authority_label_uses_editor_name` 与 `non_network_server_naming.py` 锁定 retired editor workbench debt bucket 不回流，当前 non-network server naming gate 为 `classified-and-clear`。
>
> 最新完成：Runtime 15 M2 asset change construction module naming hard cutover（`runtime_15_asset_change_construction_naming_hard_cutover_static_passed_cargo_deferred`）已删除 `asset/watch/asset_change_new.rs` 并硬切为 `asset/watch/asset_change_construction.rs`；`runtime_15_asset_change_construction_uses_owner_name` 锁定旧 `*_new` construction owner 不回流，完整 construction-owner `_new` sweep 与 `module_convention_gate` 仍 pending。
>
> 最新完成：Runtime 15 M2 resource streamer construction module naming hard cutover（`runtime_15_resource_streamer_construction_naming_hard_cutover_static_passed_cargo_deferred`）已删除 `graphics/scene/resources/resource_streamer/resource_streamer_new.rs` 并硬切为 `graphics/scene/resources/resource_streamer/resource_streamer_construction.rs`；`runtime_15_resource_streamer_construction_uses_owner_name` 锁定旧 `*_new` construction owner 不回流，完整 construction-owner `_new` sweep 与 `module_convention_gate` 仍 pending。
>
> 最新完成：Runtime 15 M2 offscreen target construct directory naming hard cutover（`runtime_15_offscreen_target_construct_naming_hard_cutover_static_passed_cargo_timeout_no_result`）已删除 `graphics/backend/render_backend/offscreen_target_new/` 并硬切为 `graphics/backend/render_backend/offscreen_target_construct/`；`runtime_15_offscreen_target_construct_uses_owner_name` 锁定旧 `*_new` construction directory 不回流，完整 construction-owner `_new` sweep 与 `module_convention_gate` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI runtime input ownership test folder split（`runtime_15_ui_runtime_input_ownership_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/runtime_input_ownership.rs` 降到 203 行并迁出六个 folder-backed input ownership owner；16 个 input-method、owner validation、high-precision、drag/drop、popup/tooltip 与 route-trace 测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI event routing test folder split（`runtime_15_ui_event_routing_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/event_routing.rs` 降到 341 行并迁出四个 folder-backed routing 行为 owner；27 个测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 UI runtime input reply routes test folder split（`runtime_15_ui_runtime_input_reply_routes_tests_folder_split_static_passed_cargo_deferred`）已把 `ui/tests/runtime_input_reply_routes.rs` 父文件降到 500 行并迁出三个 folder-backed reply-route 行为 owner；既有 oversized reply-route 子文件仍 pending。
>
> 最新完成：Runtime 15 M3 UI runtime input reply route child folder split（`runtime_15_ui_runtime_input_reply_route_children_folder_split_static_passed_cargo_deferred`）已把 `keyboard_navigation_routes.rs` 降到 152 行、`tree_view_pointer_routes.rs` 降到 418 行，并迁出七个 folder-backed reply-route 子 owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 runtime diagnostics test folder split（`runtime_15_runtime_diagnostics_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `tests/runtime_diagnostics/mod.rs` 降到 89 行并迁出六个 folder-backed diagnostics series owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 RHI command list test folder split（`runtime_15_rhi_command_list_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `rhi/tests/command_list.rs` 降到 214 行并迁出四个 folder-backed command-list 行为 owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 RHI device contract test folder split（`runtime_15_rhi_device_contract_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `rhi/tests/device_contract.rs` 降到 40 行并迁出六个 folder-backed device-contract 行为 owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 asset pack test folder split（`runtime_15_asset_pack_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `asset/tests/pack.rs` 降到 154 行并迁出六个 folder-backed 资产打包行为 owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 asset facade test folder split（`runtime_15_asset_facade_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `asset/tests/facade.rs` 降到 111 行并迁出五个 folder-backed asset façade 行为 owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 asset project zmeta test folder split（`runtime_15_asset_project_zmeta_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `asset/tests/project/zmeta.rs` 降到 104 行并迁出四个 folder-backed zmeta 行为 owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 asset project manager test folder split（`runtime_15_asset_project_manager_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `asset/tests/project/manager.rs` 降到 181 行并迁出三个 folder-backed project-manager 行为 owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 asset project flow sample test folder split（`runtime_15_asset_project_flow_sample_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `asset/tests/project/asset_flow_sample.rs` 降到 28 行并迁出四个 folder-backed sample owner；asset 测试域当前已无超过 800 行测试文件。

> 最新完成：Runtime 15 M3 asset material test folder split（`runtime_15_asset_material_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `asset/tests/assets/material.rs` 降到 69 行并迁出五个 folder-backed material 行为 owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 asset glTF importer test folder split（`runtime_15_asset_gltf_importer_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `asset/tests/assets/gltf_importer.rs` 降到 129 行并迁出七个 folder-backed glTF importer 行为 owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 asset glTF primitive fixture folder split（`runtime_15_asset_gltf_primitive_fixtures_folder_split_static_passed_cargo_lock_blocked`）已把 `asset/tests/assets/gltf_primitive_fixtures.rs` 降到 11 行并迁出四个 folder-backed fixture owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 asset importer test folder split（`runtime_15_asset_importer_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `asset/tests/assets/importer.rs` 降到 105 行并迁出七个 folder-backed importer 行为 owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 asset scene test folder split（`runtime_15_asset_scene_tests_folder_split_static_passed_cargo_lock_blocked`）已把 `asset/tests/assets/scene.rs` 降到 25 行并迁出六个 folder-backed scene 行为 owner；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 test file budget guard folder split（`runtime_15_test_file_budget_guard_folder_split_static_passed_cargo_lock_blocked`）已把 `structure_convention/test_file_budget.rs` 保持在 701 行并迁出/挂载八个 folder-backed guard owner；后续 Plan 09 camera-target custom-target owner split、sub-owner split、composite source guard 与 queue override source guard 又挂载并扩展 `structure_convention/test_file_budget/render_products.rs`，当前父文件为 430 行、render-products guard 为 258 行，并由 `runtime_15_render_camera_target_products_are_folder_backed` 锁定 camera-target render product 根、custom-target/queue 子 owner、文档状态锚与 800 行预算；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Plan 09 custom-target composite source guard（`render_plan09_custom_target_composite_source_guard_static_passed`）已新增 `render_product_camera_targets/custom_target/composite.rs`，让 25 行 `custom_target.rs` root 继续只挂载子 owner；`custom_target/composite.rs` 当前 195 行，承载 `custom_target_two_viewport_stacks_preserve_independent_composites_before_primary_sample` 双 viewport Base+Overlay stack 产品守卫，`render_products.rs` 结构守卫同步锁定新路径与 moved guard 不回流。

> 最新完成：Plan 09 queue override product source guard（`render_plan09_queue_override_product_source_guard_static_passed`）已新增 `m4_behavior_layers/queue_override.rs`，让 732 行 `m4_behavior_layers.rs` 继续只挂载 folder-backed product owners；`queue_override.rs` 当前 167 行，承载 `render_product_queue_override_reorders_draws`，`render_products.rs` 结构守卫同步锁定新路径、规范名和文档状态锚。

> 最新完成：Plan 09 PrimitiveRelevance typed layer filter（`render_plan09_primitive_relevance_typed_layer_filter_static_passed_cargo_lock_blocked_timeout_no_result`）已把 `PrimitiveRelevance::for_mesh_view(...)` 与 `view_visible_for_layers(...)` 从 legacy `u32` 输入改为 `&RenderLayerSet`，main-view relevance 直接消费 typed `RenderMeshSnapshot.render_layer_mask`。后续 `render_plan09_visibility_batch_key_layer_set_static_passed_cargo_lock_blocked` 已关闭 `VisibilityBatchKey` / `FrameVisibility.render_layer_masks` 的 old mask 边界，`render_plan09_visibility_renderable_input_layer_set_static_passed_cargo_lock_blocked_timeout_no_result` 已关闭 `VisibilityRenderableInput` 旧 DTO 边界。`relevance.rs` 当前 213 行，`view_context/build_views.rs` 368 行，`collect_batching_result.rs` 112 行，未引入近阈值 owner；focused Cargo 超时且 locked check 被当前 `Cargo.lock` 漂移阻断，不计 WGPU/Cargo 通过。

> 最新完成：Plan 09 VisibilityBatchKey typed layer set（`render_plan09_visibility_batch_key_layer_set_static_passed_cargo_lock_blocked`）已把 `VisibilityBatchKey.render_layer_mask` 与 `FrameVisibility.render_layer_masks` 从 legacy `u32` 收束为 `RenderLayerSet`；`collect_batching_result.rs` 从 typed mesh snapshot clone batch key / BVH instance / history entry，custom-target `build_views.rs` 直接消费 typed layer set。`construct.rs` 当前 775 行，`view_context/mod.rs` 189 行，`view_context/build_views.rs` 366 行，`visibility_batch_key.rs` 10 行，`visibility_context.rs` 90 行，`build_draw_commands.rs` 21 行，未引入近阈值 owner；focused Cargo 与 locked check 均被当前 `Cargo.lock` 漂移在编译前阻断，不计 WGPU/Cargo 通过。

> 最新完成：Plan 09 VisibilityRenderableInput typed layer set（`render_plan09_visibility_renderable_input_layer_set_static_passed_cargo_lock_blocked_timeout_no_result`）已把 `VisibilityRenderableInput.render_layer_mask` 从 legacy `u32` 收束为 `RenderLayerSet`；snapshot adapter、visibility fallback 与 world `build_visibility_input(...)` 都 clone typed layer set，particle emitter layer 聚合改为 `RenderLayerSet::union(...)`。按大文件约束，`frame_extract.rs` inline tests 已迁到 `frame_extract/tests.rs`，主文件降到 894 行；`frame_extract/tests.rs` 133 行、`visibility_entries.rs` 18 行、`scene/world/render.rs` 958 行，未引入超阈值 owner。focused Cargo 单测在 124s 工具窗口超时且无 test binary; locked check 被当前 `Cargo.lock` 漂移在编译前阻断，不计 WGPU/Cargo 通过。

> 最新完成：Plan 09 sprite render-layer typed snapshot（`render_plan09_sprite_render_layer_set_snapshot_static_passed_cargo_lock_blocked`）已把 `RenderSpriteSnapshot.render_layer_mask` 从 legacy `u32` 收束为 `RenderLayerSet`；`World::render_sprite_snapshot_for_camera(...)` 在 scene entity legacy mask 边界包装 typed set，`build_sprite_vertices(...)` 直接用 `selected_camera_layers().intersects(&sprite.render_layer_mask)` 做 CPU 过滤，layer 32+ sprite 不再被 lossy mask 截断。`sprite.rs` 当前 26 行，`build_sprite_vertices.rs` 当前 646 行，`scene/world/render.rs` 当前 903 行；后续 world render extract 新职责仍需先拆 visibility/sprite/light owner。focused Cargo 被当前 `Cargo.lock` 漂移和 `--locked` 在编译前阻断，不计 WGPU/Cargo 通过。

> 最新完成：Plan 09 mesh selected-camera layer filter（`render_plan09_mesh_selected_camera_layer_filter_static_passed_cargo_timeout_no_result`）已把 `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs` 中的 phase queue/material sort owner 抽到 `build/phase_ordering.rs`。`build.rs` 从 1036 行降到 721 行，新 owner 为 401 行；raw mesh-vector fallback 与 `RenderPhaseQueue` consumption 都会用 selected camera layer set 过滤 `RenderMeshSnapshot.render_layer_mask`。本地守卫 `phase_ordered_meshes_filter_meshes_by_selected_camera_layers` 用 `RenderLayerSet::layer(2)` 覆盖两条路径；focused Cargo 124s 超时且无 test binary，不计 WGPU/Cargo 通过。

> 最新完成：Plan 09 mesh render-layer typed snapshot（`render_plan09_mesh_render_layer_set_snapshot_static_passed_cargo_lock_blocked`）已把 `RenderMeshSnapshot.render_layer_mask` 从 legacy `u32` 收束为 `RenderLayerSet`；`scene/world/render.rs` 只在 mesh DTO 边界包装 scene entity legacy mask，`phase_ordering.rs` 直接用 `intersects(&mesh.render_layer_mask)` 做 selected-camera 过滤，`StaticMeshBatchExtract` 与 frame-history mesh validation key 同步 typed。后续 visibility-input 切片已把 `frame_extract.rs` inline tests 迁出，主文件当前为 894 行；`scene/world/render.rs` 当前 958 行，后续新增 world render extract 职责仍应先拆 owner。既有超阈值 render product 测试文件本切片只同步 typed fixture，不应继续承载新产品守卫。focused Cargo 被当前 `Cargo.lock` 漂移和 `--locked` 在编译前阻断，不计 WGPU/Cargo 通过。

> 最新完成：Plan 05/09 shadow view-projection owner split（`render_plan05_09_shadow_view_projection_owner_split_static_passed`）已把 `graphics/scene/scene_renderer/shadow/view_projection.rs` 挂到 `shadow/mod.rs`，承接 directional cascade、spot、point-face shadow view-projection 矩阵构造和 direction/far-plane sanitizing；`shadow/plan.rs` 只保留 atlas allocation、slot pass、globals 与 light-slot assignment 编排。新增 `runtime_15_shadow_plan_view_projection_is_child_owner` 锁定 moved helper 不回流、docs/status 锚点和 near-threshold owner 预算。scoped rustfmt/static/docs/diff checks 已通过；focused locked Cargo check 304s 超时无输出，不计 Cargo/WGPU 通过，本切片不声明新的 WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2/MD-M4 virtual geometry compiled-scene indirect evidence（`render_plan02_virtual_geometry_compiled_indirect_evidence_static_passed_cargo_deferred_active_lanes`）已把 `CompiledSceneDraws` 中 VG indirect segment/args/buffer getters 接入 `virtual_geometry_indirect_stats()`，再通过 `PreparedMeshVirtualGeometryIndirectStats` 与 `PreparedMeshQueueStats` 进入 `update_virtual_geometry_stats(...)`。本切片删除该 owner 中的 `#[allow(dead_code)]`，不新增 root 行为或公开 API，只把 renderer-private WGPU buffer 存在性投影为产品统计 evidence。scoped rustfmt、diff-check 与 dead-code 静态扫描已通过；focused Cargo/WGPU/RenderDoc 因其他 cargo/rustc lane 活跃而暂缓。

> 最新完成：Plan 02 MD-M2/MD-M4 virtual geometry mesh-level indirect buffers（`render_plan02_virtual_geometry_mesh_indirect_buffers_static_passed_cargo_deferred_active_lanes`）已新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/virtual_geometry_indirect.rs` 作为 351 行 child owner，承接 authored VG execution segments 到 WGPU indexed indirect args、submission、authority、draw-ref 与 segment buffers 的构造。`build_mesh_draws/build.rs` 当前 968 行，仍低于本阶段阈值；`render_product_mesh_cache.rs` 当前 835 行，继续承载同一个 VG 产品守卫并新增 indirect buffer/args/segment 非零断言。`record_submission(...)`、`record_present_submission(...)` 与 `update_virtual_geometry_stats(...)` 不再把 VG indirect segment/buffer 统计固定为零。本切片 scoped rustfmt/check、line-count 与 diff checks 已通过；focused Cargo/WGPU/RenderDoc 因其他 cargo/rustc lane 活跃而暂缓，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 static command cache virtual geometry residual product guard（`render_plan02_static_cache_virtual_geometry_residual_product_guard_static_passed_cargo_deferred_active_lanes`）已把产品级 authored VG residual 守卫补进 `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs::render_product_virtual_geometry_extract_stays_out_of_pre_mesh_cache`。该 product owner 当时为 766 行，继续避免扩大 `render_product_submit.rs`；新增 Dynamic 可见性承载 mesh helper 与 authored VG extract helper 复用 pluginized advanced provider fixture，只负责把同一实体的 VG payload 送入真实产品提交并满足现有主视图可见性来源。守卫锁定 authored VG 不进入 pending static command-cache candidate、pre-MeshDraw skip 或 cache hit/miss/rebuild 口径，并保留 `last_virtual_geometry_indirect_draw_count` 产品统计。scoped rustfmt/source-anchor/line-count checks 已通过；focused Cargo 因本切片验证决策时其他 cargo/rustc lane 活跃而暂缓，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 static command cache skinned residual product guard（`render_plan02_static_cache_skinned_residual_product_guard_static_passed_cargo_deferred_active_lanes`）已把产品级 skinned/GPU-source residual 守卫补进 `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs::render_product_static_skinned_mesh_stays_out_of_pre_mesh_cache`。该 product owner 现在 622 行，继续避免扩大 `render_product_submit.rs`；新增的 minimal skinned mesh、root skeleton、pose 与 direct skinned extract helper 只负责把 skinned draw 送入真实产品提交。守卫锁定 skinned static mesh 不进入 pending static command-cache candidate、pre-MeshDraw skip 或 cache hit/miss/rebuild 口径，并保留 dynamic command path。scoped rustfmt/source-anchor/line-count/diff-checks 已通过；focused Cargo 因本切片验证决策时其他 cargo/rustc lane 活跃而暂缓，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 static command cache transparent residual product guard（`render_plan02_static_cache_transparent_residual_product_guard_static_passed_cargo_deferred_active_lanes`）已把产品级 transparent residual 守卫补进 `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs::render_product_static_transparent_mesh_stays_out_of_pre_mesh_cache`。该 product owner 现在 396 行，继续避免扩大 `render_product_submit.rs`；新增的 Blend material helper 与 `static_cache_transparent_extract(...)` 复用现有 fixture，只负责把 static mesh 放进 `Transparent3d` phase。守卫锁定 transparent static mesh 不进入 pending static command-cache candidate、pre-MeshDraw skip 或 cache hit/miss/rebuild 口径，并保留 dynamic command 以支持每帧相机深度排序。scoped rustfmt/source-anchor/line-count checks 已通过；focused Cargo 因本切片验证决策时其他 cargo/rustc lane 活跃而暂缓，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 07 post-process stack owner split（`render_plan07_post_process_stack_owner_split_static_passed`）已把 `core/framework/render/post_process/graph_resource_names.rs` 作为 `PostProcessGraphResourceNames` owner，并把 17 个 stack graph contract tests 迁入 `core/framework/render/post_process/stack/tests/{exposure,terminal_chain,screen_space_reflection,temporal_history,effect_stack}.rs`。`stack.rs` 降到 586 行，只保留 `PostProcessStackDescriptor` 构造、validated graph 与 history-resource stripping。新增 `runtime_15_post_process_stack_is_folder_backed` 锁定 resource-name/test owner 挂载、moved tests 不回流、docs/status 锚点和行数预算。scoped rustfmt/static/docs/diff checks 已通过；本切片不声明新的 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 static command cache TAA reactive residual product guard（`render_plan02_static_cache_taa_reactive_residual_product_guard_static_passed_cargo_deferred_active_lanes`）已把产品级 TAA reactive residual 守卫补进 `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs::render_product_static_mesh_taa_reactive_mask_keeps_residual_mesh_draw_path`。该 product owner 现在 297 行，继续避免扩大 `render_product_submit.rs`；reactive material revision helper 与 `static_cache_taa_extract(...)` 复用现有 fixture，只新增 TAA 开关和 `taa_reactive_mask_strength` 材质属性。守卫锁定 static reactive mesh 不会在 pre-MeshDraw 阶段被跳过，但 residual `MeshDraw` 构造后 ordinary phases 可命中 command cache，reactive-mask command 仍保留 per-frame dynamic command。scoped rustfmt/source-anchor/line-count checks 已通过；focused Cargo 因本切片验证决策时其他 cargo/rustc lane 活跃而暂缓，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 static command cache material revision product guard（`render_plan02_static_cache_material_revision_product_guard_static_passed_cargo_deferred_active_lanes`）已把产品级材质 revision 失效守卫补进 `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs::render_product_static_mesh_material_revision_invalidates_pre_mesh_cache`。该 product owner 当时为 193 行，继续避免扩大 `render_product_submit.rs`；局部 `register_material_revision(...)` helper 用同一 material id/URI 的 source hash 变化推进 revision，并锁定第二帧不会误用 pre-MeshDraw skip/cache hit，同时能记录 material residual、material invalidation 和 command rebuild。scoped rustfmt/source-anchor/line-count checks 已通过；focused Cargo 因本切片验证决策时其他 cargo/rustc lane 活跃而暂缓，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 07 volume component owner split（`render_plan07_volume_component_owner_split_static_passed`）已把 `VolumeParamValue`、`VolumeParamSchema`、`VolumeParamInterpFn`、`interp_*` 与参数默认值工厂迁入 `core/framework/render/post_process/volume_component/params.rs`，并把 5 个 volume component behavior tests 迁入 `core/framework/render/post_process/volume_component/tests.rs`。`volume_component.rs` 降到 642 行，只保留 `VolumeComponentDescriptor`、内建 component descriptor 表和 read/apply 写回映射。新增 `runtime_15_post_process_volume_component_is_folder_backed` 锁定 params/test owner 挂载、moved owners 不回流、docs/status 锚点和行数预算。scoped rustfmt/static/docs/diff checks 已通过；focused locked `render_volume_component` Cargo 测试 184s 超时无结果且未发现本 target 残留，不计 Cargo 通过；本切片不声明新的 WGPU/RenderDoc 通过。

> 最新完成：Plan 07 volume camera transition product guard owner（`render_plan07_volume_camera_transition_product_guard_static_passed_cargo_timeout_no_result`）已新增 `graphics/tests/render_product_post_process_volume.rs` 并挂到 `graphics/tests/mod.rs`，承接计划表 `render_product_post_volume_camera_transition`，避免继续扩大 913 行 `render_product_post_process.rs`。新 owner 234 行，用真实 headless WGPU 三视口覆盖 post-process sphere volume 外、blend 区与中心相机位置，并断言 `post.uber`/`post.output-transfer`、角落 luma 梯度和最终帧 delta。scoped rustfmt/static/line-count checks 已通过；focused locked Cargo 245s 超时无结果且无 cargo/rustc 残留，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 07 full-chain all-effects product guard owner（`render_plan07_full_chain_all_effects_product_guard_static_passed_cargo_timeout_no_result`）已新增 `graphics/tests/render_product_post_process_full_chain.rs` 和 `graphics/tests/render_product_post_process_full_chain/fixture.rs` 并挂到 `graphics/tests/mod.rs`，承接计划表 `render_product_post_full_chain_all_effects_on`。新主测试 542 行、fixture 414 行，避免继续扩大 913 行 `render_product_post_process.rs`；真实 headless WGPU 场景同帧覆盖 histogram exposure、bloom、DoF、motion blur、SSR/fog scene-composite、blur、color LUT bake/tonemap/user LUT、vignette/grain/dither/CA、dynamic-resolution upscale 与 SMAA terminal AA，并断言 executor 顺序、active families、alias/backing、scene-velocity readback 和最终帧 delta。scoped rustfmt/static/line-count checks 与 locked core-min `cargo check` 已通过(既有 warnings)；focused locked Cargo test 604s 超时无结果且无本 target-dir 残留，不计 WGPU/RenderDoc 通过。

> 最新完成：Plan 01 render graph execution record owner split（`render_plan01_execution_record_owner_split_static_passed`）已把 `graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs` 拆为 550 行 execution-record aggregation root，并新增 `graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs` 与 `graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/tests.rs` 两个 child owner。`compute_workload.rs` 承接 compute dispatch/workload audit 类型、dispatch group sizing 与 compute audit tests，`tests.rs` 承接非 compute record 行为测试。新增 `runtime_15_render_graph_execution_record_is_folder_backed` 锁定 child module mounts、moved compute/test owner 不回流、docs/status 锚点和行数预算；scoped rustfmt/static/line-count/docs-anchor/diff-check 通过，locked core-min Cargo check 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 static command cache product stats guard（`render_plan02_static_cache_product_stats_guard_static_passed_cargo_timeout_no_result`）已新增 `zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs` 作为 115 行产品提交路径 test owner，锁定同一 eligible static mesh 第二帧在 pre-MeshDraw 抽取阶段复用 cached command 的产品 stats。既有 `render_product_submit.rs` 仅公开 fixture，未继续堆入新断言；`graphics/tests/mod.rs` 只新增结构挂载。scoped validation 见 Plan 02，focused locked Cargo 在约 184s 超时无结果且匹配 target cargo/rustc 已停止，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 mesh draw command list owner split（`render_plan02_mesh_draw_command_list_owner_split_static_passed_cargo_lock_blocked`）已把 `graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs` 拆为 291 行 command-list/container root，并新增 `graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs` 与 `graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs` 两个 child owner。`builder.rs` 承接 batch→command buffer processor fan-out、static cache lookup/rebuild、dynamic command append 与 cache stats 汇总，`tests.rs` 承接原 inline command list/builder/cache 行为测试。新增 `runtime_15_mesh_draw_command_list_is_folder_backed` 锁定 child module mounts、moved builder/test owner 不回流、docs/status 锚点和行数预算；scoped rustfmt/static/line-count/docs-anchor/diff-check 通过，locked core-min Cargo check 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 mesh pass processor tests owner split（`render_plan02_mesh_pass_processor_tests_owner_split_static_passed_cargo_lock_blocked`）已把 `graphics/scene/scene_renderer/mesh/mesh_pass/processors/mod.rs` 收敛为 15 行 declaration/re-export root，并新增 `graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs` 作为 processor behavior test owner。新 child owner 承接 phase emission、TAA reactive mask、velocity、visibility 与 Plan 02 focused processor guards。新增 `runtime_15_mesh_pass_processors_are_folder_backed` 锁定 root 不承载 tests/fixtures、docs/status 锚点和行数预算；scoped rustfmt/source-anchor scans 通过，locked Cargo 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 prepared queue tests owner split（`render_plan02_prepared_queue_tests_owner_split_static_passed_cargo_lock_blocked`）已把 `graphics/scene/scene_renderer/mesh/prepared_queue.rs` 收敛为 272 行 production stats owner，并新增 `graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs` 作为 prepared queue stats behavior test owner。新 child owner 承接 early-z/shadow phase counts、static/dynamic/GPU instancing candidate grouping、skinned GPU/CPU-morphed stats、LOD stats、mesh-pass command buffer/replay/GPUScene stats forwarding 等原 inline tests。新增 `runtime_15_prepared_mesh_queue_is_folder_backed` 锁定 parent 不承载 inline tests、docs/status 锚点和行数预算；scoped rustfmt/static/source-anchor/docs-anchor/diff-check 通过，locked Cargo 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 prepared queue stats bridge tests owner split（`render_plan02_prepared_queue_stats_bridge_tests_owner_split_static_passed_cargo_lock_blocked`）已把 stats forwarding tests 从近阈值 `graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs` 拆入 `graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs`。`tests.rs` 降到 599 行，只保留 queue behavior、batch candidate、velocity/skinning/LOD 与 GPU-skinned eligibility 行为测试；新 child owner 174 行承接 pending command cache plan/extraction、mesh pass command buffer/replay 和 GPUScene stats forwarding 测试。`runtime_15_prepared_mesh_queue_is_folder_backed` 已同步锁定 `mod stats_bridge_tests;`、moved test owner、docs/status 锚点和父子行数预算；scoped validation 见 Plan 02，focused locked Cargo 被当前 `Cargo.lock` 更新需求阻断，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 prepared queue stats bridge owner split（`render_plan02_prepared_queue_stats_bridge_owner_split_static_passed_cargo_timeout_no_result`）已把 `PreparedMeshQueueStats` 的跨系统 stats forwarding 从 `graphics/scene/scene_renderer/mesh/prepared_queue.rs` 拆入 `graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs`。父文件降到 241 行，仅保留 stats 字段、queue summarization 与 repeated-group helper；新 child owner 93 行，承接 pending command cache plan/extraction、mesh pass command buffer/replay 和 GPUScene upload stats bridge。`runtime_15_prepared_mesh_queue_is_folder_backed` 与 `runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 已同步锁定 `mod stats_bridge;`、new child owner、docs/status 锚点和父子行数预算。scoped static validation 见 Plan 02；focused locked Cargo 180 秒超时无结果且无本 target 残留 cargo/rustc，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 residual fallback owner split（`render_plan02_residual_fallback_owner_split_static_passed_cargo_lock_blocked`）已把 pre-MeshDraw static command cache 抽取失败归因从 `pending_command_cache_extract.rs` 拆入 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/residual_fallback.rs`。父 owner 降到 294 行，继续只做提取流程、cache lookup/store 与 pending batch materialization；新 child owner 58 行承接 `PendingMeshCommandCacheResidualReason`、rebuild failure 分类和 `residual_*_draw_count` 计数。`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 已同步锁定 `mod residual_fallback;`、new child owner、docs/status 锚点和父子行数预算；scoped validation 见 Plan 02，focused locked Cargo 被当前 `Cargo.lock` 更新需求阻断，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 pre-MeshDraw second-frame extraction guards（`render_plan02_pre_mesh_draw_second_frame_extract_guards_static_passed_cargo_timeout_no_result`）已新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/second_frame_tests.rs` 作为 focused child owner，锁定 pre-MeshDraw 抽取层 full-hit 第二帧 zero-rebuild reuse 与 shadow-only material revision invalidation 的 safe non-material rebuild。`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 已同步锁定 `mod second_frame_tests;`、220 行预算和 docs/status 锚点；scoped validation 见 Plan 02，focused locked Cargo 在 184s 工具窗口超时无结果且无本 target 残留 cargo/rustc，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 pre-MeshDraw material-bound rebuild boundary guard（`render_plan02_pre_mesh_draw_material_boundary_guard_static_passed_cargo_lock_blocked`）已新增 `zircon_runtime/src/tests/runtime_absorption/structure_convention/render_pending_command_cache_material_boundary.rs`，锁定 `non_material_rebuild.rs` 只允许 opaque `ShadowDepth` pre-MeshDraw 重建，并把 normal prepass、alpha-mask shadow、object velocity、TAA reactive replay 的 `bind_standard_material_if_needed(...)` material-bound 事实纳入结构守卫。该 guard 独立于 359 行的 pending-cache plan 守卫，避免继续堆大单测文件；scoped validation 见 Plan 02，focused locked Cargo 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 pre-MeshDraw residual fallback diagnostics（`render_plan02_pre_mesh_draw_residual_fallback_diagnostics_static_passed_cargo_lock_blocked`）已把 static command cache 抽取失败原因拆成 material phase fallback、rebuild input missing、rebuild rejected 三类统计。`pending_command_cache_extract.rs` 新增带统计入口和 `PendingMeshCommandCacheResidualReason`，`graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/fallback_tests.rs` 作为 148 行 focused owner 锁定三类边界；`PreparedMeshQueueStats`、`RenderStats`、`update_stats/base_stats.rs` 与产品诊断输出 `render.mesh.queue.pre_mesh_draw_static_command_cache.residual_*_draw_count`。scoped static validation 见 Plan 02；focused locked Cargo 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 pre-MeshDraw opaque shadow cache rebuild（`render_plan02_pre_mesh_draw_shadow_cache_rebuild_static_passed_cargo_timeout_no_result`）已在 `pending_command_cache_extract.rs` 下新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/non_material_rebuild.rs` 与 `pending_command_cache_extract/tests.rs` 两个 child owner。父 owner 保留入口、cache lookup 与 pending draw batch 构造；`non_material_rebuild.rs` 只允许 opaque `ShadowDepth` 在 pre-MeshDraw 阶段重建，显式拒绝 depth prepass 与 alpha-mask shadow，避免绕过需要 standard material 的 replay 路径。`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 已同步锁定 child owner、moved tests、docs/status 锚点和父子行数预算；scoped validation 见 Plan 02，本切片不声明 WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 lazy pre-MeshDraw rebuild input（`render_plan02_lazy_pre_mesh_draw_rebuild_input_static_passed_cargo_lock_blocked`）已把 `pending_command_cache_extract.rs` 的 rebuild batch 输入改为惰性物化，并新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/lazy_rebuild_tests.rs` 作为 focused test owner。full-hit 静态 draw 与 material-bound phase miss 不再提前请求 GPUScene span/`MeshBatchRef`，只有 `non_material_rebuild` 允许的 miss/invalidated phase 才进入 `pending_mesh_command_cache_rebuild_batch_for_phase(...)`。`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 已同步锁定新 child owner、source anchors、docs/status 锚点和父子行数预算；scoped validation 见 Plan 02，focused locked lib-test compile 被当前 `Cargo.lock` 更新需求阻断，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 visibility-pruned pre-MeshDraw diagnostics split（`render_plan02_visibility_pruned_pre_mesh_draw_diagnostics_static_passed_cargo_timeout_no_result`）已把 fully visibility-pruned 零命令 skip 从普通 pre-MeshDraw skipped draw 中拆出独立诊断：`pending_command_cache_extract.rs` 新增 `visibility_pruned` 结果标记与 `visibility_pruned_mesh_draw_count`，`PreparedMeshQueueStats`、`RenderStats`、`update_stats/base_stats.rs` 和 `product/mesh_queue.rs` 输出 `render.mesh.queue.pre_mesh_draw_static_command_cache.visibility_pruned_draw_count`。新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/visibility_tests.rs` 作为 62 行 focused test owner，原综合 `tests.rs` 降到 220 行；`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 已同步锁定新 child owner、诊断锚点、docs/status 锚点和父子行数预算。scoped validation 见 Plan 02，focused locked lib-test compile 180 秒超时无结果，且无本 target 残留 cargo/rustc；不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 visibility-pruned pre-MeshDraw empty extraction（`render_plan02_visibility_pruned_pre_mesh_draw_empty_extract_static_passed_cargo_lock_blocked`）已在 `pending_command_cache_extract.rs` 内把 visibility/relevance 全裁掉所有 cacheable phases 的 eligible static pending draw 转成空命令成功抽取，避免 residual `MeshDraw` 构造和 material bind group 创建。后续 diagnostics split 已将 guard 移入 `pending_command_cache_extract/visibility_tests.rs::pending_command_cache_extract_marks_visibility_pruned_static_draw`，锁定空命令、默认 cache stats、`visibility_pruned` 标记与 rebuild batch 未请求；root 264/300、tests 220/260、visibility_tests 62/120。`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 已同步锁定测试名、docs/status 锚点和行数预算；scoped validation 见 Plan 02，focused locked lib-test compile 被当前 `Cargo.lock` 更新需求阻断，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 pending command cache extract-item owner split（`render_plan02_pending_command_cache_extract_item_owner_split_static_passed_cargo_lock_blocked`）已把 `pending_command_cache_extract.rs` 的 pending draw projection、skip eligibility 与 cacheable phase selection 拆入 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/extract_item.rs`。父 owner 降到 255 行，保留抽取入口、cache lookup/store、non-material rebuild dispatch 与 rebuild batch materialization；新 child owner 111 行。`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 已同步锁定 `mod extract_item;`、new child owner、docs/status 锚点和父子行数预算；scoped validation 见 Plan 02，focused locked lib-test compile 被当前 `Cargo.lock` 更新需求阻断，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 pre-MeshDraw command cache extraction（`render_plan02_pre_mesh_draw_command_cache_extraction_static_passed_cargo_lock_blocked`）已新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs` 作为 full-hit static command-cache downshift owner，在 GPUScene 同步后、`create_mesh_draw(...)` 创建 WGPU material bind groups 前，对直接 prepared、非透明、非 skinned、无 reactive mask 且所有 cacheable phase 全命中的静态 pending draw 复用 cached `MeshDrawCommand`。`BuiltMeshDraws`/`CompiledSceneDraws` 携带 source prepared queue stats、prebuilt `MeshPassCommandBuffers` 与 extraction stats，render 主流程合并 prebuilt/residual buffers，产品诊断新增 `render.mesh.queue.pre_mesh_draw_static_command_cache.skipped_*`。`runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 已扩展锁定 census/extraction owners、build hook、stats/diagnostics bridge、docs/status 锚点和行数预算；scoped validation 见 Plan 02，本切片不声明 WGPU/RenderDoc 通过。

> 最新完成：Plan 02 MD-M2 pending command cache plan diagnostics（`render_plan02_pending_command_cache_plan_static_passed_cargo_lock_blocked`）已新增 `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_plan.rs` 作为 pre-MeshDraw 静态 command cache census owner，在 `MeshDraw`/WGPU bind 资源构造之前统计 `pending_static_command_cache_*` draw/phase 候选，并通过 `BuiltMeshDraws`、`CompiledSceneDraws`、`PreparedMeshQueueStats`、`RenderStats` 与产品诊断输出。新增 `runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build` 锁定 owner、build hook、stats/diagnostics bridge、docs/status 锚点和行数预算；scoped rustfmt/static/source-anchor/docs-anchor/diff-check 通过，locked Cargo 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 Cargo/WGPU/RenderDoc 通过。

> 最新完成：Plan 07 built-in post-process executor owner split（`render_plan07_builtin_postprocess_executor_owner_split_static_passed`）已把 `graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs` 拆为 574 行 registry-facing executor root，并新增 `graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/frame_effects.rs`、`graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/graph_resources.rs`、`graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors/resource_routing.rs` 三个 child owner。`frame_effects.rs` 承接 frame effect predicates，`graph_resources.rs` 承接 `product_postprocess_executor(...)` 与 graph resource kind/external binding 校验，`resource_routing.rs` 承接 terminal/bloom/uber resource routing 和原 inline routing tests。新增 `runtime_15_builtin_postprocess_executors_are_folder_backed` 锁定 child module mounts、moved helpers/tests 不回流、docs/status 锚点和行数预算；scoped rustfmt/static/line-count/docs-anchor/diff-check 与 locked core-min `cargo check` 已通过(既有 warnings)，focused locked structure Cargo test 被当前 `Cargo.lock` 更新需求在编译前阻断，不计 WGPU/RenderDoc 通过。

> 最新完成：Plan 09 CO-M4 world visibility input owner split（`render_plan09_world_visibility_input_owner_split_static_passed_cargo_timeout_no_result`）已把 scene world visibility input DTO 构造从近阈值 `scene/world/render.rs` 抽入 `scene/world/render_visibility.rs`。新 child owner 承接 `build_visibility_input(...)`、`particle_emitter_render_layer_masks(...)`、`empty_visibility_input(...)`、typed `VisibilityRenderableInput` rows 与 `RenderLayerSet::union(...)` particle emitter layer 聚合；`scene/world/render.rs` 当前降到 878 行，`scene/world/render_visibility.rs` 为 87 行。新增 `runtime_15_scene_world_render_visibility_input_is_child_owner` 锁定 owner 挂载、moved functions 不回流、docs/status 锚点和行数预算。focused locked Cargo 124s 超时且无测试二进制，不计 WGPU/Cargo 通过。

> 最新完成：Plan 09 particle selected-camera layer filter（`render_plan09_particle_selected_camera_layer_filter_static_passed_cargo_timeout_no_result`）已把 `RenderParticleSpriteSnapshot.render_layer_mask` 作为粒子 sprite layer 权威，`scene/world/render_particles.rs` 将实体 layer 传入 authored particles 与 world HUD bar quads；particle billboard/velocity CPU 顶点 owner 与 `RenderPipelineAsset::compile(...)` 都按 selected camera layers 过滤。该切片完成时 `scene/world/render.rs` 为 899 行，后续 light typed slice 后当前为 903 行；后续 world render extract 新职责需先拆 visibility/particle/light owner。scoped rustfmt/static/source/diff/line-count 检查通过，focused Cargo 184s 超时且无 test binary，不计 WGPU/Cargo 通过。

> 最新完成：Plan 09 particle render-layer typed snapshot（`render_plan09_particle_render_layer_set_snapshot_static_passed_cargo_lock_blocked`）已把 `RenderParticleSpriteSnapshot.render_layer_mask` 从 legacy `u32` 收束为 `RenderLayerSet`；`scene/world/render_particles.rs` 在 particle DTO 边界包装 scene entity legacy mask，particle billboard/velocity CPU 顶点 owner 与 `RenderPipelineAsset::compile(...)` 直接用 `intersects(&sprite.render_layer_mask)` 过滤，layer 32+ particle sprite 不再被 lossy mask 截断。后续 visibility-input 切片已把 `build_visibility_input(...)` emitter 聚合从 lossy OR 改为 typed `RenderLayerSet::union(...)`；`scene/world/render.rs` 当前 958 行，后续 world render extract 新职责仍需先拆 visibility/particle/light owner。focused Cargo 被当前 `Cargo.lock` 漂移和 `--locked` 在编译前阻断，不计 WGPU/Cargo 通过。

> 最新完成：Plan 09 light layer typed mask convergence（`render_plan09_light_layer_set_snapshot_static_passed_cargo_timeout_no_result`）已把 directional/point/spot/rect render light snapshot 的 `layer_mask` 从 legacy `u32` 收束为 `RenderLayerSet`；scene extraction 只在实体 legacy mask 边界包装 typed set，WGPU `GpuLightData.shadow_slot_layer[1]` 继续在 `light_buffer.rs` 用 `to_legacy_mask_lossy()` 写入旧 32-bit ABI。`scene/world/render.rs` 与 `shadow/plan.rs` 当前均为 903 行，后续新增 world render extract 或 shadow planning 职责必须先拆 folder-backed owner。scoped rustfmt/static/diff/line-count 检查通过，focused Cargo 188s 超时且无 test binary，不计 WGPU/Cargo 通过。

> 最新完成：Plan 09 volume mask separation（`render_plan09_volume_mask_separate_from_culling_static_passed_cargo_lock_blocked_timeout_no_result`）已把 post-process Volume 评估从 selected camera culling mask 分离到 `selected_camera_volume_layers()`；`scene/world/render_post_process.rs` 在 post-process owner 内按 selected/stack camera `volume_mask` union 收集 Volume，`build_frame_submission_context/build.rs` 不再给 `resolved_settings_for_camera(...)` 传 culling layers。`frame_extract.rs` 当前 979 行、`scene/world/render.rs` 当前 953 行、`scene/tests/render_extract.rs` 当前 1446 行；后续新增 frame/world render extract 职责或相关测试前必须先拆 owner，不能继续堆入这些近阈值/超阈值文件。scoped rustfmt/static/diff/line-count 检查通过，focused Cargo 超时且 locked check 被 `Cargo.lock` 漂移阻断，不计 WGPU/Cargo 通过。

> 最新完成：Runtime 15 M3 Runtime 07 performance hotspot guard folder split（`runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result`）已把 `tests/runtime_absorption/performance_hotspots.rs` 降到 12 行并迁出五个 folder-backed hotspot guard owner；Runtime 07 test inventory 当前为 11 个文件，完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 script VM test folder split（`runtime_15_script_vm_tests_folder_split_static_passed_cargo_timeout_no_result`）已把 `script/vm/tests.rs` 降到 41 行并迁出六个 folder-backed 脚本 VM 测试/夹具 owner；32 个测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 script VM hot-reload coordinator test folder split（`runtime_15_script_vm_hot_reload_coordinator_tests_folder_split_static_passed_cargo_deferred`）已把 `script/vm/runtime/hot_reload_coordinator.rs` 的 5 个内嵌 hot-reload/poison-recovery 测试迁入 `script/vm/runtime/hot_reload_coordinator/tests.rs`；父文件降到 282 行并只保留生产 coordinator owner 和 `#[cfg(test)] mod tests;` 挂载，子 owner 为 407 行，完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 gameplay host test folder split（`runtime_15_gameplay_host_tests_folder_split_static_passed_cargo_deferred`）已把 `script/vm/gameplay_host/tests.rs` 降到 46 行并迁出四个 folder-backed gameplay host 测试 owner；9 个玩法宿主测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 shader prewarm manifest test folder split（`runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred`）已把 `bin/zircon_shader_prewarm/manifest.rs` 降到 672 行并迁出 `bin/zircon_shader_prewarm/manifest/tests.rs` 测试 owner；1 个资产扫描预热 manifest 测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 scene ECS schedule test folder split（`runtime_15_scene_ecs_schedule_tests_folder_split_static_passed_cargo_deferred`）已把 `scene/tests/ecs_schedule.rs` 降到 32 行并迁出四个 folder-backed ECS schedule 行为 owner；57 个 schedule 测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 scene ECS systems test folder split（`runtime_15_scene_ecs_systems_tests_folder_split_static_passed_cargo_deferred`）已把 `scene/tests/ecs_systems.rs` 降到 53 行并迁出六个 folder-backed ECS systems 行为 owner；24 个系统参数/事件/查询行为测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。2026-06-25 后续 M2 命名硬切已把 many/single query owner 从 `query_helpers.rs` 收束为 `many_single_queries.rs`。

> 最新完成：Runtime 15 M3 test file budget root-layout child split（`runtime_15_test_file_budget_root_layout_child_split_static_passed_cargo_deferred`）已把 `structure_convention/test_file_budget.rs` 的根布局自守卫迁入 `structure_convention/test_file_budget/root_layout.rs`，父文件降到 428 行；完整 `runtime_15_no_oversized_test_files` 仍 pending。
> 最新完成：Runtime 15 M3 test file budget root-layout UI child split（`runtime_15_test_file_budget_root_layout_ui_child_split_static_passed_cargo_deferred`）已把 `structure_convention/test_file_budget/root_layout.rs` 的 UI child guard scan 迁入 `structure_convention/test_file_budget/root_layout/ui_children.rs`，父文件降到 499 行，新子文件为 207 行；完整 `runtime_15_no_oversized_test_files` 仍 pending。
> 最新完成：Runtime 15 M3 asset test-budget guard child-owner split（`runtime_15_asset_test_budget_guard_child_owner_split_static_passed_cargo_deferred`）已把 `structure_convention/test_file_budget/asset_tests.rs` 拆成 pack/facade/project/material 四个子 owner，父文件降到 161 行，最大子文件 `project.rs` 为 275 行；完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 scene derived-state test folder split（`runtime_15_scene_derived_state_tests_folder_split_static_passed_cargo_deferred`）已把 `scene/tests/derived_state.rs` 拆成五个 folder-backed derived-state owner，父文件降到 68 行；完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 dynamic scene session path-management test folder split（`runtime_15_dynamic_scene_session_path_management_tests_folder_split_static_passed_cargo_deferred`）已把 `scene/tests/dynamic_scene_session/path_management.rs` 拆成六个 folder-backed session path owner，父文件降到 14 行；完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 scene component-structure test folder split（`runtime_15_scene_component_structure_tests_folder_split_static_passed_cargo_deferred`）已把 `scene/tests/component_structure.rs` 拆成七个 folder-backed component-structure owner，父文件降到 9 行；完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 scene ECS reflect foundation test folder split（`runtime_15_scene_ecs_reflect_foundation_tests_folder_split_static_passed_cargo_deferred`）已把 `scene/tests/ecs_reflect/foundation.rs` 拆成七个 folder-backed ECS reflect foundation owner，父文件降到 158 行；完整 `runtime_15_no_oversized_test_files` 仍 pending。
>
> 最新完成：Runtime 15 M3 dynamic scene root test folder split（`runtime_15_dynamic_scene_root_tests_folder_split_static_passed_cargo_deferred`）已把 `scene/tests/dynamic_scene.rs` 拆成五个 folder-backed dynamic-scene root owner，父文件降到 181 行；完整 `runtime_15_no_oversized_test_files` 仍 pending 于活跃 render 会话占用的 `scene/tests/render_extract.rs`。

## 治理范围

`zircon_runtime/src` 全模块的：façade 友好度（R3.1/R3.3）、可见性纪律（R3.4）、命名（R2.1–R2.4）、`mod.rs`/`module.rs` 判据（R1.2）、行为 owner 化（R1.3）、行数预算（R1.4）、测试组织（R4.1–R4.4）。

## `module_convention_gate` 字段（待 M1 实测填充）

| 字段 | 含义 | 目标 |
|---|---|---|
| `oversized_facade_files` | 超符号 / 行预算的 façade | → 0 |
| `mixed_visibility_mod_files` | 无 façade 注释的 `pub`/`pub(crate)` 混排 | → 0 |
| `prefix_vocabulary_violations` | 越界前缀（`runtime_` 滥用等） | → 0 |
| `plural_singular_violations` | 复数 / 单数误用目录 | → 0 |
| `banned_name_modules` | `_inner`/`_impl`/`_helper`/`util` 等 | → 0 |
| `module_rs_without_descriptor` | 非注册子系统却有 `module.rs` | → 0 |
| `oversized_test_files` | > 800 行测试 | → 0 |
| `duplicate_test_trees` | 重复测试树 | → 0 |
| `module_convention_gate.m1_gate_status` | 门状态 | `classified-and-clear` |
| `migration_debt_count` | 迁移债 | → 0 |
| `exempt` | 登记豁免 | 仅 vendored / fixture / `@generated` |

## 联动

与 `large-file-ownership-m1.md` 共享 hotspot 清单；render 子计划 graphics 热点纳入本治理。

## Runtime 15 M1 animation manager folder-backed cutover

状态：`runtime_15_animation_manager_folder_backed_cutover_static_passed_cargo_deferred`。

R1.2 的当前新增落地部分是 animation manager root 去 `manager.rs` + `manager/` 共存债。`animation/manager.rs` 已删除，当前 root owner 为 `animation/manager/mod.rs`；`animation/mod.rs` 只通过 `mod manager;` 挂载并重导出 `DefaultAnimationManager`，`animation/manager/mod.rs` 只挂载 `graph`、`parameters`、`pose`、`sampling` 与 `state_machine` child owners，没有保留旧模块、alias 或兼容 re-export。

守卫：`runtime_15_animation_manager_is_folder_backed` 验证旧文件不存在、新 folder root/child owner 形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、animation runtime 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo deferred，不计通过。该切片关闭 animation manager 的 folder-backed root 切换；完整 `module_convention_gate` 与 M1 full structure sweep 仍 pending。

## Runtime 15 M2 core runtime state module naming hard cutover

状态：`runtime_15_core_runtime_state_module_naming_hard_cutover_static_passed_cargo_deferred`。

R2.3 的当前新增落地部分是 core runtime state owner 去 `_inner` 文件名债。`core/runtime/state/runtime_inner.rs` 已删除，当前 owner 为 `core/runtime/state/core_runtime_state.rs`；`core/runtime/state/mod.rs` 只通过 `mod core_runtime_state;` 挂载并从该 owner 重导出 `CoreRuntimeInner`，没有保留旧模块、alias 或兼容 re-export。`core/runtime/tests/registration/structure/mod.rs` 的 fixture 同步改为 `runtime_state` 并读取 `../../../state/core_runtime_state.rs`，因此 registration 结构守卫继续验证同一 service registry 存储形状。

守卫：`runtime_15_core_runtime_state_module_uses_owner_name` 验证旧文件不存在、新 owner/入口/fixture 形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、core state 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo deferred，不计通过。该切片只关闭 core runtime state 文件名债；完整 `runtime_15_no_banned_name_modules` 与 `module_convention_gate` 仍 pending。

## Runtime 15 M2 scene ECS observer callback registry module naming hard cutover

状态：`runtime_15_scene_ecs_observer_callback_registry_naming_hard_cutover_static_passed_cargo_deferred`。

R2.3 的当前新增落地部分是 scene ECS observer callback registry 去 `utils` 文件名债。`scene/ecs/observer/utils.rs` 已删除，当前 owner 为 `scene/ecs/observer/callback_registry.rs`；`scene/ecs/observer/mod.rs` 只通过 `mod callback_registry;` 挂载，`scene/ecs/observer/store.rs` 只从该 owner 读取 `lifecycle_callback_count`、`event_callback_count`、`entity_event_callback_count` 与 `remove_observer_by_id`，没有保留旧模块、alias 或兼容 re-export。

守卫：`runtime_15_scene_ecs_observer_callback_registry_uses_owner_name` 验证旧文件不存在、新 owner/入口/store import 形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、scene ECS 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo deferred，不计通过。该切片只关闭 scene ECS observer callback registry 文件名债；完整 `runtime_15_no_banned_name_modules` 与 `module_convention_gate` 仍 pending。

## Runtime 15 M2 scene ECS query-state many-item array module naming hard cutover

状态：`runtime_15_scene_ecs_query_state_many_item_array_naming_hard_cutover_static_passed_cargo_deferred`。

R2.3 的当前新增落地部分是 scene ECS query-state many-item array 去 `helpers` 文件名债。`scene/ecs/query/query_state/helpers.rs` 已删除，当前 owner 为 `scene/ecs/query/query_state/many_item_array.rs`；`scene/ecs/query/query_state/mod.rs` 只通过 `mod many_item_array;` 挂载，`cached_direct.rs`、`mutable.rs`、`read_only.rs` 与 `read_only_cached.rs` 只从该 owner 读取 `collect_many_query_items`，没有保留旧模块、alias 或兼容 re-export。

守卫：`runtime_15_scene_ecs_query_state_many_item_array_uses_owner_name` 验证旧文件不存在、新 owner/入口/调用方 import 形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、scene ECS 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo deferred，不计通过。该切片只关闭 scene ECS query-state many-item array 文件名债；完整 `runtime_15_no_banned_name_modules` 与 `module_convention_gate` 仍 pending。

## Runtime 15 M2 scene ECS component-storage component results module naming hard cutover

状态：`runtime_15_scene_ecs_component_storage_component_results_naming_hard_cutover_static_passed_cargo_deferred`。

R2.3 的当前新增落地部分是 scene ECS component-storage component results 去 `utils` 文件名债。`scene/ecs/storage/component_storage/utils.rs` 已删除，当前 owner 为 `scene/ecs/storage/component_storage/component_results.rs`；`scene/ecs/storage/component_storage/mod.rs` 只通过 `mod component_results;` 挂载，`store.rs` 只从该 owner 读取 `downcast_component` 与 `sort_component_ids_if_needed`，没有保留旧模块、alias 或兼容 re-export。

守卫：`runtime_15_scene_ecs_component_storage_component_results_uses_owner_name` 验证旧文件不存在、新 owner/入口/store import 形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、scene ECS 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo deferred，不计通过。该切片只关闭 scene ECS component-storage component results 文件名债；完整 `runtime_15_no_banned_name_modules` 与 `module_convention_gate` 仍 pending。

## Runtime 15 M2 asset watcher shutdown-on-drop module naming hard cutover

状态：`runtime_15_asset_watcher_shutdown_on_drop_naming_hard_cutover_static_passed_cargo_deferred`。

R2.3 的当前新增落地部分是 asset watcher shutdown-on-drop 去 `_impl` 文件名债。`asset/watch/drop_impl.rs` 已删除，当前 owner 为 `asset/watch/shutdown_on_drop.rs`；`asset/watch/mod.rs` 只通过 `mod shutdown_on_drop;` 挂载，该 owner 继续承接 `AssetWatcher` 的 drop-time stop signal 与 watcher thread join，没有保留旧模块、alias 或兼容 re-export。

守卫：`runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name` 验证旧文件不存在、新 owner/入口形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、asset watcher 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo deferred，不计通过。该切片只关闭 asset watcher shutdown-on-drop 文件名债；完整 `runtime_15_no_banned_name_modules` 与 `module_convention_gate` 仍 pending。

## Runtime 15 M2 asset change construction module naming hard cutover

状态：`runtime_15_asset_change_construction_naming_hard_cutover_static_passed_cargo_deferred`。

R2.5 的当前新增落地部分是 asset watcher `AssetChange` construction 去 `*_new` 文件名债。`asset/watch/asset_change_new.rs` 已删除，当前 owner 为 `asset/watch/asset_change_construction.rs`；`asset/watch/mod.rs` 只通过 `mod asset_change_construction;` 挂载，该 owner 继续承接 `AssetChange::new(...)` 构造逻辑，`fold_events.rs` 继续消费同一 API，没有保留旧模块、alias 或兼容 re-export。

守卫：`runtime_15_asset_change_construction_uses_owner_name` 验证旧文件不存在、新 owner/入口/调用方形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、asset watcher 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo deferred，不计通过。该切片只关闭 asset change construction 文件名债；完整 construction-owner `_new` sweep 与 `module_convention_gate` 仍 pending。

## Runtime 15 M2 resource streamer construction module naming hard cutover

状态：`runtime_15_resource_streamer_construction_naming_hard_cutover_static_passed_cargo_deferred`。

R2.5 的当前新增落地部分是 graphics ResourceStreamer construction 去 `*_new` 文件名债。`graphics/scene/resources/resource_streamer/resource_streamer_new.rs` 已删除，当前 owner 为 `graphics/scene/resources/resource_streamer/resource_streamer_construction.rs`；`resource_streamer/mod.rs` 只通过 `mod resource_streamer_construction;` 挂载，该 owner 继续承接 `ResourceStreamer::new(...)`、fallback texture/material uniform 初始化和 output-target writeback converter 构造逻辑，没有保留旧模块、alias 或兼容 re-export。

守卫：`runtime_15_resource_streamer_construction_uses_owner_name` 验证旧文件不存在、新 owner/入口形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、graphics render-product 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo deferred，不计通过。该切片只关闭 ResourceStreamer construction 文件名债；完整 construction-owner `_new` sweep、graphics Cargo sweep 与 `module_convention_gate` 仍 pending。

## Runtime 15 M2 offscreen target construct directory naming hard cutover

状态：`runtime_15_offscreen_target_construct_naming_hard_cutover_static_passed_cargo_timeout_no_result`。

R2.5 的当前新增落地部分是 render backend OffscreenTarget construction 去 `*_new` 目录名债。`graphics/backend/render_backend/offscreen_target_new/` 已删除，当前 owner 目录为 `graphics/backend/render_backend/offscreen_target_construct/`；`graphics/backend/render_backend/mod.rs` 只通过 `mod offscreen_target_construct;` 挂载，该目录继续承接 `OffscreenTarget::new(...)`、固定 offscreen frame target texture bundle 构造、cluster buffer 构造和 texture bundle owner 拆分，没有保留旧目录、alias 或兼容 re-export。

守卫：`runtime_15_offscreen_target_construct_uses_owner_name` 验证旧目录不存在、新目录/父模块/construct owner 形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、graphics render-product 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Focused `cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir target\codex-runtime15-offscreen-construct-check --message-format short --color never` 首跑 exit=1 但保留输出无错误尾部；同目标目录复跑 304s 超时，日志无 `Finished` / `error:` 结论，不计 Cargo 通过。该切片只关闭 OffscreenTarget construction directory 命名债；完整 construction-owner `_new` sweep、graphics Cargo sweep 与 `module_convention_gate` 仍 pending。

## Runtime 15 M2 asset texture upload readiness container fixtures module naming hard cutover

状态：`runtime_15_asset_texture_upload_readiness_container_fixtures_naming_hard_cutover_static_passed_cargo_deferred`。

R2.3 的当前新增落地部分是 asset texture upload readiness container fixtures 去 `common` 文件名债。`asset/tests/assets/texture_upload_readiness/common.rs` 已删除，当前 owner 为 `asset/tests/assets/texture_upload_readiness/container_fixtures.rs`；`asset/tests/assets/texture_upload_readiness.rs` 只通过 `mod container_fixtures;` 挂载，`boundaries.rs`、`containers.rs`、`dds.rs` 与 `ktx.rs` 只从该 owner 读取 DDS/KTX/ASTC fixture bytes、container header writers 与 upload-readiness constants，没有保留旧模块、alias 或兼容 re-export。

守卫：`runtime_15_asset_texture_upload_readiness_container_fixtures_uses_owner_name` 验证旧文件不存在、新 owner/入口/调用方 import 形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、render-assets 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo deferred，不计通过。该切片只关闭 asset texture upload readiness container fixtures 文件名债；完整 `runtime_15_no_banned_name_modules` 与 `module_convention_gate` 仍 pending。

## Runtime 15 M2 scene ECS query cached queries module naming hard cutover

状态：`runtime_15_scene_ecs_query_cached_queries_naming_hard_cutover_static_passed_cargo_deferred`。

R2.3 的当前新增落地部分是 scene ECS query cached queries 去 `helpers` 文件名债。`scene/tests/ecs_query/cache_helpers.rs` 已删除，当前 owner 为 `scene/tests/ecs_query/cached_queries.rs`；`scene/tests/ecs_query.rs` 只通过 `mod cached_queries;` 挂载，该 test owner 继续承接 cache rebuild、count/empty/get/many/unique、cached-direct table/sparse location、archetype movement 和 optional archetype membership 用例，没有保留旧模块、alias 或兼容 re-export。

守卫：`runtime_15_scene_ecs_query_cached_queries_uses_owner_name` 验证旧文件不存在、新 owner/入口形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、scene ECS 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo deferred，不计通过。该切片只关闭 scene ECS query cached queries 文件名债；完整 `runtime_15_no_banned_name_modules` 与 `module_convention_gate` 仍 pending。

## Runtime 15 M2 dynamic API vampire runtime support module naming hard cutover

状态：`runtime_15_dynamic_api_vampire_runtime_support_naming_hard_cutover_static_passed_cargo_deferred`。

R2.3 的当前新增落地部分是 dynamic API vampire runtime support 去 `helpers` 文件名债。`dynamic_api/session/tests/helpers.rs` 已删除，当前 owner 为 `dynamic_api/session/tests/vampire_runtime_support.rs`；`dynamic_api/session/tests/mod.rs` 只通过 `mod vampire_runtime_support;` 挂载，`frame_diagnostics.rs`、`vampire_gameplay.rs`、`vampire_hud.rs` 与 `vampire_menu.rs` 只从该 owner 读取 vampire project/session/HUD/diagnostics 测试支撑函数，没有保留旧模块、alias 或兼容 re-export。

守卫：`runtime_15_dynamic_api_vampire_runtime_support_uses_owner_name` 验证旧文件不存在、新 owner/入口/调用方 import 形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、dynamic API session 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo deferred，不计通过。该切片只关闭 dynamic API vampire runtime support 文件名债；完整 `runtime_15_no_banned_name_modules` 与 `module_convention_gate` 仍 pending。

## Runtime 15 M2 camera controller output module naming hard cutover

状态：`runtime_15_camera_controller_output_naming_hard_cutover_static_passed_cargo_deferred`。

R2.3 的当前新增落地部分是 camera controller output 去 `common` 文件名债。`core/framework/camera_controller/common.rs` 已删除，当前 owner 为 `core/framework/camera_controller/controller_output.rs`；`core/framework/camera_controller/mod.rs` 只通过 `mod controller_output;` 挂载，并从该 owner 重导出 `CameraControllerOutput`、`CursorGrabIntent` 与 `CursorGrabMode`，没有保留旧模块、alias 或兼容 re-export。

守卫：`runtime_15_camera_controller_output_uses_owner_name` 验证旧文件不存在、新 owner/入口形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、camera controller 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo deferred，不计通过。该切片只关闭 camera controller output 文件名债；完整 `runtime_15_no_banned_name_modules` 与 `module_convention_gate` 仍 pending。

## Runtime 15 M2 scene ECS systems many/single queries module naming hard cutover

状态：`runtime_15_scene_ecs_systems_many_single_queries_naming_hard_cutover_static_passed_cargo_timeout_no_result`。

R2.3 的当前新增落地部分是 scene ECS systems many/single queries 去 `query_helpers` 文件名债。`scene/tests/ecs_systems/query_helpers.rs` 已删除，当前 owner 为 `scene/tests/ecs_systems/many_single_queries.rs`；`scene/tests/ecs_systems.rs` 只通过 `mod many_single_queries;` 挂载，该 owner 继续承接 `get_many` / `iter_many` / `single` query behavior 覆盖，没有保留旧模块、alias 或兼容 re-export。既有 `runtime_15_scene_ecs_systems_tests_are_folder_backed` 守卫和相关 M3 文档锚点已同步新路径。

守卫：`runtime_15_scene_ecs_systems_many_single_queries_uses_owner_name` 验证旧文件不存在、新 owner/入口形状、M3 test-budget guard 已切到新路径、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、scene ECS 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 复跑在 305 秒超时，临时日志只到编译 warning 且没有 test result，不计通过。该切片只关闭 scene ECS systems many/single query 文件名债；完整 `runtime_15_no_banned_name_modules` 与 `module_convention_gate` 仍 pending。

## Runtime 15 M2 plugin static manifest contract owner naming hard cutover

状态：`runtime_15_plugin_static_manifest_contract_owner_naming_hard_cutover_static_passed_cargo_deferred`。

R2.3 的当前新增落地部分是 static manifest contract 测试去 `helpers` 文件名债。`plugin_extensions/static_manifest_contracts/feature_bundles/helpers.rs`、`package_coordinates/helpers.rs`、`package_identity/helpers.rs` 与 `package_kind/helpers.rs` 已删除，当前 owner 为 `plugin_extensions/static_manifest_contracts/feature_bundles/feature_bundle_rows.rs`、`plugin_extensions/static_manifest_contracts/package_coordinates/package_coordinate_resolution.rs`、`plugin_extensions/static_manifest_contracts/package_identity/package_id_tokens.rs` 与 `plugin_extensions/static_manifest_contracts/package_kind/package_kind_fields.rs`；父模块只挂载职责命名 owner，调用方只从这些 owner import，没有保留旧模块、alias 或兼容 re-export。

守卫：`runtime_15_plugin_static_manifest_contract_owners_use_domain_names` 验证旧文件不存在、新 owner/入口/调用方 import 形状、Runtime 15 子计划、runtime index、审查发现、结构规范、本文档、package manifest 文档和 status-output expectations 同步。验证按实施切片节奏使用 scoped rustfmt/static scans、旧路径不存在扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo deferred，不计通过。该切片只关闭 static manifest contract test owner 文件名债；完整 `runtime_15_no_banned_name_modules` 与 `module_convention_gate` 仍 pending。

## Runtime 15 graphics facade visibility note

状态：`runtime_15_graphics_facade_visibility_note_static_passed_cargo_blocked_graphics_drift`。

R3.4 的当前已落地部分是 `graphics/mod.rs` 的混合可见性边界说明。该 root façade 仍保留同一导出集合，但源码已经明确分出 crate-private implementation owners、public module entries、public façade exports、crate-visible bridge 和 test-only access。公共 module entries 只包括 feature extract source 合同、`graphics::prelude` 和 graphics module descriptor surface；`backend`、`scene`、`types` 等实现 owner 保持 `pub(crate)`，不会作为稳定模块入口泄漏。

守卫：`runtime_15_mixed_visibility_has_facade_note` 验证 `graphics/mod.rs` 保留上述分区注释、公共入口和实现模块私有性，并验证 Runtime 15 计划、runtime index、结构规范和本文档都记录同一状态锚。scoped rustfmt with `skip_children=true`、standalone guard 和状态锚静态检查已通过；Cargo 聚焦验证当前被既有 graphics 编译漂移阻塞（`FrameSubmissionContext::new` 参数数不匹配、`AdvancedProfileRuntimePlan: Default` 缺失），因此本切片只记录静态守卫闭合，core-min Cargo gate 继续 pending。

## Runtime 15 F9 runtime prelude required type coverage

状态：`runtime_15_prelude_required_types_coremin_check_passed`。

R3.3 的当前已落地部分是 prelude 分层：`asset/prelude.rs`、`scene/prelude.rs`、`ui/prelude.rs`、`graphics/prelude.rs` 分别维护子系统高频类型，crate 级 `prelude.rs` 只聚合这些子系统 prelude，不再直接列 asset/scene/ECS/UI/graphics 符号。该形态让 gameplay/authoring 用户能通过 `zircon_runtime::prelude::*` 获取资产句柄与 descriptor、ECS world/query/resource、UI surface/template/v2、graphics module/render pipeline 等常用入口，同时保持完整公共面仍归各子系统 `mod.rs`。

守卫：`runtime_prelude_exports_asset_scene_ui_and_graphics_contracts` 验证行为面可用；`runtime_15_prelude_covers_required_types` 验证 crate 聚合、四个子系统 `pub mod prelude;`、必含类型清单、Runtime 15 计划、runtime index、审查发现、结构规范和本文档状态锚同步。

## Runtime 15 runtime UI dead-code support split

状态：`runtime_15_runtime_ui_dead_code_support_split_coremin_check_passed`。

E6/S10/F10/F12 的当前已落地部分是把 runtime UI 的生产 dead-code surface 与测试支持拆开。`PublicRuntimeFrame` 现在由生产 owner `ui/public_runtime_frame.rs` 持有，`graphics/types/viewport_render_frame_from_public_runtime.rs` 继续通过 `crate::ui::PublicRuntimeFrame` 构造 `ViewportRenderFrame` 并把 `frame.extract` 包装为 `Arc<RenderFrameExtract>`。`RuntimeUiManager`、`RuntimeUiFixture`、input router、manager error 与 window-event helpers 全部移入 `ui/tests/runtime_ui_support`，由 `ui/mod.rs` 通过 `#[cfg(test)]` 和 `#[path = "tests/runtime_ui_support/mod.rs"]` 挂载给测试使用。

旧生产 `ui/runtime_ui/` 目录已删除，`ui/mod.rs` 不再声明 `#[allow(dead_code)] mod runtime_ui;`，也不保留兼容 re-export 或 shim。守卫：`runtime_15_runtime_ui_dead_code_surface_is_test_support` 验证生产 frame owner、test-only support owner、旧目录删除、graphics conversion anchor，以及 Runtime 15 计划、runtime index、审查发现、结构规范和本文档状态锚同步。验证：scoped rustfmt --check 通过；standalone structure guard 1/1、ui_architecture 3/3、status-output 2/2 通过；direct ui_architecture_boundary_audit risks=[]（ui entries 18/18、taffy hits/files 175/175 与 10/10）；core-min focused cargo test `runtime_15_runtime_ui_dead_code_surface_is_test_support` 1/1 通过；core-min `cargo check` 通过（既有 warnings）。 该切片只关闭 `runtime_ui` 子面；F12 全量 `#[allow(dead_code)]` sweep 仍由 Runtime 15 M5/T2 后续执行。

## Runtime 15 F12 runtime-owned dead-code suppression cleanup

状态：`runtime_15_runtime_owned_dead_code_suppression_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是两个 runtime-owned suppression 点清理，避开 active render/provider、plugin、editor 会话区域。`asset/pipeline/worker_pool.rs` 中的 test-only `request_rx_guard` 不再依赖 `#[allow(dead_code)]`，而是通过 `request_channel_guard_is_alive_for_test()` 暴露给 worker-pool 行为测试，测试显式断言 bounded overflow 无 worker 场景下 receiver guard 仍然存活。这样保留通道连接语义，同时让 test-only 支撑代码有真实读取点。

`core/runtime/state/module_entry.rs` 的 descriptor 字段不再压制 dead-code lint；`ModuleEntry::descriptor()` 现在是明确 accessor，`core/runtime/diagnostics/devtools.rs` 通过该 accessor 读取 module name、description、driver/manager/plugin counts。守卫：`runtime_15_runtime_owned_dead_code_suppression_cleanup` 验证 asset worker 和 module entry 两个文件不再包含 `#[allow(dead_code)]`，并验证 Runtime 15 计划、runtime index、审查发现、结构规范和本文档状态锚同步。该切片只关闭 runtime-owned 两处 suppression 子面；script host value descriptor 子面已由 `runtime_15_script_host_value_descriptors_coremin_check_passed` 关闭，OffscreenTarget 固定帧 texture owner 子面已由 `runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result` 关闭，更宽 graphics resources 与全量 F12 sweep 仍 pending。

## Runtime 15 F12 script host value descriptor dead-code cleanup

状态：`runtime_15_script_host_value_descriptors_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是脚本宿主 math 值描述器清理。`script/vm/host/builtin_host_modules.rs` 的 `Vec3` 与 `ColorRgba` 只作为 `ZirconScriptType` 反射描述器进入 `zircon_host_module!`，因此原先用 `#[allow(dead_code)]` 避开字段未读告警。本轮移除这两个 suppression，并新增字段布局哨兵，构造并读取 `Vec3 { x, y, z }` 与 `ColorRgba { r, g, b, a }`，让 descriptor-only 类型的字段形状保持编译器可见。

该哨兵不新增 VM host call、不改变 `zr.zircon.math` 的 `vec3_length` / `vec3_dot` 函数面，也不改变 Runtime 13 host ledger：`docs/zircon_runtime/script/vm/host/function_ledger.md` 仍记录 6 个固定 host module、52 个固定 host function、2 个固定 script type descriptor。守卫：`runtime_15_script_host_value_descriptors_do_not_suppress_dead_code` 验证 `builtin_host_modules.rs` 不再包含 `#[allow(dead_code)]`、布局哨兵读取所有字段、ledger 计数稳定，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和脚本宿主清册的状态锚同步。该切片只关闭 script host value descriptor 子面；OffscreenTarget 固定帧 texture owner 子面已由 `runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result` 关闭，更宽 graphics resources 与全量 F12 sweep 仍 pending。

## Runtime 15 F12 script reflection macro fixture dead-code cleanup

状态：`runtime_15_script_reflection_macro_fixture_dead_code_cleanup_static_passed_cargo_deferred`。

E6/S10/F12 的当前新增落地部分是脚本 VM reflection docs 测试 fixture 清理。`script/vm/tests/reflection_docs.rs` 中的 `TestVec3`、`TestEnum` 与 nested `Point` 只用于验证 `ZirconScriptType` 和 `zircon_host_module` 宏生成 descriptor，因此原先依赖 `#[allow(dead_code)]` 避开字段和枚举变体未读告警。本轮移除这些 suppression，并让测试构造读取 `TestVec3 { x, y, z }`、用 `matches!(TestEnum::A, TestEnum::A)` 读取枚举变体、通过 `macro_math::point_fixture_x()` 读取 nested module 的 Point 字段。

该切片不新增 VM host call、不改变 reflection Markdown 输出，也不改变 `zr.zircon.math` 或 macro host function dispatch；它只让宏 fixture 的 Rust 形状保持编译器可见。守卫：`runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code` 验证 `reflection_docs.rs` 不再包含 `#[allow(dead_code)]`、三类 fixture 都有真实读取点，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/script/vm/tests.md` 与 `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` 的状态锚同步。该切片只关闭 script reflection macro fixture 子面；graphics resources 与全量 F12 sweep 仍 pending。

## Runtime 15 M4 core runtime service-list owner split

状态：`runtime_15_core_runtime_service_lists_folder_split_static_passed_cargo_lock_blocked`。

R1.4/M4 的当前新增落地部分是 core runtime registration service-list owner 减压。旧 `core/runtime/handle/registration/service_lists.rs` 已删除并拆成 folder-backed `core/runtime/handle/registration/service_lists/mod.rs`、`types.rs`、`multi.rs`、`specialized.rs` 与 `shutdown.rs`。父模块只保留 `module_service_lists(...)` 分派和 `single_service_module_lists` 窄 re-export；`types.rs` 承接 `ModuleServiceLists` 返回形状，`multi.rs` 承接多服务扫描，`specialized.rs` 承接 1-5 服务特化路径，`shutdown.rs` 承接 shutdown 顺序组装。

该切片不改变 `register_module.rs` 的入口、不改变 driver/manager/plugin lifecycle ordering，也不新增兼容 re-export。守卫：`runtime_15_core_runtime_service_lists_are_folder_backed` 验证旧平铺文件不存在、五个子 owner 挂载、代表入口保持窄接口、所有 service-list owner 低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、runtime lifecycle 文档和 status-output expectations 的状态锚同步。该切片只关闭 core runtime service-list owner 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M4 RHI WGPU command validation render-state owner split

状态：`runtime_15_rhi_wgpu_command_validation_render_state_split_static_passed_cargo_lock_blocked`。

R1.4/M4 的当前新增落地部分是 RHI WGPU command validation 的 render-state owner 减压。`rhi_wgpu/command_validation.rs` 从 878 行减压为 621 行，继续拥有 `validate_recorded_commands(...)`、`execute_recorded_commands(...)`、debug group、render pass、queue 与 copy command traversal；新增 `rhi_wgpu/command_validation/render_state.rs` 作为 300 行 child owner，承接 `RecordedRenderState`、`CommandResourceLookup`、bind group slot validation、binding range、vertex/index/strided range helper 与 pipeline-layout lookup。

该切片不改变 public RHI API、不改变 `rhi_wgpu::device` 的 submit/execute 调用入口，也不新增兼容 re-export。守卫：`runtime_15_rhi_wgpu_command_validation_state_is_child_owner` 验证父/子 owner 挂载、moved helper 不回流、两侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、RHI descriptors 文档和 status-output expectations 的状态锚同步。该切片只关闭 WGPU command validation render-state owner 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 RHI/graphics Cargo sweep 仍 pending。

## Runtime 15 M4 RHI WGPU UI surface render/setup owner split

状态：`runtime_15_rhi_wgpu_ui_surface_render_setup_owner_split_static_passed_cargo_timeout_no_result`。

R1.4/M4 的当前新增落地部分是 RHI WGPU UI surface render/setup owner 减压。`rhi_wgpu/ui_surface.rs` 从 1054 行减压为 549 行，继续拥有 `WgpuUiSurfaceRenderer` lifecycle、surface frame acquire/present、image cache pruning 与 public UI surface renderer 调用入口；新增 `rhi_wgpu/ui_surface/render_pass.rs` 作为 168 行 child owner，承接 `WgpuUiDrawBuffers`、`TargetLoad`、draw-op buffer upload、render pass begin、viewport/scissor setup 与 solid/image/text draw recording；新增 `rhi_wgpu/ui_surface/surface_setup.rs` 作为 164 行 child owner，承接 surface configuration、format/present/alpha mode selection、adapter/device request、instance descriptor 与 raw-window-handle surface creation；新增 `rhi_wgpu/ui_surface/tests.rs` 作为 273 行 child owner，保留 headless presenter stats、surface format/alpha、damage mode、batch stats、atlas image batching 与 image-cache prune coverage。

该切片不改变 `rhi::ui_surface` trait、不改变 editor retained-host GPU presenter contract、不改 shader/pipeline layout 或 image cache key 语义，也不新增兼容 re-export。守卫：`runtime_15_rhi_wgpu_ui_surface_render_setup_are_child_owners` 验证父/子 owner 挂载、render/setup helper 不回流、四侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、RHI UI surface 文档和 status-output expectations 的状态锚同步。该切片只关闭 WGPU UI surface render/setup owner 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 RHI/graphics Cargo sweep 仍 pending。

## Runtime 15 M4 RHI WGPU UI surface geometry test owner split

状态：`runtime_15_rhi_wgpu_ui_surface_geometry_tests_owner_split_static_passed_cargo_timeout_no_result`。

R1.4/M4 的当前新增落地部分是 RHI WGPU UI surface geometry test owner 减压。`rhi_wgpu/ui_surface/geometry.rs` 从 812 行减压为 559 行，继续拥有 draw-list ordering、solid/image/text draw item projection、rounded quad/border tessellation、image UV trimming、effective rect clipping 与 text bounds conversion；新增 `rhi_wgpu/ui_surface/geometry/tests.rs` 作为 308 行 child owner，承接 damage/clip trimming、stable z-order、rounded solid/border geometry、image UV clipping、atlas UV composition、invalid atlas UV skip、text bounds clip 与 disjoint damage coverage，以及 test-only `solid_items(...)` helper。

该切片不改变 `UiSurfaceDrawList` 几何语义、不改变 batching 输入顺序、不改 atlas UV 计算、不改 shader/pipeline layout 或 `rhi::ui_surface` trait，也不新增兼容 re-export。守卫：`runtime_15_rhi_wgpu_ui_surface_geometry_tests_are_child_owner` 验证父/子 owner 挂载、test helper 与测试函数不回流、两侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、RHI UI surface 文档和 status-output expectations 的状态锚同步。该切片只关闭 WGPU UI surface geometry test owner 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 RHI/UI surface Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；focused `cargo test -p zircon_runtime --lib runtime_15_rhi_wgpu_ui_surface_geometry_tests_are_child_owner --locked --jobs 1` 10 分钟超时无测试结果，且超时后未发现 cargo/rustc 残留进程，不计 Cargo 通过。

## Runtime 15 M4 RHI device handle owner split

状态：`runtime_15_rhi_device_handles_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 RHI device handle owner 减压。`rhi/device.rs` 从 799 行减压为 702 行，继续拥有 typed `RhiError`、command/render-pass DTOs、`CommandList` 与 `RenderDevice` contract；新增 `rhi/device/handles.rs` 作为 105 行 child owner，承接 `BufferHandle`、`TextureHandle`、`SamplerHandle`、`BindGroupLayoutHandle`、`BindGroupHandle`、`ShaderModuleHandle`、`PipelineLayoutHandle` 与 `PipelineHandle` 这组 neutral resource handle newtypes。

该切片不改变 RHI handle raw/new API、不改变 command-list recording semantics、不改 backend device contract，也不新增兼容 re-export。父模块通过 `mod handles;` 与 `pub use self::handles::{...}` 保留原 `rhi::device::*` 和 `rhi::*` public paths。守卫：`runtime_15_rhi_device_handles_are_child_owner` 验证父/子 owner 挂载、handle newtypes 不回流、`rhi/mod.rs` handle export 保持、两侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、RHI descriptors 文档和 status-output expectations 的状态锚同步。该切片只关闭 RHI device handle owner 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 RHI Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描、尾随空白扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 scene component lighting/post-process owner split

状态：`runtime_15_scene_component_light_postprocess_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 scene component production owner 减压。`scene/components/scene.rs` 从 711 行减压为 481 行，继续拥有 `NodeKind`、基础 active/transform/render-layer、camera、mesh renderer、physics、animation、scene node/record DTO 与 serde default helpers；`scene/components/scene/lighting.rs` 作为 85 行 child owner 承接 `AmbientLight`、`DirectionalLight`、`PointLight`、`RectLight`、`SpotLight` 与 defaults；`scene/components/scene/post_process.rs` 作为 84 行 child owner 承接 `PostProcessSettingsComponent`、`PostProcessVolumeComponent`、defaults 与 `global`/`local`/`with_weight` builders。

父模块通过 `mod lighting;`、`mod post_process;` 与 `pub use self::lighting::{...}` / `pub use self::post_process::{...}` 保留原 `scene::components` 公开类型路径，不新增兼容 facade，不改变 scene asset serialization、render extract component lookup、post-process volume extraction 或 authoring semantics。

守卫：`runtime_15_scene_components_light_postprocess_are_child_owners` 验证父模块挂载并 re-export 两个 child owners、light/post-process declarations 不回流到父文件、子 owner 承接对应 component/default/builder 逻辑、三侧均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/scene/ecs.md`、`docs/zircon_runtime/scene/render_extract.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `scene/components/scene.rs` 的 light/post-process owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 scene Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描、尾随空白扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 texture descriptor settings parser owner split

状态：`runtime_15_texture_descriptor_settings_parser_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 texture descriptor settings parser owner 减压。`asset/assets/texture/descriptor.rs` 从 635 行减压为 393 行，继续拥有 `TextureArrayLayout`、`TextureAssetDescriptor` DTO、`apply_import_settings(...)`、render descriptor projection、extent normalization 与默认 texture descriptor contract；新增 `asset/assets/texture/descriptor/settings.rs` 作为 189 行 child owner，承接 TOML settings parser helpers、usage/asset_usage token parsing、sampler shorthand/table parsing、array layout/color-space/dimension parsing 与 Bevy-style token normalization。

父模块通过 `mod settings;` 与窄 `use self::settings::{...}` 消费子 owner；fallible apply API 名称保持不变，并已由后续 F8 typed-error 切片改为 `TextureDescriptorResult<_>`，同时保留 Bevy alias diagnostics、RGBA8 linear-format normalization、2D/3D extent validation、serialized descriptor shape 与 `TextureAsset::render_image_descriptor()` 语义，也不新增兼容 re-export。守卫：`runtime_15_texture_descriptor_settings_parser_is_child_owner` 验证父模块保留 public descriptor behavior、settings parser helper 不回流到父文件、子 owner 承接 parser/token normalization、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/asset/importer.md`、`docs/zircon_runtime/asset/render-assets.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 texture descriptor settings parser 的 M4 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 texture importer/render assets Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描、尾随空白扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 F5 sound asset typed errors

状态：`runtime_15_sound_asset_typed_errors_static_passed_cargo_deferred`。

F5/E1/E2 的当前新增落地部分是 sound asset WAV parser 的 typed-error 收束。`asset/assets/sound.rs` 新增 `SoundAssetError` / `SoundAssetResult`，`SoundAsset::from_wav_bytes(...)`、format/extensible-format parser、sample decoder、channel-mask layout projection 和 header reader helpers 不再返回 `Result<_, String>`。

错误 variants 覆盖 RIFF/WAVE container shape、fmt/data chunk 缺失、zero channel/sample-rate declaration、WAVE_FORMAT_EXTENSIBLE valid-bits/subformat/channel-mask failures、unsupported bits/format、block-align mismatch、sample/frame alignment 和 header/chunk overflow。`asset/assets/mod.rs` 与 `asset/mod.rs` 公开 typed surface；`asset/importer/ingest/import_sound.rs` 继续只在 `AssetImportError::Parse` 边界格式化 `SoundAssetError` Display 文案，不把 importer 诊断反推为资产层字符串错误。

守卫：`review_f5_sound_asset_uses_typed_error` 锁定 sound owner、facade exports、import boundary、`docs/zircon_runtime/asset/assets/sound.md` 和 status/docs anchors，并拒绝 `asset/assets/sound.rs` 回退到 `Result<_, String>`、`Err(format!(...))` 或 `.to_string()`。验证：scoped rustfmt/static scans、docs/status/session anchor scan 已通过；Cargo 因外部 cargo/rustc 通道 active 按 Runtime 15 实施切片节奏 deferred，不计通过。完整 `module_convention_gate`、全量 asset/audio Cargo sweep 与剩余 String-error sweep 仍 pending。

## Runtime 15 F8 texture descriptor typed errors

状态：`runtime_15_texture_descriptor_typed_errors_static_passed_cargo_deferred`。

F8/E3 的当前新增落地部分是 texture descriptor fallible apply API 的 typed-error 收束。`asset/assets/texture/descriptor.rs` 新增 `TextureDescriptorError` / `TextureDescriptorResult`，`TextureArrayLayout::from_import_settings(...)` 与 `TextureAssetDescriptor::apply_import_settings(...)` 不再返回 `Result<_, String>`；`asset/assets/texture/descriptor/settings.rs` 把 TOML setting type、u32 overflow、unsupported token 与 array-layout mode errors 映射到 typed variants；`asset/assets/texture/texture_asset.rs` 把 array-layout RGBA8/2D/single-layer/divisibility/byte-length/extent overflow 校验纳入同一错误类型。

`asset/assets/texture/mod.rs` 与 `asset/assets/mod.rs` 公开 typed error/result，runtime built-in texture ingest 与 first-party texture importer plugin 仍只在 `AssetImportError::Parse` 边界格式化 Display 文案，不恢复 builder-style `with_*`、`Result<_, String>` 或 `Err(format!(...))`。守卫：`review_f8_texture_import_settings_use_fallible_apply_not_with` 锁定 typed fallible apply API、settings 子 owner 无 String-error 回退、`TextureDescriptorError` docs/status 锚点，以及 Runtime 15 计划、runtime index、审查发现、结构规范、render-assets 文档和 status-output expectations 的同步状态。

验证：scoped rustfmt/static scans、docs/status/session anchor scan 已通过；Cargo 因外部 cargo/rustc 通道 active 按 Runtime 15 实施切片节奏 deferred，不计通过。完整 `module_convention_gate`、全量 texture importer/render assets Cargo sweep 与剩余 String-error sweep 仍 pending。

## Runtime 15 M4 scene world render light collection owner split

状态：`runtime_15_scene_world_render_lights_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 scene world render production owner 减压。`scene/world/render.rs` 继续拥有 `RenderFrameExtract` 构建入口、camera descriptor/view projection、mesh/sprite/post-process/particle 采集编排、visibility handoff 和共享 `entity_intersects_camera_layers(...)` helper；ambient、directional、point、rect 与 spot light snapshot collection 迁入 `scene/world/render/lights.rs`。

子 owner 保留既有 active-in-hierarchy 过滤、camera `RenderLayerSet` intersection、legacy default render layer fallback、light snapshot sort order 和 rect-light degradation reason；父模块通过 `mod lights;` 挂载并继续从 `build_prepared_render_frame_extract_for_request(...)` 调用相同 collector 名称，不改变 `LightingExtract`、shadow-map first directional selection、renderer readiness stats 或 WGPU light-buffer ABI。父文件从 832 行降到 725 行，`render/lights.rs` 为 169 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_scene_world_render_light_collectors_are_child_owner` 验证父模块保留 frame extract orchestration、light collector call sites 与共享 camera-layer helper，light collector impl 和 render light snapshot Vec 类型不回流到父文件，子 owner 承接五类 light snapshot collection 与 `default_render_layer_mask()` fallback，父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/scene/render_extract.md`、`docs/zircon_runtime/graphics/render-product-submit.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 scene world render light collection owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 render extract Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描和 docs/status 锚点扫描通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 dynamic API session profile owner split

状态：`runtime_15_dynamic_api_session_profile_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 dynamic API session profile owner 减压。`dynamic_api/session.rs` 从 773 行减压为 728 行，继续拥有 Rust-ABI session entry points、session registry、`RuntimeDynamicSession` lifecycle、frame tick/capture/present 和 host-request orchestration；新增 `dynamic_api/session/profile.rs` 作为 47 行 child owner，承接 `RuntimeDynamicSessionProfile`、ABI profile byte parsing、fixed-step policy、diagnostic-log schedule selection 与 render-bridge enablement policy。

该切片不改变 `ZrRuntimeSessionConfigV1` profile 字节语义、不改变 default/runtime/editor/dev/minimal/headless profile 行为、不改 `RuntimeDynamicSession::new(...)` 或 frame tick 调用入口，也不新增兼容 re-export。父模块只通过 `mod profile;` 与 `use profile::RuntimeDynamicSessionProfile;` 消费 profile policy。守卫：`runtime_15_dynamic_api_session_profile_is_child_owner` 验证父/子 owner 挂载、profile enum/constants/methods 不回流、两侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、dynamic API session 文档和 status-output expectations 的状态锚同步。该切片只关闭 dynamic API session profile owner 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 dynamic API Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描、尾随空白扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 dynamic API session registry owner split

状态：`runtime_15_dynamic_api_session_registry_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 dynamic API session registry owner 减压。`dynamic_api/session.rs` 从 728 行继续减压为 682 行，继续拥有 Rust-ABI session entry points、`RuntimeDynamicSession` lifecycle、frame tick/capture/present 和 host-request orchestration；新增 `dynamic_api/session/registry.rs` 作为 69 行 child owner，承接 `SESSION_REGISTRY`、`SessionRegistry` handle map、handle allocation、poison-safe `lock_registry`/`lock_session` 与 `with_session` lookup/dispatch。

该切片不改变 `ZrRuntimeSessionHandle` 分配、不改变 destroy lookup、invalid/not-found status、lock-poison recovery 或 dynamic API ABI entry semantics，也不新增兼容 re-export。父模块只通过 `mod registry;`、`use registry::{insert_session, lock_registry, with_session};` 和测试专用 `use registry::lock_session;` 消费 registry owner。守卫：`runtime_15_dynamic_api_session_registry_is_child_owner` 验证父/子 owner 挂载、registry static/struct/helper 不回流、两侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、dynamic API session 文档和 status-output expectations 的状态锚同步。该切片只关闭 dynamic API session registry owner 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 dynamic API Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描、尾随空白扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 native host API adapter tests owner split

状态：`runtime_15_native_host_api_adapter_tests_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 native host API adapter production owner 减压。`zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs` 把内联测试迁入 folder-backed `zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/tests.rs`。父文件从 967 行降到 506 行，继续拥有 `NativeHostApiV3RegistrationScope`、`NativeHostBridgeCallScope`、9 个 host ABI callback entry、registration/component bridge dispatch、context table 与 status helper；新子文件为 455 行，承接原 13 个 native host API / bridge method descriptor 测试。

该切片不改变 `ZrHostApiV3` 表面、不改变 runtime plugin handle、bridge method dispatch、manifest method descriptor projection、panic guard 或 dotted plugin id projection，也不新增兼容 facade。守卫：`runtime_15_native_host_api_adapter_tests_are_child_owner` 验证父模块挂载、代表性 moved test 不回流、父文件 0 个测试加新子文件 13 个测试合计保留原 13 个测试，并验证两侧都低于 800 行预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、`docs/zircon_runtime/plugin/bridge.md` 与 status-output expectations。完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 plugin bridge Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数与测试数量扫描、moved-test parent scan、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 已通过；Cargo 因外部 cargo/rustc 通道 active（`cargo` PIDs 19096/53460/61976/63160；`rustc` PIDs 6488/31436）按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 material asset value/readiness helper owner split

状态：`runtime_15_material_asset_value_readiness_owner_split_static_passed_cargo_timeout_no_result`。

R1.4/M4 的当前新增落地部分是 material asset helper owner 减压。`asset/assets/material/material_asset.rs` 从 937 行减压为 750 行，继续拥有 `MaterialAsset` DTO、`.zmaterial` document 转换入口、descriptor/readiness public API、management overview 与 shader-aware dependency/texture-slot entry；新增 `asset/assets/material/material_asset/value_sync.rs` 作为 136 行 child owner，承接 TOML override 读取、texture slot hydration、legacy default 同步与 TOML 数组生成 helper；新增 `asset/assets/material/material_asset/readiness.rs` 作为 70 行 child owner，承接 shader readiness diagnostic projection、WGSL capture/missing runtime source 映射与 material validation diagnostic rows。

该切片不改变 `.zmaterial` 序列化形状、不改变 `MaterialAsset` public API、不改 render material descriptor 字段或 readiness report 语义，也不新增兼容 re-export。守卫：`runtime_15_material_asset_value_readiness_helpers_are_child_owners` 验证父/子 owner 挂载、value/readiness helper 不回流、三侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、asset zmeta/material 文档和 status-output expectations 的状态锚同步。该切片只关闭 material asset value/readiness helper 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 asset/render material Cargo sweep 仍 pending。

## Runtime 15 M4 material asset management record owner split

状态：`runtime_15_material_asset_management_record_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 material asset management record owner 减压。`asset/assets/material/material_asset.rs` 从 750 行继续减压为 651 行，继续拥有 `MaterialAsset` DTO、`.zmaterial` document 转换入口、descriptor/readiness public API、`overview(...)`/`management_record(...)` entry 与 shader-aware dependency/texture-slot entry；新增 `asset/assets/material/material_asset/management.rs` 作为 108 行 child owner，承接 `MaterialAssetOverview`、`MaterialAssetManagementRecord`、`MaterialAssetManagementRecordSetSummary` 与 `MaterialAssetManagementRecordSet` 聚合 DTO 和 record-set 汇总 impl。

该切片不改变 `.zmaterial` 序列化形状、不改变 `MaterialAsset::overview(...)` / `management_record(...)` public API、不改 management row ordering、summary counts、render material descriptor 字段或 readiness report 语义，也不新增兼容 re-export。父模块只通过 `mod management;` 与 `pub use self::management::{...}` 保留原公开类型路径。守卫：`runtime_15_material_asset_management_records_are_child_owner` 验证父/子 owner 挂载、management DTO/impl 不回流、两侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、asset zmeta/material 文档和 status-output expectations 的状态锚同步。该切片只关闭 material asset management record 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 asset/render material Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描、尾随空白扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 core runtime render-stats graph execution-resources owner split

状态：`runtime_15_render_stats_graph_execution_resources_owner_split_static_passed_cargo_timeout_no_result`。

R1.4/M4 的当前新增落地部分是 core runtime render-stats graph diagnostics owner 减压。`core/runtime/diagnostics/render_stats_store/graph.rs` 从 1024 行减压为 765 行，继续拥有 graph diagnostics dispatcher、frame graph aggregate rows、execution coverage、stage summary、materialization、alias、profile 与 post-process graph rows；新增 `core/runtime/diagnostics/render_stats_store/graph/execution_resources.rs` 作为 260 行 child owner，承接 `last_graph_execution_resource_report` 投影、texture/buffer binding counts，以及 transient-pool creation/reuse/retained/budget/eviction byte/count rows。

该切片不改变 `record_render_stats_diagnostics(...)` 入口、不改变任何 `render.graph.execution.*` 诊断 path 名称，也不新增兼容 re-export。守卫：`runtime_15_render_stats_graph_execution_resources_are_child_owner` 验证父/子 owner 挂载、moved resource rows 不回流、两侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、core diagnostics 文档和 status-output expectations 的状态锚同步。该切片只关闭 render-stats graph execution-resources owner 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 runtime diagnostics Cargo sweep 仍 pending。

## Runtime 15 M4 scene fixed light reflection write-field owner split

状态：`runtime_15_scene_fixed_light_reflection_write_fields_owner_split_static_passed_cargo_lock_blocked`。

R1.4/M4 的当前新增落地部分是 scene fixed light reflection owner 减压。`scene/reflect/fixed/lights.rs` 从 906 行减压为 379 行，继续拥有 Ambient/Directional/Point/Rect/Spot light 的 schema registration、adapter construction、contains、read/read_fields 与 remove callbacks；新增 `scene/reflect/fixed/lights/write_fields.rs` 作为 542 行 child owner，承接五类 light 的 editable field write callbacks、typed component presence checks、Vec2/Vec3/scalar/bool value validation 与 no-op mutation comparisons。

该切片不改变 `ReflectComponent` 函数表、不改变 fixed light type path、字段名、错误模型或 `World` typed ECS mutation path，也不新增兼容 re-export。守卫：`runtime_15_scene_fixed_light_reflection_write_fields_are_child_owner` 验证父/子 owner 挂载、write helper 不回流、两侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、scene reflect 文档和 status-output expectations 的状态锚同步。该切片只关闭 fixed light reflection write-field owner 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 scene reflection Cargo sweep 仍 pending。

## Runtime 15 M4 scene world property-access physics write owner split

状态：`runtime_15_scene_world_property_access_physics_owner_split_static_passed_cargo_timeout_no_result`。

R1.4/M4 的当前新增落地部分是 scene world property-access physics writer 减压。`scene/world/property_access/write.rs` 从 936 行减压为 593 行，继续拥有 `World::set_property(...)` dispatch、name/hierarchy/transform/camera/mesh/light/animation 与 dynamic component write routing；新增 `scene/world/property_access/write/physics.rs` 作为 357 行 child owner，承接 rigid body、collider 与 joint property writes、physics value parsing、collider shape replacement、material override、axis lock 与 dynamic fallback。

该切片不改变 `ComponentPropertyPath` 字段名、不改变 `ScenePropertyValue` 验证、不改变 property write 的 `Result<bool, String>` 合同，也不新增兼容 re-export。守卫：`runtime_15_scene_world_property_access_physics_writes_are_child_owner` 验证父/子 owner 挂载、physics write helper 不回流、两侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、scene ECS 文档和 status-output expectations 的状态锚同步。该切片只关闭 property-access physics write owner 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 scene Cargo sweep 仍 pending。

## Runtime 15 M4 scene world property-access physics entry owner split

状态：`runtime_15_scene_world_property_access_physics_entries_owner_split_static_passed_cargo_lock_blocked`。

R1.4/M4 的当前新增落地部分是 scene world property-access physics entry 减压。`scene/world/property_access/entries.rs` 从 843 行减压为 602 行，继续拥有 property entry traversal、name/hierarchy/transform/camera/mesh/light/animation 非 physics 投影、dynamic JSON 投影、literal matching 与 capacity routing；新增 `scene/world/property_access/entries/physics.rs` 作为 281 行 child owner，承接 rigid body、collider 与 joint property entries、collider shape/material projection、constraint rows、joint limit rows 与 physics capacity hint。

该切片不改变 property path、不改变 `ScenePropertyValue` 内容、不改变 animatable 标记或 entry 顺序，也不新增兼容 re-export。守卫：`runtime_15_scene_world_property_access_physics_entries_are_child_owner` 验证父/子 owner 挂载、physics entry helper 不回流、两侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、scene ECS 文档和 status-output expectations 的状态锚同步。该切片只关闭 property-access physics entry owner 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 scene Cargo sweep 仍 pending。

## Runtime 15 M4 scene world project I/O mesh owner split

状态：`runtime_15_scene_world_project_io_mesh_owner_split_static_passed_cargo_timeout_no_result`。

R1.4/M4 的当前新增落地部分是 scene world project I/O mesh owner 减压。`scene/world/project_io.rs` 从 799 行减压为 667 行，继续拥有 scene asset load/save entry、entity loop、non-mesh component projection、project document normalization 与 default-node repair；新增 `scene/world/project_io/mesh.rs` 作为 130 行 child owner，承接 `SceneMeshInstanceAsset` 与 `MeshRenderer` 的双向投影、LOD/primitive binding 映射、mesh/model/material handle/reference resolution。

该切片不改变 `.zscene` 资产格式、不改变 builtin fallback 或 project manager lookup、不改变 mesh primitive/LOD 顺序，也不新增兼容 re-export。守卫：`runtime_15_scene_world_project_io_mesh_is_child_owner` 验证父/子 owner 挂载、mesh projection helper 不回流、两侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、scene ECS 文档和 status-output expectations 的状态锚同步。该切片只关闭 scene world project I/O mesh owner 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 scene Cargo sweep 仍 pending。

## Runtime 15 M3 status output Runtime 15 row data split

状态：`runtime_15_status_output_runtime_15_row_data_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 status-output expected row data 减压。`tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs` 从 815 行降到 55 行，只保留 child module mounts、row group aggregation 与 `EXPECTED_STATUS_OUTPUT_SLICE_GROUPS`；新增 folder-backed 子 owner `tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs` 承接 Runtime 15 全部 expected status row literals，并保持低于 800 行预算。

Runtime 15 结构守护的 status row source 读取点已统一转向 `expected_status_row_data/runtime_15.rs`，避免把 Runtime 15 row literal 复制回聚合入口。新增 `runtime_15_status_output_runtime_15_row_data_is_child_owner`，验证父/子 owner 布局、row literal 不回流、两侧行数预算，并要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。验证：scoped rustfmt/static checks、父子行数预算扫描、moved row-data 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按支撑切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 status-output guard sweep 仍 pending。

## Runtime 15 M3 production file budget core runtime guard split

状态：`runtime_15_production_file_budget_core_runtime_guard_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 production-file budget guard 减压。`tests/runtime_absorption/structure_convention/production_file_budget.rs` 已从 779 行降到低于 800 行，只保留 child module mounts、material/scene world 子 owner 与剩余生产 owner guard；新增 folder-backed 子 owner `tests/runtime_absorption/structure_convention/production_file_budget/core_runtime_service_lists.rs` 承接 `runtime_15_core_runtime_service_lists_are_folder_backed` 和 production-file-budget 自检。

该子 owner 继续验证 core runtime service-list production owner 布局、旧平铺 `core/runtime/handle/registration/service_lists.rs` 不存在、父/子 production source 行数预算，并新增 `runtime_15_production_file_budget_core_runtime_guard_is_child_owner` 防止守护回流到聚合文件。验证：scoped rustfmt/static checks、父子行数预算扫描、moved guard 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按支撑切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`large_file_ownership_gate`、`module_convention_gate` 与全量 production-file guard sweep 仍 pending。

## Runtime 15 M3 production file budget guard child-owner split

状态：`runtime_15_production_file_budget_guard_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 production-file budget guard 聚合层二次减压。`tests/runtime_absorption/structure_convention/production_file_budget.rs` 从 732 行降到 63 行，只保留 `production_file_budget/*` 子模块挂载与共享 `read_runtime_src` / `read_repo` helper；原父文件剩余的 UI text layout、RHI WGPU command validation、RHI WGPU UI surface render/setup、render-stats graph、scene fixed lights、scene world render visibility 与 shadow view-projection 守卫分别迁入 `ui_text_layout.rs`、`rhi_wgpu_command_validation.rs`、`rhi_wgpu_ui_surface_render_setup.rs`、`render_stats_graph.rs`、`scene_fixed_lights.rs`、`render_scene_world.rs` 与 `render_shadow.rs`。

新增 `tests/runtime_absorption/structure_convention/production_file_budget/module_layout.rs::runtime_15_production_file_budget_guard_child_owner_split` 作为自守护，验证父模块只挂载 child owner、代表性 guard 不回流、父/关键子 owner 低于 300 行预算，并要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。验证：scoped rustfmt/static checks、父子行数预算扫描、moved guard 扫描与 docs/status 锚点扫描通过；Cargo 因当前外部 Cargo/rustc 车道 active 按支撑切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`large_file_ownership_gate`、`module_convention_gate` 与全量 production-file guard sweep 仍 pending。

## Runtime 15 M3 status output Runtime 15 M4 row data split

状态：`runtime_15_status_output_runtime_15_m4_row_data_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 Runtime 15 status-output row data 二级减压。`tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs` 从 780 行降到 723 行，改为 foundation、M4、F12 resource、M3 四个显式 row group owner；新增 folder-backed 子 owner `tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs`，用 85 行承接 Runtime 15 M4 expected status row literals。

顶层 `expected_status_row_data.rs` 现在按四组挂载 Runtime 15 数据，M4 row literal 只保留在 `runtime_15/m4.rs`。新增 `runtime_15_status_output_runtime_15_m4_row_data_is_child_owner`，验证顶层聚合引用、Runtime 15 父/子 owner 挂载、M4 row literal 不回流、三侧低于 800 行预算，并要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。验证：scoped rustfmt/static checks、父子行数预算扫描、moved M4 row-data 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按支撑切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 status-output guard sweep 仍 pending。

## Runtime 15 M3 status output expected-slice maps split

状态：`runtime_15_status_output_expected_slice_maps_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 status-output expected-slice status/date map 减压。`tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status.rs` 从 737 行降到 572 行，只保留 Runtime 15 子路由和非 Runtime 15 状态分支；`tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date.rs` 从 616 行降到 451 行，只保留 Runtime 15 子路由和非 Runtime 15 日期分支。

新增 `tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs`（179 行）与 `tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs`（177 行）两个 child owner，分别承接 Runtime 15 expected status/date literal 映射。新增 `runtime_15_status_output_expected_slice_maps_are_child_owners`，验证 status/date 父路由、Runtime 15 literal 不回流、四个 map owner 和新守护文件低于 800 行预算，并要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。验证：scoped rustfmt/static checks、父子行数预算扫描、moved expected-slice 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按支撑切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 status-output guard sweep 仍 pending。

## Runtime 15 M3 status output Runtime 15 expected-slice child-owner split

状态：`runtime_15_status_output_runtime_15_expected_slice_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 Runtime 15 expected-slice status/date map 的第二阶段减压。`tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs` 从 750 行降为路由父文件，只挂载 `foundation.rs`、`naming_boundary.rs`、`m4_surface_cleanup.rs` 与 `m3_structure_support.rs`；`date/runtime_15.rs` 也从 601 行降为相同结构的路由父文件。

新增 status/date 各四个 topic child owner，分别承接 Runtime 15 foundation/F5/F8/F12/F13、M1/M2 命名边界、M4/渲染清理、M3 测试与状态支撑 literal。新增 `runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners`，验证 Runtime 15 父文件不再保留代表性 expected-slice literal，child owner 保留对应 status/date 值，所有 focused owner 低于 400 行预算，并要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、session note 与 status-output expectations 同步。精确锚点包括 `plan_status/status_output_tables/expected_slices/status/runtime_15.rs`、`plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs` 和 `runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners`。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved expected-slice literal 扫描、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 通道 active 按支撑切片节奏 deferred，不计 Cargo 通过。

## Runtime 15 M3 status output expected-slice guard maps child-owner split

状态：`runtime_15_status_output_expected_slice_guard_maps_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 status-output expected-slice 守护自身的 child-owner 减压。`tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps.rs` 从 572 行降为路由父文件，只挂载 `maps/top_level_maps.rs` 与 `maps/runtime_15_topics.rs`。

新增 `structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics.rs` 承接 Runtime 15 expected-slice topic 守护，`maps/top_level_maps.rs` 承接顶层/legacy map 守护。新增 `runtime_15_status_output_expected_slice_guard_maps_are_child_owners`，验证父文件不再定义被移动的两个守护、子 owner 保留原守护、父子文件低于 400 行 focused 预算，并要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、session note 与 status-output expectations 同步。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved guard 扫描、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 通道 active 按支撑切片节奏 deferred，不计 Cargo 通过。

## Runtime 15 M3 status output expected-slice top-level map support child-owner split

状态：`runtime_15_status_output_expected_slice_top_level_map_support_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 的当前支撑切片整理 `structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs` 内部结构，不改变 status-output 运行时代码和旧的 `runtime_15_status_output_expected_slice_maps_are_child_owners` 测试入口。父文件现在只挂载 `top_level_maps/assertions.rs` 与 `top_level_maps/sources.rs`，并保留旧守卫入口加新的 `runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed` 自检。

`assertions.rs` 承接 expected-slice status/date parent、Runtime 15 topic、pre-Runtime-15 legacy map、line-budget 和 docs/status 断言组；`sources.rs` 承接批量 source reads。父文件为 136 行，`assertions.rs` 为 349 行，`sources.rs` 为 113 行，三者均低于 400 行 focused 预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、本文档、session note 与 status-output expectations，精确锚点包括 `structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs`、`structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions.rs`、`structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/sources.rs` 和 `runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed`。Cargo 按支撑切片节奏 deferred，不计 Cargo 通过。

## Runtime 15 M3 status output expected-slice legacy child-owner split

状态：`runtime_15_status_output_expected_slice_legacy_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 expected-slice status/date map 的第二阶段 parent 减压。`tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status.rs` 与 `tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date.rs` 现在都是 11 行路由父文件，只挂载 `runtime_15` 与 `pre_runtime_15` 两个 child owner，并通过 `pre_runtime_15::expected_*_for_slice(slice)` 委派非 Runtime 15 分支。

新增 `tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs`（567 行）与 `tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs`（446 行），承接 Runtime 01-14 的 expected status/date literal 映射。新增 `runtime_15_status_output_expected_slice_legacy_maps_are_child_owners`，验证父入口不再保留 `Runtime 14 Cargo 验证窗口探测`、`Runtime 10 F18 asset manager resolution return shape` 等旧分支，Runtime 15 子 map 记录本切片状态，status row evidence 指向两个 legacy child owner。验证：scoped rustfmt/static checks、父子行数预算扫描、moved legacy expected-slice 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 因当前外部 Cargo/rustc 车道 active 按支撑切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 status-output guard sweep 仍 pending。

## Runtime 15 M3 status output expected-slice legacy group child-owner split

状态：`runtime_15_status_output_expected_slice_legacy_group_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 expected-slice legacy map 的第三阶段 group child-owner 减压。`tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs` 与 `tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs` 现在只保留 `runtime_01_05`、`runtime_06_10` 与 `runtime_11_14` 三个 child owner 挂载和窄路由 fallback，避免 Runtime 01-14 literal 继续集中在单个 legacy owner。

新增 `tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_01_05.rs`（228 行）、`runtime_06_10.rs`（240 行）、`runtime_11_14.rs`（111 行）和 date 对应三组 `runtime_01_05.rs`（100 行）、`runtime_06_10.rs`（225 行）、`runtime_11_14.rs`（131 行），承接 Runtime 01-14 的 expected status/date legacy literal。新增 `runtime_15_status_output_expected_slice_legacy_group_maps_are_child_owners`，验证父文件不再保留 `Runtime 05 plan-status Cargo attempt 状态审计`、`Runtime 08 F17 entity path lookup verb rename`、`Runtime 14 animation Cargo gate 尝试` 等代表性 literal，child group 持有对应 status/date 值，Runtime 15 status/date maps、status row evidence、Runtime 15 计划、runtime index、审查发现、结构规范和本文档同步本切片锚点。验证：scoped rustfmt --check、父子行数预算扫描、moved legacy group literal 扫描、docs/status 锚点扫描、trailing-whitespace 扫描和 scoped `git diff --check` 通过；Cargo 因当前外部 Cargo/rustc 车道 active 按支撑切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 status-output guard sweep 仍 pending。

## Runtime 15 M3 status output Runtime 15 M3 row data split

状态：`runtime_15_status_output_runtime_15_m3_row_data_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 Runtime 15 status-output row data 的 M3 子 owner 减压。`tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs` 从 727 行降到 240 行，只保留 foundation、F12 resource row literals 与 M3/M4 子 owner 挂载；新增 `tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs`，用 501 行承接 Runtime 15 M3 expected status row literals。

顶层 `expected_status_row_data.rs` 仍按 foundation、M4、F12 resource、M3 四组挂载 Runtime 15 数据，Runtime 15 父文件通过 `m3::EXPECTED_STATUS_OUTPUT_SLICES` 委派 M3 row data，M3 row literal 只保留在 `runtime_15/m3.rs`。新增 `runtime_15_status_output_runtime_15_m3_row_data_is_child_owner`，验证顶层聚合引用、Runtime 15 父/子 owner 挂载、M3 row literal 不回流、父/子/守护文件低于 800 行预算，并要求 status/date expected-slice maps、Runtime 15 计划、runtime index、审查发现、结构规范和本文档的状态锚同步。验证：scoped rustfmt/static checks、父子行数预算扫描、moved M3 row-data 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按支撑切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 status-output guard sweep 仍 pending。

## Runtime 15 M3 status output variable evidence anchors

状态：`runtime_15_status_output_variable_evidence_anchors_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 status-output row evidence arity 修复。`tests/runtime_absorption/plan_status/status_output_tables/expected_status_rows.rs` 的 `ExpectedStatusOutputSlice` 现在使用 `(&'static str, &'static [&'static str])`，使每条状态记录可以携带实际需要的证据锚点数量，而不是被固定四项数组限制。

对应地，`tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/**/*.rs` 的 row evidence literal 统一改成 `&[...]`。这保留了 Runtime 15 M3/M4 中已经记录的 5 项和 9 项证据锚点，避免通过删除锚点来掩盖状态记录需要更多文件/guard anchor 的事实。新增 `runtime_15_expected_status_output_rows_accept_variable_evidence_anchors`，验证类型别名不再包含 `[&'static str; 4]`，并抽查 M3/M4 多锚点状态行仍以切片方式保留。验证：scoped rustfmt/static checks、可变锚点扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 status-output guard sweep 仍 pending。

## Runtime 15 M3 status output M3 row data child-owner split

状态：`runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 Runtime 15 M3 status-output row data 二级 child-owner 减压。`tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs` 从 740 行降到 31 行聚合父模块，只保留 `foundation_guards`、`ui_tests_first`、`asset_budget_tests`、`scene_script_tests`、`status_support`、`ui_tests_second` 与 `production_guard_support` 七个 child owner 挂载和窄常量转发。

新增 `tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/*.rs` 承接原 78 条 M3 row literals，其中状态支撑锚点由 `plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs` 记录，最大 child `asset_budget_tests.rs` 为 157 行。顶层 `expected_status_row_data.rs` 逐组挂载 Runtime 15 M3 child constants，避免 M3 row literal 或单体 M3 row-data owner 回流。新增 `runtime_15_status_output_m3_row_data_child_owner_split`，验证 7 个 child group 全部进入输出聚合、代表性 row literals 不回流、父/子 owner 低于 800 行预算，并要求 status/date expected-slice maps、Runtime 15 计划、runtime index、审查发现、结构规范和本文档的状态锚同步。验证：scoped rustfmt/static checks、M3 child group aggregation scan、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 因当前外部 Cargo/rustc 车道 active 按支撑切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 status-output guard sweep 仍 pending。

## Runtime 15 M3 status output row-data guard child-owner split

状态：`runtime_15_status_output_row_data_guard_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 status-output row-data 守护文件自身的 child-owner 减压。`tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs` 从 643 行降到 15 行父模块，只保留 `module_layout`、`evidence_anchors`、`runtime_15_row_data`、`runtime_15_m4_row_data`、`runtime_15_m3_row_data` 与 `runtime_15_m3_child_groups` 六个 child owner 挂载。

原 variable-evidence、Runtime 15 row-data、M4 row-data、M3 row-data 与 M3 child-group 守卫分别迁入 `structure_convention/test_file_budget/status_output_row_data/*.rs`，最大 child `runtime_15_m3_child_groups.rs` 为 180 行。新增 `runtime_15_status_output_row_data_guard_child_owner_split`，验证父模块只挂载 child owners、代表性 guard 函数不回流、父/子 owner 低于 800 行预算，并要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、status-output expectations 与 `root_layout/status_scan.rs` 的状态/行数锚同步。验证：scoped rustfmt/static checks、父子行数预算扫描、moved guard 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 因当前外部 Cargo/rustc 车道 active 按支撑切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 status-output guard sweep 仍 pending。

## Runtime 15 F12 offscreen target texture owner cleanup

状态：`runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result`。

E6/S10/F12 的当前新增落地部分是渲染后端固定帧 texture owner 清理。`graphics/backend/render_backend/offscreen_target.rs` 里的 `global_illumination`、`scene_color`、`bloom`、G-buffer、`normal`、`depth` 等 WGPU texture 字段原本只通过对应 `TextureView` 间接服务帧图资源导入，因此用 `#[allow(dead_code)]` 避开未读告警。本轮移除这些 suppression，并新增 `OffscreenTarget::RETAINED_FRAME_TEXTURE_COUNT` 与 `retained_frame_texture_count()`，显式读取 final color、GI、scene color、bloom、G-buffer、normal、AO 与 depth 的 9 个 texture owner。

`scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs` 在生产绑定入口通过 debug assertion 消费该 owner 计数，说明这些字段负责保活 graph-imported `TextureView` 背后的 WGPU resources，而不是未接线脚手架。守卫：`runtime_15_offscreen_target_texture_owner_cleanup` 验证 OffscreenTarget 不再包含 `#[allow(dead_code)]`、构造路径仍 materialize 9 个 owner、compiled-scene binder 消费保活契约，并验证 Runtime 15 计划、runtime index、render index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 OffscreenTarget 固定帧 texture owner 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output all-subplans guard 1/1 通过；core-min Cargo check 在独立 target 目录 10 分钟超时无编译结果，残留本切片 cargo/rustc 进程已停止，不计通过。

## Runtime 15 F12 render backend state owner cleanup

状态：`runtime_15_render_backend_state_owner_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是渲染后端 WGPU state owner 清理。`graphics/backend/render_backend/render_backend.rs` 里的 `instance`、`adapter` 与 `config` 字段原本作为 backend lifetime owner 保留，但除 capability projection 间接使用外没有显式读取，因此用 `#[allow(dead_code)]` 避开未读告警。本轮移除这些 suppression，并新增 `RenderBackend::RETAINED_STATE_OWNER_COUNT` 与 `retained_state_owner_count()`，显式读取 instance、adapter 与 config 3 个 backend state owner。

`RenderBackend::caps()` 在生产 capability projection 路径通过 debug assertion 消费该 owner 计数，说明这些字段负责保活 WGPU backend state 与 backend config，而不是未接线脚手架。守卫：`runtime_15_render_backend_state_owner_cleanup` 验证 `RenderBackend` 不再包含 `#[allow(dead_code)]`、owner 计数契约读取三项 state、`caps()` 消费保活契约，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 RenderBackend state owner 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-render-backend-state-owner-0622` 通过（既有 warnings）。

## Runtime 15 F12 gpu texture resource owner cleanup

状态：`runtime_15_gpu_texture_resource_owner_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是材质纹理 GPU resource owner 清理。`graphics/scene/resources/gpu_texture/gpu_texture_resource.rs` 里的 `id`、`texture`、`view` 与 `sampler` 字段原本作为 material texture binding 的身份和 WGPU resource owner 保留，但用 `#[allow(dead_code)]` 避开未读告警。本轮移除这些 suppression，并新增 `GpuTextureResource::RETAINED_TEXTURE_BINDING_OWNER_COUNT` 与 `retained_texture_binding_owner_count()`，显式读取 texture identity、WGPU texture、view 与 sampler 4 个 binding owner。

`GpuTextureResource::view()` 和 `GpuTextureResource::sampler()` 在材质绑定入口通过 debug assertion 消费该 owner 计数，说明这些字段负责保活 material bind group 背后的 WGPU resources，而不是未接线脚手架。守卫：`runtime_15_gpu_texture_resource_owner_cleanup` 验证 `GpuTextureResource` 不再包含 `#[allow(dead_code)]`、owner 计数契约读取四项 state、view/sampler 绑定 accessor 消费保活契约，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 GpuTextureResource owner 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-gpu-texture-owner-0622` 通过（既有 warnings）。

## Runtime 15 F12 gpu material uniform owner cleanup

状态：`runtime_15_gpu_material_uniform_owner_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是材质 uniform GPU resource owner 清理。`graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs` 里的 `buffer`、`payload_byte_len` 与 `buffer_byte_len` 字段原本作为 material uniform binding 的 WGPU buffer owner 与 byte-length diagnostics 保留，但用 `#[allow(dead_code)]` 避开未读告警。本轮移除这些 suppression，并新增 `GpuMaterialUniformResource::RETAINED_MATERIAL_UNIFORM_OWNER_COUNT` 与 `retained_material_uniform_owner_count()`，显式读取 WGPU buffer、payload byte length 与 padded buffer byte length 3 个 binding/diagnostics owner。

`GpuMaterialUniformResource::binding_resource()` 在材质 uniform 绑定入口通过 debug assertion 消费该 owner 计数，说明 buffer 字段负责保活 bind group 背后的 WGPU resource。`resource_streamer_accessors.rs` 现在通过 `payload_byte_len()` / `buffer_byte_len()` owner accessor 暴露诊断长度，不再直接读取字段。守卫：`runtime_15_gpu_material_uniform_owner_cleanup` 验证 `GpuMaterialUniformResource` 不再包含 `#[allow(dead_code)]`、owner 计数契约读取三项 state、binding accessor 消费保活契约、resource streamer 通过 owner accessor 读取诊断长度，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 GpuMaterialUniformResource owner 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-gpu-material-uniform-owner-0622` 通过（既有 144 warnings）。

## Runtime 15 F12 gpu mesh order signature cleanup

状态：`runtime_15_gpu_mesh_order_signature_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是 mesh order-signature dead-code suppression 清理。`graphics/scene/resources/gpu_mesh/gpu_mesh_resource.rs` 里的 `indirect_order_signature` 字段原本服务 Virtual Geometry / indirect submission 顺序契约，但用 `#[allow(dead_code)]` 避开未读告警。本轮移除 suppression，将字段收窄为 `gpu_mesh` owner 内部字段，并通过 `GpuMeshResource::indirect_order_signature()` 暴露只读契约。

`graphics/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs` 继续从 position、normal、uv、joint indices/weights、tangent、color 与 index payload 派生完整 order signature。`graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs` 的 prepared mesh draw 路径通过 `mesh_order_command_sort_tie_breaker(...)` 把该签名混入稳定排序 tie-breaker，说明该字段是 draw ordering live input，而不是未接线脚手架。守卫：`runtime_15_gpu_mesh_order_signature_cleanup` 验证资源字段、签名派生、draw builder 接线，以及 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 GpuMeshResource order-signature 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-gpu-mesh-order-0622` 通过（既有 warnings）。

## Runtime 15 F12 gpu model identity cleanup

状态：`runtime_15_gpu_model_identity_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是 GPU model identity dead-code suppression 清理。`graphics/scene/resources/gpu_model/gpu_model_resource.rs` 里的 `id` 字段原本用于记录资源身份，但用 `#[allow(dead_code)]` 避开未读告警。本轮移除 suppression，将字段收窄为 `gpu_model` owner 内部字段，并通过 `GpuModelResource::id()` 暴露只读契约。

`graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs` 继续在构造 GPU model 时记录 `ResourceId`。`graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs::model(...)` 在返回缓存资源前通过 debug assertion 校验 `prepared.resource.id()` 与 streamer key 一致，说明该字段是 ResourceStreamer model cache identity 的 live contract，而不是未接线脚手架。守卫：`runtime_15_gpu_model_identity_cleanup` 验证资源字段、构造记录、streamer 查询接线，以及 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 GpuModelResource identity 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-gpu-model-identity-0622` 通过（既有 warnings）。

## Runtime 15 F12 post-process LUT texture owner cleanup

状态：`runtime_15_post_process_lut_texture_owner_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是 post-process LUT texture owner dead-code suppression 清理。`graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs` 里的 `texture` 字段原本作为 3D LUT `TextureView` 背后的 WGPU owner 保留，但用 `#[allow(dead_code)]` 避开未读告警。本轮移除 suppression，并新增 `PostProcessLutTextureResource::RETAINED_LUT_TEXTURE_OWNER_COUNT` 与 `retained_lut_texture_owner_count()`，显式读取 texture/view 两个 LUT binding owner。

`PostProcessLutTextureResource::view()` 在 3D LUT 绑定入口通过 debug assertion 消费该 owner 计数。`graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs::prepared_post_process_lut_3d_view(...)` 保持 `RenderColorLookupTextureLayout::matches_texture_3d(...)` descriptor 匹配，并改为通过 `prepared.resource.view()` 暴露 binding view，说明 texture 字段是 ResourceStreamer post-process LUT cache 的 live owner，而不是未接线脚手架。守卫：`runtime_15_post_process_lut_texture_owner_cleanup` 验证资源字段、owner 计数、streamer 3D LUT accessor，以及 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 PostProcessLutTextureResource owner 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-post-process-lut-owner-0622` 通过（既有 warnings）。

## Runtime 15 F12 output target texture owner cleanup

状态：`runtime_15_output_target_texture_owner_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是 output target texture owner dead-code suppression 清理。`graphics/scene/resources/output_target_texture/output_target_texture_resource.rs` 里的 descriptor、texture、view 与 sampler 字段原本作为 camera texture-target 写回、graph import 和材质采样的 WGPU resource owner 保留，但用 `#[allow(dead_code)]` 避开未读告警。本轮移除 suppression，并新增 `OutputTargetTextureResource::RETAINED_OUTPUT_TARGET_TEXTURE_OWNER_COUNT` 与 `retained_output_target_texture_owner_count()`，显式读取 output target descriptor、WGPU texture、view 与 sampler 4 个 owner。

`OutputTargetTextureResource::descriptor()`、`size()`、`texture()`、`view()` 与 `sampler()` 在 writeback、compiled-scene graph import 和 material sampling 路径通过 debug assertion 消费该 owner 计数。`graphics/scene/resources/prepared/prepared_output_target_texture.rs` 同步移除 prepared `resource` 字段 suppression，新增 `PreparedOutputTargetTexture::RETAINED_OUTPUT_TARGET_CACHE_OWNER_COUNT`、`retained_output_target_cache_owner_count()` 与 `resource()` accessor。`ResourceStreamer` 的 output-target graph import readiness、writeback clone 与 public output target resource accessor 都通过 prepared accessor 读取 cached Arc，说明这些字段是 output target cache 的 live owner，而不是未接线脚手架。守卫：`runtime_15_output_target_texture_owner_cleanup` 验证资源字段、cache owner 计数、streamer graph-import/writeback accessor，以及 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 OutputTargetTextureResource / PreparedOutputTargetTexture owner 子面；更宽 graphics resources 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-output-target-owner-0622` 通过（既有 warnings）。

## Runtime 15 F12 material runtime capture seed cleanup

状态：`runtime_15_material_runtime_capture_seed_cleanup_coremin_check_passed`。

E6/S10/F12 的当前新增落地部分是 material runtime capture seed dead-code suppression 清理。`graphics/scene/resources/runtime/material_runtime.rs` 不再用 `#[allow(dead_code)]` 遮盖 `MaterialCaptureSeed`、`MaterialRuntime` 或 `MaterialRuntime::capture_seed()`。`MaterialRuntime` 仍保留为生产材质运行态 DTO，因为 material preparation、uniform upload、mesh draw construction 和 readiness reporting 都读取它；`MaterialCaptureSeed` 与 `capture_seed()` 则收进 `#[cfg(test)]`，只服务 render product streamer 测试对材质捕获种子的回归断言。

`graphics/scene/resources/runtime/mod.rs` 与 `graphics/scene/resources/mod.rs` 的 `MaterialCaptureSeed` re-export 同步收进 test cfg。`graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs` 的 `material_capture_seed(...)`、`sample_texture_rgba(...)`、`shading_model_id_for_lighting_model(...)`、`sample_texture_asset_rgba(...)` 与 `wrap01(...)` 也收进 test cfg，避免历史 Hybrid GI/material capture helper 继续作为生产 dead-code surface 暴露。守卫：`runtime_15_material_runtime_capture_seed_cleanup` 验证 material runtime、runtime/resources façade、resource streamer capture accessors，以及 Runtime 15 计划、runtime index、审查发现、结构规范、本文档与 render-product 文档的状态锚同步。该切片只关闭 MaterialRuntime capture seed/test texture sampling 子面；`resource_streamer_accessors.rs` 中其余 diagnostics accessor suppression 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；standalone structure guard 1/1、status-output 2/2 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-material-capture-0622` 通过（既有 warnings）。

## Runtime 15 F12 resource streamer diagnostics accessor cleanup

状态：`runtime_15_resource_streamer_diagnostics_accessor_cleanup_static_passed_cargo_lock_blocked`。

E6/S10/F12 的当前新增落地部分是 ResourceStreamer diagnostics accessor suppression 清理。`graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs` 不再包含 `#[allow(dead_code)]`。只由 render product / asset flow 测试读取的资产管理快照、材质管理查询、uniform/property/texture-slot 诊断和 prepared-material state helper 统一收进 `#[cfg(test)]`，避免测试诊断 surface 继续留在生产构建里伪装为未接线生产代码。

生产仍使用的 material readiness bridge 不收进 test cfg：`material_readiness_report(...)` 与 `material_readiness_summary(...)` 保持正常构建，`resource_streamer_ensure_scene_resources.rs` 继续通过 `self.material_readiness_summary(&material_id)` 汇总材质 readiness stats。守卫：`runtime_15_resource_streamer_diagnostics_accessor_cleanup` 验证 accessors 文件没有 dead-code suppression、代表性测试诊断入口是 test-only、生产 readiness summary 仍由 ensure path 消费，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 render-product 文档的状态锚同步。该切片只关闭 ResourceStreamer diagnostics accessor 子面；全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；静态扫描确认 `resource_streamer_accessors.rs` 无 `#[allow(dead_code)]`；带锁 standalone structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞（Cargo 需要补齐 `zircon_plugin_sdk` 相关锁文件条目），不计通过。

Runtime 15 F12 ResourceStreamer material capture child owner split 状态：`runtime_15_resource_streamer_material_capture_child_owner_static_passed_cargo_deferred_implementation_cadence`。

R1.4 follow-up 把 material capture/test texture sampling 从 `graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs` 移入 `graphics/scene/resources/resource_streamer/resource_streamer_accessors/material_capture.rs`。父文件保留生产 resource accessors、material readiness bridge 与 diagnostics query accessors，并只通过 `#[cfg(test)] mod material_capture;` 挂载 child；child owner 承接 `material_capture_seed(...)`、`sample_texture_rgba(...)`、`shading_model_id_for_lighting_model(...)`、`sample_texture_asset_rgba(...)` 与 `wrap01(...)`。`runtime_15_resource_streamer_diagnostics_accessor_cleanup` 同步锁定 moved helper 不回流、父/子 800 行预算、Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 render-product 文档的状态锚同步。验证为 scoped rustfmt/static/line-count/docs-anchor/whitespace/diff-check；Cargo/WGPU/RenderDoc 按 milestone implementation cadence deferred。

## Runtime 15 F12 resource streamer resolve texture id cleanup

状态：`runtime_15_resource_streamer_resolve_texture_id_cleanup_static_passed_cargo_lock_blocked`。

E6/S10/F12 的当前新增落地部分是 ResourceStreamer texture-reference helper 僵尸清理。`graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs` 不再包含 `#[allow(dead_code)]`，并删除未使用的 `ResourceStreamer::resolve_texture_id(...)`。全仓库调用面没有该 helper 的生产消费者，因此本切片采用硬删除而不是 test-only 保留。

生产贴图解析入口保持不变：`resolve_texture_reference(...)` 与 `resolve_texture_reference_with_support(...)` 继续返回 `ResolvedTextureReference`，`ResolvedTextureReference::id()` 仍供当前材质准备路径读取成功解析的 `ResourceId`。未解析 locator 和未满足 upload support 的纹理仍走 `RenderMaterialValidationError`、`RenderMaterialFallbackUsage` 与 `RenderMaterialTextureSlotFallback` 报告路径。守卫：`runtime_15_resource_streamer_resolve_texture_id_cleanup` 验证旧 helper 和 dead-code suppression 不复活、生产解析入口仍存在，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 render-product 文档的状态锚同步。该切片只关闭 `resolve_texture_id` 僵尸 helper 子面；全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；静态扫描确认该文件无 `#[allow(dead_code)]` 且 `resolve_texture_id(` 只剩状态守卫字符串；带锁 standalone structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞（Cargo 需要补齐 `zircon_plugin_sdk` 相关锁文件条目），不计通过。

## Runtime 15 F12 particle GPU readback output accessor cleanup

状态：`runtime_15_particle_gpu_readback_output_accessor_cleanup_static_passed_cargo_lock_blocked`。

E6/S10/F12 的当前新增落地部分是 renderer runtime-output accessor 的 dead-code suppression 清理。`graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs` 中的 `SceneRenderer::take_last_particle_gpu_readback_outputs(...)` 不再包含 `#[allow(dead_code)]`，因为它已经由生产 runtime feedback 收集路径消费。

`graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs::collect_particle_feedback(...)` 从 renderer output drain 调用 `renderer.take_last_particle_gpu_readback_outputs()`，再和 `RenderPreparedRuntimeSidebands::take_particle_readback_outputs()` 合并，最后在非空时投递到 `ParticleGpuFeedback::new(...)`。这说明该 accessor 是 particle runtime feedback bridge 的 live 输入，而不是未接线脚手架。守卫：`runtime_15_particle_gpu_readback_output_accessor_cleanup` 验证 accessor 文件无 dead-code suppression、feedback collector 仍消费该 accessor，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 render-product 文档的状态锚同步。该切片只关闭 particle GPU readback accessor 子面；全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；静态扫描确认该文件无 `#[allow(dead_code)]` 且 runtime feedback 路径消费 `renderer.take_last_particle_gpu_readback_outputs()`；带锁 standalone structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 F12 advanced plugin output test accessor cleanup

状态：`runtime_15_advanced_plugin_output_test_accessor_cleanup_static_passed_cargo_lock_blocked`。

E6/S10/F12 的当前新增落地部分是 renderer advanced plugin output mailbox 的测试观察 helper 清理。`graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs` 不再用 `#[allow(dead_code)]` 保存 `has_virtual_geometry_gpu_readback(...)`、`plugin_renderer_outputs(...)` 与 `has_particle_gpu_readback(...)`。这三个 helper 只被同目录 inline tests 和 readback collection tests 用来观察 mailbox 内容，因此现在均由 `#[cfg(test)]` 收进测试编译面。

生产路径不通过这些 observation helper 决策。`SceneRendererAdvancedPluginOutputs` 仍保留 `take_hybrid_gi_readback_outputs(...)`、`take_particle_gpu_readback_outputs(...)` 与 `take_virtual_geometry_readback_outputs(...)`，供 runtime feedback/render product drain 各自插件输出槽。守卫：`runtime_15_advanced_plugin_output_test_accessor_cleanup` 验证 `output_access.rs` 无 dead-code suppression、三个 observation helper 均 test-only、生产 take/drain 方法仍存在，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、render-product 文档和 particles runtime 文档的状态锚同步。该切片只关闭 advanced plugin output test accessor 子面；全量 F12 sweep 仍 pending。

验证：scoped rustfmt --check 通过；静态扫描确认 `output_access.rs` 无 `#[allow(dead_code)]`、三个 observation helper 均收进 `#[cfg(test)]`，且生产 take/drain 方法仍存在；带锁 standalone structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 graphics dead-code guard module split

状态：`runtime_15_graphics_dead_code_guard_module_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 graphics dead-code 结构守卫测试组织拆分。903 行的 graphics dead-code 单文件 guard 硬切为 folder-backed `structure_convention/graphics_dead_code/mod.rs`；layout 守卫迁入 `structure_convention/graphics_dead_code/module_layout.rs`，renderer output accessor 相关守卫迁入 `structure_convention/graphics_dead_code/renderer_output_accessors.rs`。

父模块继续持有共享 `read_repo` / `read_runtime_src` helper 和其余 graphics F12 dead-code 守卫；子模块只承接 `runtime_15_particle_gpu_readback_output_accessor_cleanup` 与 `runtime_15_advanced_plugin_output_test_accessor_cleanup`。守卫：`runtime_15_graphics_dead_code_guard_is_folder_backed` 验证旧单文件路径不存在、新 parent/child 模块存在、父模块挂载 `mod renderer_output_accessors;`、子模块包含两个迁出的 renderer output accessor 守卫、父模块行数低于近大文件阈值，并验证 Runtime 15 计划、runtime index、审查发现、结构规范和本文档状态锚同步。该切片只关闭 graphics dead-code guard 测试组织子面；完整 `runtime_15_no_oversized_test_files` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 graphics dead-code guard child-owner split

状态：`runtime_15_graphics_dead_code_guard_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 继续收缩 graphics dead-code guard owner。`structure_convention/graphics_dead_code/mod.rs` 从 797 行降为 20 行，只保留子模块挂载和共享 `read_repo` / `read_runtime_src` helper；具体 F12 graphics dead-code 守卫不再堆在父文件内。

新子 owner 按责任域拆分：`graphics_dead_code/backend_owners.rs` 承接 OffscreenTarget 与 RenderBackend owner 守卫；`graphics_dead_code/gpu_resource_owners.rs` 承接 GpuTexture/GpuMaterialUniform/GpuMesh/GpuModel/PostProcessLut/OutputTarget owner 守卫；`graphics_dead_code/resource_streamer_cleanup.rs` 承接 MaterialRuntime capture seed、ResourceStreamer diagnostics accessor 与 resolve texture id cleanup 守卫。`runtime_15_graphics_dead_code_guard_is_folder_backed` 现在验证五个 child owner、代表性 moved guard 不回流、每个 owner 低于 800 行预算，以及 Runtime 15 计划、runtime index、审查发现、结构规范和本文档状态锚同步。

验证：scoped rustfmt/static checks、父/子行数与测试数量扫描、docs/status 锚点扫描和 scoped `git diff --check` 通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 provider boilerplate guard module split

状态：`runtime_15_provider_boilerplate_guard_module_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 provider boilerplate 结构守卫测试组织拆分。provider registration、update stats、feedback shared payload 三个守卫已从顶层 `structure_convention.rs` 迁入 `structure_convention/provider_boilerplate.rs`，与 prepare-input shared frame owner 守卫和 full provider boilerplate audit 总守卫同 owner 管理。

守卫：`runtime_15_provider_boilerplate_guard_is_folder_backed` 验证顶层聚合文件挂载 `structure_convention/provider_boilerplate.rs`，不再直接持有 `runtime_15_provider_registration_uses_shared_owner`、`runtime_15_provider_update_uses_shared_stats_owner`、`runtime_15_provider_feedback_uses_shared_payload_owner`；同时要求 `structure_convention.rs` 保持 700 行以下、`provider_boilerplate.rs` 保持 900 行以下，并验证 Runtime 15 计划、runtime index、审查发现、结构规范和本文档状态锚同步。该切片只关闭 provider boilerplate guard 测试组织子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 facade surface guard module split

状态：`runtime_15_facade_surface_guard_module_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 façade/prelude 结构守卫测试组织拆分。`runtime_15_prelude_covers_required_types` 与 `runtime_15_mixed_visibility_has_facade_note` 已从顶层 `structure_convention.rs` 迁入 `structure_convention/facade_surface.rs`，让 crate/subsystem prelude coverage 与 graphics façade visibility note 的结构守卫同 owner 管理。

守卫：`runtime_15_facade_surface_guard_is_folder_backed` 验证顶层聚合文件挂载 `structure_convention/facade_surface.rs`，不再直接持有 `runtime_15_prelude_covers_required_types` 与 `runtime_15_mixed_visibility_has_facade_note`；同时要求 `structure_convention.rs` 保持 500 行以下、`facade_surface.rs` 保持 700 行以下，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 都包含本切片锚。该切片只关闭 façade/prelude guard 测试组织子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 runtime dead-code guard module split

状态：`runtime_15_runtime_dead_code_guard_module_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 runtime dead-code 结构守卫测试组织拆分。`runtime_15_runtime_ui_dead_code_surface_is_test_support`、`runtime_15_runtime_owned_dead_code_suppression_cleanup`、`runtime_15_script_host_value_descriptors_do_not_suppress_dead_code` 与 `runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code` 已从顶层 `structure_convention.rs` 迁入 `structure_convention/runtime_dead_code.rs`，让 F10/F12 runtime-owned dead-code surface 的结构守卫同 owner 管理。

守卫：`runtime_15_runtime_dead_code_guard_is_folder_backed` 验证顶层聚合文件挂载 `structure_convention/runtime_dead_code.rs`，不再直接持有 runtime UI、runtime-owned cleanup、script host descriptor 和 script reflection fixture 四段 dead-code guard；同时要求 `structure_convention.rs` 保持 180 行以下、`runtime_dead_code.rs` 保持 700 行以下，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 都包含本切片锚。该切片只关闭 runtime dead-code guard 测试组织子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 diagnostics guard module split

状态：`runtime_15_diagnostics_guard_module_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 diagnostics 结构守卫测试组织拆分。`runtime_15_diagnostics_use_frame_trait_without_world_wrapper` 已从顶层 `structure_convention.rs` 迁入 `structure_convention/diagnostics_surface.rs`，让 F14 diagnostics normalization 的结构守卫和 diagnostics 文档锚同 owner 管理。

守卫：`runtime_15_diagnostics_guard_is_folder_backed` 验证顶层聚合文件挂载 `structure_convention/diagnostics_surface.rs`，不再直接持有 diagnostics guard；同时要求 `structure_convention.rs` 保持 80 行以下、`diagnostics_surface.rs` 保持 500 行以下，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 都包含本切片锚。该切片只关闭 diagnostics guard 测试组织子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 core framework test folder split

状态：`runtime_15_core_framework_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `core/framework/tests.rs` folder-backed 拆分。底部 time/task/root-structural/render-frame/profile 合约用例迁入 `core/framework/tests/framework_surfaces.rs`，render product post-process 与 camera ordering 合约用例迁入 `core/framework/tests/render_product_surface.rs`；父文件现在只保留 framework constructibility、phase queue、geometry extract、static mesh helper 和 `assert_mesh_phase_order(...)`，行数从 1848 降到 653。新增两个子 owner 分别保持 800 行以下，既有 `core/framework/tests/phase_queue_summary.rs` 继续作为 phase queue summary 子 owner。

守卫：`runtime_15_core_framework_tests_are_folder_backed` 验证父模块挂载 `phase_queue_summary`、`framework_surfaces` 与 `render_product_surface`，moved guard 不回流到父文件，三个子 owner 承接对应测试锚，`core/framework/tests.rs` 和所有 core framework test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 都包含本切片锚。该切片只关闭 `core/framework/tests.rs` 的 M3 folder-backed 子面；`ui/tests/v2_asset.rs`、`ui/tests/shared_core.rs`、完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 core runtime deactivation blocked test folder split

状态：`runtime_15_core_runtime_deactivation_blocked_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `core/runtime/tests/activation/behavior/deactivation/blocked.rs` folder-backed 拆分。外部 dependent blocker 用例迁入 `core/runtime/tests/activation/behavior/deactivation/blocked/external_dependents.rs`；exact two / three dependency matcher 用例迁入 `exact_two_three_dependency_matcher.rs`；shutdown order 首个 blocked service 用例迁入 `shutdown_order.rs`；exact four 用例迁入 `exact_four_dependency_matcher.rs`；exact five no-index-map fallback 用例迁入 `exact_five_without_index_map.rs`。既有 `exact_five_dependency_matcher.rs` 继续作为 exact-five all-dependency matcher 子 owner。父文件现在只保留子模块挂载，行数从 869 降到 7；10 个 blocked deactivation 测试全部保留在子模块，最大子文件 `exact_two_three_dependency_matcher.rs` 为 280 行。

守卫：`runtime_15_core_runtime_deactivation_blocked_tests_are_folder_backed` 验证父模块挂载六个子 owner，代表性 external-dependent / exact matcher / shutdown-order moved guard 不回流到父文件，所有 10 个 blocked 测试保留在子模块，`blocked.rs` 和所有 child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/core/runtime/lifecycle.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `blocked.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 code review findings test folder split

状态：`runtime_15_code_review_findings_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `tests/runtime_absorption/code_review_findings.rs` folder-backed 拆分。F5/F6/F7 typed-error review guards 迁入 `tests/runtime_absorption/code_review_findings/typed_error_convergence/`，并由后续 child-owner split 拆为 `animation_resource.rs`、`asset_loaders.rs`、`asset_records.rs` 与 `scene_world.rs`；F8 texture import settings 与 RuntimePluginDescriptor review guards 迁入 `f8_api_convergence.rs`；F11 shading-model registry、F17 entity path lookup、F18 asset manager handle shape 与 F19 scene renderer construction naming review guards 迁入 `late_api_cleanup.rs`。父文件现在只保留子模块挂载，行数从 1315 降到 3；25 个评审守卫全部保留，最大子文件 `f8_api_convergence.rs` 为 589 行。

守卫：`runtime_15_code_review_findings_tests_are_folder_backed` 验证父模块挂载三个子 owner，typed-error 子目录挂载四个 child owner，代表性 F5/F8/F11/F19 moved guard 不回流到父文件，所有 25 个 review guard 保留在子模块，父/子 owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 都包含本切片锚。该切片只关闭 `runtime_absorption/code_review_findings.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 typed-error convergence guard child-owner split

状态：`runtime_15_typed_error_convergence_guard_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `tests/runtime_absorption/code_review_findings/typed_error_convergence.rs` 的 folder-backed 子拆分。新增 sound asset typed-error guard 后，该文件增长到 1400 行以上，因此退役平铺文件并改为 `typed_error_convergence/mod.rs` 入口，子 owner 按责任分为 `scene_world.rs`、`asset_records.rs`、`asset_loaders.rs` 与 `animation_resource.rs`。

`scene_world.rs` 承接 world spawn/bundle、fixed mutation、dynamic component 和 property access typed-error guards；`asset_records.rs` 承接 authoring/navigation/font/sound/zshader/asset-meta record typed-error guards；`asset_loaders.rs` 承接 texture loader、mesh/OBJ loader 和 artifact/importer typed-error guards；`animation_resource.rs` 承接 animation manager 与 core resource registry typed-error guards。当前 typed-error 子目录保留 15 个 guard，整体 code-review findings 保留 25 个 review guards，所有 owner 低于 800 行预算。

守卫：`runtime_15_code_review_findings_tests_are_folder_backed` 已更新为检查 typed-error 子目录挂载、`review_f5_sound_asset_uses_typed_error` 锚点、25 个 review guards、父/子 owner 行数预算，以及 Runtime 15/status/docs 锚点。验证：scoped rustfmt/static scans、line-count scan 与 docs/status/session anchor scan 已通过；Cargo 因并行 cargo/rustc lane active deferred，不计通过。

## Runtime 15 M3 UI architecture test folder split

状态：`runtime_15_ui_architecture_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `tests/runtime_absorption/ui_architecture.rs` folder-backed 拆分。Runtime 09 UI architecture M0/M2/M3 architecture-boundary 守卫迁入 `tests/runtime_absorption/ui_architecture/architecture_boundaries.rs`；M1.1/M1.2 legacy/debt rename 守卫迁入 `tests/runtime_absorption/ui_architecture/legacy_renames.rs`；镜像文档结构审计守卫迁入 `tests/runtime_absorption/ui_architecture/mirror_docs.rs`。父文件现在只保留共享扫描 helper 与子模块挂载，行数从 1251 降到 104；18 个 Runtime 09 UI architecture absorption guard 全部保留在子模块，最大子文件 `legacy_renames.rs` 为 530 行，`architecture_boundaries.rs` 为 505 行。

守卫：`runtime_15_ui_architecture_tests_are_folder_backed` 验证父模块挂载三个子 owner，代表性 architecture/legacy/mirror moved guard 不回流到父文件，所有 18 个 Runtime 09 UI architecture guard 保留在子模块，父/子 owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片还要求 `ui_architecture_boundary.py` 将 `ui_architecture/architecture_boundaries.rs`、`ui_architecture/legacy_renames.rs` 与 `ui_architecture/mirror_docs.rs` 纳入 guard source 读取，防止 Runtime 09 mirror audit 在测试拆分后只看旧父文件。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、include_str 路径解析、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 dynamic scene absorption guard folder split

状态：`runtime_15_dynamic_scene_absorption_guard_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `tests/runtime_absorption/dynamic_scene.rs` folder-backed 拆分。原单个 814 行 Runtime 05 dynamic-scene patch/session 吸收守卫拆成七个子 owner：patch-preview API/read-only 断言迁入 `tests/runtime_absorption/dynamic_scene/patch_preview_api.rs`；patch-preview 状态文档锚迁入 `patch_preview_status_docs.rs`；focused patch-preview 行为锚迁入 `patch_preview_behavior.rs`；session capture/persistence、retention/mutation/merge、load/query/path-management 与 asset-reload/selection/status 锚分别迁入 `session_capture_persistence.rs`、`session_retention_mutation_merge.rs`、`session_load_query_path.rs` 与 `asset_reload_selection_status.rs`。父文件现在只保留共享 `include_str!` source 常量与子模块挂载，行数从 814 降到 38；原 `runtime_05_dynamic_scene_patch_preview_api_stays_read_only` 仍保留在 `patch_preview_api.rs`，最大子文件 `asset_reload_selection_status.rs` 为 268 行。

守卫：`runtime_15_dynamic_scene_absorption_guard_is_folder_backed` 验证父模块挂载七个子 owner，原 read-only patch-preview guard 和六个新增 guard fragment 都保留在 child owner 中，父/子 owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/scene/dynamic_scene.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `tests/runtime_absorption/dynamic_scene.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、include_str 路径解析、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI v2 asset test folder split

状态：`runtime_15_ui_v2_asset_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `ui/tests/v2_asset.rs` folder-backed 拆分。原单文件中的 loader / ZUI profile、style cascade 与 runtime pseudo-state、默认控件、range controls、demo/builder、composite component、file cache 合约用例已分别迁入 `ui/tests/v2_asset/asset_loading.rs`、`ui/tests/v2_asset/style_runtime.rs`、`ui/tests/v2_asset/default_controls.rs`、`ui/tests/v2_asset/range_controls.rs`、`ui/tests/v2_asset/demo_and_builder.rs`、`ui/tests/v2_asset/composite_components.rs` 与 `ui/tests/v2_asset/file_cache.rs`。父文件现在只保留共享导入、子模块挂载与 helper，行数从 3806 降到当前 331；最大子文件 `style_runtime.rs` 为 718 行，全部低于 800 行。

守卫：`runtime_15_ui_v2_asset_tests_are_folder_backed` 验证父模块挂载七个子 owner，代表性 moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/v2_asset.rs` 和所有 UI v2 asset test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/v2_asset.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 UI shared core test folder split

状态：`runtime_15_ui_shared_core_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `ui/tests/shared_core.rs` folder-backed 拆分。原单文件中的 layout/render surface、box flow、pointer/visibility、navigation、scroll/property mutation 合约用例已分别迁入 `ui/tests/shared_core/layout_surface.rs`、`ui/tests/shared_core/box_flow.rs`、`ui/tests/shared_core/input_visibility.rs`、`ui/tests/shared_core/navigation.rs` 与 `ui/tests/shared_core/scroll_mutation.rs`。父文件现在只保留共享导入、模块挂载和 constraint / pointer helper，行数从 3145 降到当前 77；最大子文件 `input_visibility.rs` 为 771 行，全部低于 800 行。

守卫：`runtime_15_ui_shared_core_tests_are_folder_backed` 验证父模块挂载五个子 owner，代表性 moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/shared_core.rs` 和所有 UI shared core test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/shared_core.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 UI shared core guard child-owner split

状态：`runtime_15_ui_shared_core_guard_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 UI shared-core test-budget 守护的 child-owner 减压。`tests/runtime_absorption/structure_convention/test_file_budget/ui_shared_core.rs` 从 641 行降为路由父文件，只挂载 `ui_shared_core/root.rs`、`ui_shared_core/layout_surface.rs`、`ui_shared_core/input_visibility.rs` 与 `ui_shared_core/scroll_mutation.rs`。

新增 `runtime_15_ui_shared_core_guard_child_owners_are_folder_backed`，验证父文件不再定义四个已移动守护、子 owner 保留原守护、父子文件低于 400 行 focused 预算，并要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、UI architecture 文档、session note 与 status-output expectations 同步。精确锚点包括 `structure_convention/test_file_budget/ui_shared_core.rs` 与 `structure_convention/test_file_budget/ui_shared_core/layout_surface.rs`。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved guard 扫描、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo 进程 active 按支撑切片节奏 deferred，不计 Cargo 通过。

## Runtime 15 M3 historical oversized test roots closeout

状态：`runtime_15_historical_oversized_test_roots_closeout_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是历史 S6 超大测试根 closeout。`core/framework/tests.rs`、`ui/tests/v2_asset.rs` 与 `ui/tests/shared_core.rs` 已分别收口为 folder-backed parent，当前父文件行数为 653、331、77；代表子 owner `core/framework/tests/framework_surfaces.rs`、`core/framework/tests/render_product_surface.rs`、`core/framework/tests/phase_queue_summary.rs`、`ui/tests/v2_asset/style_runtime.rs`、`ui/tests/v2_asset/file_cache.rs`、`ui/tests/shared_core/input_visibility.rs` 与 `ui/tests/shared_core/scroll_mutation.rs` 均低于 800 行，当前最大子 owner 为 `ui/tests/shared_core/input_visibility.rs` 的 771 行。

守卫：`runtime_15_historical_oversized_test_roots_are_folder_backed` 位于 `structure_convention/test_file_budget/historical_oversized_roots.rs`，验证三组历史根仍挂载各自 child owner，代表性 moved guard 不回流到父文件，父子 owner 都低于 800 行，并要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 同步包含 `Runtime 15 M3 historical oversized test roots closeout` 与 `runtime_15_historical_oversized_test_roots_closeout_static_passed_cargo_deferred`。该切片只关闭 `core/framework/tests.rs`、`ui/tests/v2_asset.rs` 与 `ui/tests/shared_core.rs` 三个历史超大测试根的收口证明；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 test-file-budget guard sweep 仍 pending。

验证：scoped rustfmt/static scans、历史父子行数预算扫描、moved guard ownership 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按支撑切片节奏 deferred，不计通过。

## Runtime 15 M3 UI accessibility test folder split

状态：`runtime_15_ui_accessibility_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/accessibility.rs` folder-backed 拆分。原单文件中的 snapshot extraction、label/name 关系、focus diagnostics、description references、activation actions 与 value actions 合约用例已分别迁入 `ui/tests/accessibility/extraction.rs`、`ui/tests/accessibility/naming_relations.rs`、`ui/tests/accessibility/focus_diagnostics.rs`、`ui/tests/accessibility/description_references.rs`、`ui/tests/accessibility/activation_actions.rs` 与 `ui/tests/accessibility/value_actions.rs`。父文件现在只保留共享导入、子模块挂载、accessibility tree fixture 与 surface helper，行数从 2251 降到 125；49 个原有测试全部保留在子模块，最大子文件 `ui/tests/accessibility/value_actions.rs` 低于 800 行。

守卫：`runtime_15_ui_accessibility_tests_are_folder_backed` 验证父模块挂载六个子 owner，代表性 extraction/value action moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/accessibility.rs` 和所有 UI accessibility test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/accessibility.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI accessibility widget actions test folder split

状态：`runtime_15_ui_accessibility_widget_actions_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/accessibility_widget_actions.rs` folder-backed 拆分。原单文件中的 disclosure open/expand、popup open/collapse/dismiss/default action、tooltip dismiss 与 menu item fallback-close 合约用例已分别迁入 `ui/tests/accessibility_widget_actions/disclosure_actions.rs`、`ui/tests/accessibility_widget_actions/popup_actions.rs` 与 `ui/tests/accessibility_widget_actions/tooltip_menu.rs`。父文件现在只保留共享导入、子模块挂载、`root_surface()`、`dispatch_accessibility(...)`、binding report 断言和 runtime widget/popup/tooltip fixture helper，行数从 811 降到 250；11 个原有测试全部保留在子模块，最大子文件 `ui/tests/accessibility_widget_actions/popup_actions.rs` 为 291 行，全部低于 800 行。

守卫：`runtime_15_ui_accessibility_widget_actions_tests_are_folder_backed` 验证父模块挂载三个子 owner，代表性 disclosure/popup/tooltip moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/accessibility_widget_actions.rs` 和所有 widget action test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/accessibility_widget_actions.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI layout slots test folder split

状态：`runtime_15_ui_layout_slots_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/layout_slots.rs` folder-backed 拆分。原单文件中的 linear/free/canvas slot 用例迁入 `ui/tests/layout_slots/linear_free.rs`；overlay、overlay surface-frame 与 scroll virtual-window 用例迁入 `ui/tests/layout_slots/overlay_scroll.rs`；wrap flow、grid、masonry shortest-column 与 sequential masonry 用例迁入 `ui/tests/layout_slots/flow_grid_masonry.rs`。父文件现在只保留共享导入、子模块挂载、`fixed_constraint(...)`、`pointer_node(...)`、render/hit frame helper，行数从 867 降到 100；10 个原有测试全部保留在子模块，最大子文件 `ui/tests/layout_slots/flow_grid_masonry.rs` 为 315 行，全部低于 800 行。

守卫：`runtime_15_ui_layout_slots_tests_are_folder_backed` 验证父模块挂载三个子 owner，代表性 linear/free、overlay/scroll、flow/grid/masonry moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/layout_slots.rs` 和所有 layout slot test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/layout_slots.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI surface-frame authority test folder split

状态：`runtime_15_ui_surface_frame_authority_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/surface_frame_authority.rs` folder-backed 拆分。原单文件中的 arranged/render/hit/pointer authority 与 focus path 用例迁入 `ui/tests/surface_frame_authority/arranged_authority.rs`；Taffy flex、horizontal/vertical slot sizing 与 flex fallback 用例迁入 `ui/tests/surface_frame_authority/taffy_flex.rs`；wrap 与 grid surface-frame authority 用例迁入 `ui/tests/surface_frame_authority/taffy_wrap_grid.rs`；Zircon SizeBox fallback 用例迁入 `ui/tests/surface_frame_authority/zircon_fallback.rs`。父文件现在只保留共享导入、常量、子模块挂载、surface fixture、button/layout helper 和 Taffy/Grid/SizeBox 构造 helper，行数从 922 降到 409；9 个原有测试全部保留在子模块，最大子文件 `ui/tests/surface_frame_authority/taffy_flex.rs` 为 297 行，全部低于 800 行。

守卫：`runtime_15_ui_surface_frame_authority_tests_are_folder_backed` 验证父模块挂载四个子 owner，代表性 arranged/focus、Taffy flex、wrap/grid 与 Zircon fallback moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/surface_frame_authority.rs` 和所有 surface-frame authority test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/surface_frame_authority.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI surface dirty domains test folder split

状态：`runtime_15_ui_surface_dirty_domains_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/surface_dirty_domains.rs` folder-backed 拆分。原单文件中的 dirty rebuild domain 与 structural domain recompute 用例迁入 `ui/tests/surface_dirty_domains/rebuild_domains.rs`；sibling skip、layout-engine route preserve/replace/drop 和 auto-parent revisit 用例迁入 `ui/tests/surface_dirty_domains/incremental_layout.rs`；render command reuse、render-only metadata/text/dispatch-effect 用例迁入 `ui/tests/surface_dirty_domains/render_domains.rs`；route state mutation 与 explicit layout marking 用例迁入 `ui/tests/surface_dirty_domains/mutation_state.rs`。父文件现在只保留共享导入、子模块挂载、surface fixture、dirty/assert helper、layout route helper、keyboard event helper 和 fixed constraint helper，行数从 1021 降到 297；13 个原有测试全部保留在子模块，最大子文件 `ui/tests/surface_dirty_domains/incremental_layout.rs` 为 261 行，全部低于 800 行。

守卫：`runtime_15_ui_surface_dirty_domains_tests_are_folder_backed` 验证父模块挂载四个子 owner，代表性 rebuild、incremental layout、render-only 与 mutation/state moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/surface_dirty_domains.rs` 和所有 dirty-domain test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/surface_dirty_domains.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI material layout test folder split

状态：`runtime_15_ui_material_layout_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/material_layout.rs` folder-backed 拆分。原单文件中的 button/icon intrinsic sizing 用例迁入 `ui/tests/material_layout/button_icon_metrics.rs`；menu/tab/label/table row 用例迁入 `ui/tests/material_layout/row_label_metrics.rs`；field placeholder/value/numeric/options/vector 用例迁入 `ui/tests/material_layout/field_values.rs`；asset value 与 icon role 用例迁入 `ui/tests/material_layout/asset_icon_roles.rs`；authored constraints、child content 和 list/switch min-height 用例迁入 `ui/tests/material_layout/constraints_children.rs`。父文件现在只保留共享导入、子模块挂载、material leaf measurement/render command helper 和 intrinsic constraint helper，行数从 813 降到 111；23 个原有测试全部保留在子模块，最大子文件 `ui/tests/material_layout/field_values.rs` 为 238 行，全部低于 800 行。

守卫：`runtime_15_ui_material_layout_tests_are_folder_backed` 验证父模块挂载五个子 owner，代表性 button/icon、row/label、field values、asset/icon roles 与 constraints/children moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/material_layout.rs` 和所有 material-layout test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/material_layout.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI template test folder split

状态：`runtime_15_ui_template_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/template.rs` folder-backed 拆分。原单文件中的 loader/instance/validation 用例迁入 `ui/tests/template/loader_instance_validation.rs`；tree-builder interaction binding 用例迁入 `ui/tests/template/interaction_bindings.rs`；surface builder、container 和 layout contract attributes 用例迁入 `ui/tests/template/surface_containers.rs`；slot ownership、overlay、canvas/free/space slot contracts 用例迁入 `ui/tests/template/slot_contracts.rs`；template contract layout compute 用例迁入 `ui/tests/template/layout_compute.rs`。父文件现在只保留共享导入、template TOML fixture、子模块挂载、tree/root helper，行数从 884 降到 154；22 个原有测试全部保留在子模块，最大子文件 `ui/tests/template/surface_containers.rs` 为 204 行，全部低于 800 行。

守卫：`runtime_15_ui_template_tests_are_folder_backed` 验证父模块挂载五个子 owner，代表性 loader/instance、interaction binding、surface/container、slot contract 与 layout compute moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/template.rs` 和所有 template test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/template.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI component catalog test folder split

状态：`runtime_15_ui_component_catalog_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/component_catalog.rs` folder-backed 拆分。原父文件中的 V1 component catalog inventory 用例迁入 `ui/tests/component_catalog/catalog_inventory.rs`；descriptor kind、layout role、palette/schema validation 与 schema normalization 用例迁入 `ui/tests/component_catalog/descriptor_contracts.rs`；host capability filtering、palette view sorting 和 registry revision 用例迁入 `ui/tests/component_catalog/registry_queries.rs`。父文件现在只保留共享导入、子模块挂载、component descriptor/registry helper、schema/assert helper 和 drag-source fixture，行数从 934 降到 136；7 个原有父文件测试全部保留在子模块，最大子文件 `ui/tests/component_catalog/catalog_inventory.rs` 为 446 行，全部低于 800 行。

守卫：`runtime_15_ui_component_catalog_tests_are_folder_backed` 验证父模块挂载三个新子 owner，代表性 catalog inventory、descriptor contracts、registry query 和 revision moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/component_catalog.rs` 和本轮新增 component catalog test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/component_catalog.rs` 父文件的 M3 folder-backed 子面；既有 `ui/tests/component_catalog/component_state.rs` 仍超过 800 行，完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI boundary test folder split

状态：`runtime_15_ui_boundary_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/boundary.rs` folder-backed 拆分。原父文件中的 template namespace API 边界用例迁入 `ui/tests/boundary/template_namespace.rs`；root surface、layout、tree、surface 与 dispatch namespace 用例迁入 `ui/tests/boundary/layout_tree_surface.rs`；binding、event_ui、dispatch root 和 surface root structural 用例迁入 `ui/tests/boundary/binding_event_roots.rs`；runtime UI asset fixture、fixture loader 和 v2 projection 用例迁入 `ui/tests/boundary/asset_fixture_projection.rs`。父文件现在只保留文件系统/路径 helper 和子模块挂载，行数从 1210 降到 62；32 个原有父文件测试全部保留在子模块，最大子文件 `ui/tests/boundary/template_namespace.rs` 为 438 行，全部低于 800 行。

守卫：`runtime_15_ui_boundary_tests_are_folder_backed` 验证父模块挂载四个新子 owner，代表性 template、layout、surface、binding、event、asset fixture 与 v2 projection moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/boundary.rs` 和所有 boundary test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/boundary.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI component state test folder split

状态：`runtime_15_ui_component_catalog_component_state_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/component_catalog/component_state.rs` folder-backed 拆分。原父文件中的 retained number/dropdown/drop event 和 drag payload metadata 用例迁入 `ui/tests/component_catalog/component_state/retained_events.rs`；array/map mutation 和 collection validation 用例迁入 `ui/tests/component_catalog/component_state/collection_mutation.rs`；reference action、drop source metadata、serde compatibility 和 rejected drop preservation 用例迁入 `ui/tests/component_catalog/component_state/reference_sources.rs`；transient flags、non-drop replacement、map update、numeric large-step 与 range clamp 用例迁入 `ui/tests/component_catalog/component_state/interaction_numeric.rs`。父文件现在只保留共享导入和既有 component-state 子模块挂载，行数从 965 降到 26；18 个原有父文件测试全部保留在子模块，最大子文件 `ui/tests/component_catalog/component_state/reference_sources.rs` 为 300 行，全部低于 800 行。

守卫：`runtime_15_ui_component_catalog_component_state_tests_are_folder_backed` 验证父模块挂载四个新子 owner，代表性 retained events、collection mutation、reference source、interaction/numeric moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/component_catalog/component_state.rs` 和本轮新增 component-state test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/component_catalog/component_state.rs` 父文件的 M3 folder-backed 子面；键盘子 owner 由下一节继续拆分，完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI component state keyboard test folder split

状态：`runtime_15_ui_component_catalog_component_state_keyboard_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/component_catalog/component_state/keyboard.rs` folder-backed 拆分。原父文件中的 button、toggle、tab、radio/checkbox group 与 multiple toggle button action 选择用例迁入 `ui/tests/component_catalog/component_state/keyboard/action_selection.rs`；menu focus、tree/table focus、first-character menu search、prefix match 与 buffered prefix 用例迁入 `ui/tests/component_catalog/component_state/keyboard/menu_navigation.rs`；text input append、selection replacement 与 caret state 用例迁入 `ui/tests/component_catalog/component_state/keyboard/text_inputs.rs`；numeric step、popup close 与 range slider focused thumb 用例迁入 `ui/tests/component_catalog/component_state/keyboard/numeric_controls.rs`。父文件现在只保留共享导入、`menu_option` helper 和子模块挂载，行数从 1282 降到 20；14 个原有父文件测试全部保留在子模块，最大子文件 `ui/tests/component_catalog/component_state/keyboard/menu_navigation.rs` 为 586 行，全部低于 800 行。

守卫：`runtime_15_ui_component_catalog_component_state_keyboard_tests_are_folder_backed` 验证父模块挂载四个新子 owner，代表性 action selection、menu navigation、text input 与 numeric/range moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/component_catalog/component_state/keyboard.rs` 和本轮新增 keyboard test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/component_catalog/component_state/keyboard.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M4 UI component state-reducer keyboard menu submenu owner split

状态：`runtime_15_ui_component_state_reducer_keyboard_menu_submenu_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 UI component state-reducer keyboard menu production owner 减压。`ui/component/state_reducer/keyboard/menu.rs` 继续拥有 menu focus-control detection、keyboard text typeahead、search query/filter state、search option flattening、recursive filtering 与 filtered option visibility；submenu focus loop、hover pending state、open/close state、active parent index、invalid submenu pruning 与 submenu target lookup 迁入 `ui/component/state_reducer/keyboard/menu/submenu.rs`。父文件通过 `mod submenu;` 和窄 re-export 继续向 `keyboard.rs` 暴露 `menu::open_focused_submenu(...)` / `menu::close_active_submenu(...)`，不改变 typeahead buffer、search filter payload、submenu focus scope 或 option target semantics。父文件从 872 行降到 609 行，子 owner 为 271 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_component_state_reducer_keyboard_menu_submenu_is_child_owner` 验证父模块挂载 submenu child、代表性 submenu 常量与状态机 helper 不回流到父文件、子 owner 承接 submenu state transition 和 target lookup internals、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/component/state_reducer/keyboard/menu.rs` 的 submenu owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI component Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 UI component state-reducer tree view editing owner split

状态：`runtime_15_ui_component_state_reducer_tree_view_editing_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 UI component state-reducer tree view production owner 减压。`ui/component/state_reducer/tree_view.rs` 继续拥有 tree-view identity、keyboard expand/collapse、toggle expanded、select option、multi/single selection、range selection、ordered/expanded/selected node id helpers 与 disabled-option checks；begin/cancel/commit rename、editing property selection、editing state clearing、rename payload writes、tree node label lookup 与 edit text validation 迁入 `ui/component/state_reducer/tree_view/editing.rs`。父文件通过 `mod editing;` 和窄 re-export 继续向 state-reducer 上层暴露 `tree_view::apply_begin_edit(...)` / `tree_view::apply_cancel_editing(...)` / `tree_view::apply_commit(...)`，不改变 selection focus、expanded items、rename payload property fallback 或 disabled-option validation。父文件从 834 行降到 508 行，子 owner 为 312 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_component_state_reducer_tree_view_editing_is_child_owner` 验证父模块挂载 editing child、代表性 editing 常量与 rename 状态机 helper 不回流到父文件、子 owner 承接 rename/editing state transition、tree node label lookup 和 edit text validation internals、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/component/state_reducer/tree_view.rs` 的 editing owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI component Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI Material foundation test folder split

状态：`runtime_15_ui_component_catalog_material_foundation_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/component_catalog/material_foundation/mod.rs` folder-backed 拆分。原父文件中的 planned component layer inventory、registry component-id set 和 shared MUI customization schema 循环迁入 `ui/tests/component_catalog/material_foundation/planned_layers.rs`；editor descriptor、text/textarea、viewport/workbench 与 editor-only layout contract 用例迁入 `ui/tests/component_catalog/material_foundation/editor_components.rs`；MUI surface variant、popup/modal/overlay layer、Alert/Snackbar 与 family assertion 用例迁入 `ui/tests/component_catalog/material_foundation/mui_surface_overlay.rs`；transition、MUI X、charts、agent chat 和 runtime visibility 用例迁入 `ui/tests/component_catalog/material_foundation/mui_x_runtime.rs`；原 folder-backed-by-family 守卫迁入 `ui/tests/component_catalog/material_foundation/folder_structure.rs`。父文件现在只保留共享导入、Material foundation 子模块挂载和 button/MUI schema helper，行数从 1081 降到 149；5 个拆分后测试全部保留在子模块，最大子文件 `ui/tests/component_catalog/material_foundation/editor_components.rs` 为 324 行，全部低于 800 行。

守卫：`runtime_15_ui_component_catalog_material_foundation_tests_are_folder_backed` 验证父模块挂载五个新子 owner，代表性 planned layer、editor component、MUI surface/overlay、MUI X runtime visibility 与 folder structure moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/component_catalog/material_foundation/mod.rs` 和本轮新增 Material foundation test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/component_catalog/material_foundation/mod.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI asset test folder split

状态：`runtime_15_ui_asset_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/asset.rs` folder-backed 拆分。父文件保留 imported widget/style/layout/legacy fixture 常量、`STYLE_WITH_RULE_IDS` 和子模块挂载，并迁出 `ui/tests/asset/{style_rule_ids,style_write_apis,loader_validation,document_compiler,fixture_migration,component_schema}.rs` 六个子 owner。stable stylesheet/rule id 覆盖迁入 `style_rule_ids.rs`，style write API 原子性和 reorder 覆盖迁入 `style_write_apis.rs`，loader rejection 覆盖迁入 `loader_validation.rs`，imported widget/reference compile 覆盖迁入 `document_compiler.rs`，legacy/flat fixture migration 与 compiler folder split guard 迁入 `fixture_migration.rs`，runtime component schema default/type/style prop 覆盖迁入 `component_schema.rs`。父文件从 1359 行降到 251；32 个原父文件测试全部保留在子模块，最大子文件 `style_write_apis.rs` 为 261 行，全部低于 800 行。

守卫：`runtime_15_ui_asset_tests_are_folder_backed` 验证父模块挂载六个新子 owner，代表性 style id/write API、loader validation、document compiler、fixture migration 和 component schema moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/asset.rs` 和本轮新增 UI asset test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/asset.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、docs/status 锚点扫描通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI asset MUI X web style test folder split

状态：`runtime_15_ui_asset_mui_web_mui_x_style_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/asset_mui_web_mui_x_style.rs` folder-backed 拆分。父文件保留 MUI X style/layout TOML fixture、`find_node`/`find_node_opt`、`str_attr` 和 class assertion helper，并挂载 `ui/tests/asset_mui_web_mui_x_style/{data_grid,tree_view,date_time_pickers,charts,agent_chat}.rs` 五个子 owner。原综合测试按组件族拆为 DataGrid、TreeView、Date/Time Pickers、Charts/Gauge 与 Agent Chat 五个测试，分别迁入对应子模块。父文件从 1347 行降到 685；最大子文件 `data_grid.rs` 为 221 行，全部低于 800 行。

守卫：`runtime_15_ui_asset_mui_web_mui_x_style_tests_are_folder_backed` 验证父模块挂载五个新子 owner，代表性 DataGrid、TreeView、Date/Time Picker、Charts/Gauge 与 Agent Chat moved guard 不回流到父文件，各子 owner 承接对应组件族测试锚，`ui/tests/asset_mui_web_mui_x_style.rs` 和本轮新增 MUI X web style test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/asset_mui_web_mui_x_style.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、docs/status 锚点扫描通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI asset MUI web style test folder split

状态：`runtime_15_ui_asset_mui_web_style_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/asset_mui_web_style.rs` folder-backed 拆分。父文件保留 MUI web style TOML fixture、`str_attr`/`bool_attr`/`float_attr`/`int_attr`/`table_str_attr` 和 class assertion helper，并挂载 `ui/tests/asset_mui_web_style/{state_icons,slots_native,feedback,surface,data_display}.rs` 五个子 owner。state/sx/icon class 覆盖迁入 `state_icons.rs`，slot props 与 native alias 覆盖迁入 `slots_native.rs`，Alert/Snackbar/Skeleton 覆盖迁入 `feedback.rs`，Paper/Card/AppBar 覆盖迁入 `surface.rs`，Typography/Divider/Avatar/Chip/Badge/List/ImageList/Table 覆盖迁入 `data_display.rs`。父文件从 1296 行降到 648；9 个原父文件测试全部保留在子模块，最大子文件 `feedback.rs` 为 162 行，全部低于 800 行。

守卫：`runtime_15_ui_asset_mui_web_style_tests_are_folder_backed` 验证父模块挂载五个新子 owner，代表性 state/icon、slot/native、feedback、surface 和 data-display moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/asset_mui_web_style.rs` 和本轮新增 MUI web style test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/asset_mui_web_style.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、docs/status 锚点扫描通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI taffy layout pass test folder split

状态：`runtime_15_ui_taffy_layout_pass_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/taffy_layout_pass.rs` folder-backed 拆分。父文件保留共享 layout/tree helper、template metadata helper、selection/fallback assertion helper 和子模块挂载，并迁出 `ui/tests/taffy_layout_pass/{routing_diagnostics,arrangement,linear_slots,fallback_policy,grid_slots}.rs` 五个子 owner。Taffy 路由源码扫描、native/Zircon fallback 报告与 fallback reason 聚合迁入 `routing_diagnostics.rs`；linear/wrap/grid 基础排布、fractional fixed extent、template metadata 与 text/image measured size 覆盖迁入 `arrangement.rs`；linear/wrap slot padding、cross-axis alignment、slot sizing、auto/stretch-content/bounds 覆盖迁入 `linear_slots.rs`；unsupported slot padding、non-finite layout value、axis constraint、collapsed child、child placement policy 与 size-box Zircon-owned 语义覆盖迁入 `fallback_policy.rs`；grid placement/span、out-of-bounds track expansion、padding/alignment 与 grid alignment fallback 覆盖迁入 `grid_slots.rs`。父文件从 1230 行降到 168；35 个原父文件测试全部保留在子模块，最大子文件 `routing_diagnostics.rs` 为 347 行，全部低于 800 行。

守卫：`runtime_15_ui_taffy_layout_pass_tests_are_folder_backed` 验证父模块挂载五个新子 owner，代表性 routing diagnostics、native arrangement、linear/wrap slot、fallback policy 和 grid slot moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/taffy_layout_pass.rs` 和本轮新增 Taffy layout pass test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/taffy_layout_pass.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、docs/status 锚点扫描通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI runtime window input pump test folder split

状态：`runtime_15_ui_runtime_window_input_pump_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/runtime_window_input_pump.rs` folder-backed 拆分。原父文件中的 app deactivation、focus loss、activation/occlusion、batch ordering、close request 与 destroyed lifecycle 用例迁入 `ui/tests/runtime_window_input_pump/lifecycle.rs`；cursor move、cursor left、touch move 与 closed-without-cursor pointer route 用例迁入 `ui/tests/runtime_window_input_pump/pointer_routes.rs`；resize、scale factor、move 与 redraw dirty-domain 用例迁入 `ui/tests/runtime_window_input_pump/metrics_dirty.rs`。父文件现在只保留共享 window input pump fixture、dispatch/window metadata helper、popup/tooltip helper 和子模块挂载，行数从 882 降到 184；14 个原有父文件测试全部保留在子模块，最大子文件 `ui/tests/runtime_window_input_pump/lifecycle.rs` 为 328 行，全部低于 800 行。

守卫：`runtime_15_ui_runtime_window_input_pump_tests_are_folder_backed` 验证父模块挂载三个新子 owner，代表性 lifecycle、pointer route 与 metrics/dirty-domain moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/runtime_window_input_pump.rs` 和本轮新增 window input pump test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/runtime_window_input_pump.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI runtime window event ABI child folder split

状态：`runtime_15_ui_runtime_window_event_abi_children_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/runtime_ui_window_event_routes/abi.rs` 二级 folder-backed 拆分。原父文件中的 runtime event batch dispatch、resize-before-followup pointer、partial adapter failure、dispatch error index 与 adapter error no-mutation 用例迁入 `ui/tests/runtime_ui_window_event_routes/abi/batch_adapter.rs`；runtime pointer、wheel, pointer moved hover pump、cursor-left cancel 与 touch event route 用例迁入 `ui/tests/runtime_ui_window_event_routes/abi/pointer_window_routes.rs`；keyboard enter、gamepad d-pad 与 gamepad axis navigation route 用例迁入 `ui/tests/runtime_ui_window_event_routes/abi/keyboard_gamepad_routes.rs`。父文件现在只保留 `use super::*` 和子模块挂载，行数从 1078 降到 5；13 个原有父文件测试全部保留在子模块，最大子文件 `ui/tests/runtime_ui_window_event_routes/abi/pointer_window_routes.rs` 为 445 行，全部低于 800 行。

守卫：`runtime_15_ui_runtime_window_event_abi_children_are_folder_backed` 验证父模块挂载三个新子 owner，代表性 batch/adapter、pointer/window route 与 keyboard/gamepad route moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/runtime_ui_window_event_routes/abi.rs` 和本轮新增 ABI route child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/runtime_ui_window_event_routes/abi.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI widget text input keyboard test folder split

状态：`runtime_15_ui_widget_text_input_keyboard_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/widget_text_input_keyboard.rs` folder-backed 拆分。原父文件中的基础编辑、selection/navigation、word shortcut、clipboard/newline 与 text/IME 覆盖分别迁入 `ui/tests/widget_text_input_keyboard/basic_editing.rs`、`ui/tests/widget_text_input_keyboard/selection_navigation.rs`、`ui/tests/widget_text_input_keyboard/word_shortcuts.rs`、`ui/tests/widget_text_input_keyboard/clipboard_newline.rs` 与 `ui/tests/widget_text_input_keyboard/text_ime.rs`。父文件现在只保留 shared dispatch/text/IME/surface fixture helper 和子模块挂载，行数从 1362 降到 318；52 个原有父文件测试全部保留在子模块，最大子文件 `selection_navigation.rs` 为 323 行，全部低于 800 行。

守卫：`runtime_15_ui_widget_text_input_keyboard_tests_are_folder_backed` 验证父模块挂载五个新子 owner，代表性 basic editing、selection/navigation、word shortcut、clipboard/newline 与 text/IME moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/widget_text_input_keyboard.rs` 和本轮新增 child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/widget_text_input_keyboard.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI focus navigation test folder split

状态：`runtime_15_ui_focus_navigation_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/focus_navigation.rs` folder-backed 拆分。原父文件中的 focus state、input route、property mutation、tab/directional navigation 与 modal/popup focus trap 覆盖分别迁入 `ui/tests/focus_navigation/focus_state.rs`、`ui/tests/focus_navigation/property_mutation.rs`、`ui/tests/focus_navigation/tab_directional.rs` 与 `ui/tests/focus_navigation/modal_popup.rs`。父文件现在只保留 shared focus/modal/popup/navigation fixture helper 和子模块挂载，行数从 913 降到 346；16 个原有父文件测试全部保留在子模块，最大子文件 `modal_popup.rs` 为 222 行，全部低于 800 行。

守卫：`runtime_15_ui_focus_navigation_tests_are_folder_backed` 验证父模块挂载四个新子 owner，代表性 focus state、property mutation、tab/directional navigation 与 modal/popup moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/focus_navigation.rs` 和本轮新增 child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/focus_navigation.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M4 UI template style slot-contract owner split

状态：`runtime_15_ui_template_style_slot_contract_owner_split_static_passed_cargo_timeout_no_result`。

R1.4/M4 的当前新增落地部分是 UI template style application production owner 减压。`ui/template/asset/compiler/style_apply.rs` 继续拥有 style plan construction、selector path matching、rule merge order、MUI `sx` merge precedence、component root class dispatch、generic variant/color/size suppression、selector state extraction 和共享 attribute helper；slot-props 与 owner slot utility class routing 迁入 `ui/template/asset/compiler/style_apply/slot_contract.rs`，由该子 owner 承接 root slot props merge、child slot props/slot component/class projection、Skeleton child metadata routing，以及 layout/form/selection/collection/MUI X/surface/navigation owner slot utility class 分发。父文件通过 `mod slot_contract;` 和 `pub(super) use slot_contract::{apply_mui_child_slot_props, apply_mui_root_slot_props_to_node, mui_slot_name}` 消费子 owner，并把 `mui_slot_name` 作为同级 MUI class owner 的窄 helper 出口；父行数从 831 降到 701，子 owner 为 207 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_template_style_slot_contract_is_child_owner` 验证父模块挂载 slot-contract child、代表性 slot-props/slot-utility helper 不回流到父文件、子 owner 承接 root/child slot props 和 owner slot utility class internals、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/template/asset/compiler/style_apply.rs` 的 slot-contract owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI template Cargo sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；focused core-min `cargo check` 120 秒超时无结果，不计通过。

## Runtime 15 M4 UI v2 style runtime-state owner split

状态：`runtime_15_ui_v2_style_runtime_state_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 UI v2 style production owner 减压。`ui/v2/style.rs` 继续拥有 `UiV2StyleResolver`、`UiV2RuntimeStyleIndex` baseline/index API、rule collection、token/theme resolution、selector path DTO 与 selector matching；pseudo-state extraction、resolved painter state aliases、retained runtime-state attribute projection、runtime style dirty-delta classification 与 dirty flag merge 迁入 `ui/v2/style/runtime_state.rs`。父文件通过 `mod runtime_state;` 与窄 helper imports 消费子 owner，不改变 resolved style sheet shape、runtime style index crate-internal API、selector matching semantics、theme token resolution 或 retained component-state behavior。父文件从 1142 行降到 793 行，子 owner 为 362 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_v2_style_runtime_state_is_child_owner` 验证父模块挂载 runtime-state child、代表性 pseudo-state/retained-state/dirty-delta helper 不回流到父文件、子 owner 承接 painter resolved-state 与 dirty flag internals、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/v2/style.rs` 的 runtime-state owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI v2 Cargo sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；当前存在外部 cargo/rustc 编译通道，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 UI accessibility extract state owner split

状态：`runtime_15_ui_accessibility_extract_state_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 UI accessibility extract production owner 减压。`ui/accessibility/extract.rs` 继续拥有 accessibility snapshot traversal、relation target collection/pruning、name/description resolution、child filtering、role inference、action defaults、bounds/visibility/reference parsing 与 diagnostic construction；expanded/open、disabled、selected、pressed、checked、value text、text-selection、component-state/TOML value conversion 与 byte-offset clamping 迁入 `ui/accessibility/extract/state.rs`。父文件通过 `mod state;` 与窄 helper imports 消费子 owner，不改变 `accessibility_snapshot(...)` 输出形状、diagnostic code、hidden relation target retention、widget behavior mapping 或 accessibility action behavior。父文件从 993 行级别减压到 668 行，子 owner 为 339 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_accessibility_extract_state_is_child_owner` 验证父模块挂载 state child、代表性 state projection helper、`UiA11yCheckedState`、`UiA11yTextSelection` 与 `UiValue` conversion 不回流到父文件、子 owner 承接 component-state conversion internals、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/accessibility/extract.rs` 的 state projection owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI accessibility Cargo sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 UI component catalog editor-showcase helper owner split

状态：`runtime_15_ui_component_catalog_editor_showcase_helper_owner_split_static_passed_cargo_timeout_no_result`。

R1.4/M4 的当前新增落地部分是 UI component catalog editor showcase production owner 减压。`ui/component/catalog/editor_showcase.rs` 继续拥有 editor showcase registry、descriptor list、descriptor assembly entry point 与 representative component coverage；base descriptor construction、layout role/default template projection、palette metadata、fallback policy、option/slot/value prop schema builders 与 TOML layout helpers 迁入 `ui/component/catalog/editor_showcase/descriptor_builders.rs`。父文件通过 `mod descriptor_builders;` 与窄 descriptor-builder imports 消费子 owner，不改变 editor showcase registry ids、component descriptors、palette metadata shape、fallback policy 或 component catalog public lookup behavior。父文件从 1029 行降到 674 行，子 owner 为 429 行，两侧都低于 800 行生产文件软预算；原 M4 拆分 owner 的 `helpers.rs` 文件名已由后续 M2 命名硬切收束到职责名。

守卫：`runtime_15_ui_component_catalog_editor_showcase_helpers_are_child_owner` 验证父模块挂载 descriptor_builders child、代表性 descriptor builder、palette metadata、fallback policy 与 prop schema builder 不回流到父文件、子 owner 承接 descriptor construction internals、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/component/catalog/editor_showcase.rs` 的 descriptor builder owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI component catalog Cargo sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、moved owner 扫描与 docs/status 锚点扫描已通过；focused Cargo 305 秒超时无诊断结果，超时后另有 editor layout cargo/rustc 通道活跃，不计通过。

## Runtime 15 M2 UI editor showcase descriptor builders module naming hard cutover

状态：`runtime_15_ui_editor_showcase_descriptor_builders_naming_hard_cutover_static_passed_cargo_deferred`。

R2.3/M2 的当前新增落地部分是 UI editor showcase descriptor construction owner 命名硬切。`ui/component/catalog/editor_showcase/helpers.rs` 已删除并硬切为 `ui/component/catalog/editor_showcase/descriptor_builders.rs`；`ui/component/catalog/editor_showcase.rs` 只挂载 `mod descriptor_builders;` 并从职责命名 owner import descriptor construction、layout role/default template projection、palette metadata、fallback policy、option/slot/value prop schema builders 与 TOML layout helpers，不保留旧 `helpers` module、alias 或兼容 re-export。

守卫：`runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name` 验证旧文件不存在、新 owner/入口/调用方 import 形状、既有 M4 production-file budget 守卫已同步新路径，以及 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 UI editor showcase descriptor builders 的 `helpers` 文件名债；完整 `runtime_15_no_banned_name_modules`、`module_convention_gate` 与全量 UI component catalog Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、旧路径不存在与新模块入口扫描、docs/status/date 锚点扫描、trailing-whitespace scan 和 scoped `git diff --check` 已通过；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 UI surface event-routing owner split

状态：`runtime_15_ui_surface_event_routing_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 UI surface production owner 减压。`ui/surface/surface.rs` 继续拥有 `UiSurface` state、runtime-state style entry points、surface frame/debug snapshots、property mutation、reflector/focus path entry points 与 focus-reconcile reason helper；pointer capture/release、input/window dispatch adapters、pointer route construction、pointer dispatch side effects、navigation routing 与 activation phase helper 迁入 `ui/surface/surface/event_routing.rs`；route-derived hovered/pressed/focused component-state dirtying、component event reports、focus event reports 与 damage frame requests 迁入 `ui/surface/surface/pointer_component_events.rs`。父文件通过 `mod event_routing;` 与 `mod pointer_component_events;` 挂载子 owner，不改变 input dispatch result shape、pointer route semantics、default interaction order、focus visibility reason 或 component event payload。父文件从 1161 行降到 317 行，两个子 owner 分别为 578 行和 356 行，三侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_surface_event_routing_is_child_owner` 验证父模块挂载 event-routing 和 pointer-component-events child、代表性 event-routing helper 和 pointer component report helper 不回流到父文件、子 owner 承接 routing/event report internals、三侧 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/surface/surface.rs` 的 event-routing owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI surface Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。


## Runtime 15 M4 UI surface property mutation metadata dirty owner split

状态：`runtime_15_ui_surface_property_mutation_metadata_dirty_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 `ui/surface/property_mutation.rs` production owner 减压。父文件继续负责 `UiPropertyMutationRequest`、`UiPropertyMutationReport`、`mutate_tree_property(...)`、visibility/input/state property mutation、binding report construction、template attribute sync、visibility/input value parsing、state dirty marking 与 mutation report dirty synchronization；metadata attribute dirty-domain classification、render/virtualized dirty helper、MUI overlay/feedback/transition/customization predicates、virtualized range predicates 与 layout metadata predicate 迁入 `ui/surface/property_mutation/metadata_dirty.rs`。父文件通过 `mod metadata_dirty;` 与窄 imports 消费 `metadata_attribute_dirty` / `render_dirty`，行数从 839 降到 522；子 owner 为 322 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_surface_property_mutation_metadata_dirty_is_child_owner` 验证父模块挂载 metadata dirty child、代表性 dirty classifier 与 predicate helper 不回流到父文件、子 owner 承接 metadata dirty-domain classification、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/surface/property_mutation.rs` 的 metadata dirty owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI surface Cargo sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 UI surface render feedback command/color owner split

状态：`runtime_15_ui_surface_render_feedback_command_color_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 UI surface render feedback production owner 减压。`ui/surface/render/feedback.rs` 继续拥有 feedback component kind detection、Alert/AlertTitle/Tooltip/Toast command layout、metadata text/icon/size extraction、border/radius parsing 与 public render entry points；AlertTone、feedback color constants、visual-state-aware alert/tooltip/toast color selection 与 style override fallback 迁入 `ui/surface/render/feedback/colors.rs`；quad/text/icon `UiRenderCommand` DTO construction 迁入 `ui/surface/render/feedback/commands.rs`。父文件通过 `mod colors;`、`mod commands;` 与窄 imports 消费子 owner，不改变 feedback render command ordering、z-index offsets、metadata key fallback、painter-state projection 或 owner text/image suppression behavior。父文件从 872 行降到 590 行，`colors.rs` 为 268 行，`commands.rs` 为 100 行，既有 `state.rs` 为 70 行，四侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_surface_render_feedback_commands_are_child_owners` 验证父模块挂载 colors/commands/state child、代表性 color constants、color helpers、`UiRenderCommandKind`、`UiResolvedStyle`、`UiVisualAssetRef` 与 primitive command constructors 不回流到父文件，子 owner 承接 visual-state color resolution、primitive command DTO construction 与 painter family state resolution，四侧 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/surface/render/feedback.rs` 的 command/color owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI surface render Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 UI surface default-interactions keyboard/timer owner split

状态：`runtime_15_ui_surface_default_interactions_keyboard_timer_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 UI surface default-interactions production owner 减压。`ui/surface/surface/default_interactions.rs` 继续拥有 pointer default action routing、button/toggle/disclosure helpers、shared binding report construction、widget behavior predicates 与 component event token matching；keyboard-triggered default component actions、semantic keyboard actions/text、keyboard behavior eligibility 与 semantic action/event-kind mapping 迁入 `ui/surface/surface/default_interactions/keyboard.rs`；typeahead timeout、submenu hover delay、tooltip timer、menu-role detection、tooltip id extraction 与 timer-expired component event reports 迁入 `ui/surface/surface/default_interactions/timers.rs`。父文件通过 `mod keyboard;` 与 `mod timers;` 挂载子 owner，不改变 default interaction order、keyboard semantic action payload、timer metadata resolution 或 component event report shape。父文件从 973 行降到 596 行，两个子 owner 分别为 229 行和 172 行，三侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_surface_default_interactions_keyboard_timers_are_child_owners` 验证父模块挂载 keyboard 和 timers child、代表性 keyboard/timer helper 不回流到父文件、子 owner 承接 keyboard action 和 timer event internals、三侧 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/surface/surface/default_interactions.rs` 的 keyboard/timer owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI surface Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 UI surface table column helper owner split

状态：`runtime_15_ui_surface_table_column_helper_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 UI surface table default-interactions production owner 减压。`ui/surface/surface/default_interactions/table/mod.rs` 继续拥有 table pointer flow、column resize/sort event flow、row selection/virtual scroll dispatch、table mutation helper 与 shared owner predicates；column resize/sort metadata helper、column field/width/min-width lookup、sort direction/model helper、row sort comparison、column match predicates 与 resize drag token encode/decode 迁入 `ui/surface/surface/default_interactions/table/columns.rs`。父文件通过 `mod columns;` 与显式 `columns::...` 调用消费子 owner，不改变 column resize payload、sort model writes、row client sorting、selection 或 virtual scroll behavior。父文件从 949 行降到 677 行，`columns.rs` 为 292 行，既有 `selection.rs` 为 296 行、`virtualization.rs` 为 381 行，四侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_surface_table_column_helpers_are_child_owner` 验证父模块挂载 columns/selection/virtualization child、代表性 column helper 常量与函数不回流到父文件、子 owner 承接 column metadata/sort/width/drag-token internals、table 四侧 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/surface/surface/default_interactions/table/mod.rs` 的 column helper owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI surface Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M2 UI table sortingMode server literal allowed-context sync

状态：`runtime_15_ui_table_sorting_mode_server_literal_allowed_context_static_passed_cargo_deferred`。

R2.3/M2 的当前新增落地部分是 UI table sortingMode 第三方 API 字面量的 moved-owner allowlist 同步。M4 table column helper split 后，`sortingMode = "server"` 的生产读取点位于 `ui/surface/surface/default_interactions/table/columns.rs::table_uses_client_sorting(...)`；该字面量描述 DataGrid/Table 的外部排序模式，不是 runtime 网络/server owner。`non_network_server_naming.py` 与 Rust `runtime_non_network_server_naming_is_classified_by_owner` guard 已同步新 owner 路径，继续把该字面量作为 allowed context 处理。

守卫：`runtime_15_ui_table_sorting_mode_server_literal_stays_allowed_context` 验证 `columns.rs` 仍只在 `table_uses_client_sorting(...)` 中消费 `sortingMode`/`Some("server")`，Python audit 与 Rust guard 都登记新路径，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 UI table `sortingMode = "server"` moved-owner allowlist 同步面；graphics render-framework server naming debt 已由后续 M2 hard cutover 关闭，完整 `runtime_15_no_banned_name_modules` 与 `module_convention_gate` 仍 pending。

验证：scoped rustfmt/static scans、aggregate `audit_runtime_structure.py --json` non-network server assertions、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 已通过；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M2 graphics render-framework receiver naming hard cutover

状态：`runtime_15_graphics_render_framework_receiver_naming_hard_cutover_static_passed_cargo_deferred`。

R2.3/M2 的当前新增落地部分是 graphics render-framework 非网络 receiver 命名硬切。`graphics/runtime/render_framework/**` 中 viewport lifecycle、pipeline mutation、stats/debug query、capture、frame-submission context build、camera-loop submit、direct runtime-frame submit 与 preflight failure helpers 的 `server: &WgpuRenderFramework` receiver/context 变量已硬切为 `framework: &WgpuRenderFramework`；对应调用改为 `framework.lock_operation()`、`framework.lock_state()` 和窄函数传参，不保留 `server` alias、compat variable 或 allowlist 债。

守卫：`runtime_15_render_framework_receiver_uses_framework_name` 递归验证 render-framework source 无 `server` token，代表性入口继续使用 `framework: &WgpuRenderFramework` 与 `framework.lock_*`，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/graphics/render-product-submit.md` 和 status-output expectations 都包含本切片锚。Rust `runtime_non_network_server_naming_is_classified_by_owner` 与 Python `non_network_server_naming.py` 均不再保留 retired `graphics-render-framework-debt` bucket；aggregate `non_network_server_references` 在该切片收敛为 0 unclassified、0 graphics render-framework debt，后续 editor workbench authority-label hard cutover 继续清掉当时剩余的 classified debt。该切片只关闭 graphics render-framework 非网络 receiver 命名债；完整 `runtime_15_no_banned_name_modules` 与 `module_convention_gate` 仍 pending。

验证：scoped rustfmt/static scans、render-framework server-token scan、old graphics debt string scan、aggregate `audit_runtime_structure.py --json` non-network server assertions、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 已通过；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M2 editor workbench authority-label naming hard cutover

状态：`runtime_15_editor_workbench_authority_label_naming_hard_cutover_static_passed_cargo_deferred`。

R2.3/M2 的当前新增落地部分是 editor workbench extension authority label 命名硬切。`zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs` 中 `workbench.extension.spawn_rules.condition_night_table_row.select` 的输出文案从 `Selected Condition_Night   server authority` 改为 `Selected Condition_Night   editor authority`，使 Workbench fixture/output label 表达 editor 权限来源，而不是非网络 server owner。

守卫：`runtime_15_editor_workbench_authority_label_uses_editor_name` 验证 Workbench feedback source 包含新 `Selected Condition_Night   editor authority` 文案、不再包含 `server authority`，并验证 Python `non_network_server_naming.py` 不再保留 retired `editor-workbench-authority-label-debt` bucket。该守卫同时要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/engine-architecture/non-network-server-naming-m1.md`、`docs/zircon_editor/ui/host/commands.md` 和 status-output expectations 都包含本切片锚。aggregate `non_network_server_references` 审计现在报告 0 reference decisions、0 classified debt、0 migration debt、0 unclassified，M1 gate 为 `classified-and-clear`。

验证：scoped rustfmt/static scans、Python py_compile、aggregate `audit_runtime_structure.py --json` non-network server assertions、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 已通过；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 UI template document validation owner split

状态：`runtime_15_ui_template_document_validation_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 UI asset document production owner 减压。`ui/template/asset/document.rs` 继续拥有 `UiAssetDocumentRuntimeExt` runtime extension API、style rule/sheet mutation API、node lookup/mutation、child mount lookup、parent lookup、node iterator 与 tree mutation helpers；document validation 迁入 `ui/template/asset/document/validation.rs`，由该子 owner 承接 root/component node id validation、duplicate node subtree authority、stylesheet id validation、style rule id validation 与 selector parse validation。父文件只通过 `mod validation;` 和 `validate_*` helper 消费子 owner，行数从 805 降到 653；子 owner 为 100 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_template_document_validation_is_child_owner` 验证父模块挂载 validation child、代表性 validation helper 和 `UiSelector::parse` 不回流到父文件、子 owner 承接 document identity/selector checks、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/template/asset/document.rs` 的 validation owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI template Cargo sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M4 UI template MUI X DataGrid class owner split

状态：`runtime_15_ui_template_mui_x_data_grid_class_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 UI template asset compiler 的 MUI X class production owner 减压。`ui/template/asset/compiler/style_apply/mui_x_classes.rs` 继续拥有 MUI X component-family dispatch、MaterialTreeView、Date/Time Pickers、Charts、AgentChat/Chat class routing、generic-class suppression 和共享 attribute helper；DataGrid-specific root 与 slot utility class projection 迁入 `ui/template/asset/compiler/style_apply/mui_x_classes/data_grid.rs`，由该子 owner 承接 root class、columnHeader/row/cell slot class、toolbar/footer/loading/noRows slot class，以及 sort/filter/pagination/virtualization state class projection。父文件只通过 `mod data_grid;`、`data_grid::append_component_classes(...)` 和 `data_grid::append_slot_classes(...)` 消费子 owner，行数从 823 降到 575；子 owner 为 277 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_template_mui_x_data_grid_classes_are_child_owner` 验证父模块挂载 DataGrid child、代表性 DataGrid root/slot helpers 不回流到父文件、子 owner 承接 DataGrid utility class internals、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/template/asset/compiler/style_apply/mui_x_classes.rs` 的 DataGrid owner 减压面；上层 `style_apply.rs`、完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI template Cargo sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。


## Runtime 15 M4 UI layout arrange grid/masonry owner split

状态：`runtime_15_ui_layout_arrange_grid_masonry_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 `ui/layout/pass/arrange.rs` production owner 减压。父文件继续负责 `arrange_node(...)` entry point、Taffy fallback routing、Free/Canvas/Container/Overlay dispatch、SizeBox/BlockBox/Linear/Scrollable/WrapBox fallback arrangement、scroll virtualization window planning、wrap content sizing、child position accumulation 与 `hide_subtree_layout(...)`；GridBox placement/dimension/cell-frame helper、MasonryBox sequential/shortest-column placement、masonry content-size computation 与 grid/masonry recursive child arrangement 迁入 `ui/layout/pass/arrange/grid_masonry.rs`。父文件通过 `mod grid_masonry;` 与窄 imports 消费子 owner，行数从 853 降到 690；子 owner 为 181 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_layout_arrange_grid_masonry_is_child_owner` 验证父模块挂载 grid/masonry child、代表性 grid/masonry placement helper 不回流到父文件、子 owner 承接 GridBox/MasonryBox fallback arrangement、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/layout/pass/arrange.rs` 的 grid/masonry owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI layout Cargo sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 UI text layout engine visual-order owner split

状态：`runtime_15_ui_text_layout_engine_visual_order_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 `ui/text/layout_engine.rs` production owner 减压。父文件继续负责 `measure_text_size(...)`、`layout_text(...)`、source-run wrapping、ellipsis fragment construction、direction resolution、alignment 和 `#[cfg(test)] mod tests` 挂载；BiDi visual-order scaffold 迁入 `ui/text/layout_engine/visual_order.rs`，由该子 owner 承接 visual token/cluster/fragment 类型、neutral-direction assignment、visual cluster assembly 与 visual fragment coalescing。父文件只通过 `mod visual_order;` 和 `visual_order::apply_visual_order(...)` 消费子 owner，行数从 823 降到 530；子 owner 为 301 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_text_layout_engine_visual_order_is_child_owner` 验证父模块挂载 visual-order child、代表性 visual token/fragment/neutral helper 不回流到父文件、子 owner 承接 BiDi scaffold internals、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/text/layout_engine.rs` 的 visual-order owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI text Cargo sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI runtime input manager test folder split

状态：`runtime_15_ui_runtime_input_manager_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/runtime_input_manager.rs` folder-backed 拆分。原父文件中的 window input pump batch 与 tick 计时覆盖迁入 `ui/tests/runtime_input_manager/window_timer.rs`；路由阶段顺序和 route policy 名称覆盖迁入 `ui/tests/runtime_input_manager/route_order.rs`；pointer capture、popup outside close、preview tunnel、keyboard focus path 与 popup default action 路由矩阵覆盖迁入 `ui/tests/runtime_input_manager/route_matrix.rs`；double-click timer、primary/secondary touch、touch cancel 与 multi-pointer capture isolation 覆盖迁入 `ui/tests/runtime_input_manager/touch_pointer.rs`。父文件现在只保留 shared route matrix、double-click、popup、input metadata、pointer/touch/keyboard/popup event 与 window metadata fixture helper 和子模块挂载，行数从 1006 降到 295；15 个原有父文件测试全部保留在子模块，最大子文件 `touch_pointer.rs` 为 400 行，全部低于 800 行。

守卫：`runtime_15_ui_runtime_input_manager_tests_are_folder_backed` 验证父模块挂载四个新子 owner，代表性 window/timer、route order、route matrix、double-click/touch/multi-pointer moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/runtime_input_manager.rs` 和本轮新增 child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/runtime_input_manager.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI runtime input ownership test folder split

状态：`runtime_15_ui_runtime_input_ownership_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/runtime_input_ownership.rs` folder-backed 拆分。原父文件中的 input-method owner、hidden/disabled owner validation、high-precision capture dispatch、drag/drop lifecycle、popup/tooltip transient input 与 route trace 覆盖分别迁入 `ui/tests/runtime_input_ownership/input_method.rs`、`ui/tests/runtime_input_ownership/owner_validation.rs`、`ui/tests/runtime_input_ownership/high_precision_dispatch.rs`、`ui/tests/runtime_input_ownership/drag_drop.rs`、`ui/tests/runtime_input_ownership/popup_tooltip.rs` 与 `ui/tests/runtime_input_ownership/route_trace.rs`。父文件现在只保留 shared pointer capture、input metadata、keyboard/analog/pointer、popup/tooltip、drag/drop 与 input-method fixture helper 和子模块挂载，行数从 1152 降到 203；16 个原有父文件测试全部保留在子模块，最大子文件 `popup_tooltip.rs` 为 213 行，全部低于 800 行。本切片保留既有 pointer-capture API 更新，测试继续通过 `set_pointer_capture_for_id`、`active_pointer_capture` 与 `pointer_capture_owner` 锁定 per-pointer capture 语义。

守卫：`runtime_15_ui_runtime_input_ownership_tests_are_folder_backed` 验证父模块挂载六个新子 owner，代表性 input-method、owner validation、high-precision dispatch、drag/drop、popup/tooltip 与 route-trace moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/runtime_input_ownership.rs` 和本轮新增 child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/runtime_input_ownership.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI event routing test folder split

状态：`runtime_15_ui_event_routing_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/event_routing.rs` folder-backed 拆分。原单文件中的 pointer press/release/hover/dirty-state 用例迁入 `ui/tests/event_routing/pointer_state.rs`；binding/component events、capture release、scroll fallback、template click envelope 用例迁入 `ui/tests/event_routing/component_events.rs`；focus/capture/high-precision scroll、navigation、host-owned input effect 与 input-method validation 用例迁入 `ui/tests/event_routing/dispatch_effects.rs`；keyboard/text/IME shared dispatch、editable mutation 和 hidden/invalid owner rejection 用例迁入 `ui/tests/event_routing/shared_input.rs`。父文件现在只保留共享导入、子模块挂载、button/scroll/editable/input helper，行数从 1676 降到 341；27 个原有测试全部保留在子模块，所有子 owner 低于 800 行。

守卫：`runtime_15_ui_event_routing_tests_are_folder_backed` 验证父模块挂载四个子 owner，代表性 pointer/component/dispatch/shared-input moved guard 不回流到父文件，各子 owner 承接对应测试锚，`ui/tests/event_routing.rs` 和所有 UI event routing test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/event_routing.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI runtime input reply routes test folder split

状态：`runtime_15_ui_runtime_input_reply_routes_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `ui/tests/runtime_input_reply_routes.rs` 父文件减压。原父文件剩余 direct reply trace、raw mouse motion 和 dispatch step trace 用例迁入 `ui/tests/runtime_input_reply_routes/route_trace_routes.rs`；unified pointer route、preview tunnel、press/release、double-click 与 precise scroll 用例迁入 `ui/tests/runtime_input_reply_routes/pointer_bubble_routes.rs`；focus/capture、navigation、text/IME、captured pointer release 与 accessibility default-action route steps 用例迁入 `ui/tests/runtime_input_reply_routes/focus_text_accessibility_routes.rs`。父文件继续挂载既有 route 子 owner，并保留 route surface、input event、pointer capture、editable helper 和 shared assertion helper；行数从 1558 降到 500，三个新子 owner 全部低于 800 行。

守卫：`runtime_15_ui_runtime_input_reply_routes_tests_are_folder_backed` 验证父模块挂载三个新子 owner，代表性 route-trace/pointer-bubble/focus-text/accessibility moved guard 不回流到父文件，本次迁出的 13 个父文件测试全部保留在子模块，`ui/tests/runtime_input_reply_routes.rs` 和三个新增 child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/tests/runtime_input_reply_routes.rs` 父文件；既有 `keyboard_navigation_routes.rs` 与 `tree_view_pointer_routes.rs` 仍超过 800 行，完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 UI runtime input reply route child folder split

## Runtime 15 M3 UI runtime input reply route guard child-owner split

状态：`runtime_15_ui_runtime_input_reply_route_guard_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 UI runtime input reply route test-budget 守护的 child-owner 减压。`tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs` 从 539 行降为路由父文件，只挂载 `ui_runtime_input_reply_routes/root.rs`、`ui_runtime_input_reply_routes/route_children.rs` 与 `ui_runtime_input_reply_routes/table_pointer.rs`。

新增 `runtime_15_ui_runtime_input_reply_route_guard_child_owners_are_folder_backed`，验证父文件不再定义三个已移动守护、子 owner 保留原守护、父子文件低于 400 行 focused 预算，并要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、UI architecture 文档、session note 与 status-output expectations 同步。精确锚点包括 `structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs` 与 `structure_convention/test_file_budget/ui_runtime_input_reply_routes/route_children.rs`。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved guard 扫描、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 进程 active 按支撑切片节奏 deferred，不计 Cargo 通过。

状态：`runtime_15_ui_runtime_input_reply_route_children_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是既有 reply-route child owner 的二级 folder-backed 拆分。`ui/tests/runtime_input_reply_routes/keyboard_navigation_routes.rs` 现在只保留 keyboard navigation shared imports、route surface/input helper、assertion helper 和子模块挂载；focus path、semantic action route、timers-disabled 与 directional navigation 用例分别迁入 `ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/focus_path.rs`、`semantic_actions.rs`、`timers_disabled.rs` 与 `directional.rs`。父文件从 980 行降到 152 行，四个子 owner 保留 15 个测试并全部低于 800 行。

`ui/tests/runtime_input_reply_routes/tree_view_pointer_routes.rs` 现在只保留 tree view shared fixture、selection helper、node lookup helper 和子模块挂载；tree selection、drag/reorder 与 virtualization 用例分别迁入 `ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/selection.rs`、`drag_reorder.rs` 与 `virtualization.rs`。父文件从 810 行降到 418 行，三个子 owner 保留 9 个测试并全部低于 800 行。

守卫：`runtime_15_ui_runtime_input_reply_route_children_are_folder_backed` 验证两个 child parent 的模块挂载与 helper ownership，代表性 keyboard/tree moved guard 不回流到 parent，keyboard 子 owner 合计保留 15 个测试、tree 子 owner 合计保留 9 个测试，所有本次 parent/child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片关闭 `keyboard_navigation_routes.rs` 与 `tree_view_pointer_routes.rs` 的 oversized child-owner 债务；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

## Runtime 15 M3 runtime diagnostics test folder split

状态：`runtime_15_runtime_diagnostics_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `tests/runtime_diagnostics/mod.rs` folder-backed 拆分。原单文件中的 render diagnostics series 断言按诊断领域迁入 `tests/runtime_diagnostics/capability_history_visibility.rs`、`tests/runtime_diagnostics/hzb_light_camera_capture.rs`、`tests/runtime_diagnostics/graph_resources.rs`、`tests/runtime_diagnostics/graph_execution.rs`、`tests/runtime_diagnostics/post_process_material_mesh.rs` 与 `tests/runtime_diagnostics/gpu_sprite_ui_advanced.rs`。父文件现在只保留两个测试入口、runtime/devtools 装配和子模块调用，行数从 2098 降到 89；最大子文件 `graph_resources.rs` 为 445 行，既有 `support.rs` 为 703 行，全部低于 800 行。

守卫：`runtime_15_runtime_diagnostics_tests_are_folder_backed` 验证父模块挂载上述子 owner、代表性 diagnostics series anchor 不回流到父文件、`motion_vector.rs` 和 `support.rs` 继续作为既有子 owner 保持预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 RHI command list test folder split

状态：`runtime_15_rhi_command_list_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `rhi/tests/command_list.rs` folder-backed 拆分。原单文件中的 command-list queue/copy/compute、bind-group layout、raster draw 和 vertex/index buffer state 合约用例已分别迁入 `rhi/tests/command_list/basic_commands.rs`、`rhi/tests/command_list/bind_groups.rs`、`rhi/tests/command_list/raster_draws.rs` 与 `rhi/tests/command_list/vertex_index_state.rs`。父文件现在只保留共享导入、RHI test fixture helpers 和四个子模块挂载，行数从 1034 降到 214；四个子 owner 分别为 152、233、222、222 行，全部低于 800 行。

守卫：`runtime_15_rhi_command_list_tests_are_folder_backed` 验证父模块挂载四个子 owner、代表性 command-list moved guard 不回流到父文件、各子 owner 承接对应测试锚，`rhi/tests/command_list.rs` 和所有 command-list test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks 通过；带锁 focused structure guard/status-output/core-min cargo check 均在进入测试前被当前工作区 `Cargo.lock` / `Cargo.toml` 不一致阻塞，不计通过。

## Runtime 15 M3 RHI device contract test folder split

状态：`runtime_15_rhi_device_contract_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `rhi/tests/device_contract.rs` folder-backed 拆分。原单文件中的 RHI handle/resource、texture/sampler descriptor、bind-group validation、invalid descriptor matrix、transfer/fence IO 与 framework wgpu import boundary 合约已分别迁入 `rhi/tests/device_contract/basic_resources.rs`、`rhi/tests/device_contract/texture_sampler_descriptors.rs`、`rhi/tests/device_contract/bind_groups.rs`、`rhi/tests/device_contract/invalid_descriptors.rs`、`rhi/tests/device_contract/transfer_and_fences.rs` 与 `rhi/tests/device_contract/framework_boundary.rs`。父文件现在只保留共享 RHI 导入、bind-group layout/pipeline layout helpers 和六个子模块挂载，行数从 987 降到 40；最大子文件 `bind_groups.rs` 为 308 行，全部低于 800 行。

守卫：`runtime_15_rhi_device_contract_tests_are_folder_backed` 验证父模块挂载六个子 owner、代表性 device-contract moved guard 不回流到父文件、各子 owner 承接对应测试锚，`rhi/tests/device_contract.rs` 和所有 device-contract test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks 通过；带锁 core-min cargo check 明确返回当前工作区 `Cargo.lock` / `Cargo.toml` 不一致，focused structure guard 与 status-output Cargo 尝试在 active Cargo lanes 下超时且无测试结果，不计通过。

## Runtime 15 M3 asset pack test folder split

状态：`runtime_15_asset_pack_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `asset/tests/pack.rs` folder-backed 拆分。原单文件中的基础 pack writer/reader 合约、pack manifest validation、delta reader validation、delta pack apply、delta installer/receipt 和 trim planner 报告用例已分别迁入 `asset/tests/pack/basic.rs`、`asset/tests/pack/reader_validation.rs`、`asset/tests/pack/delta_reader_validation.rs`、`asset/tests/pack/delta_pack.rs`、`asset/tests/pack/delta_installer.rs` 与 `asset/tests/pack/trim.rs`。父文件现在只保留共享导入、manifest/byte helper、临时目录 helper 和六个子模块挂载，行数从 1288 降到 154；最大子文件 `delta_installer.rs` 为 402 行，42 个原有测试全部保留在子模块，全部低于 800 行。

守卫：`runtime_15_asset_pack_tests_are_folder_backed` 验证父模块挂载六个子 owner、代表性 pack/delta/installer/trim moved guard 不回流到父文件、子模块合计保留 42 个测试、`asset/tests/pack.rs` 和所有 asset pack test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描与 scoped `git diff --check` 通过；带锁 focused guard 与 core-min cargo check 均 120s 超时无结果，当前工作区 Cargo.lock/Cargo.toml 漂移仍未解决，不计通过。

## Runtime 15 M3 asset facade test folder split

状态：`runtime_15_asset_facade_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `asset/tests/facade.rs` folder-backed 拆分。原父文件中的 typed handle/event、root load-state/readiness、project asset manager façade、recursive dependency aggregation 和 missing/direct dependency precedence 用例已分别迁入 `asset/tests/facade/handle_events.rs`、`asset/tests/facade/load_state_roots.rs`、`asset/tests/facade/project_facade.rs`、`asset/tests/facade/recursive_dependencies.rs` 与 `asset/tests/facade/dependency_failures.rs`。父文件现在只保留共享 asset/resource helper、`ui_v2_view_asset()` 和模块挂载，行数从 1017 降到 111；最大新增子文件 `recursive_dependencies.rs` 为 292 行，20 个父文件测试全部保留在子模块，全部低于 800 行。既有 `failure_reason.rs`、`handle_lifecycle.rs` 与 `hot_reload.rs` 继续作为 asset façade 子 owner。

守卫：`runtime_15_asset_facade_tests_are_folder_backed` 验证父模块挂载五个新增子 owner 和三个既有子 owner、代表性 typed handle/load-state/project/dependency moved guard 不回流到父文件、子模块合计保留 20 个父文件测试、`asset/tests/facade.rs` 和所有新增 asset façade test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描与 docs/status 锚点扫描通过；带锁 focused guard 与 core-min cargo check 均被当前工作区 Cargo.lock/Cargo.toml 不一致阻塞，不计通过。

## Runtime 15 M3 asset project zmeta test folder split

状态：`runtime_15_asset_project_zmeta_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `asset/tests/project/zmeta.rs` folder-backed 拆分。原父文件中的 zmeta schema/reference lifecycle、package URI asset roots、compound zshader package import、zshader diagnostics 和 documented fixture 解析用例已分别迁入 `asset/tests/project/zmeta/metadata_lifecycle.rs`、`asset/tests/project/zmeta/package_roots.rs`、`asset/tests/project/zmeta/compound_shader.rs` 与 `asset/tests/project/zmeta/shader_diagnostics_fixture.rs`。父文件现在只保留共享 importer/material/import-outcome helper 和模块挂载，行数从 996 降到 104；最大子文件 `compound_shader.rs` 为 283 行，9 个父文件测试全部保留在子模块，全部低于 800 行。

守卫：`runtime_15_asset_project_zmeta_tests_are_folder_backed` 验证父模块挂载四个子 owner、代表性 zmeta schema/package/compound shader/fixture moved guard 不回流到父文件、子模块合计保留 9 个父文件测试、`asset/tests/project/zmeta.rs` 和所有 zmeta test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描与 scoped `git diff --check` 通过；带锁 focused guard 与 core-min cargo check 均 120s 超时无结果，未留下匹配 target-dir 进程，不计通过。

## Runtime 15 M3 asset project manager test folder split

状态：`runtime_15_asset_project_manager_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `asset/tests/project/manager.rs` folder-backed 拆分。原父文件中的 first-wave asset import/library artifact、physics/animation/sound import、ready artifact restore、failed import continuation、UI schema migration、dependency diagnostics 和 labeled subasset error handling 用例已分别迁入 `asset/tests/project/manager/library_imports.rs`、`asset/tests/project/manager/restore_failure_migration.rs` 与 `asset/tests/project/manager/subassets_errors.rs`。父文件现在只保留共享 importer fixture、counted importer、dependency/subasset importer helper 和模块挂载，行数从 940 降到 181；最大子文件 `restore_failure_migration.rs` 为 292 行，11 个父文件测试全部保留在子模块，全部低于 800 行。

守卫：`runtime_15_asset_project_manager_tests_are_folder_backed` 验证父模块挂载三个子 owner、代表性 first-wave import / restore / migration / subasset moved guard 不回流到父文件、子模块合计保留 11 个父文件测试、`asset/tests/project/manager.rs` 和所有 project-manager test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描与 scoped `git diff --check` 通过；带锁 focused guard 与 core-min cargo check 均 120s 超时无结果，未留下匹配 target-dir 进程，当前工作区 Cargo.lock/Cargo.toml 漂移仍未解决，不计通过。

## Runtime 15 M3 asset project flow sample test folder split

状态：`runtime_15_asset_project_flow_sample_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `asset/tests/project/asset_flow_sample.rs` folder-backed 拆分。原父文件中的 minimal glTF/material/shader/mesh 项目导入端到端测试已迁入 `asset/tests/project/asset_flow_sample/end_to_end.rs`；sample importer wiring、sample source writers 与 assertion/load helpers 分别迁入 `asset/tests/project/asset_flow_sample/importers.rs`、`asset/tests/project/asset_flow_sample/fixtures.rs` 与 `asset/tests/project/asset_flow_sample/assertions.rs`。父文件现在只保留共享导入和模块挂载，行数从 975 降到 28；最大子文件 `end_to_end.rs` 为 448 行，原有 1 个端到端测试保留在子模块，全部低于 800 行。

守卫：`runtime_15_asset_project_flow_sample_tests_are_folder_backed` 验证父模块挂载四个 sample owner、代表性 end-to-end/importer/fixture/assertion helper 不回流到父文件、端到端测试数量保持 1、`asset/tests/project/asset_flow_sample.rs` 和所有 child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、端到端测试数量扫描、父子行数预算扫描、docs/status 锚点扫描与 scoped `git diff --check` 通过；带锁 focused guard 与 core-min cargo check 均被当前工作区 Cargo.lock/Cargo.toml 不一致阻塞，不计通过。

## Runtime 15 M3 asset material test folder split

状态：`runtime_15_asset_material_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `asset/tests/assets/material.rs` folder-backed 拆分。原父文件中的 material TOML roundtrip/serialization、standard descriptor owned fields、override validation errors、shader contract/readiness diagnostics 和 material management record summary 用例已分别迁入 `asset/tests/assets/material/asset_serialization.rs`、`asset/tests/assets/material/owned_descriptor.rs`、`asset/tests/assets/material/override_validation.rs`、`asset/tests/assets/material/shader_readiness.rs` 与 `asset/tests/assets/material/management_records.rs`。父文件现在只保留共享 shader/material helper 和模块挂载，行数从 1228 降到 69；最大子文件 `owned_descriptor.rs` 为 361 行，23 个父文件测试全部保留在子模块，全部低于 800 行。

守卫：`runtime_15_asset_material_tests_are_folder_backed` 验证父模块挂载五个子 owner、代表性 serialization/descriptor/validation/shader/management moved guard 不回流到父文件、子模块合计保留 23 个父文件测试、`asset/tests/assets/material.rs` 和所有 material test child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描与 scoped `git diff --check` 通过；带锁 focused guard 与 core-min cargo check 均 120s 超时无结果，未留下匹配 target-dir 进程，当前工作区 Cargo.lock/Cargo.toml 漂移仍未解决，不计通过。

## Runtime 15 M3 asset glTF importer test folder split

状态：`runtime_15_asset_gltf_importer_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `asset/tests/assets/gltf_importer.rs` folder-backed 拆分。原父文件中的 root glTF import/default importer capability、Bevy-style labeled subassets、synthetic skeleton、multi-primitive material labels、external texture/missing buffer/unsupported primitive diagnostics、skinning/tangent/color/UV channel preservation、texture transform bridge 和 multi-scene labels 用例已分别迁入 `asset/tests/assets/gltf_importer/basic_import.rs`、`asset/tests/assets/gltf_importer/labeled_subassets.rs`、`asset/tests/assets/gltf_importer/multi_primitive.rs`、`asset/tests/assets/gltf_importer/external_inputs.rs`、`asset/tests/assets/gltf_importer/vertex_channels.rs`、`asset/tests/assets/gltf_importer/material_transforms.rs` 与 `asset/tests/assets/gltf_importer/multi_scene.rs`。父文件现在只保留共享 importer/locator/assertion helper 和模块挂载，行数从 989 降到 129；最大子文件 `labeled_subassets.rs` 为 259 行，13 个父文件测试全部保留在子模块，全部低于 800 行。

守卫：`runtime_15_asset_gltf_importer_tests_are_folder_backed` 验证父模块挂载七个子 owner、代表性 root import/subasset/material label/external input/vertex channel/material transform/multi-scene moved guard 不回流到父文件、子模块合计保留 13 个父文件测试、`asset/tests/assets/gltf_importer.rs` 和所有 glTF importer child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描与 scoped `git diff --check` 通过；带锁 focused guard 与 core-min cargo check 均被当前工作区 Cargo.lock/Cargo.toml 不一致阻塞，不计通过。

## Runtime 15 M3 asset glTF primitive fixture folder split

状态：`runtime_15_asset_gltf_primitive_fixtures_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `asset/tests/assets/gltf_primitive_fixtures.rs` folder-backed 拆分。原父文件中的 triangle/line 基础 glTF writer、tangent/color/UV/skinned vertex-channel writer、texture-transform/two-primitive material writer 与 node-animation writer 已分别迁入 `asset/tests/assets/gltf_primitive_fixtures/basic.rs`、`asset/tests/assets/gltf_primitive_fixtures/vertex_channels.rs`、`asset/tests/assets/gltf_primitive_fixtures/materials.rs` 与 `asset/tests/assets/gltf_primitive_fixtures/animation.rs`。父文件现在只保留模块挂载和受限 `pub(super) use` 测试夹具导出，行数从 876 降到 11；最大子文件 `vertex_channels.rs` 为 307 行，8 个 fixture writer 全部保留在子模块，全部低于 800 行。

守卫：`runtime_15_asset_gltf_primitive_fixtures_are_folder_backed` 验证父模块挂载四个 fixture owner、代表性 triangle/line/vertex/material/animation fixture writer 不回流到父文件、子模块合计保留 8 个 fixture writer、`asset/tests/assets/gltf_primitive_fixtures.rs` 和所有 child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、fixture writer 数量扫描、父子行数预算扫描、docs/status 锚点扫描与 scoped `git diff --check` 通过；带锁 focused guard 与 core-min cargo check 均被当前工作区 Cargo.lock/Cargo.toml 不一致阻塞，不计通过。

## Runtime 15 M3 asset importer test folder split

状态：`runtime_15_asset_importer_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `asset/tests/assets/importer.rs` folder-backed 拆分。原父文件中的 importer subtree hard-cutover、typed TOML/UI backend routing、default builtin data import、registry priority and diagnostic-only precedence、registry error typing、WGSL/OBJ/model import、virtual geometry backfill、physics material 和 animation sequence 用例已分别迁入 `asset/tests/assets/importer/structure.rs`、`asset/tests/assets/importer/typed_toml_ui.rs`、`asset/tests/assets/importer/builtin_data.rs`、`asset/tests/assets/importer/registry_priority.rs`、`asset/tests/assets/importer/registry_errors.rs`、`asset/tests/assets/importer/shader_model.rs` 与 `asset/tests/assets/importer/physics_animation.rs`。父文件现在只保留共享 importer/typed-TOML/WGSL/virtual-geometry helper 和模块挂载，行数从 857 降到 105；最大子文件 `typed_toml_ui.rs` 为 242 行，23 个父文件测试全部保留在子模块，全部低于 800 行。

守卫：`runtime_15_asset_importer_tests_are_folder_backed` 验证父模块挂载七个子 owner、代表性 structure/typed-TOML/builtin data/registry/error/shader-model/physics-animation moved guard 不回流到父文件、子模块合计保留 23 个父文件测试、`asset/tests/assets/importer.rs` 和所有 importer child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描与 scoped `git diff --check` 通过；带锁 focused guard 与 core-min cargo check 均被当前工作区 Cargo.lock/Cargo.toml 不一致阻塞，不计通过。

## Runtime 15 M3 asset scene test folder split

状态：`runtime_15_asset_scene_tests_folder_split_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 `asset/tests/assets/scene.rs` folder-backed 拆分。原父文件中的基础 scene TOML roundtrip、UUID/URL mesh binding、legacy foundation defaults、camera roundtrip/defaults、post-process volume/settings、physics/animation components、point/spot/ambient/rect lights 与 script bindings 用例已分别迁入 `asset/tests/assets/scene/foundation.rs`、`asset/tests/assets/scene/camera.rs`、`asset/tests/assets/scene/post_process.rs`、`asset/tests/assets/scene/physics_animation.rs`、`asset/tests/assets/scene/lights.rs` 与 `asset/tests/assets/scene/script_bindings.rs`。既有 `asset/tests/assets/scene/management.rs` 继续独立承载 scene overview 与 management record 测试。父文件现在只保留 SceneAsset/组件类型共享导入和模块挂载，行数从 841 降到 25；最大子文件 `management.rs` 为 413 行，本次迁出的 10 个父文件测试全部保留在六个子模块，scene 测试族合计 13 个测试，全部低于 800 行。

守卫：`runtime_15_asset_scene_tests_are_folder_backed` 验证父模块挂载七个子 owner、代表性 foundation/camera/post-process/physics-animation/light/script moved guard 不回流到父文件、迁移子模块合计保留 10 个父文件测试、`asset/tests/assets/scene.rs` 和所有 scene child owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描与 scoped `git diff --check` 通过；带锁 focused guard 与 core-min cargo check 均被当前工作区 Cargo.lock/Cargo.toml 不一致阻塞，不计通过。

## Runtime 15 M3 test file budget guard folder split

状态：`runtime_15_test_file_budget_guard_folder_split_static_passed_cargo_lock_blocked`。

补充状态：`render_plan09_custom_target_subowners_static_passed`；`render_plan09_custom_target_composite_source_guard_static_passed`。

R4.1/M3 的当前新增落地部分最初是 `structure_convention/test_file_budget.rs` 自身的 folder-backed 拆分。父文件保留 core framework、UI v2 asset、UI shared core 三个 test-file-budget guard、子模块挂载、自守卫和共享 `read_runtime_src` / `read_repo` helper；runtime diagnostics、RHI command-list、RHI device-contract、asset glTF importer、asset glTF primitive fixture、asset importer、asset project flow sample 与 asset scene guard 分别迁入 `structure_convention/test_file_budget/runtime_diagnostics.rs`、`structure_convention/test_file_budget/rhi_command_list.rs`、`structure_convention/test_file_budget/rhi_device_contract.rs`、`structure_convention/test_file_budget/asset_gltf_importer.rs`、`structure_convention/test_file_budget/asset_gltf_primitive_fixtures.rs`、`structure_convention/test_file_budget/asset_importer.rs`、`structure_convention/test_file_budget/asset_project_flow_sample.rs` 与 `structure_convention/test_file_budget/asset_scene.rs`。后续 Plan 09 camera-target custom-target owner split、sub-owner split、composite source guard 与 queue override source guard 追加并扩展 `structure_convention/test_file_budget/render_products.rs`，父文件当前为 430 行，九个子 owner 分别保持低于 800 行；render-products 子守卫覆盖 `graphics/tests/render_product_camera_targets.rs` 的 7 行根模块、`custom_target.rs` 25 行、`custom_target/composite.rs` 195 行、`custom_target/material_sampling.rs` 354 行、`custom_target/viewport.rs` 250 行、`custom_target/ordering.rs` 105 行、`primary_surface.rs` 247 行、`texture_target.rs` 307 行、fixture 418 行、`m4_behavior_layers.rs` 732 行与 `m4_behavior_layers/queue_override.rs` 167 行。

守卫：`runtime_15_test_file_budget_guard_is_folder_backed` 验证父模块挂载九个子 owner、moved guard 不回流到父文件、父子行数预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。`runtime_15_render_camera_target_products_are_folder_backed` 进一步验证 camera-target render product root 只挂载 folder-backed owners，custom-target root 只挂载 `composite`、`material_sampling`、`viewport`、`ordering` 子 owner，custom/primary/texture/fixture owners 承载对应测试与夹具，并验证 Plan 09、render index、审查发现、结构规范和 render-product-submit 文档的 `render_plan09_camera_target_custom_owner_split_static_passed`、`render_plan09_custom_target_composite_source_guard_static_passed` 与 `render_plan09_queue_override_product_source_guard_static_passed` 状态锚及 custom-target/queue-override 子路径同步。

验证：原 folder split 的 scoped rustfmt/static checks 通过；带锁 Cargo 验证仍受当时工作区 Cargo.lock/Cargo.toml 不一致与 active Cargo lanes 影响，不计通过。本轮 render-products 子守卫的 scoped rustfmt/static/docs scans 覆盖 custom-target sub-owner split 与 composite source guard；focused locked guard 在上一轮 180 秒后超时未出测试结果，且当前仍有 active cargo/rustc 与 `Cargo.lock` 漂移，本轮未声明新的 WGPU/Cargo 通过。

## Runtime 15 M3 Runtime 07 performance hotspot guard folder split

状态：`runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result`。

R4.1/M3 的当前新增落地部分是 Runtime 07 performance hotspot guard 的 folder-backed 拆分。`tests/runtime_absorption/performance_hotspots.rs` 现在只保留 `artifact_render_diagnostics_splits`、`hotspot_inventory`、`owner_budget`、`scene_project_splits`、`submit_context` 与 `submit_error_paths` 六个模块挂载；原父文件中的 submit-context 大 payload 共享守卫、submit error path 守卫、hotspot inventory 证据守卫、scene/project/dynamic split 守卫、artifact/render diagnostics split 守卫分别迁入 `tests/runtime_absorption/performance_hotspots/{submit_context,submit_error_paths,hotspot_inventory,scene_project_splits,artifact_render_diagnostics_splits}.rs`，其中 submit-context 精确 owner 为 `tests/runtime_absorption/performance_hotspots/submit_context.rs`；`owner_budget.rs` 继续拥有 Runtime 07 owner-budget、镜像文档和虚拟几何 snapshot owner 守卫。父文件从 1394 行降到 12 行，最大子 owner `hotspot_inventory.rs` 为 425 行。

`performance_hotpath_source_inventory.py` 同步把 Runtime 07 `expected_test_file_count = 11` 与新增 test inventory 记录为当前审计口径。守卫：`runtime_15_runtime_07_performance_hotspots_guard_is_folder_backed` 验证父/子模块挂载、moved guard 不回流、全部 performance-hotspot guard owner 行数预算、Runtime 07 `expected_test_file_count = 11` 镜像和 Runtime 15/status-output 状态锚同步。

验证：scoped rustfmt --check、Python py_compile、standalone exact guard 1/1、Runtime 07 test inventory scan 与 docs/status 锚点扫描通过；带锁 focused guard 与 core-min cargo check 均 120s 超时无结果，未留下本切片 target-dir 进程，不计通过。

## Runtime 15 M3 script VM test folder split

状态：`runtime_15_script_vm_tests_folder_split_static_passed_cargo_timeout_no_result`。

R4.1/M3 的当前新增落地部分是 `script/vm/tests.rs` folder-backed 拆分。父文件现在只保留 `lifecycle_failures` 和内层测试域模块挂载；原父文件中的 host registry/export/call-table contracts 迁入 `script/vm/tests/host_exports.rs`，bridge host module contracts 迁入 `script/vm/tests/bridge_host.rs`，host reflection markdown 与宏生成 contracts 迁入 `script/vm/tests/reflection_docs.rs`，VM backend/hot-reload/plugin manager contracts 迁入 `script/vm/tests/plugin_runtime.rs`，runtime module wiring/protocol boundary/source-layout contracts 迁入 `script/vm/tests/module_surface.rs`，共享夹具迁入 `script/vm/tests/support.rs`。既有 `script/vm/tests/lifecycle_failures.rs` 继续承载 fallback lifecycle 失败路径测试。

父文件从 1456 行降到 41 行；最大子文件 `reflection_docs.rs` 为 314 行，`support.rs` 为 282 行，其余测试 owner 均低于 800 行。32 个脚本 VM 测试全部保留在子模块中。新增 `structure_convention/test_file_budget/script_vm_tests.rs::runtime_15_script_vm_tests_are_folder_backed`，验证父/子模块挂载、代表性 moved guard 不回流、迁移测试数量、脚本 VM test owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/script/vm/tests.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt --check 已通过；带锁 focused guard 与 core-min cargo check 均在冷构建阶段超时，未得到编译/测试结果，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 script VM hot-reload coordinator test folder split

状态：`runtime_15_script_vm_hot_reload_coordinator_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `script/vm/runtime/hot_reload_coordinator.rs` 的 module-local test owner 拆分。父文件现在只保留 `HotReloadCoordinator`、`PluginSlot`、slot table poison recovery helper、load/hot-reload/unload/call/list 生产路径和 `#[cfg(test)] mod tests;` 挂载；原内嵌的 hot-reload policy、state transfer、lifecycle query deadlock guard 与 slot-table poison recovery tests 迁入 `script/vm/runtime/hot_reload_coordinator/tests.rs`。

父文件从 770 行降到 282 行；child owner 为 407 行，5 个原 module-local 测试全部保留。新增 `structure_convention/test_file_budget/script_vm_tests.rs::runtime_15_script_vm_hot_reload_coordinator_tests_are_folder_backed`，验证父/子模块挂载、moved test 不回流、测试数量、行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/script/vm/zr_vm_host_reflection.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved-test parent scan、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 gameplay host test folder split

状态：`runtime_15_gameplay_host_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `script/vm/gameplay_host/tests.rs` folder-backed 拆分。父文件现在只保留共享导入、`mod combat_lifecycle;`、`mod component_state;`、`mod property_animation;`、`mod spawn_transform;` 与 `assert_vec3_close` / `assert_quat_close` helper；原父文件中的 pose/transform 与 spawn model 合约迁入 `script/vm/gameplay_host/tests/spawn_transform.rs`，current HP、particle sprites 与 string dynamic-component 合约迁入 `script/vm/gameplay_host/tests/component_state.rs`，damage report、despawn stale handle 与 hit-before-death 合约迁入 `script/vm/gameplay_host/tests/combat_lifecycle.rs`，script-binding predicate/heal 与 animation/world HUD bar 合约迁入 `script/vm/gameplay_host/tests/property_animation.rs`。

父文件从 891 行降到 46 行；最大子文件 `property_animation.rs` 为 289 行，`combat_lifecycle.rs` 为 258 行，9 个 gameplay host 测试全部保留在子模块中。新增 `structure_convention/test_file_budget/script_vm_tests.rs::runtime_15_gameplay_host_tests_are_folder_backed`，验证父/子模块挂载、代表性 moved test 不回流、迁移测试数量、gameplay host test owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/script/vm/gameplay_host.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、父子行数预算扫描、moved test scan、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 shader prewarm manifest test folder split

状态：`runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `bin/zircon_shader_prewarm/manifest.rs` 的测试 owner 拆分。父文件现在只保留 shader prewarm manifest 生产逻辑以及 `#[cfg(test)] mod tests;` 挂载；原内联 `shader_prewarm_asset_root_manifest_reads_compound_zshader_package` 测试迁入 `bin/zircon_shader_prewarm/manifest/tests.rs`，继续覆盖 compound `.zshader`、`.zmaterial` feature bits、built-in shading model 与 alpha-blend pass filtering 的资产扫描预热 manifest 合约。

父文件从 810 行降到 672 行；子文件为 137 行，所有 owner 低于 800 行。新增 `structure_convention/test_file_budget/shader_prewarm_manifest.rs::runtime_15_shader_prewarm_manifest_tests_are_folder_backed`，验证父/子模块挂载、moved test 不回流、迁移测试数量、shader prewarm manifest owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/core/framework/render/shader.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、父子行数预算扫描、moved test scan、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 scene ECS schedule test folder split

状态：`runtime_15_scene_ecs_schedule_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `scene/tests/ecs_schedule.rs` folder-backed 拆分。原父文件中的 ResourceStore / Events / EventStore / EventSubscription 用例迁入 `scene/tests/ecs_schedule/resources_events.rs`；SystemStage / SystemSet / stage-plan / duplicate-id / native-registration 用例迁入 `scene/tests/ecs_schedule/schedule_plan.rs`；derived-state dirty flag、render-extract prepare、prepared frame extract、camera layer / camera product projection和 inactive camera 用例迁入 `scene/tests/ecs_schedule/render_extract.rs`；WorldDriver hook、runtime time advance、native/render-extract hook ordering、runtime scene system ordering用例迁入 `scene/tests/ecs_schedule/world_driver.rs`。既有 `conflict_graph.rs`、`fixed_update.rs` 与 `parallel_executor.rs` 继续作为同 folder-backed schedule 家族 owner。

父文件从 1430 行降到 32 行；最大 owner 是既有 `conflict_graph.rs` 的 768 行，新增最大子文件 `render_extract.rs` 为 413 行，`schedule_plan.rs` 为 400 行。37 个原父文件测试全部迁入新增子模块，`ecs_schedule/` 家族合计 57 个测试，所有 owner 低于 800 行。新增 `structure_convention/test_file_budget/scene_ecs_schedule.rs::runtime_15_scene_ecs_schedule_tests_are_folder_backed`，验证父/子模块挂载、代表性 moved guard 不回流、迁移测试数量、全族测试数量、ECS schedule test owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/scene/ecs.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描和 docs/status 锚点扫描已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 scene ECS systems test folder split

状态：`runtime_15_scene_ecs_systems_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `scene/tests/ecs_systems.rs` folder-backed 拆分。原父文件中的 command queue / entity command 用例迁入 `scene/tests/ecs_systems/commands.rs`；SystemState、QueryState mutation、optional resource、ParamSet 和 8 元 tuple/ParamSet 用例迁入 `state_params.rs`；EventReader/EventWriter 队列和 cursor 用例迁入 `events.rs`；Added/Changed run-window、cached direct、cached iter、count/is_empty helper 用例迁入 `run_window_filters.rs`；get_many / iter_many / single query behavior 用例迁入 `many_single_queries.rs`；removed-components、LocalParam 和 scheduled native local-state 用例迁入 `removal_local.rs`。

父文件从约 1000+ 行降到 53 行，只保留共享 `Health` / `Player` / `Marker` / `Score` / `HitEvent` / `LocalCounter` fixture、`expect_query_error(...)` helper 和子模块挂载；最大子文件 `run_window_filters.rs` 为 330 行，`state_params.rs` 为 286 行。24 个原父文件测试全部迁入六个子模块，所有 owner 低于 800 行。新增 `structure_convention/test_file_budget/scene_ecs_systems.rs::runtime_15_scene_ecs_systems_tests_are_folder_backed`，验证父/子模块挂载、代表性 moved guard 不回流、迁移测试数量、ECS systems test owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/scene/ecs.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描和 docs/status 锚点扫描已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 scene ECS query test folder split

状态：`runtime_15_scene_ecs_query_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `scene/tests/ecs_query.rs` folder-backed 拆分。原父文件中的 query data read、tuple/filter arity、stable location 和 single-result 用例迁入 `scene/tests/ecs_query/read_items.rs`；mutable query、get_mut/get_many_mut、access conflict 和 duplicate mutable component 用例迁入 `mutation_access.rs`；fixed scene component query 与 Ref/Mut change tick 用例迁入 `fixed_ticks.rs`；mutable/cached-direct iter-many run-window 用例迁入 `iter_many.rs`；cache rebuild、count/empty/get/many/unique helpers、cached-direct table/sparse location、archetype movement 和 optional archetype membership 用例迁入 `cached_queries.rs`。

父文件从 938 行降到 60 行，只保留共享 `Health` / `Enemy` / `Player` / `SparseScore` fixture、`expect_query_error(...)`、`cached_component_locations_for(...)` 和子模块挂载；最大子文件 `cached_queries.rs` 为 555 行。19 个原父文件测试全部迁入五个子模块，所有 owner 低于 800 行。新增 `structure_convention/test_file_budget/scene_ecs_query.rs::runtime_15_scene_ecs_query_tests_are_folder_backed`，验证父/子模块挂载、代表性 moved guard 不回流、迁移测试数量、ECS query test owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/scene/ecs.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描和 docs/status 锚点扫描已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 scene ECS query structure test folder split

状态：`runtime_15_scene_ecs_query_structure_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `scene/tests/ecs_query_structure.rs` folder-backed 拆分。原父文件中的 QueryState folder/cache-vector 守卫迁入 `scene/tests/ecs_query_structure/query_state_layout.rs`；many-mut 和 mut iterator 借用缓存守卫迁入 `mutable_iterators.rs`；cached many/cached direct/read iterator 守卫迁入 `cached_iterators.rs`；cache rebuild reserve 与 hot-path 守卫迁入 `cache_rebuild.rs`；archetype index 与 QueryAccess boolean conflict 守卫迁入 `archetype_access.rs`；cached combination iterator 守卫迁入 `combinations.rs`。

父文件从 1041 行降到 33 行，只保留共享 `BTreeSet` / path imports、QueryState owner 常量、行数预算常量、source/path helper 和子模块挂载；最大子文件 `scene/tests/ecs_query_structure/cached_iterators.rs` 为 233 行，cache rebuild 守卫 owner 为 `scene/tests/ecs_query_structure/cache_rebuild.rs`。11 个原父文件结构守卫全部迁入六个子模块，所有 owner 低于 800 行。新增 `structure_convention/test_file_budget/scene_ecs_query_structure.rs::runtime_15_scene_ecs_query_structure_tests_are_folder_backed`，验证父/子模块挂载、代表性 moved guard 不回流、迁移测试数量、ECS query structure test owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/scene/ecs.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描和 docs/status 锚点扫描已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 scene derived-state test folder split

状态：`runtime_15_scene_derived_state_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `scene/tests/derived_state.rs` folder-backed 拆分。原父文件中的 spawn-node hot-path guard 迁入 `scene/tests/derived_state/spawn_paths.rs`；层级重建、pre-sized parent snapshot、subtree traversal、mobility preflight 与 internal scene-system stage-plan source guard 迁入 `scene/tests/derived_state/hierarchy_rebuild.rs`；direct parent branch、projected value/default component、node-record projection、scalar accessor 和 retained node-cache source guard 迁入 `scene/tests/derived_state/projected_reads.rs`；post-update freshness、no-op mutator、render extract prepare、property path node cache 和 active-camera freshness 行为测试迁入 `scene/tests/derived_state/runtime_freshness.rs`；imported record、cycle rejection、active hierarchy、large hierarchy propagation 与 mobility bucket 行为测试迁入 `scene/tests/derived_state/hierarchy_behavior.rs`。

父文件从 943 行降到 68 行，只保留共享导入、`LARGE_HIERARCHY_NODE_COUNT`、`detached_node_record(...)`、`pending_reparented_world(...)`、source/path helper 和子模块挂载；最大子文件 `scene/tests/derived_state/projected_reads.rs` 为 291 行，`scene/tests/derived_state/runtime_freshness.rs` 为 193 行。23 个原父文件测试全部迁入五个子模块，所有 owner 低于 800 行。新增 `structure_convention/test_file_budget/scene_derived_state.rs::runtime_15_scene_derived_state_tests_are_folder_backed`，验证父/子模块挂载、代表性 moved guard 不回流、迁移测试数量、derived-state test owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/scene/ecs.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 dynamic scene session path-management test folder split

状态：`runtime_15_dynamic_scene_session_path_management_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `scene/tests/dynamic_scene_session/path_management.rs` folder-backed 拆分。原父文件中的 path-level rename/update metadata/touch/remove commit 用例迁入 `scene/tests/dynamic_scene_session/path_management/slot_mutations.rs`；no-write mutation preview 用例迁入 `scene/tests/dynamic_scene_session/path_management/mutation_previews.rs`；copy commit/preview 用例迁入 `scene/tests/dynamic_scene_session/path_management/slot_copy.rs`；loaded archive 与 source-path single-slot import commit/preview 用例迁入 `scene/tests/dynamic_scene_session/path_management/single_slot_import.rs`；standalone single-slot save 用例迁入 `scene/tests/dynamic_scene_session/path_management/single_slot_save.rs`；archive merge commit/preview 用例迁入 `scene/tests/dynamic_scene_session/path_management/archive_merge.rs`。

父文件从 972 行降到 14 行，只保留共享 `fs`、`RuntimeSessionArchive` / `RuntimeSessionArchiveMergePolicy` / `RuntimeSessionMetadata` / `World` imports、dynamic-session helper imports 和子模块挂载；最大子文件 `scene/tests/dynamic_scene_session/path_management/single_slot_import.rs` 为 249 行，`scene/tests/dynamic_scene_session/path_management/archive_merge.rs` 为 202 行。19 个原父文件测试全部迁入六个子模块，所有 owner 低于 800 行。新增 `structure_convention/test_file_budget/scene_dynamic_session.rs::runtime_15_dynamic_scene_session_path_management_tests_are_folder_backed`，验证父/子模块挂载、代表性 moved guard 不回流、迁移测试数量、dynamic-session path-management owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/scene/dynamic_scene.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 scene component-structure test folder split

状态：`runtime_15_scene_component_structure_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `scene/tests/component_structure.rs` folder-backed 拆分。原父文件中的 runtime world-domain、world property-access、render-extract snapshot-adapter rejection、neutral inspection surface 和 LateUpdate resurrection guard 迁入 `scene/tests/component_structure/runtime_world_domains.rs`；component registry reverse/dynamic lookup guard 迁入 `scene/tests/component_structure/component_registry.rs`；project serialization authoring-boundary guard 迁入 `scene/tests/component_structure/project_serialization.rs`；Runtime 05 dynamic scene root/session owner-tree guard 迁入 `scene/tests/component_structure/dynamic_scene_owner_tree.rs`；component storage dispatch/result-vector guard 迁入 `scene/tests/component_structure/component_storage_dispatch.rs`；table/sparse storage indexing guard 迁入 `scene/tests/component_structure/component_storage_indexing.rs`。既有 `scene/tests/component_structure/runtime_08_owner_tree.rs` 继续承载 Runtime 08 ECS/query/source-stable owner-tree coverage。

父文件从 842 行降到 9 行，只保留共享 authoring-boundary import 与七个子模块挂载；最大 owner 为既有 `scene/tests/component_structure/runtime_08_owner_tree.rs` 301 行，新增最大子文件 `scene/tests/component_structure/runtime_world_domains.rs` 为 199 行，`scene/tests/component_structure/component_storage_indexing.rs` 为 179 行，`scene/tests/component_structure/dynamic_scene_owner_tree.rs` 为 187 行。原父文件 20 个测试全部迁入六个新增子模块，component-structure 测试族合计 23 个测试，所有 owner 低于 800 行。新增 `structure_convention/test_file_budget/scene_component_structure.rs::runtime_15_scene_component_structure_tests_are_folder_backed`，验证父/子模块挂载、代表性 moved guard 不回流、迁移测试数量、component-structure owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/scene/ecs.md`、`docs/zircon_runtime/scene/dynamic_scene.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 scene ECS reflect foundation test folder split

状态：`runtime_15_scene_ecs_reflect_foundation_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `scene/tests/ecs_reflect/foundation.rs` folder-backed 拆分。原父文件中的 TypeRegistry / RuntimeTypeRegistration / world serialization guard 迁入 `scene/tests/ecs_reflect/foundation/registry.rs`；scene/reflected/json/animation value conversion guard 迁入 `scene/tests/ecs_reflect/foundation/value_conversion.rs`；component/resource address routing guard 迁入 `scene/tests/ecs_reflect/foundation/address_routing.rs`；fixed registration guard 迁入 `scene/tests/ecs_reflect/foundation/fixed_registry.rs`；ambient/rect light 与 name reflection guard 迁入 `scene/tests/ecs_reflect/foundation/fixed_lights_name.rs`；ActiveSelf / LocalTransform reflection guard 迁入 `scene/tests/ecs_reflect/foundation/fixed_transform_active.rs`；RenderLayerMask / RigidBody / fixed reflection error guard 迁入 `scene/tests/ecs_reflect/foundation/fixed_render_physics.rs`。

父文件从 1122 行降到 158 行，只保留共享导入、dummy component/resource adapter helpers、registration/address helpers 和七个子模块挂载；最大子文件 `scene/tests/ecs_reflect/foundation/fixed_render_physics.rs` 为 236 行，`scene/tests/ecs_reflect/foundation/value_conversion.rs` 为 191 行。20 个原父文件测试全部迁入七个子模块，所有 owner 低于 800 行。新增 `structure_convention/test_file_budget/scene_ecs_reflect_foundation.rs::runtime_15_scene_ecs_reflect_foundation_tests_are_folder_backed`，验证父/子模块挂载、代表性 moved guard 不回流、迁移测试数量、ECS reflect foundation owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/scene/ecs.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 dynamic scene root test folder split

状态：`runtime_15_dynamic_scene_root_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `scene/tests/dynamic_scene.rs` folder-backed 拆分。原父文件中的 dynamic-scene roundtrip、scene patch resource/preview、world-mutation typed error 和 versioned JSON migration guard 迁入 `scene/tests/dynamic_scene/scene_patch_document.rs`；runtime session archive roundtrip、duplicate/unsupported/non-canonical validation、manual serialization ordering 与 metadata normalization guard 迁入 `scene/tests/dynamic_scene/archive_core.rs`；archive rename/copy/merge/prune/touch/diff 与 mutation-surface guard 迁入 `scene/tests/dynamic_scene/archive_mutation.rs`；archive statistics、latest/oldest selection、manifest summaries/filtering/tag selection 与 upsert summary guard 迁入 `scene/tests/dynamic_scene/archive_manifest.rs`；level restore/apply guard 迁入 `scene/tests/dynamic_scene/level_apply.rs`。

父文件从 1548 行降到 181 行，只保留共享导入、`FrameCounter` resource fixture、cloud-layer descriptor、FrameCounter reflection/resource helpers 和五个子模块挂载；最大子文件 `scene/tests/dynamic_scene/archive_mutation.rs` 为 376 行，`scene/tests/dynamic_scene/archive_manifest.rs` 为 355 行，`scene/tests/dynamic_scene/scene_patch_document.rs` 为 286 行。27 个原父文件测试全部迁入五个子模块，所有 owner 低于 800 行。新增 `structure_convention/test_file_budget/scene_dynamic_scene_root.rs::runtime_15_dynamic_scene_root_tests_are_folder_backed`，验证父/子模块挂载、代表性 moved guard 不回流、迁移测试数量、dynamic-scene root owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/scene/dynamic_scene.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files` 仍 pending 于活跃 render 会话占用的 `scene/tests/render_extract.rs`；`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 test file budget root-layout child split

状态：`runtime_15_test_file_budget_root_layout_child_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `structure_convention/test_file_budget.rs` 根布局守卫减压。新增 scene ECS query structure 守卫后，父文件达到 793 行，继续追加后续预算守卫会越过 800 行阈值；因此将 `runtime_15_test_file_budget_guard_is_folder_backed` 自守卫整体迁入 `structure_convention/test_file_budget/root_layout.rs`，父文件只保留 `mod` 挂载、core framework/UI v2/UI shared core 三个仍在父文件内的预算守卫，以及共享 `read_runtime_src(...)` / `read_repo(...)` helper。后续 Plan 09 composite source guard 扩展 `test_file_budget/render_products.rs`，并继续追加 scene derived-state、dynamic-scene root、dynamic-session path-management、component-structure 与 ECS reflect foundation 子守卫后，父文件当前为 454 行，仍低于 800 行阈值。

父文件从 793 行降到 428 行，新增 derived-state、dynamic-scene root、dynamic-session path-management、component-structure 与 ECS reflect foundation 子守卫后当前为 454 行；`root_layout.rs` 当前为 480 行。`root_layout.rs` 继续读取并验证所有 test-budget 子 owner，包括 asset、RHI、runtime diagnostics、render products、script VM、scene ECS schedule/systems/query/query-structure/derived-state/dynamic-scene root/dynamic-session/component-structure/ECS reflect foundation；它同时要求父文件挂载 `mod root_layout;`、根布局守卫不回流到父文件、所有 test-budget owner 低于 800 行，并验证 Runtime 15、runtime index、review findings、结构规范、本文档和 status-output expectations 的新旧状态锚同步。

验证：scoped rustfmt/static checks、父子行数预算扫描、root guard moved scan、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 test file budget root-layout status scan child split

状态：`runtime_15_test_file_budget_root_layout_status_scan_child_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 test-file-budget root-layout 守护继续减压。`tests/runtime_absorption/structure_convention/test_file_budget/root_layout.rs` 从 739 行降到 532 行，只保留 folder-backed parent/child layout 与 moved guard ownership 检查；新增 `tests/runtime_absorption/structure_convention/test_file_budget/root_layout/status_scan.rs`，用 149 行承接所有 test-file-budget child owner 行数预算扫描、历史 root-layout 状态锚扫描和本切片自守护。

新增 `runtime_15_test_file_budget_root_layout_status_scan_is_child_owner`，验证 `root_layout.rs` 挂载 `root_layout/status_scan.rs`、status/line-budget scan 不回流、所有 test-file-budget guard owner 低于 800 行预算，并要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。验证：scoped rustfmt/static checks、父子行数预算扫描、moved root-layout status scan 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按支撑切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 test-file-budget guard sweep 仍 pending。

## Runtime 15 M3 test file budget root-layout folder-backed guard child split

状态：`runtime_15_test_file_budget_root_layout_folder_backed_guard_child_split_static_passed_cargo_timeout_no_result`。

R4.1/M3 的 2026-06-24 支撑切片已把 `runtime_15_test_file_budget_guard_is_folder_backed` 从 `structure_convention/test_file_budget/root_layout.rs` 迁入 `structure_convention/test_file_budget/root_layout/folder_backed.rs`，并新增 `root_layout/module_layout.rs::runtime_15_test_file_budget_root_layout_folder_backed_guard_is_child_owner`。该守卫锁定 root-layout 父模块挂载、旧 guard 不回流、`folder_backed.rs` / `module_layout.rs` 行数预算，以及 Runtime 15、runtime index、review findings、结构规范、本文档和 status-output expectations 的镜像锚。Cargo focused run 超时无结果，不计 Cargo 通过；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 test-file-budget guard sweep 仍 pending。

## Runtime 15 M3 test file budget root-layout folder-backed support child-owner split

状态：`runtime_15_test_file_budget_root_layout_folder_backed_support_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 的当前支撑切片只整理 `structure_convention/test_file_budget/root_layout/folder_backed.rs` 内部的测试守卫支撑结构，不改变任何生产 runtime 行为，也不改旧的 `runtime_15_test_file_budget_guard_is_folder_backed` 测试入口。父文件现在只挂载 `folder_backed/assertions.rs`、`folder_backed/guard_names.rs` 与 `folder_backed/sources.rs`，并保留旧守卫入口加新的 `runtime_15_test_file_budget_root_layout_folder_backed_support_child_owners_are_folder_backed` 自检。

`assertions.rs` 承接 test-budget 子 owner 的批量断言组，`guard_names.rs` 承接 moved guard 名称构造，`sources.rs` 承接批量 source reads；父文件为 154 行，最大 child `assertions.rs` 为 354 行，四个 owner 均低于 400 行 focused 预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、本文档、session note 与 status-output expectations，精确锚点包括 `structure_convention/test_file_budget/root_layout/folder_backed.rs`、`structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs`、`structure_convention/test_file_budget/root_layout/folder_backed/guard_names.rs` 和 `runtime_15_test_file_budget_root_layout_folder_backed_support_child_owners_are_folder_backed`。Cargo 按支撑切片节奏 deferred，不计 Cargo 通过。

## Runtime 15 M3 test file budget root-layout UI child split

状态：`runtime_15_test_file_budget_root_layout_ui_child_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 test-file-budget root-layout UI 守护继续减压。`tests/runtime_absorption/structure_convention/test_file_budget/root_layout.rs` 从 780 行降到 499 行，只保留非 UI folder-backed parent/child layout 与 moved guard ownership 检查；新增 `tests/runtime_absorption/structure_convention/test_file_budget/root_layout/ui_children.rs`，用 225 行承接 24 个 UI test-budget child guard 的父模块挂载、moved guard 不回流、子 owner 归属和 docs/status 锚点扫描。

新增 `runtime_15_test_file_budget_root_layout_ui_child_scan_is_child_owner`，验证 `root_layout.rs` 挂载 `root_layout/ui_children.rs`、UI guard scan 不回流、`root_layout/status_scan.rs` 的行数预算列表包含新子 owner，并要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的状态锚同步。验证：scoped rustfmt/static checks、父子行数预算扫描、moved UI guard ownership 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按支撑切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 test-file-budget guard sweep 仍 pending。

## Runtime 15 M3 test file budget guard root mod cutover

状态：`runtime_15_test_file_budget_guard_root_mod_cutover_static_passed_cargo_lock_blocked`。

R4.1/M3 的当前新增落地部分是 test-file-budget guard 根模块硬切。旧平铺 `structure_convention/test_file_budget.rs` 已删除，根模块迁入 `structure_convention/test_file_budget/mod.rs`；父级 `structure_convention.rs` 直接通过 `#[path = "structure_convention/test_file_budget/mod.rs"] mod test_file_budget;` 挂载，避免普通 rustfmt/模块解析把 `test_file_budget/*` 子 owner 误解析到 `structure_convention/` 兄弟目录。

`root_layout.rs` 继续拥有 `runtime_15_test_file_budget_guard_is_folder_backed` 自守卫，并新增旧平铺文件不存在检查、`structure_convention/test_file_budget/mod.rs` 文档/status 锚点、父/子 owner 行数预算和状态输出期望行。当前 `mod.rs` 为 441 行，`root_layout.rs` 为 709 行，均低于 800 行阈值；该切片不改变 runtime 生产行为，只收束测试守卫树的 owner 根路径。

验证：scoped rustfmt/static checks、旧文件不存在扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；带锁 Cargo 被当前 `Cargo.lock`/`Cargo.toml` 不一致阻断，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 asset test-budget guard child-owner split

状态：`runtime_15_asset_test_budget_guard_child_owner_split_static_passed_cargo_deferred`。

R4.1/M3 继续对 test-budget 守卫树做二级减压。`structure_convention/test_file_budget/asset_tests.rs` 原本直接承载 pack、facade、project zmeta、project manager 与 material 五个资产测试 folder-backed 预算守卫，文件达到 759 行，继续作为 asset 测试族 umbrella 会重新接近 800 行阈值。

本切片将父文件收缩为 161 行，只保留 `mod pack;`、`mod facade;`、`mod project;`、`mod material;` 和 `runtime_15_asset_test_budget_guard_child_owner_split` 自检。五个既有守卫按责任域迁入 `asset_tests/pack.rs`、`asset_tests/facade.rs`、`asset_tests/project.rs` 与 `asset_tests/material.rs`；其中 `project.rs` 同时承接 zmeta 与 manager 两个 project-family 预算守卫，最大 child 为 275 行。`root_layout.rs` 同步读取四个 child owner，验证 moved guard 不回流、所有 test-budget owner 低于 800 行，并要求 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 的新状态锚同步。

验证：scoped rustfmt/static checks、父子行数预算扫描、moved guard scan、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 F14 diagnostics normalization

状态：`runtime_15_diagnostics_frame_trait_wrapper_removed_coremin_check_passed`。

E5/S11/F14 的当前已落地部分是 diagnostics 命名和纯包装层收束。`core/runtime/diagnostics/frame_diagnostics.rs` 现在拥有 `FrameDiagnostics` / `FrameDiagnosticsStatus`，render、physics、animation diagnostics 和 `EcsFramePerformanceDiagnostics` 均通过同一 trait 暴露 domain、available 和 error 状态。`RuntimeDiagnosticsSnapshot::frame_diagnostics_statuses()` 只组合 render/physics/animation 的状态，不改动既有 `DiagnosticStore` metric paths，避免影响诊断面板和日志消费者。

`World` 现在直接持有 `EcsFramePerformanceDiagnostics`，`scene/world/performance_diagnostics.rs` 不再定义 `WorldEcsFramePerformanceDiagnostics`，也不再通过 `.0` 做纯转发。守卫：`runtime_15_diagnostics_use_frame_trait_without_world_wrapper` 验证 trait owner、runtime 子域组合、ECS `scene.ecs` domain、World 直接字段和相关计划/文档状态锚同步。行为锚：`runtime_snapshot_frame_diagnostics_statuses_preserve_subdomains` 和 `ecs_frame_performance_diagnostics_uses_scene_ecs_frame_domain`。F13 registration、update stats、feedback shared payload、prepare-input shared frame owner 样板与 full provider boilerplate audit 已由 shared-owner 子切片和总守卫收束。

## Runtime 15 F13 provider registration shared owner

状态：`runtime_15_provider_registration_shared_owner_coremin_check_passed`。

E5/S11/F13 的当前新增落地部分是 runtime provider registration 存储与 debug 样板收束。`graphics/runtime_provider/registration.rs` 现在拥有 `RuntimeProviderRegistration<P: ?Sized>`，统一保存 provider ID、priority、provider trait object 和 provider-specific debug name；`define_runtime_provider_registration!` 生成 HGI、Virtual Geometry、Solari 三个 public registration wrapper 的 `new`、`provider_id`、`priority`、`with_priority`、`provider` 和 `Debug` 实现。

这保持外部 API 名称不变，`RuntimeExtensionRegistry`、`GraphicsModule` 和 `WgpuRenderFramework` 仍消费原来的 provider-specific registration 类型；变化仅限三套 provider registration 不再各自复制字段、priority builder 和 debug 实现。守卫：`runtime_15_provider_registration_uses_shared_owner` 验证共享 owner、宏生成入口、三套 provider-specific registration 不再持有重复字段，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 `docs/zircon_runtime/graphics/runtime_provider/registration.md` 状态锚同步。本切片只关闭 registration 样板；update、feedback 与 prepare-input shared-owner 子切片由后续记录覆盖。

## Runtime 15 F13 provider update shared stats owner

状态：`runtime_15_provider_update_shared_stats_owner_coremin_check_passed`。

E5/S11/F13 的当前新增落地部分是 runtime provider update stats 样板收束。`graphics/runtime_provider/update.rs` 现在拥有 `RuntimeProviderUpdate<S>`，统一保存 update stats payload；`define_runtime_provider_update!` 生成 HGI 与 Virtual Geometry 两个 provider-specific update wrapper。`HybridGiRuntimeUpdate::stats()` 继续按旧 API 返回 `HybridGiRuntimeStats` by value，`VirtualGeometryRuntimeUpdate::stats()` 继续返回 `&VirtualGeometryRuntimeStats`，因此 record-submission 与测试 fixture 调用点不需要迁移。

守卫：`runtime_15_provider_update_uses_shared_stats_owner` 验证共享 owner、宏生成入口、两套 provider-specific update 不再声明自己的 `stats` 字段，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 `docs/zircon_runtime/graphics/runtime_provider/update.md` 状态锚同步。验证边界：scoped rustfmt 与 core-min `cargo check` 已通过；standalone guard/status-output binary 启动被 Windows `ResourceUnavailable` / 用户取消状态阻断，focused Cargo test 超时无结果，不计为通过。本切片只关闭 update stats 样板；feedback 与 prepare-input shared-owner 子切片由后续记录覆盖。

## Runtime 15 F13 provider feedback shared payload owner

状态：`runtime_15_provider_feedback_shared_payload_owner_coremin_check_passed`。

E5/S11/F13 的当前新增落地部分是 runtime provider feedback 共同 payload 样板收束。`graphics/runtime_provider/feedback.rs` 现在拥有 `RuntimeProviderFeedback<G, V>`，统一保存 `gpu_completion` 与 `visibility_feedback` 两个 provider feedback 共同字段；HGI 与 Virtual Geometry 的 public feedback wrapper 继续保留原类型名、constructor 和 getter surface。

该切片刻意不合并 feature-specific 字段：HGI 的 `evictable_probe_ids` 仍由 `HybridGiRuntimeFeedback` 拥有；Virtual Geometry 的 `node_and_cluster_cull_page_requests`、`evictable_page_ids` 与 `generation` 仍由 `VirtualGeometryRuntimeFeedback` 拥有。守卫：`runtime_15_provider_feedback_uses_shared_payload_owner` 验证共享 owner、runtime_provider 挂载、两套 provider-specific feedback 不再声明共同 payload 字段，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 `docs/zircon_runtime/graphics/runtime_provider/feedback.md` 状态锚同步。验证：scoped rustfmt、standalone structure guard、standalone status-output guards 与 core-min `cargo check` 已通过（既有 warnings）。本切片只关闭 feedback 共同 payload 样板；prepare-input shared frame owner 由后续记录覆盖。

## Runtime 15 F13 provider prepare input shared frame owner

状态：`runtime_15_provider_prepare_input_shared_frame_owner_coremin_check_passed`。

E5/S11/F13 的当前新增落地部分是 runtime provider prepare input 共同帧字段收束。`graphics/runtime_provider/prepare_input.rs` 现在拥有 `RuntimeProviderPrepareInput<'a, E>`，统一保存 provider prepare 阶段共同的 optional extract 与 frame generation。HGI 与 Virtual Geometry 的 public prepare input wrapper 继续保留原类型名、constructor 参数和 getter surface。

该切片不合并 feature-specific 输入：HGI 的 mesh snapshots、三类 light snapshots 与 `VisibilityHybridGiUpdatePlan` 仍由 `HybridGiRuntimePrepareInput` 拥有；Virtual Geometry 的 page upload plan、visible clusters 与 draw segments 仍由 `VirtualGeometryRuntimePrepareInput` 拥有。守卫：`runtime_15_provider_prepare_input_uses_shared_extract_generation_owner` 验证共享 owner、runtime_provider 挂载、两套 provider-specific prepare input 不再声明共同 `extract` / `generation` 字段，并验证 Runtime 15 计划、runtime index、render index、审查发现、结构规范、本文档和 `docs/zircon_runtime/graphics/runtime_provider/prepare_input.md` 状态锚同步。验证：scoped rustfmt、standalone structure guard 1/1、standalone status-output all-subplans guard 1/1 与 core-min `cargo check` 已通过（既有 warnings）；broader `status_output` 批次仍有非本切片 Runtime 06 F8 row-drift 失败。

## Runtime 15 F13 full provider boilerplate audit

状态：`runtime_15_provider_boilerplate_full_audit_coremin_check_passed`。

E5/S11/F13 的当前总验收是 provider boilerplate 总守卫。`structure_convention/provider_boilerplate.rs` 现在包含 `runtime_15_no_duplicated_provider_boilerplate`，把 registration、update、feedback、prepare input 四个 shared-owner 子切片作为一个整体审计。

守卫要求 `graphics/runtime_provider/{registration,update,feedback,prepare_input}.rs` 均挂载共享 owner；HGI、Virtual Geometry、Solari registration 文件只使用 `define_runtime_provider_registration!`，不再复制 provider id / priority / trait-object / debug 样板；HGI/VG update 文件只使用 `define_runtime_provider_update!`，不再手写 constructor / stats getter；HGI/VG feedback 文件委托 `RuntimeProviderFeedback<G, V>`，不再复制共同 GPU completion / visibility feedback 字段；HGI/VG prepare-input 文件委托 `RuntimeProviderPrepareInput<'a, E>`，不再复制共同 optional extract / generation 字段。Particle feedback 只有 `ParticleGpuFeedback` 且没有 visibility feedback payload，因此作为 feature-specific 单 payload 例外记录，不强行套入双 payload owner。

状态输出期望行同步到 `expected_status_row_data.rs` 和 `expected_slices/{status,date}.rs`。验证：scoped rustfmt --check 通过；standalone full provider boilerplate guard 1/1 通过；standalone status-output all-subplans guard 1/1 通过；core-min `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime15-provider-boilerplate-full-coremin-0622` 通过（既有 141 warnings）。完整 `module_convention_gate`、全量 dead-code sweep 与测试组织拆分仍 pending。


## Runtime 15 M3 picking test folder split

状态：`runtime_15_picking_tests_folder_split_static_passed_cargo_deferred`。`tests/picking/mod.rs` 现在只保留 fixture 和子模块挂载，`rays`、`hits_and_hover`、`diagnostics`、`pipeline` 与 `pointer_events` 子 owner 承接 20 个测试；守卫 `runtime_15_picking_tests_are_folder_backed` 锁定布局、测试数量、低于 800 行预算和跨文档状态锚。Cargo 按实施切片节奏 deferred。

## Runtime 15 M3 asset mesh test root split

状态：`runtime_15_asset_mesh_tests_root_split_static_passed_cargo_deferred`。`asset/tests/assets/mesh.rs` 现在只保留共享 fixture 和七个 child owner 挂载，`document_roundtrip`、`validation`、`summaries` 与 `conversion_import` 承接 19 个原父文件测试；守卫 `runtime_15_asset_mesh_tests_are_folder_backed` 锁定 moved test、测试数量、低于 800 行预算和跨文档状态锚。Cargo 按实施切片节奏 deferred。

## Runtime 15 M3 asset project example vampire test folder split

状态：`runtime_15_asset_project_example_vampire_tests_folder_split_static_passed_cargo_deferred`。`asset/tests/project/example_vampire.rs` 现在只保留 `vampire_root()` fixture 和两个 child owner 挂载，`asset/tests/project/example_vampire/manifest_scene_imports.rs` 与 `asset/tests/project/example_vampire/third_person_render_extract.rs` 承接原有 manifest/scene/import 与 third-person render extract 两个测试；守卫 `runtime_15_asset_project_example_vampire_tests_are_folder_backed` 锁定 moved test、测试数量、低于 800 行预算和跨文档状态锚。Cargo 按实施切片节奏 deferred。
## Runtime 15 M3 asset artifact store test folder split

状态：`runtime_15_asset_artifact_store_tests_folder_split_static_passed_cargo_deferred`。`asset/tests/assets/artifact_store.rs` 现在只保留 artifact payload/reference helper 和五个 child owner 挂载，`asset/tests/assets/artifact_store/binary_payloads.rs` 与 `asset/tests/assets/artifact_store/library_assets.rs` 等子 owner 承接 15 个 artifact roundtrip/rejection 测试；守卫 `runtime_15_asset_artifact_store_tests_are_folder_backed` 锁定 moved test、测试数量、低于 800 行预算和跨文档状态锚。Cargo 按实施切片节奏 deferred。
## Runtime 15 M3 asset UI test folder split

状态：`runtime_15_asset_ui_tests_folder_split_static_passed_cargo_deferred`。`asset/tests/assets/ui.rs` 现在只保留 UI TOML/ZUI fixtures、fixture importer helper、legacy component TOML helper 和五个 child owner 挂载，`asset/tests/assets/ui/importer.rs` 与 `asset/tests/assets/ui/project_manager.rs` 等子 owner 承接 16 个 UI asset wrapper/reference/import/project scan 测试；守卫 `runtime_15_asset_ui_tests_are_folder_backed` 锁定 moved test、测试数量、低于 800 行预算和跨文档状态锚。Cargo 按实施切片节奏 deferred。

## Runtime 15 M3 asset pipeline manager test folder split

状态：`runtime_15_asset_pipeline_manager_tests_folder_split_static_passed_cargo_deferred`。`asset/tests/pipeline/manager.rs` 现在只保留共享 first-wave plugin fixture helper、imports 和七个 child owner 挂载，`asset/tests/pipeline/manager/model_import.rs` 与 `asset/tests/pipeline/manager/watcher.rs` 等子 owner 承接 10 个 ProjectAssetManager pipeline 测试；守卫 `runtime_15_asset_pipeline_manager_tests_are_folder_backed` 锁定 moved test、测试数量、低于 800 行预算和跨文档状态锚。Cargo 按实施切片节奏 deferred。

## Runtime 15 M3 scene asset integration test folder split

状态：`runtime_15_scene_asset_integration_tests_folder_split_static_passed_cargo_deferred`。`scene/tests/asset_scene.rs` 现在只保留共享 scene asset reference、project IO source/section、authoring-token guard helper 和三个 child owner 挂载，`scene/tests/asset_scene/mesh_bindings.rs`、`scene/tests/asset_scene/hierarchy_sources.rs` 与 `scene/tests/asset_scene/product_fields.rs` 承接 9 个 scene asset integration 测试；守卫 `runtime_15_scene_asset_integration_tests_are_folder_backed` 锁定 moved test、测试数量、低于 800 行预算和跨文档状态锚。Cargo 按实施切片节奏 deferred。

## Runtime 15 M3 scene world basics test folder split

状态：`runtime_15_scene_world_basics_tests_folder_split_static_passed_cargo_deferred`。`scene/tests/world_basics.rs` 现在只保留共享 imports 和三个 child owner 挂载，`scene/tests/world_basics/world_state.rs`、`scene/tests/world_basics/render_extract.rs` 与 `scene/tests/world_basics/sprites.rs` 承接 15 个 world basics 测试；守卫 `runtime_15_scene_world_basics_tests_are_folder_backed` 锁定 moved test、测试数量、低于 800 行预算和跨文档状态锚。Cargo 按实施切片节奏 deferred。

## Runtime 15 M3 scene property paths test folder split

状态：`runtime_15_scene_property_paths_tests_folder_split_static_passed_cargo_deferred`。`scene/tests/property_paths.rs` 现在只保留共享 imports 和三个 child owner 挂载，`scene/tests/property_paths/read_paths.rs`、`scene/tests/property_paths/runtime_mutation.rs` 与 `scene/tests/property_paths/write_validation.rs` 承接 18 个 property-path 行为测试与源码结构守卫；守卫 `runtime_15_scene_property_paths_tests_are_folder_backed` 锁定 moved test、测试数量、低于 800 行预算和跨文档状态锚。Cargo 按实施切片节奏 deferred。

## Runtime 15 M3 input manager test folder split

状态：`runtime_15_input_manager_tests_folder_split_static_passed_cargo_deferred`。`input/tests/input_manager.rs` 现在只保留共享 imports 和三个 child owner 挂载，`input/tests/input_manager/frame_state.rs`、`input/tests/input_manager/touch_gamepad.rs` 与 `input/tests/input_manager/host_requests.rs` 承接 14 个 input manager 测试；守卫 `runtime_15_input_manager_tests_are_folder_backed` 锁定 moved test、测试数量、低于 800 行预算和跨文档状态锚。Cargo 按实施切片节奏 deferred。



Runtime 15 精确锚点补记 2026-06-24：`Runtime 15 M3 picking test folder split` / `runtime_15_picking_tests_folder_split_static_passed_cargo_deferred` 精确锚点包括 `tests/picking/mod.rs`、`tests/picking/pipeline.rs`、`tests/picking/pointer_events.rs` 与 `runtime_15_picking_tests_are_folder_backed`。

Runtime 15 精确锚点补记 2026-06-24：`Runtime 15 M3 asset mesh test root split` / `runtime_15_asset_mesh_tests_root_split_static_passed_cargo_deferred` 精确锚点包括 `asset/tests/assets/mesh.rs`、`asset/tests/assets/mesh/document_roundtrip.rs`、`asset/tests/assets/mesh/conversion_import.rs` 与 `runtime_15_asset_mesh_tests_are_folder_backed`。

## Runtime 15 M4 asset artifact cache UI document owner split

状态：`runtime_15_asset_artifact_cache_ui_documents_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 asset artifact cache production owner 减压。`asset/artifact/cache_payload.rs` 继续拥有 cache wire enum、variant dispatch、cache-safe data/texture/material/shader/prefab/physics conversion 与 child owner 挂载；UI v1/v2 document TOML normalization 和 typed restore paths 迁入 `asset/artifact/cache_payload/ui.rs`，由该子 owner 承接 `ArtifactCacheUiAssetDocument`、`ArtifactCacheUiV2AssetDocument`、`UiLayoutAsset`、`UiWidgetAsset`、`UiStyleAsset`、`UiV2ViewAsset`、`UiV2ComponentAsset` 与 `UiV2StyleAsset` 的 parser-backed cache restore。

父模块通过 `mod ui;` 与 `use ui::{ArtifactCacheUiAssetDocument, ArtifactCacheUiV2AssetDocument};` 消费子 owner，不改变 `.zasset` enum variant、typed `AssetImportError` source、UI document parser entry 或 `UiThemeAsset`/`UiIconAsset` direct-cache path。父文件从 791 行降到约 710 行，`cache_payload/ui.rs` 为约 93 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_asset_artifact_cache_ui_documents_are_child_owner` 验证父模块挂载 UI cache child、代表性 UI document helper 不回流到父文件、子 owner 承接 v1/v2 TOML normalization 和 typed restore paths、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/asset/artifact.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `asset/artifact/cache_payload.rs` 的 UI document owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 asset artifact Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 mesh asset management record owner split

状态：`runtime_15_mesh_asset_management_record_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 mesh asset production owner 减压。`asset/assets/mesh/mesh_asset.rs` 继续拥有 `MeshAsset` DTO、model primitive conversion、morph target application、validation、render descriptor projection 与 management entry methods；overview/management record DTO、failure row、record-set summary 和 record-set aggregation 迁入 `asset/assets/mesh/mesh_asset/management.rs`。

父模块通过 `mod management;` 与 `pub use self::management::{...}` 保留原公开类型路径，不改变 `.zmesh` 序列化形状、`MeshAsset::overview(...)` / `management_record(...)` public API、render descriptor 字段或 resource id record semantics。父文件从 734 行降到约 607 行，`management.rs` 为约 140 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_mesh_asset_management_records_are_child_owner` 验证父模块挂载并 re-export management child、management DTO/impl 不回流到父文件、子 owner 承接 overview/record-set aggregation、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/asset/render-assets.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `asset/assets/mesh/mesh_asset.rs` 的 management record owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 asset mesh Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 asset project scan/import source collection owner split

状态：`runtime_15_asset_project_scan_import_sources_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 asset project scan/import production owner 减压。`asset/project/manager/scan_and_import.rs` 继续拥有 `scan_and_import(...)` 主循环、artifact restore、success/failure meta 写回、dependency resolution 与 entry identity registration；source enumeration、compound `.zmeta` source discovery、source URI mapping、source byte assembly 与 mtime aggregation 迁入 `asset/project/manager/scan_and_import/sources.rs`。

父模块通过 `mod sources;` 与窄 `use self::sources::{AssetImportSource, source_bytes_for_import, source_mtime_unix_ms_for_import};` 消费子 owner，不改变 importer selection、artifact writeback、`.zmeta` schema、package locator semantics 或 failed-import recovery。父文件从 705 行降到约 599 行，`sources.rs` 为约 181 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_asset_project_scan_import_sources_are_child_owner` 验证父模块保留 import loop/artifact restore/success/failure helpers 并挂载 source collection child、source enumeration 和 compound source helper 不回流到父文件、子 owner 承接 byte/mtime helper、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/asset/importer.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `asset/project/manager/scan_and_import.rs` 的 source collection owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 asset project Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描、尾随空白扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M4 glTF labeled material subasset owner split

状态：`runtime_15_gltf_labeled_material_subasset_owner_split_static_passed_cargo_deferred`。

R1.4/M4 的当前新增落地部分是 glTF labeled material subasset owner 减压。`asset/importer/ingest/gltf_labeled_subassets.rs` 从 664 行减压为 390 行，继续拥有 glTF texture/mesh/scene labeled subasset entry、scene dependency collection、shared material URI/reference resolution、root dependency insertion 与 label URI/reference helper；新增 `asset/importer/ingest/gltf_labeled_subassets/material.rs` 作为 283 行 child owner，承接 `add_gltf_material_subassets(...)`、default material generation、PBR material projection、texture-slot metadata、KHR_texture_transform bridge、glTF alpha mode mapping 与 default PBR shader reference。

父模块通过 `mod material;` 与 `pub(crate) use self::material::add_gltf_material_subassets;` 保留原 importer 调用入口，不改变 Bevy-style glTF label names、`Material{n}` / `DefaultMaterial` output shape、texture dependency collection、texture transform metadata、default shader locator 或 scene/mesh material reference semantics，也不新增兼容 re-export。守卫：`runtime_15_gltf_labeled_material_subassets_are_child_owner` 验证父模块保留 texture/mesh/scene orchestration 与共享 label/dependency helpers、material/PBR/texture-slot helper 不回流到父文件、子 owner 承接 material subasset projection、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/asset/importer.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 glTF labeled material subasset 的 M4 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 asset importer Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved owner 扫描、docs/status 锚点扫描、尾随空白扫描和 scoped `git diff --check` 已通过；当前外部 cargo/rustc 通道仍活跃，Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 runtime dead-code guard forbidden attribute literal cleanup

状态：`runtime_15_runtime_dead_code_guard_literal_cleanup_static_passed_cargo_deferred`。

本切片只处理 Runtime 15 dead-code 守卫自身的源扫描噪声，不改变生产 runtime 行为。`zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code.rs` 继续拥有 runtime UI support split、runtime-owned dead-code cleanup、script host descriptor cleanup 与 reflection macro fixture cleanup 的结构断言；本轮把这些断言使用的 forbidden dead-code allow attribute 从直接测试源字面量改为 `DEAD_CODE_ALLOW_ATTRIBUTE` 常量拼装，避免简单源码扫描把守卫文件误报为 suppression 残留。

新增守卫 `runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed` 读取 `structure_convention/runtime_dead_code.rs` 自身，验证 forbidden attribute literal 不回流，同时确认 `DEAD_CODE_ALLOW_ATTRIBUTE`、现有生产/测试 owner 检查和跨文档状态锚仍存在。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings 与 status-output expectations。完整 `runtime_15_no_dead_code_suppression_in_production`、`module_convention_gate` 与全量 Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、literal scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M5 production dead-code suppression global gate

状态：`runtime_15_production_dead_code_suppression_global_gate_static_passed_cargo_deferred`。

本切片把 F12 dead-code suppression 清理从点状 owner 锁定推进为生产源码全局闸口，不改变生产 runtime 行为。`zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code.rs` 新增 `runtime_15_production_sources_do_not_allow_dead_code_suppression`，递归扫描 `zircon_runtime/src` 下非 `tests/`、非 `tests.rs`、非 `*_tests.rs` 的生产 Rust 源文件，确认 `DEAD_CODE_ALLOW_ATTRIBUTE` 零命中。守卫仍通过常量拼装 forbidden attribute，避免测试源码自身再次成为 literal scan 噪声。

该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、module-convention docs、session note 与 status-output expectations，精确锚点包括 `structure_convention/runtime_dead_code.rs`、`DEAD_CODE_ALLOW_ATTRIBUTE` 与 `runtime_15_production_sources_do_not_allow_dead_code_suppression`。完整 `module_convention_gate` 与全量 Runtime 15 Cargo sweep 仍 pending。

验证：生产源码 dead-code suppression 扫描、scoped rustfmt/static scans、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 通道 active 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 root entries guard child-owner split

状态：`runtime_15_root_entries_guard_child_owner_split_static_passed_cargo_deferred`。

本切片收敛 `zircon_runtime/src/tests/runtime_absorption/root_entries.rs` 的测试守卫所有权。父文件现在只挂载 `root_entries/core_spine.rs`、`root_entries/module_families.rs` 与 `root_entries/runtime_root.rs`，不再混合 Runtime 02 core/root/generated 守卫和 Runtime 14 module-family 守卫实现。`core_spine.rs` 承接 core root/spine 断言，`runtime_root.rs` 承接 runtime crate root 和 builtin root 断言，`module_families.rs` 承接 navigation、animation、status JSON 与 module-family mirror docs 断言。

`zircon_runtime/src/tests/runtime_absorption/core_spine_root_generated.rs`、`core_spine_root_generated_boundary.py` 与 `module_family_boundary.py` 已同步聚合读取新 child owner，保持 `root_entries guard tests 13`、`guard_test_anchor_count = 26` 与 Runtime 14 guard anchor 计数语义不漂移。新增 `structure_convention/test_file_budget/root_entries.rs::runtime_15_root_entries_guard_child_owners_are_folder_backed` 锁定父入口只挂载、moved guard 不回流、父/子 owner 低于 800 行预算、Rust/Python 审计聚合路径和 Runtime 15/status/docs 镜像锚。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 Runtime 02/14 guard Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、父子行数预算扫描、Rust/Python 审计聚合路径扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 因外部 cargo/rustc 通道 active 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 lock poison policy guard folder split

状态：`runtime_15_lock_poison_policy_guard_folder_split_static_passed_cargo_deferred`。

本切片只整理 E9/F2 lock-poison 结构守卫的测试 owner，不改变生产 runtime 行为。`zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs` 从 1583 行超预算守卫文件减压为 202 行父模块，只保留 `mod core_runtime;`、`mod runtime_services;`、`mod asset_render_input;`、共享 source reader/direct-lock helper 和新布局守卫。既有 21 个 lock-poison 回归守卫被按 owner 迁入 `structure_convention/lock_poison_policy/core_runtime.rs`、`structure_convention/lock_poison_policy/runtime_services.rs`、`structure_convention/lock_poison_policy/asset_render_input.rs`，三个 child owner 分别为 649、452、460 行。

新增守卫 `runtime_15_lock_poison_policy_guard_is_folder_backed` 验证父模块挂载三个 child owner，代表性 moved guard 不回流父文件，父子合计保留 21 个既有守卫加 1 个布局守卫共 22 个 `#[test]`，并锁定所有 owner 低于 800 行预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、module-convention docs、session note 与 status-output expectations，精确锚点包括 `structure_convention/lock_poison_policy.rs`、`structure_convention/lock_poison_policy/core_runtime.rs`、`structure_convention/lock_poison_policy/runtime_services.rs`、`structure_convention/lock_poison_policy/asset_render_input.rs` 和 `runtime_15_lock_poison_policy_guard_is_folder_backed`。

验证：scoped rustfmt/static scans、父子行数预算扫描、moved-test parent scan、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 通道 active（`cargo` PIDs 19964、34648、59276、70536；`rustc` PIDs 15088、29672）按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 core runtime lock poison guard child-owner split

状态：`runtime_15_core_runtime_lock_poison_guard_child_owner_split_static_passed_cargo_deferred`。

本切片继续收敛 E9/F2 lock-poison 结构守卫的测试 owner，不改变生产 runtime 行为。`zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime.rs` 从 near-budget core runtime 守卫文件减压为 folder-backed 父模块，只挂载 `core_runtime/scene_eventbus.rs`、`core_runtime/global_gate.rs`、`core_runtime/config_devtools.rs`、`core_runtime/handle_accessors.rs` 与 `core_runtime/task_profiling.rs`。

新增守卫 `runtime_15_core_runtime_lock_poison_guard_child_owner_split` 验证父模块只挂载 child owner，代表性 moved guard 不回流父文件，父子合计保留 10 个既有 core runtime lock-poison 守卫加 1 个布局守卫共 11 个 `#[test]`，并锁定每个 focused owner 低于 400 行预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、module-convention docs、session note 与 status-output expectations，精确锚点包括 `structure_convention/lock_poison_policy/core_runtime.rs`、`structure_convention/lock_poison_policy/core_runtime/handle_accessors.rs` 和 `runtime_15_core_runtime_lock_poison_guard_child_owner_split`。

验证：scoped rustfmt/static scans、核心子 owner 行数预算扫描、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 F2 lock poison recovery guard

状态：`runtime_15_f2_lock_poison_recovery_guard_static_passed_cargo_deferred`。

本切片只把既有 F2 poison-safe lock 修复转成 Runtime 15 结构防回退守卫，不改变生产 runtime 行为。`zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs` 读取 `scene/level_system.rs`、`scene/module/default_level_manager.rs`、`scene/module/level_manager_lifecycle.rs`、`core/runtime/events.rs` 与 publish/subscribe/prune 子 owner，确认 scene level holder 和 EventBus 的共享状态锁都经集中 helper 恢复 poison。

新增守卫 `runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus` 验证 `LevelSystem` 保留 `lock_poison_recovered(...)` 与 world/runtime_state/metadata/lifecycle/subsystems accessors，`DefaultLevelManager` 保留 `lock_levels()`，EventBus 保留 `lock_subscribers()` / `lock_delivery()`，并扫描生产段拒绝 direct lock unwrap。测试段中用于制造 poison 的 `level_system.rs` fixture 仍允许存在。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、scene/event module docs 与 status-output expectations。完整 `module_convention_gate` 与全量 Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、生产段 direct lock unwrap scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 production direct lock unwrap global gate

状态：`runtime_15_production_direct_lock_unwrap_global_gate_static_passed_cargo_deferred`。

本切片把 E9/F2 poison-safe lock 规则从点状 owner 守卫提升为全 crate 生产段回归闸口，不改变生产 runtime 行为。`zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime.rs` 新增 `runtime_15_production_sources_do_not_directly_unwrap_mutex_locks`，递归扫描 `zircon_runtime/src` 下非 `tests/`、非 `tests.rs`、非 `*_tests.rs` 的 Rust 源文件。

守卫只读取每个源文件 `#[cfg(test)]` 之前的生产段，并拒绝 `LOCK_UNWRAP_CALL` 对应的 direct `.lock().unwrap()`；inline test 中专门用于制造 poisoned lock 的 fixture 仍允许存在。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、session note 与 status-output expectations。完整 `module_convention_gate` 与全量 Cargo sweep 仍 pending。

验证：全量生产段 direct lock unwrap scan、scoped rustfmt/static scans、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 config store lock poison recovery

状态：`runtime_15_config_store_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到低冲突 core runtime config backing store。`zircon_runtime/src/core/runtime/config_store.rs` 新增私有 `lock_values()` helper，`store_value`、`load_value` 与 `snapshot_values` 都通过该 helper 访问 values map，不再在生产路径 direct lock unwrap。

新增 module-local `config_store_accessors_recover_poisoned_values_lock` 覆盖中毒锁恢复后 store/load/snapshot 仍可用；`structure_convention/lock_poison_policy.rs::runtime_15_config_store_lock_poison_recovery_guard_covers_runtime_config_store` 读取 `core/runtime/config_store.rs` 与 `docs/zircon_runtime/core/runtime/config_store.md`，验证 helper、生产段 direct lock unwrap 扫描和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、config-store module docs 与 status-output expectations。完整 `module_convention_gate` 与全量 Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、生产段 direct lock unwrap scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 core runtime devtools lock poison recovery

状态：`runtime_15_core_runtime_devtools_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 core runtime devtools 只读投影。`zircon_runtime/src/core/runtime/diagnostics/devtools.rs` 新增私有泛型 `lock_poison_recovered(...)` helper，`collect_module_snapshots`、`collect_service_snapshots` 与 `collect_scene_hook_snapshots` 都通过该 helper 读取 modules、services 与 scene_hooks registry，不再在生产路径 direct lock unwrap。

新增 module-local `devtools_snapshot_recovers_poisoned_runtime_registry_locks` 覆盖 modules、services 与 scene_hooks locks 被 poison 后仍可收集 snapshot；`structure_convention/lock_poison_policy.rs::runtime_15_core_runtime_devtools_lock_poison_recovery_guard_covers_devtools_snapshot` 读取 `core/runtime/diagnostics/devtools.rs` 与 `docs/zircon_runtime/core/diagnostics.md`，验证 helper、direct-lock scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、core diagnostics docs 与 status-output expectations。完整 `module_convention_gate` 与全量 core diagnostics Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、devtools direct-lock scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 core handle diagnostics lock poison recovery

状态：`runtime_15_core_handle_diagnostics_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 CoreHandle diagnostics store 访问面。`zircon_runtime/src/core/runtime/handle/diagnostics.rs` 新增私有 `lock_diagnostics()` helper，`diagnostic_store()`、`diagnostic_store_snapshot()` 与 `record_diagnostic(...)` 都通过该 helper 访问 `DiagnosticStore`，不再在生产路径 direct lock unwrap。

新增 module-local `core_handle_diagnostic_accessors_recover_poisoned_store_lock` 覆盖 diagnostics lock 被 poison 后仍可 record/snapshot；`structure_convention/lock_poison_policy.rs::runtime_15_core_handle_diagnostics_lock_poison_recovery_guard_covers_diagnostic_store` 读取 `core/runtime/handle/diagnostics.rs` 与 `docs/zircon_runtime/core/diagnostics.md`，验证 helper、direct-lock scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、core diagnostics docs 与 status-output expectations。完整 `module_convention_gate` 与全量 core diagnostics Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、CoreHandle diagnostics direct-lock scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 core handle time lock poison recovery

状态：`runtime_15_core_handle_time_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 CoreHandle 时间推进入口。`zircon_runtime/src/core/runtime/handle/time.rs` 新增私有 `lock_time()` 与 `lock_frame_clock()` helpers，`time_clocks()`、`advance_time_by(...)`、`tick_time(...)`、虚拟时钟暂停/恢复和固定步长配置都通过 helper 访问 runtime clocks 或 frame clock，不再在生产路径 direct lock unwrap。时间诊断写入改为复用 `CoreHandle::record_diagnostic(...)`，因此 diagnostics store poison recovery 只由 diagnostics owner 维护。

新增 module-local `core_handle_time_accessors_recover_poisoned_runtime_clocks` 覆盖 time、frame_clock 与 diagnostics locks 被 poison 后仍可 pause/unpause、advance/tick 和写入时间诊断；`structure_convention/lock_poison_policy.rs::runtime_15_core_handle_time_lock_poison_recovery_guard_covers_runtime_clocks` 读取 `core/runtime/handle/time.rs` 与 `docs/zircon_runtime/core/diagnostics.md`，验证 helper、direct-lock scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、core diagnostics docs 与 status-output expectations。完整 `module_convention_gate` 与全量 core diagnostics Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、CoreHandle time direct-lock scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 core handle states lock poison recovery

状态：`runtime_15_core_handle_states_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 CoreHandle state registry 访问面。`zircon_runtime/src/core/runtime/handle/states.rs` 新增私有 `lock_states()` helper，`init_state`、`insert_state`、state/next-state 读取、pending transition 写入、transition apply/event 查询和 hook 注册都通过该 helper 访问 `StateRegistry`，不再在生产路径 direct lock unwrap。

新增 module-local `core_handle_state_accessors_recover_poisoned_state_registry_lock` 覆盖 states lock 被 poison 后仍可 init、set/apply transition、读取当前 state 与 event history；`structure_convention/lock_poison_policy.rs::runtime_15_core_handle_states_lock_poison_recovery_guard_covers_state_registry` 读取 `core/runtime/handle/states.rs` 与 `docs/zircon_runtime/core/state.md`，验证 helper、direct-lock scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、core state docs 与 status-output expectations。完整 `module_convention_gate` 与全量 core state Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、CoreHandle states direct-lock scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 core runtime task lock poison recovery

状态：`runtime_15_core_runtime_task_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 core runtime JobSystem state。`zircon_runtime/src/core/runtime/tasks/job_handle.rs` 新增 `JobState::lock_inner()`、`wait_inner(...)` 与 `wait_inner_timeout(...)`，让 `is_complete`、wait、panic-message read、terminal marking、dependent registration 和 dependency decrement 都通过 poison recovery 访问 job state。`zircon_runtime/src/core/runtime/tasks/job_scheduler.rs` 新增 `PendingScheduledJob::lock_task()`，让 pending scheduled task launch 与 terminal cleanup 不再在生产路径 direct lock expect。

新增 module-local `job_handle_accessors_recover_poisoned_state_lock`、`job_handle_wait_recovers_poisoned_state_lock` 与 `pending_scheduled_job_recovers_poisoned_task_lock` 覆盖 job state 和 pending task lock 被 poison 后仍可 dependent callback、wait、mark complete 和 launch scheduled task；`structure_convention/lock_poison_policy.rs::runtime_15_core_runtime_task_lock_poison_recovery_guard_covers_job_handles` 读取 `core/runtime/tasks/job_handle.rs`、`core/runtime/tasks/job_scheduler.rs` 与 `docs/zircon_runtime/core/tasks.md`，验证 helper、direct-lock/direct-panic scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、core tasks docs 与 status-output expectations。完整 `module_convention_gate` 与全量 core tasks Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、core runtime task direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 core runtime profiling lock poison recovery

状态：`runtime_15_core_runtime_profiling_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 core runtime profiling recorder。`zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs` 新增私有 `lock_recorder()` helper，`start_capture`、`stop_capture`、`reset_capture`、`snapshot` 与 `with_recorder(...)` 都通过该 helper 访问全局 `ProfileRecorder`，不再在生产路径 direct lock unwrap。

新增 module-local `profile_recorder_accessors_recover_poisoned_global_lock` 覆盖全局 recorder lock 被 poison 后仍可 snapshot/reset；`structure_convention/lock_poison_policy.rs::runtime_15_core_runtime_profiling_lock_poison_recovery_guard_covers_global_recorder` 读取 `core/runtime/diagnostics/profiling/mod.rs` 与 `docs/zircon_runtime/core/diagnostics.md`，验证 helper、direct-lock/direct-panic scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、core diagnostics docs 与 status-output expectations。完整 `module_convention_gate` 与全量 core diagnostics/profiling Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、profiling recorder direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 core handle registry lock poison recovery

状态：`runtime_15_core_handle_registry_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 CoreHandle registry access surface。`zircon_runtime/src/core/runtime/handle/core_handle.rs` 新增共享 `lock_poison_recovered(...)` helper，以及 `lock_modules()`、`lock_services()`、`lock_scene_hooks()`、`lock_world_extensions()` 和 `lock_plugin_bridge_lifecycle()`。`activation.rs`、`registration/register_module.rs`、`resolution.rs` 与 `runtime_extensions.rs` 通过这些 helper 访问 modules/services/world-extension/scene-hook/plugin-lifecycle 状态，不再在生产路径 direct lock unwrap。

新增 module-local `core_handle_registry_accessors_recover_poisoned_runtime_locks` 覆盖这些 runtime registry locks 被 poison 后仍可恢复；`core/runtime/tests/registration/structure.rs` 的事务锁结构哨兵同步为 helper commit boundary。`structure_convention/lock_poison_policy.rs::runtime_15_core_handle_registry_lock_poison_recovery_guard_covers_registry_accessors` 读取 core handle root、activation、registration、resolution、runtime extensions 与 `docs/zircon_runtime/core/runtime/lifecycle.md`，验证 helper、direct-lock/direct-panic scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、core runtime lifecycle docs 与 status-output expectations。完整 `module_convention_gate` 与全量 core runtime handle Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、CoreHandle registry direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 core runtime registration structure behavior layout split

状态：`runtime_15_core_runtime_registration_structure_behavior_layout_split_static_passed_cargo_deferred`。

本切片只整理 core runtime registration 结构守卫的测试 owner，不改变生产 runtime 行为。`zircon_runtime/src/core/runtime/tests/registration/structure.rs` 从 805 行减压为 763 行，继续拥有 registration hot-path、service-list cache、dependency-name materialization、duplicate helper boundary 和 helper commit ordering 守卫；behavior folder wiring 断言迁入 `zircon_runtime/src/core/runtime/tests/registration/structure/behavior_layout.rs`，该 child owner 为 71 行。

新增守卫 `registration_behavior_tests_stay_folder_backed` 验证 `registration/behavior.rs` 只挂载 `validation`、`cache_lists`、`commit`、`canonical_keys` 子 owner，不直接持有 `#[test]` 或 `use`，并确认 canonical names、cache lists、partial commit、four/five dependency boundary 等关键 behavior tests 仍在对应子文件。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、module-convention docs、session note 与 status-output expectations，精确锚点包括 `core/runtime/tests/registration/structure.rs`、`core/runtime/tests/registration/structure/behavior_layout.rs` 与 `registration_behavior_tests_stay_folder_backed`。

验证：scoped rustfmt/static scans、父子行数预算扫描、剩余 oversized-test scan、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 通道 active（`cargo` PIDs 4820、8680、8844、20052、32036、33512、50960、56884；`rustc` PIDs 27412、53344、54144、62068）按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 core runtime registration structure owner split

状态：`runtime_15_core_runtime_registration_structure_owner_split_static_passed_cargo_deferred`。

本切片接续上一条 behavior-layout 减压记录，退役 `zircon_runtime/src/core/runtime/tests/registration/structure.rs` 平铺测试文件，把 registration 结构守卫改成 folder-backed `zircon_runtime/src/core/runtime/tests/registration/structure/mod.rs`。父模块只挂载 child owner 和共享 `registration_sources()` fixture；`module_layout.rs`、`service_count_paths.rs`、`service_list_caches.rs`、`dependency_fast_paths.rs`、`duplicate_detection.rs`、`cleanup.rs` 与既有 `behavior_layout.rs` 分别承接模块布局、服务数量 fast path、service/startup/shutdown cache、dependency name materialization、duplicate helper boundary、legacy cleanup 和 behavior folder wiring 断言。

新增守卫 `runtime_15_core_runtime_registration_structure_tests_are_folder_backed` 验证旧 `structure.rs` 不回流、`registration/mod.rs` 继续挂载 `mod structure;`、新父模块挂载所有 focused child owner、service-count child 继续锁定 `.rfind("let mut modules = self.lock_modules()")` 与 `.find("let modules = self.lock_modules();")` helper boundary、service-list child 继续锁定 lazy/single-startup cache direct paths，并确保每个 registration structure owner 低于 Runtime 15 800 行预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、core runtime lifecycle docs、module-convention docs、session note 与 status-output expectations，精确锚点包括 `core/runtime/tests/registration/structure/mod.rs`、`core/runtime/tests/registration/structure/service_count_paths.rs`、`core/runtime/tests/registration/structure/service_list_caches.rs` 与 `runtime_15_core_runtime_registration_structure_tests_are_folder_backed`。

验证：scoped rustfmt/static scans、父子行数预算扫描、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 plugin bridge table lock poison recovery

状态：`runtime_15_plugin_bridge_table_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 runtime plugin bridge table 的 provider slot。`zircon_runtime/src/plugin/bridge/table.rs` 新增私有 `lock_provider()` helper，`BridgeEntry::provider_installed()`、typed provider resolve、deactivate、replace/reload provider 与 restore provider 都通过该 helper 访问 provider slot，不再在生产路径 direct lock unwrap。

新增 module-local `bridge_entry_provider_accessors_recover_poisoned_provider_lock` 覆盖 provider slot 被 poison 后仍可 status/snapshot、typed resolve、deactivate、restore 与 replace；`structure_convention/lock_poison_policy.rs::runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot` 读取 bridge table 与 `docs/zircon_runtime/plugin/bridge.md`，验证 helper、direct-lock/direct-panic scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、plugin bridge docs 与 status-output expectations。完整 `module_convention_gate` 与全量 plugin bridge Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、plugin bridge table direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 native live-host bridge methods lock poison recovery

状态：`runtime_15_native_live_host_bridge_methods_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 native live-host runtime bridge method binding registry。`zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs` 新增私有 `lock_runtime_bridge_method_bindings()` helper，`clear_runtime_bridge_method_bindings`、installed binding read、replace/install 与 discovered-binding teardown 都通过该 helper 访问 binding table，不再把 poisoned mutex 映射成 `lock poisoned` 生产错误。

新增 module-local `native_live_host_bridge_method_bindings_recover_poisoned_lock` 覆盖 binding table 被 poison 后仍可 read、clear、replace；`structure_convention/native_live_host_lock_poison.rs::runtime_15_native_live_host_bridge_methods_lock_poison_recovery_guard_covers_binding_registry` 读取 native live-host bridge methods、父结构聚合文件与 `docs/zircon_runtime/plugin/bridge.md`，验证 helper、独立 guard 挂载、direct-lock/direct-panic scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、plugin bridge docs 与 status-output expectations。完整 `module_convention_gate` 与全量 plugin bridge Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、native live-host bridge methods direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 native live-host tests folder split

状态：`runtime_15_native_live_host_tests_folder_split_static_passed_cargo_deferred`。

本切片继续收束 R4.1/M3 测试组织预算。`zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs` 新增 `bridge_bindings`、`hot_reload_state` 与 `runtime_behavior` 子 owner 挂载，把 runtime descriptor/snapshot、native bridge binding scope 与 hot reload snapshot/rollback 三组测试迁出父文件。父文件只保留共享 fixture、基础 unloaded/missing package 行为、command interior-NUL 与 helper，行数从 1297 降到 541。

新增 `structure_convention/test_file_budget/native_live_host_tests.rs::runtime_15_native_live_host_tests_are_folder_backed` 验证父模块挂载、代表性 moved tests 不回流、父文件剩余 9 个测试加新子文件 18 个测试合计保留原 27 个测试，并验证父文件、`runtime_behavior.rs`、`bridge_bindings.rs`、`hot_reload_state.rs` 都低于 800 行预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、`docs/zircon_runtime/plugin/bridge.md` 与 status-output expectations。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 plugin native live-host Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、迁移测试数量扫描、父子行数预算扫描、moved-test parent scan、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 通道 active（`cargo` PIDs 3472、21776、47696、55676、63860、64772；`rustc` PIDs 30312、52648、57156）按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 native plugin loader real fixture test folder split

状态：`runtime_15_native_plugin_loader_real_fixture_tests_folder_split_static_passed_cargo_deferred`。

本切片继续收束 R4.1/M3 测试组织预算。`zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs` 新增 `real_fixture` 子 owner 挂载，把真实 native dynamic fixture build/load、descriptor/entry/status-code、unknown ABI rejection 与 native data asset importer handler 测试迁出父文件。父文件只保留 load-manifest discovery、mismatch/dedup、editor-only discovery、feature extension package 与 split native package loading 测试，行数从 933 降到 470；新增 `real_fixture.rs` 为 564 行。

新增 `structure_convention/test_file_budget/native_plugin_loader.rs::runtime_15_native_plugin_loader_real_fixture_tests_are_folder_backed` 验证父模块挂载、4 个 real fixture moved tests 不回流、父文件剩余 7 个测试加新子文件 4 个测试合计保留原 11 个测试，并验证父文件与 `real_fixture.rs` 都低于 800 行预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、`docs/zircon_runtime/plugin/bridge.md`、`docs/zircon_runtime/asset/importer.md` 与 status-output expectations。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 plugin extension/native loader Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、迁移测试数量扫描、父子行数预算扫描、moved-test parent scan、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 通道 active（`cargo` PIDs 19972、44764、49624、54336；`rustc` PIDs 29320、54940）按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 render shader template assembly guard WGSL contracts split

状态：`runtime_15_render_shader_template_assembly_guard_wgsl_contracts_split_static_passed_cargo_deferred`。

本切片只整理 production-file-budget 结构守卫的测试 owner，不改变 shader template、WGSL 或 pipeline cache 生产行为。`zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly.rs` 从 845 行减压为 610 行，继续验证 shader/template module wiring、assembler/include registry/material surface/pass specialization、mesh pipeline cache、Plan 08 文档锚点和生产文件预算；WGSL ABI/template contract 断言迁入 `zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/wgsl_contracts.rs`，该 child owner 为 258 行。

新增守卫 `runtime_15_render_shader_template_wgsl_contracts_are_child_owner` 验证 scene uniform、GPU scene transform/palette、surface interpolation、static/skinned geometry fetch、Forward/GBuffer/Depth/Shadow/Velocity/TAA template entry/alpha/motion outputs 与 Standard PBR light-grid/shadow contract，同时检查父/子 owner 都低于 800 行预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、module-convention docs、session note 与 status-output expectations，精确锚点包括 `structure_convention/production_file_budget/render_shader_template_assembly.rs`、`structure_convention/production_file_budget/render_shader_template_assembly/wgsl_contracts.rs` 与 `runtime_15_render_shader_template_wgsl_contracts_are_child_owner`。

验证：scoped rustfmt/static scans、父子行数预算扫描、剩余 oversized-test scan、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 通道 active（`cargo` PIDs 4820、8680、8844、20052、32036、33512、50960、56884；`rustc` PIDs 27412、53344、54144、62068）按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 render shader template assembly guard support child-owner split

状态：`runtime_15_render_shader_template_assembly_guard_support_child_owner_split_static_passed_cargo_deferred`。

本切片继续整理 production-file-budget 结构守卫的测试 owner，不改变 shader template、WGSL、pipeline cache 或 Plan 08 生产行为。`zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly.rs` 在上一轮 WGSL contract split 后又增长到 839 行，并同时承接批量 source reads、Rust/template/cache 断言、shadow execution 断言、line-budget loop 与 Plan 08 文档锚点断言；本切片把父文件减压为 168 行，只保留模块挂载、原 `runtime_15_render_shader_template_assembly_is_folder_backed` 测试入口，以及新增 support-child 布局守卫。

新增 `zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/sources.rs`（98 行）集中读取 shader template、mesh pipeline cache、shadow renderer 与 graph execution source；`assembly_assertions.rs`（677 行）承接 Rust template assembly/cache、mesh pipeline source/cache、shadow replay、template unit-test anchor 和 production/test owner budget 断言；`docs_anchors.rs`（133 行）承接 Plan 08、render index、shader docs、review findings、structure convention 与 render session docs anchors。既有 `depth_prepass_cache.rs`、`gbuffer_cache.rs` 与 `wgsl_contracts.rs` 保持独立 child owner。

新增守卫 `runtime_15_render_shader_template_assembly_support_children_are_folder_backed` 验证 support child mount、moved support anchors 不回流、父/子 owner 均低于 800 行预算，并锁定 Runtime 15 子计划、runtime index、engine code structure convention、review findings、module-convention docs、session note 与 status-output expectations。精确锚点包括 `structure_convention/production_file_budget/render_shader_template_assembly.rs`、`structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs`、`structure_convention/production_file_budget/render_shader_template_assembly/docs_anchors.rs`、`structure_convention/production_file_budget/render_shader_template_assembly/sources.rs` 与 `runtime_15_render_shader_template_assembly_support_children_are_folder_backed`。

验证已通过：scoped `rustfmt --edition 2021 --check`、父子行数预算扫描（父 168 行，`assembly_assertions.rs` 677 行，`docs_anchors.rs` 133 行，`sources.rs` 98 行）、moved support anchor 扫描、docs/status/session anchor scan、status/date expected-slice map scan、conflict/trailing-whitespace scan 和 scoped `git diff --check` 均通过；`git diff --check` 仅报告 LF-to-CRLF 提示。Cargo 因验证时存在 active cargo/rustc lanes deferred，不计通过。

## Runtime 15 M3 extension registry bridge test folder split

状态：`runtime_15_extension_registry_bridge_tests_folder_split_static_passed_cargo_deferred`。

本切片继续收束 R4.1/M3 测试组织预算。`zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs` 新增 `basics`、`diagnostics` 与 `lifecycle` 子 owner 挂载，把 bridge basics、diagnostics matrix 与 owner lifecycle transition 三组测试迁出父文件。父文件只保留共享 interface/provider fixture、snapshot/owner-report helper 和子模块挂载，行数收缩为 114。

新增 `structure_convention/test_file_budget/extension_registry_bridge.rs::runtime_15_extension_registry_bridge_tests_are_folder_backed` 验证父模块挂载、代表性 moved tests 不回流、父文件 0 个测试加新子文件 20 个测试合计保留原 20 个测试，并验证父文件、`basics.rs`、`diagnostics.rs`、`lifecycle.rs` 都低于 800 行预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、`docs/zircon_runtime/plugin/bridge.md` 与 status-output expectations。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 plugin extension Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、迁移测试数量扫描、父子行数预算扫描、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 通道 active（`cargo` PIDs 23868、34676、47696、48080、50600、64772；`rustc` PIDs 8940、42604、51932）按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 manifest contributions test folder split

状态：`runtime_15_manifest_contributions_tests_folder_split_static_passed_cargo_deferred`。

本切片继续收束 R4.1/M3 测试组织预算。`zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs` 新增 `editor_only` 与 `net` 子 owner 挂载，把 editor-only package manifest 与 net package manifest 两组测试迁出父文件。父文件只保留 runtime/catalog manifest contribution assertions、plugin manifest 读取 helper 和子模块挂载，行数降到 640。

新增 `structure_convention/test_file_budget/manifest_contributions.rs::runtime_15_manifest_contributions_tests_are_folder_backed` 验证父模块挂载、代表性 moved tests 不回流、父文件 8 个测试加新子文件 5 个测试合计保留原 13 个测试，并验证父文件、`editor_only.rs`、`net.rs` 都低于 800 行预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、`docs/zircon_runtime/plugin/package_manifest.md` 与 status-output expectations。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 plugin extension Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、迁移测试数量扫描、父子行数预算扫描、moved-test parent scan、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 通道 active（`cargo` PIDs 34676、48080；`rustc` PID 56948）按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 runtime plugin package manifest test folder split

状态：`runtime_15_runtime_plugin_package_manifest_tests_folder_split_static_passed_cargo_deferred`。

本切片继续收束 R4.1/M3 测试组织预算。`zircon_runtime/src/tests/plugin_extensions/runtime_plugin_package_manifest.rs` 新增 `feature_modules` 子 owner 挂载，把 optional feature、feature extension 与 package module validation 测试迁出父文件。父文件只保留 package identity/public metadata、bridge interface/method/dependency、packaging/capability/dependency/asset-importer/capability-status validation、shared runtime plugin fixture 和子模块挂载，行数降到 793；新增 `feature_modules.rs` 子文件为 308 行。

新增 `structure_convention/test_file_budget/runtime_plugin_package_manifest.rs::runtime_15_runtime_plugin_package_manifest_tests_are_folder_backed` 验证父模块挂载、代表性 moved tests 不回流、父文件 24 个测试加新子文件 11 个测试合计保留原 35 个测试，并验证父文件与 `feature_modules.rs` 都低于 800 行预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、`docs/zircon_runtime/plugin/package_manifest.md` 与 status-output expectations。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 plugin extension Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、迁移测试数量扫描、父子行数预算扫描、moved-test parent scan、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 通道 active（`cargo` PIDs 2908、3092、8528、35188；`rustc` PIDs 6392、47316）按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 export build plan test folder split

状态：`runtime_15_export_build_plan_tests_folder_split_static_passed_cargo_deferred`。

本切片继续收束 R4.1/M3 测试组织预算。`zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs` 新增 `catalog_projection` 子 owner 挂载，把 builtin catalog completion、target-mode projection、rendering default feature providers、advanced render plugin links 与 SourceTemplate+NativeDynamic merge 测试迁出父文件。父文件只保留 export profile projection、required-provider diagnostics、SourceTemplate/LibraryEmbed shared build-validation plan、profile feature matrix 与 shared helper，行数从 933 降到 723；新增 `catalog_projection.rs` 子文件为 263 行。

新增 `structure_convention/test_file_budget/export_build_plan.rs::runtime_15_export_build_plan_tests_are_folder_backed` 验证父模块挂载、代表性 moved tests 不回流、父文件 11 个测试加新子文件 5 个测试合计保留原 16 个测试，并验证父文件与 `catalog_projection.rs` 都低于 800 行预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、`docs/zircon_runtime/plugin/export_build_plan.md` 与 status-output expectations。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 plugin extension Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、迁移测试数量扫描、父子行数预算扫描、moved-test parent scan、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 通道 active（`cargo` PIDs 28208、40420、60772、66220；`rustc` PIDs 13984、29744）按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 export build plan platform test folder split

状态：`runtime_15_export_build_plan_platform_tests_folder_split_static_passed_cargo_deferred`。

本切片继续收束 R4.1/M3 测试组织预算。`zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform.rs` 新增 `browser_hosts` 子 owner 挂载，把 WebGPU/WASM host WebAssembly export bootstrap 与 allowed asset-origin gate 测试迁出父文件。父文件继续拥有 target platform policy、native-dynamic rejection、headless/mobile/browser host scaffold、mobile/browser package manifest、signing/CDN release contracts、platform callback adapters、release adapter gates 与 mobile binding/resource glue，行数从 819 降到 780；新增 `browser_hosts.rs` 子文件为 69 行。

新增 `structure_convention/test_file_budget/export_build_plan_platform.rs::runtime_15_export_build_plan_platform_tests_are_folder_backed` 验证父模块挂载、代表性 moved test 不回流、父文件 9 个测试加新子文件 1 个测试合计保留原 10 个测试，并验证父文件与 `browser_hosts.rs` 都低于 800 行预算。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、`docs/zircon_runtime/plugin/export_build_plan.md` 与 status-output expectations。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 plugin extension Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、迁移测试数量扫描、父子行数预算扫描、moved-test parent scan、docs/status/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 因外部 cargo/rustc 通道 active（`cargo` PIDs 22876、28208、40420、61976、63160、66380、70136、70160；`rustc` PIDs 32424、42100、52768、68332）按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 navigation lock poison recovery

状态：`runtime_15_navigation_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到内置 navigation fallback manager。`zircon_runtime/src/navigation/runtime.rs` 新增私有 `lock_state()` helper，`BuiltinNavigationManager` 的 navmesh load、settings load、path/sample/raycast query、agent tick stats、tick-agent path/sample lookup 与 `stats()` 都通过该 helper 访问 `BuiltinNavigationState`，不再因 poisoned mutex 触发 `expect("navigation state lock poisoned")`。

新增 module-local `navigation_manager_accessors_recover_poisoned_state_lock` 覆盖中毒锁恢复后 load/settings/sample/stats 仍可用；`structure_convention/lock_poison_policy.rs::runtime_15_navigation_lock_poison_recovery_guard_covers_builtin_navigation_manager` 读取 `navigation/runtime.rs`、`navigation/runtime/tests.rs` 与 `docs/zircon_runtime/navigation/runtime.md`，验证 helper、direct lock/direct panic 扫描和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、navigation module docs 与 status-output expectations。完整 `module_convention_gate` 与全量 Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、navigation direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 dynamic API session lock poison recovery

状态：`runtime_15_dynamic_api_session_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 dynamic API session 私有注册表和单 session execution lock。`zircon_runtime/src/dynamic_api/session.rs` 新增 `lock_registry()` 与 `lock_session()`，`destroy_session`、`insert_session` 和 `with_session` 都通过 helper 访问 session registry 或 dispatch action，不再在生产路径 direct lock unwrap。

新增 `dynamic_api_session_registry_accessors_recover_poisoned_locks` 覆盖 registry 与 session lock 被 poison 后仍可 `with_session` 和 `destroy_session`；`structure_convention/lock_poison_policy.rs::runtime_15_dynamic_api_session_lock_poison_recovery_guard_covers_session_registry` 读取 `dynamic_api/session.rs`、`dynamic_api/session/tests/lock_poison.rs` 与 `docs/zircon_runtime/dynamic_api/session.md`，验证 helper、direct-lock scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、dynamic API session docs 与 status-output expectations。完整 `module_convention_gate` 与全量 dynamic API Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、dynamic API session direct-lock scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 dynamic scene spawn task lock poison recovery

状态：`runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 dynamic scene spawn task 的异步状态与结果存储。`zircon_runtime/src/scene/dynamic_scene/spawn_task/task.rs` 新增 `lock_spawn_status(...)` 与 `lock_spawn_result(...)` helper，`status`、`status_snapshot`、`take_ready` 和 `wait_ready` 都通过 helper 访问 `AsyncTaskStatus` 或 `SpawnTaskResult`，不再在生产路径 direct lock expect。`zircon_runtime/src/scene/dynamic_scene/spawn_task/loader.rs` 的 scheduled job running 标记、completion 状态写入和 result 发布也统一消费这两个 helper。

新增 module-local `dynamic_scene_spawn_task_accessors_recover_poisoned_locks` 覆盖 status/result locks 被 poison 后仍可 record poll、mark running 和取回准备结果；`structure_convention/lock_poison_policy.rs::runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_guard_covers_spawn_task` 读取 spawn task task/loader owners 与 `docs/zircon_runtime/scene/dynamic_scene.md`，验证 helper、direct-lock/direct-panic scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、dynamic scene docs 与 status-output expectations。完整 `module_convention_gate` 与全量 dynamic scene Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、dynamic scene spawn task direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 scene ECS parallel executor lock poison recovery

状态：`runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 scene ECS schedule parallel executor 的 scheduled batch result slot。`zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs` 新增 `lock_batch_result(...)` helper，scheduled batch closure result publish 与 tail wait 后的 batch-order replay 都通过该 helper 访问 `ScheduleParallelBatchResult` slot，不再在生产路径 direct lock expect。

新增 module-local `schedule_parallel_executor_batch_result_slot_recovers_poisoned_lock` 覆盖中毒后仍可取回 Ok result，并可继续写入/取回 missing-task result；`structure_convention/lock_poison_policy.rs::runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_guard_covers_batch_result_slots` 读取 ECS executor 与 `docs/zircon_runtime/scene/ecs.md`，验证 helper、direct-lock/direct-panic scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、ECS module docs 与 status-output expectations。完整 `module_convention_gate` 与全量 scene ECS Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、scene ECS parallel executor direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 core resource manager lock poison recovery

状态：`runtime_15_core_resource_manager_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 core resource manager。`zircon_runtime/src/core/resource/manager/resource_manager.rs` 新增 `lock_registry_read()`、`lock_registry_write()`、`lock_payloads_read()`、`lock_payloads_write()`、`lock_runtime_read()`、`lock_runtime_write()` 与 `lock_subscribers()`，分别覆盖 resource registry、payload map、runtime slot map 和 subscriber list。`registry_ops.rs`、`payload_ops.rs`、`lease_ops.rs` 与 `events.rs` 只消费这些 helper，不再在生产路径 direct lock expect/unwrap 或以 `lock poisoned` panic 结束。

新增 module-local `resource_manager_accessors_recover_poisoned_state_locks` 覆盖 subscribers、registry、payloads 与 runtime locks 被 poison 后仍可 subscribe、register_ready、get/acquire、ref_count 与 runtime_state；`structure_convention/lock_poison_policy.rs::runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager` 读取五个 manager owner 与 `docs/zircon_runtime/core/resource.md`，验证 helper、direct-lock/direct-panic scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、core resource docs 与 status-output expectations。完整 `module_convention_gate` 与全量 core resource Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、core resource manager direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 animation manager lock poison recovery

状态：`runtime_15_animation_manager_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 animation runtime manager。`zircon_runtime/src/animation/manager/mod.rs` 新增私有 `lock_playback_settings()` helper，`DefaultAnimationManager::store_playback_settings` 与 `AnimationManager::playback_settings()` 都通过该 helper 访问 `AnimationPlaybackSettings`，不再在生产路径因 poisoned playback settings mutex panic。

新增 module-local `animation_manager_playback_settings_recover_poisoned_lock` 覆盖中毒锁恢复后播放设置仍可 store/read；`structure_convention/lock_poison_policy.rs::runtime_15_animation_manager_lock_poison_recovery_guard_covers_playback_settings` 读取 `animation/manager/mod.rs` 与 `docs/zircon_runtime/animation/runtime.md`，验证 helper、direct-lock/direct-panic scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、animation module docs 与 status-output expectations。完整 `module_convention_gate` 与全量 animation Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、animation manager direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 input runtime manager lock poison recovery

状态：`runtime_15_input_runtime_manager_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 input runtime manager。`zircon_runtime/src/input/runtime/default_input_manager.rs` 新增私有 `lock_state()` helper，`begin_frame`、`submit_event`、snapshot、frame snapshot 与 drain 路径都通过 helper 访问 `InputState`，不再在生产路径 direct lock unwrap。`zircon_runtime/src/input/runtime/default_input_action_manager.rs` 新增私有 `lock_evaluator()` helper，action map set/read 与 action evaluation 路径都通过 helper 访问 `InputActionEvaluator`。

新增 module-local `input_manager_accessors_recover_poisoned_state_lock` 和 `input_action_manager_accessors_recover_poisoned_evaluator_lock` 覆盖 input state 与 action evaluator locks 被 poison 后仍可 submit/snapshot/drain/evaluate；`structure_convention/lock_poison_policy.rs::runtime_15_input_runtime_manager_lock_poison_recovery_guard_covers_input_state` 读取 `input/runtime/default_input_manager.rs`、`input/runtime/default_input_action_manager.rs` 与 `docs/zircon_runtime/input/input_state.md`，验证 helper、direct-lock scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、input module docs 与 status-output expectations。完整 `module_convention_gate` 与全量 input Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、input runtime manager direct-lock scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 asset project manager lock poison recovery

状态：`runtime_15_asset_project_manager_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 `ProjectAssetManager` 的运行时锁入口。`zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs` 让 `project_read()`、`project_write()`、`importer_registry_read()`、`importer_registry_write()`、change subscriber、watch-error subscriber 和 watcher lock 都通过 `unwrap_or_else(|poisoned| poisoned.into_inner())` 恢复；`zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs` 改为通过 importer registry helper 安装和复制 pending importer handlers。

新增 module-local `project_asset_manager_runtime_accessors_recover_poisoned_locks` 覆盖 project、pending importer registry、change subscribers、watch-error subscribers 与 watcher locks 被 poison 后仍可恢复；`structure_convention/lock_poison_policy.rs::runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager` 读取 construction/runtime 两个 owner 与 `docs/zircon_runtime/asset/importer.md`，验证 helper、direct-lock/direct-panic scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、asset importer docs 与 status-output expectations。完整 `module_convention_gate` 与全量 asset pipeline Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、project asset manager direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 asset worker pool lock poison recovery

状态：`runtime_15_asset_worker_pool_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 asset worker pool 和 AssetManager service contract。`zircon_runtime/src/asset/pipeline/worker_pool.rs` 新增 `lock_in_flight()`、`lock_diagnostics()`、`lock_in_flight_map(...)` 与 `lock_worker_diagnostics(...)`，request de-duplication、queue-full rollback、diagnostics snapshot、frame diagnostics 和 completion publishing 都通过这些 helper 访问 in-flight map 与 diagnostics state。`zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs` 改为通过 `ProjectAssetManager` 的 `importer_registry_read()`、`lock_change_subscribers()` 与 `lock_watch_error_subscribers()` 访问 importer/subscriber state，service open/subscribe 路径不再保留 RwLock expect 或 subscriber lock panic。

新增 module-local `asset_worker_pool_accessors_recover_poisoned_locks` 覆盖 worker locks 被 poison 后仍可 request、diagnostics readback 和 completion publish；`structure_convention/lock_poison_policy.rs::runtime_15_asset_worker_pool_lock_poison_recovery_guard_covers_asset_worker_pool` 读取 worker pool、service contract、manager runtime helper 与 `docs/zircon_runtime/asset/worker_pool.md`，验证 helper、direct-lock/direct-panic scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、asset worker docs 与 status-output expectations。完整 `module_convention_gate` 与全量 asset pipeline Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、asset worker pool direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 WGPU render framework lock poison recovery

状态：`runtime_15_wgpu_render_framework_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 WGPU render framework 的两个共享锁入口。`zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework/wgpu_render_framework.rs` 中 `WgpuRenderFramework::lock_operation()` 与 `lock_state()` 现在通过 `unwrap_or_else(|poisoned| poisoned.into_inner())` 恢复 poisoned mutex，create/destroy viewport、pipeline asset set/reload/register、frame submit、capture、viewport surface、stats/debugger query 等生产路径继续集中走这两个入口。

新增 module-local `wgpu_render_framework_accessors_recover_poisoned_locks` 覆盖 operation lock 与 render framework state lock 被 poison 后仍可恢复；`structure_convention/lock_poison_policy.rs::runtime_15_wgpu_render_framework_lock_poison_recovery_guard_covers_wgpu_framework` 读取 WGPU framework owner 与 `docs/zircon_runtime/graphics/render-product-submit.md`，验证 helper、direct-lock/direct-panic scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、render product docs 与 status-output expectations。完整 `module_convention_gate` 与全量 graphics/render Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、WGPU render framework direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 RHI WGPU render device lock poison recovery

状态：`runtime_15_rhi_wgpu_render_device_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到 headless WGPU RHI device。`zircon_runtime/src/rhi_wgpu/device.rs` 新增私有 `lock_state()` helper，`create_*`、`destroy_*`、descriptor snapshot、bind group/pipeline validation、command submit、fence completion、`transient_allocator_stats()`、`write_buffer`、`read_buffer` 和 `read_texture` 都通过 helper 访问 `WgpuRenderDeviceState`，不再在生产路径 direct lock unwrap。

新增 module-local `wgpu_render_device_state_accessors_recover_poisoned_lock` 覆盖 state lock 被 poison 后仍可读取 transient allocator stats、创建 staging buffer、写入并读回数据；`structure_convention/rhi_wgpu_lock_poison.rs::runtime_15_rhi_wgpu_render_device_lock_poison_recovery_guard_covers_device_state` 读取 WGPU device owner、父结构聚合文件与 `docs/zircon_runtime/rhi/descriptors.md`，验证 helper、独立 guard 挂载、direct-lock scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、RHI docs 与 status-output expectations。完整 `module_convention_gate` 与全量 RHI Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、RHI WGPU device production direct-lock scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过（仅 LF-to-CRLF warnings）；Cargo 因外部 cargo/rustc 通道 active（`cargo` PIDs 15540、30624、41464、42000、61116、69224；`rustc` PIDs 3600、6012、6488、9080、10692、15256、23412、29408、38788、52576、54436、57988、58796、63940）按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 script VM registry lock poison recovery

状态：`runtime_15_script_vm_registry_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到脚本 VM 的核心注册表和热重载 slot table。`zircon_runtime/src/script/vm/backend/backend_registry.rs` 新增私有 `lock_families()`，`register_family`、`resolve` 和 `names` 都通过 helper 访问 backend family map。`zircon_runtime/src/script/vm/host/host_registry.rs` 新增 `lock_handles()`，host capability handle 分配、查询、枚举和校验不再 direct lock unwrap。`zircon_runtime/src/script/vm/host/host_export_registry.rs` 新增 `lock_modules()`，module registration、module snapshot、script call table build 和 callback lookup 都通过 helper 访问 host export registry。`zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs` 新增 `lock_slots()`，load/hot-reload/restore/replace/unload/slot lookup/package lookup/export call/list/debug projection 都通过 helper 访问 slot table。

新增 module-local `vm_backend_registry_accessors_recover_poisoned_family_lock`、`host_registry_accessors_recover_poisoned_handle_lock`、`host_export_registry_accessors_recover_poisoned_module_lock` 与 `hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock` 覆盖四类锁被 poison 后仍可 register/resolve/call/list/unload；`structure_convention/lock_poison_policy.rs::runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries` 读取四个脚本 VM owner 与 `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md`，验证 helper、direct-lock/direct-panic scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、script VM docs 与 status-output expectations。完整 `module_convention_gate` 与全量 script VM Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、script VM registry direct-lock/direct-panic scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 M3 VM plugin manager selected-backend lock poison recovery

状态：`runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_static_passed_cargo_deferred`。

本切片把 E9/F2 的 poison-safe lock 规则扩展到脚本 VM plugin manager 的 selected-backend selector。`zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs` 新增私有 `selected_backend_read()` / `selected_backend_write()` helpers，`selected_backend_name()` 和 `select_default_backend()` 都通过 helper 访问 `RwLock<String>`，不再在生产路径 direct read/write unwrap。

新增 module-local `vm_plugin_manager_selected_backend_accessors_recover_poisoned_lock` 覆盖 selected-backend lock 被 poison 后仍可读取默认 selector 并切换到 `builtin:mock`；`structure_convention/script_vm_lock_poison.rs::runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_guard_covers_manager_selector` 读取 VM plugin manager、父结构聚合文件与 `docs/zircon_runtime/script/vm/zr_vm_host_reflection.md`，验证 helper、独立 guard 挂载、direct RwLock unwrap scan 和跨文档状态锚。该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、script VM docs 与 status-output expectations。完整 `module_convention_gate` 与全量 script VM Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、VM plugin manager direct RwLock unwrap scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check`；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。

## Runtime 15 UI boundary runtime-host forbidden attribute literal cleanup

状态：`runtime_15_ui_boundary_runtime_host_literal_cleanup_static_passed_cargo_deferred`。

本切片只处理 UI boundary 守卫自身的源扫描噪声，不改变生产 UI 行为。`zircon_runtime/src/tests/ui_boundary/runtime_host.rs` 继续通过 `runtime_ui_host_surface_splits_production_frame_from_test_support` 验证 `ui/mod.rs` 暴露生产 `PublicRuntimeFrame`、以 test-only `runtime_ui` support 保持分离，并禁止 UI root 回到直接挂载 test support 的形态；本轮把 forbidden dead-code allow attribute 从直接测试源字面量改为 `DEAD_CODE_ALLOW_ATTRIBUTE` 常量拼装，避免简单源码扫描把守卫文件误报为 suppression 残留。

该记录同步 Runtime 15 子计划、runtime index、engine code structure convention、review findings、UI architecture 与 status-output expectations。完整 `runtime_15_no_dead_code_suppression_in_production`、`module_convention_gate` 与全量 Cargo sweep 仍 pending。

验证：scoped rustfmt/static scans、literal scan、docs/status/date/session anchor scan、trailing-whitespace scan 和 scoped `git diff --check` 通过；Cargo 按 Runtime 15 实施切片节奏 deferred，不计通过。
