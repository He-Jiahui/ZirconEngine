---
related_code:
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/late_api_cleanup.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
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
  - zircon_runtime/src/ui/component/catalog/editor_showcase/helpers.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget.rs
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
  - zircon_runtime/src/ui/tests/runtime_ui_support
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_public_runtime.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/prelude.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/core/runtime/state/module_entry.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/tests/reflection_docs.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_new/construct.rs
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
  - zircon_runtime/src/scene/tests/ecs_systems/query_helpers.rs
  - zircon_runtime/src/scene/tests/ecs_systems/removal_local.rs
  - zircon_runtime/src/scene/tests/ecs_query.rs
  - zircon_runtime/src/scene/tests/ecs_query/read_items.rs
  - zircon_runtime/src/scene/tests/ecs_query/mutation_access.rs
  - zircon_runtime/src/scene/tests/ecs_query/fixed_ticks.rs
  - zircon_runtime/src/scene/tests/ecs_query/iter_many.rs
  - zircon_runtime/src/scene/tests/ecs_query/cache_helpers.rs
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
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime_provider/mod.rs
  - zircon_runtime/src/graphics/runtime_provider/registration.rs
  - zircon_runtime/src/graphics/runtime_provider/update.rs
  - zircon_runtime/src/graphics/runtime_provider/feedback.rs
  - zircon_runtime/src/graphics/runtime_provider/prepare_input.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/asset/prelude.rs
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
  - zircon_runtime/src/ui/component/catalog/editor_showcase/helpers.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget.rs
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
  - zircon_runtime/src/ui/tests/runtime_ui_support
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_public_runtime.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/core/runtime/state/module_entry.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_new/construct.rs
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
  - zircon_runtime/src/scene/tests/ecs_systems/query_helpers.rs
  - zircon_runtime/src/scene/tests/ecs_systems/removal_local.rs
  - zircon_runtime/src/scene/tests/ecs_query.rs
  - zircon_runtime/src/scene/tests/ecs_query/read_items.rs
  - zircon_runtime/src/scene/tests/ecs_query/mutation_access.rs
  - zircon_runtime/src/scene/tests/ecs_query/fixed_ticks.rs
  - zircon_runtime/src/scene/tests/ecs_query/iter_many.rs
  - zircon_runtime/src/scene/tests/ecs_query/cache_helpers.rs
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
  - cargo test -p zircon_runtime --lib runtime_15_asset_test_budget_guard_child_owner_split --no-default-features --features core-min --locked
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
  - cargo test -p zircon_runtime --lib runtime_15_core_runtime_service_lists_are_folder_backed --no-default-features --features core-min --locked
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
doc_type: module-detail
---

# Runtime 模块结构规范镜像文档

> 本文是 [Runtime 15](../../plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md) 的镜像文档，固定 `module_convention_gate` 的结构审计事实，由 `runtime_15_module_convention_mirror_docs_match_structure_audit_counts` 守卫锁定计数。上游规范：[`engine-code-structure-convention.md`](../../plans/engine-code-structure-convention.md)。
>
> 状态：in_progress（Runtime 15 F9 runtime prelude required type coverage、Runtime 15 graphics facade visibility note、Runtime 15 runtime UI dead-code support split、Runtime 15 F12 runtime-owned dead-code suppression cleanup、Runtime 15 F12 script host value descriptor dead-code cleanup、Runtime 15 F12 script reflection macro fixture dead-code cleanup、Runtime 15 F12 offscreen target texture owner cleanup、Runtime 15 F12 render backend state owner cleanup、Runtime 15 F12 gpu texture resource owner cleanup、Runtime 15 F12 gpu material uniform owner cleanup、Runtime 15 F12 gpu mesh order signature cleanup、Runtime 15 F12 gpu model identity cleanup、Runtime 15 F12 post-process LUT texture owner cleanup、Runtime 15 F12 output target texture owner cleanup、Runtime 15 F12 material runtime capture seed cleanup、Runtime 15 F12 resource streamer diagnostics accessor cleanup、Runtime 15 F12 resource streamer resolve texture id cleanup、Runtime 15 F12 particle GPU readback output accessor cleanup、Runtime 15 F12 advanced plugin output test accessor cleanup、Runtime 15 M3 graphics dead-code guard module split、Runtime 15 M3 graphics dead-code guard child-owner split、Runtime 15 M3 provider boilerplate guard module split、Runtime 15 M3 facade surface guard module split、Runtime 15 M3 runtime dead-code guard module split、Runtime 15 M3 diagnostics guard module split、Runtime 15 M3 core framework test folder split、Runtime 15 M3 core runtime deactivation blocked test folder split、Runtime 15 M3 UI v2 asset test folder split、Runtime 15 M3 UI shared core test folder split、Runtime 15 M3 UI accessibility test folder split、Runtime 15 M3 UI accessibility widget actions test folder split、Runtime 15 M3 UI layout slots test folder split、Runtime 15 M3 UI surface-frame authority test folder split、Runtime 15 M3 UI surface dirty domains test folder split、Runtime 15 M3 UI material layout test folder split、Runtime 15 M3 UI event routing test folder split、Runtime 15 M3 UI runtime input reply routes test folder split、Runtime 15 M3 UI runtime input reply route child folder split、Runtime 15 M3 runtime diagnostics test folder split、Runtime 15 M3 RHI command list test folder split、Runtime 15 M3 RHI device contract test folder split、Runtime 15 M3 asset pack test folder split、Runtime 15 M3 asset facade test folder split、Runtime 15 M3 asset project zmeta test folder split、Runtime 15 M3 asset project manager test folder split、Runtime 15 M3 asset project flow sample test folder split、Runtime 15 M3 asset material test folder split、Runtime 15 M3 asset glTF importer test folder split、Runtime 15 M3 asset glTF primitive fixture folder split、Runtime 15 M3 asset importer test folder split、Runtime 15 M3 asset scene test folder split、Runtime 15 M3 test file budget guard folder split、Runtime 15 M3 Runtime 07 performance hotspot guard folder split、Runtime 15 M3 script VM test folder split、Runtime 15 M3 scene ECS schedule test folder split、Runtime 15 F14 diagnostics normalization、Runtime 15 F13 provider registration shared owner、Runtime 15 F13 provider update shared stats owner、Runtime 15 F13 provider feedback shared payload owner、Runtime 15 F13 provider prepare input shared frame owner 与 Runtime 15 F13 full provider boilerplate audit 已落地；完整 `module_convention_boundary.py` 审计计数、全量 dead-code sweep 与测试组织拆分仍 pending）。
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
> 最新完成：Runtime 15 M3 code review findings test folder split（`runtime_15_code_review_findings_tests_folder_split_static_passed_cargo_deferred`）已把 `tests/runtime_absorption/code_review_findings.rs` 降到 3 行并迁出 `typed_error_convergence.rs`、`f8_api_convergence.rs` 与 `late_api_cleanup.rs` 三个 folder-backed review guard owner；14 个评审守卫保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。
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
> 最新完成：Runtime 15 M4 UI component catalog editor-showcase helper owner split（`runtime_15_ui_component_catalog_editor_showcase_helper_owner_split_static_passed_cargo_timeout_no_result`）已把 `ui/component/catalog/editor_showcase.rs` 降到 663 行，并迁出 384 行 `ui/component/catalog/editor_showcase/helpers.rs` 承接 editor showcase descriptor helper owner；完整 `large_file_ownership_gate` 仍 pending。
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

> 最新完成：Runtime 15 M3 gameplay host test folder split（`runtime_15_gameplay_host_tests_folder_split_static_passed_cargo_deferred`）已把 `script/vm/gameplay_host/tests.rs` 降到 46 行并迁出四个 folder-backed gameplay host 测试 owner；9 个玩法宿主测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 shader prewarm manifest test folder split（`runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred`）已把 `bin/zircon_shader_prewarm/manifest.rs` 降到 672 行并迁出 `bin/zircon_shader_prewarm/manifest/tests.rs` 测试 owner；1 个资产扫描预热 manifest 测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 scene ECS schedule test folder split（`runtime_15_scene_ecs_schedule_tests_folder_split_static_passed_cargo_deferred`）已把 `scene/tests/ecs_schedule.rs` 降到 32 行并迁出四个 folder-backed ECS schedule 行为 owner；57 个 schedule 测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。

> 最新完成：Runtime 15 M3 scene ECS systems test folder split（`runtime_15_scene_ecs_systems_tests_folder_split_static_passed_cargo_deferred`）已把 `scene/tests/ecs_systems.rs` 降到 53 行并迁出六个 folder-backed ECS systems 行为 owner；24 个系统参数/事件/查询 helper 测试保留在子模块，完整 `runtime_15_no_oversized_test_files` 仍 pending。

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

## Runtime 15 M4 material asset value/readiness helper owner split

状态：`runtime_15_material_asset_value_readiness_owner_split_static_passed_cargo_timeout_no_result`。

R1.4/M4 的当前新增落地部分是 material asset helper owner 减压。`asset/assets/material/material_asset.rs` 从 937 行减压为 750 行，继续拥有 `MaterialAsset` DTO、`.zmaterial` document 转换入口、descriptor/readiness public API、management overview 与 shader-aware dependency/texture-slot entry；新增 `asset/assets/material/material_asset/value_sync.rs` 作为 136 行 child owner，承接 TOML override 读取、texture slot hydration、legacy default 同步与 TOML 数组生成 helper；新增 `asset/assets/material/material_asset/readiness.rs` 作为 70 行 child owner，承接 shader readiness diagnostic projection、WGSL capture/missing runtime source 映射与 material validation diagnostic rows。

该切片不改变 `.zmaterial` 序列化形状、不改变 `MaterialAsset` public API、不改 render material descriptor 字段或 readiness report 语义，也不新增兼容 re-export。守卫：`runtime_15_material_asset_value_readiness_helpers_are_child_owners` 验证父/子 owner 挂载、value/readiness helper 不回流、三侧低于 800 行预算，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、asset zmeta/material 文档和 status-output expectations 的状态锚同步。该切片只关闭 material asset value/readiness helper 的 M4 减压子面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 asset/render material Cargo sweep 仍 pending。

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

R4.1/M3 的当前新增落地部分是 `tests/runtime_absorption/code_review_findings.rs` folder-backed 拆分。F5/F6/F7 typed-error review guards 迁入 `tests/runtime_absorption/code_review_findings/typed_error_convergence.rs`；F8 texture import settings 与 RuntimePluginDescriptor review guards 迁入 `f8_api_convergence.rs`；F11 shading-model registry、F17 entity path lookup、F18 asset manager handle shape 与 F19 scene renderer construction naming review guards 迁入 `late_api_cleanup.rs`。父文件现在只保留子模块挂载，行数从 1315 降到 3；14 个评审守卫全部保留，最大子文件 `f8_api_convergence.rs` 为 574 行。

守卫：`runtime_15_code_review_findings_tests_are_folder_backed` 验证父模块挂载三个子 owner，代表性 F5/F8/F11/F19 moved guard 不回流到父文件，所有 14 个 review guard 保留在子模块，父/子 owner 都低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档和 status-output expectations 都包含本切片锚。该切片只关闭 `runtime_absorption/code_review_findings.rs` 的 M3 folder-backed 子面；完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 F12 sweep 仍 pending。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描、docs/status 锚点扫描和 scoped `git diff --check` 已通过；Cargo 按实施切片节奏 deferred，不计通过。

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

R1.4/M4 的当前新增落地部分是 UI component catalog editor showcase production owner 减压。`ui/component/catalog/editor_showcase.rs` 继续拥有 editor showcase registry、descriptor list、descriptor assembly entry point 与 representative component coverage；base descriptor construction、layout role/default template projection、palette metadata、fallback policy、option/slot/value prop schema builders 与 TOML layout helpers 迁入 `ui/component/catalog/editor_showcase/helpers.rs`。父文件通过 `mod helpers;` 与窄 helper imports 消费子 owner，不改变 editor showcase registry ids、component descriptors、palette metadata shape、fallback policy 或 component catalog public lookup behavior。父文件从 1029 行降到 663 行，子 owner 为 384 行，两侧都低于 800 行生产文件软预算。

守卫：`runtime_15_ui_component_catalog_editor_showcase_helpers_are_child_owner` 验证父模块挂载 helpers child、代表性 descriptor helper、palette metadata、fallback policy 与 prop schema builder 不回流到父文件、子 owner 承接 descriptor construction internals、父子 owner 均低于 800 行，并验证 Runtime 15 计划、runtime index、审查发现、结构规范、本文档、`docs/zircon_runtime/ui/architecture.md` 和 status-output expectations 都包含本切片锚。该切片只关闭 `ui/component/catalog/editor_showcase.rs` 的 descriptor helper owner 减压面；完整 `large_file_ownership_gate`、`module_convention_gate` 与全量 UI component catalog Cargo sweep 仍 pending。

验证：scoped rustfmt/static checks、父子行数预算扫描、moved owner 扫描与 docs/status 锚点扫描已通过；focused Cargo 305 秒超时无诊断结果，超时后另有 editor layout cargo/rustc 通道活跃，不计通过。

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

R4.1/M3 的当前新增落地部分是 `scene/tests/ecs_systems.rs` folder-backed 拆分。原父文件中的 command queue / entity command 用例迁入 `scene/tests/ecs_systems/commands.rs`；SystemState、QueryState mutation、optional resource、ParamSet 和 8 元 tuple/ParamSet 用例迁入 `state_params.rs`；EventReader/EventWriter 队列和 cursor 用例迁入 `events.rs`；Added/Changed run-window、cached direct、cached iter、count/is_empty helper 用例迁入 `run_window_filters.rs`；get_many / iter_many / single helper 用例迁入 `query_helpers.rs`；removed-components、LocalParam 和 scheduled native local-state 用例迁入 `removal_local.rs`。

父文件从约 1000+ 行降到 53 行，只保留共享 `Health` / `Player` / `Marker` / `Score` / `HitEvent` / `LocalCounter` fixture、`expect_query_error(...)` helper 和子模块挂载；最大子文件 `run_window_filters.rs` 为 330 行，`state_params.rs` 为 286 行。24 个原父文件测试全部迁入六个子模块，所有 owner 低于 800 行。新增 `structure_convention/test_file_budget/scene_ecs_systems.rs::runtime_15_scene_ecs_systems_tests_are_folder_backed`，验证父/子模块挂载、代表性 moved guard 不回流、迁移测试数量、ECS systems test owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/scene/ecs.md` 和 status-output expectations 的状态锚同步。

验证：scoped rustfmt/static checks、迁移测试数量扫描、父子行数预算扫描和 docs/status 锚点扫描已通过；Cargo 按实施切片节奏 deferred，不计通过。完整 `runtime_15_no_oversized_test_files`、`module_convention_gate` 与全量 dead-code sweep 仍 pending。

## Runtime 15 M3 scene ECS query test folder split

状态：`runtime_15_scene_ecs_query_tests_folder_split_static_passed_cargo_deferred`。

R4.1/M3 的当前新增落地部分是 `scene/tests/ecs_query.rs` folder-backed 拆分。原父文件中的 query data read、tuple/filter arity、stable location 和 single-result 用例迁入 `scene/tests/ecs_query/read_items.rs`；mutable query、get_mut/get_many_mut、access conflict 和 duplicate mutable component 用例迁入 `mutation_access.rs`；fixed scene component query 与 Ref/Mut change tick 用例迁入 `fixed_ticks.rs`；mutable/cached-direct iter-many run-window 用例迁入 `iter_many.rs`；cache rebuild、count/empty/get/many/unique helpers、cached-direct table/sparse location、archetype movement 和 optional archetype membership 用例迁入 `cache_helpers.rs`。

父文件从 938 行降到 60 行，只保留共享 `Health` / `Enemy` / `Player` / `SparseScore` fixture、`expect_query_error(...)`、`cached_component_locations_for(...)` 和子模块挂载；最大子文件 `cache_helpers.rs` 为 555 行。19 个原父文件测试全部迁入五个子模块，所有 owner 低于 800 行。新增 `structure_convention/test_file_budget/scene_ecs_query.rs::runtime_15_scene_ecs_query_tests_are_folder_backed`，验证父/子模块挂载、代表性 moved guard 不回流、迁移测试数量、ECS query test owner 行数预算，以及 Runtime 15 计划、runtime index、review findings、结构规范、本文档、`docs/zircon_runtime/scene/ecs.md` 和 status-output expectations 的状态锚同步。

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
