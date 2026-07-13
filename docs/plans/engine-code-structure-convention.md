---
related_code:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs
  - tests/acceptance/runtime-priority-plan-output-archive-ownership.md
  - docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md
  - docs/plans/zircon_runtime/runtime/15/2026-07-10-priority-plan-doc-current-owner-inventory.md
  - zircon_runtime/src/tests/runtime_absorption/plan_status/support/runtime_plan_archives.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/recent_static_guards/parent_routing.rs
  - tools/tests/test_runtime_plan_status_archive_ownership.py
  - tests/acceptance/runtime-plan-status-archive-ownership-sync.md
  - tools/tests/test_zui_docs_suffix_convergence.py
  - tools/tests/test_zui_docs_suffix_status_guards.py
  - tools/tests/test_zui_docs_suffix_convergence_test_owner_boundaries.py
  - tools/tests/test_zui_docs_current_status_suffix_test_owner_budget.py
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/framework/animation/error.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/animation/manager/pose.rs
  - zircon_runtime/src/animation/manager/sampling.rs
  - zircon_runtime/src/animation/sequence/apply.rs
  - zircon_runtime/src/animation/sequence/conversion.rs
  - zircon_runtime/src/core/framework/camera_controller/mod.rs
  - zircon_runtime/src/core/framework/camera_controller/controller_output.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/camera_controller.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/render_fixtures.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_core_framework.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_core_framework_render_fixtures.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/render_contracts.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/observer_callback_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/query_state_many_item_array.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/component_storage_component_results.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_core_scene.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_core_scene_ecs.rs
  - docs/zircon_runtime/core/framework/camera_controller.md
  - zircon_runtime/src/scene/tests/ecs_systems.rs
  - zircon_runtime/src/scene/tests/ecs_systems/many_single_queries.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/scene_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_systems.rs
  - zircon_runtime/src/asset/tests/assets/texture_upload_readiness.rs
  - zircon_runtime/src/asset/tests/assets/texture_upload_readiness/container_fixtures.rs
  - zircon_runtime/src/asset/watch/asset_change_construction.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_construction.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_construct/construct.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/render_fixtures.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_graphics.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/texture_containers.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_asset_dynamic.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/scene_dynamic.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_banned_names.rs
  - zircon_runtime/src/ui/platform_input/keyboard_map.rs
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
  - docs/zircon_runtime/ui/platform_input.md
  - zircon_runtime/src/ui/text/geometry.rs
  - zircon_runtime/src/graphics/text/cache/mod.rs
  - zircon_runtime/src/graphics/text/cache/frame_dedup.rs
  - zircon_runtime/src/graphics/text/cache/layout_cache.rs
  - zircon_runtime/src/graphics/text/cache/measure_cache.rs
  - zircon_runtime/src/graphics/text/cache/shaped_cache.rs
  - zircon_runtime/src/graphics/text/shaping/mod.rs
  - zircon_runtime/src/graphics/text/parallel/shape_pool.rs
  - zircon_runtime/src/graphics/text/layout/measure.rs
  - zircon_runtime/src/graphics/text/layout/line_break/mod.rs
  - zircon_runtime/src/graphics/text/layout/line_break/greedy.rs
  - zircon_runtime/src/graphics/text/layout/line_break/glyph_fallback.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/ellipsis.rs
  - zircon_runtime/src/ui/text/layout_engine/line_box.rs
  - zircon_runtime/src/ui/text/layout_engine/overflow_style.rs
  - zircon_runtime/src/ui/text/layout_engine/vertical.rs
  - zircon_runtime/src/ui/text/layout_engine/wrapping.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/graphics/text/cache/tests.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/surface/render/text_prewarm.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/tests/frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/tests/handoff.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/tests/retry_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/tests/source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/tests/source_cache.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/tests/text_pipeline/mod.rs
  - zircon_runtime/src/ui/tests/text_pipeline/fixtures.rs
  - zircon_runtime/src/ui/tests/text_pipeline/font_registry.rs
  - zircon_runtime/src/ui/tests/text_pipeline/layout_request.rs
  - zircon_runtime/src/ui/tests/text_pipeline/measure_cache.rs
  - zircon_runtime/src/ui/tests/text_pipeline/render_extract_prewarm.rs
  - zircon_runtime/src/ui/tests/text_pipeline/surface_cache.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/placement.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/placement/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/placement/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/glyphs/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion/style/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/divider/geometry/label_bounds/horizontal.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/divider/horizontal.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/chip/geometry/label.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/chip/geometry/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/alert/geometry/message.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/alert/geometry/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/avatar/geometry/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/avatar/geometry/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/badge/geometry/root_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/badge/geometry/overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/badge/geometry/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs/actions/labels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_command_palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_command_palette/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_command_palette/layout/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_command_palette/rows/indicator.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/entry.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/command_palette/options.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/command_palette_visual_screenshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/asset.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/tests.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/scrollbar_visual_screenshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime/src/core/framework/input/ime.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/support.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/drain.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/request.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/geometry.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/host_requests.rs
  - zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs
  - zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - docs/zircon_runtime/asset/render-assets.md
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/sprite/sprite.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache/tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/core/framework/render/shader/geometry_source.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_geometry_source_descriptor.rs
  - zircon_runtime/src/graphics/shader/mod.rs
  - zircon_runtime/src/graphics/shader/template/mod.rs
  - zircon_runtime/src/graphics/shader/template/assemble.rs
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
  - zircon_runtime/src/graphics/shader/template/material_surface.rs
  - zircon_runtime/src/graphics/shader/template/pass_specialization.rs
  - zircon_runtime/src/graphics/shader/template/validation.rs
  - zircon_runtime/src/graphics/shader/template/tests.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_surface_types.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_static.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_forward.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_gbuffer.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_depth.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/permutation_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_permutation_registry.rs
  - tools/zircon_build.py
  - tools/zircon_build_shader_prewarm.py
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/write.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/resource.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload/tests.rs
  - tools/tests/test_zircon_build_shader_prewarm.py
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/compiled_graph_cache_tests.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests/material_runtime.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests/material_runtime/pbr_projection.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/material_runtime_pbr_projection_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_vertices/build_particle_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_velocity_vertices/build_particle_velocity_vertices.rs
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs
  - zircon_runtime/src/graphics/tests/render_product_submit.rs
  - zircon_runtime/src/graphics/tests/render_product_submit/profiles.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_product_submit_profiles_tests.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers/particles.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers/postprocess.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers/queue_override.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers/transparent3d.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/m4_behavior_postprocess_tests.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/composite.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/material_sampling.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/ordering.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/viewport.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/fixture.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/primary_surface.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/texture_target.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests/virtual_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_prepared_mesh_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/extract_item.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/non_material_rebuild.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/rebuild_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/residual_fallback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/lazy_rebuild_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/fallback_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/visibility_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/page.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/node_cull.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot/execution.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot_streams.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot_streams/types.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/background.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_compiled_scene_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/render_structure.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/backend_types/camera_target.rs
  - zircon_runtime/src/core/framework/render/backend_types/capability.rs
  - zircon_runtime/src/core/framework/render/backend_types/graph_reports.rs
  - zircon_runtime/src/core/framework/render/backend_types/quality.rs
  - zircon_runtime/src/core/framework/render/backend_types/tests.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/mesh_queue.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_pending_command_cache_plan.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/render_products.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures.rs
  - zircon_runtime/src/graphics/tests/render_product_shadow_captures/directional.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_product_shadow_captures_directional_tests.rs
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs
  - zircon_runtime/src/graphics/tests/render_product_anti_alias/particle.rs
  - zircon_runtime/src/graphics/tests/render_product_anti_alias/reactive_mask.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_product_anti_alias_focused_tests.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_runtime/src/graphics/tests/surface_targets/texture_target.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_surface_targets_texture_target_tests.rs
  - zircon_runtime/src/graphics/tests/plugin_feature_compile.rs
  - zircon_runtime/src/graphics/tests/plugin_feature_compile/particle.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_plugin_feature_compile_particle_tests.rs
  - zircon_runtime/src/graphics/tests/renderer_data_asset.rs
  - zircon_runtime/src/graphics/tests/renderer_data_asset/asset_aware_compile.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_renderer_data_asset_compile_tests.rs
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes.rs
  - zircon_runtime/src/graphics/tests/project_render/render_quality.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_project_scene_products_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_project_render_quality_tests.rs
  - zircon_runtime/src/graphics/tests/visibility.rs
  - zircon_runtime/src/graphics/tests/visibility/virtual_geometry_page_plan.rs
  - zircon_runtime/src/graphics/tests/visibility/virtual_geometry_frontier.rs
  - zircon_runtime/src/graphics/tests/visibility/virtual_geometry_priority.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_visibility_virtual_geometry_tests.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/default_pipelines.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/dynamic_resolution.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/plugin_features.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/temporal_and_ops.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/compile_options.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/feature_descriptors.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/validation_core.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/validation_descriptors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_pipeline_compile_monolith_tests.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/stats.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/history.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/pipeline_profiles.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/neural_compute.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/advanced_providers.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_framework_bridge_tests.rs
  - zircon_runtime/src/animation/module.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/mod.rs
  - zircon_runtime/src/scene/world/property_access/path_resolution.rs
  - zircon_runtime/src/animation/sequence/target.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_texture.rs
  - zircon_runtime/src/core/runtime/handle/registration/register_module.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/mod.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/types.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/multi.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/specialized.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/shutdown.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_chrome/fill.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_chrome/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_chrome/separators.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_chrome/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows_tests/style.rs
  - zircon_plugins/gltf_importer/plugin.toml
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/capability.rs
  - zircon_plugins/gltf_importer/runtime/src/plugin.rs
  - zircon_plugins/gltf_importer/dist/Cargo.toml
  - zircon_plugins/gltf_importer/dist/src/lib.rs
  - zircon_plugins/obj_importer/plugin.toml
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/capability.rs
  - zircon_plugins/obj_importer/runtime/src/plugin.rs
  - zircon_plugins/obj_importer/dist/Cargo.toml
  - zircon_plugins/obj_importer/dist/src/lib.rs
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/capability.rs
  - zircon_plugins/texture_importer/runtime/src/plugin.rs
  - zircon_plugins/texture_importer/dist/Cargo.toml
  - zircon_plugins/texture_importer/dist/src/lib.rs
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/runtime/src/capability.rs
  - zircon_plugins/audio_importer/runtime/src/plugin.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/audio_importer/dist/Cargo.toml
  - zircon_plugins/audio_importer/dist/src/lib.rs
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/src/capability.rs
  - zircon_plugins/opus_importer/runtime/src/plugin.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/capability.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/plugin.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/capability.rs
  - zircon_plugins/ui_document_importer/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/src/capability.rs
  - zircon_plugins/asset_importers/data/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/data/plugin.toml
  - zircon_plugins/asset_importers/data/dist/Cargo.toml
  - zircon_plugins/asset_importers/data/dist/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/capability.rs
  - zircon_plugins/asset_importers/model/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/src/capability.rs
  - zircon_plugins/asset_importers/shader/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/audio/runtime/Cargo.toml
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/capability.rs
  - zircon_plugins/asset_importers/audio/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/texture/runtime/Cargo.toml
  - zircon_plugins/asset_importers/texture/plugin.toml
  - zircon_plugins/asset_importers/texture/dist/Cargo.toml
  - zircon_plugins/asset_importers/texture/dist/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/capability.rs
  - zircon_plugins/asset_importers/texture/runtime/src/plugin.rs
  - zircon_plugins/plugin_sdk/src/runtime_exports.rs
  - zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs
  - zircon_plugins/plugin_sdk/Cargo.toml
  - zircon_plugins/plugin_sdk/src/dist.rs
  - zircon_plugins/plugin_sdk/src/native.rs
  - tools/audit_plugin_structure.py
  - tools/plugin_structure_audits/capability.py
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/dist/Cargo.toml
  - zircon_plugins/ai/dist/src/lib.rs
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/lib.rs
  - zircon_plugins/ai/runtime/src/capability.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/src/tests/registration.rs
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/capability.rs
  - zircon_plugins/hybrid_gi/runtime/Cargo.toml
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/dist/Cargo.toml
  - zircon_plugins/hybrid_gi/dist/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/capability.rs
  - zircon_plugins/hybrid_gi/runtime/src/plugin.rs
  - zircon_plugins/hybrid_gi/runtime/src/tests.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/representation.rs
  - zircon_plugins/navigation/runtime/Cargo.toml
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/capability.rs
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/navigation/dist/Cargo.toml
  - zircon_plugins/navigation/dist/src/lib.rs
  - zircon_plugins/navigation/runtime/src/plugin.rs
  - zircon_plugins/navigation/runtime/src/tests/registration.rs
  - zircon_plugins/animation/plugin.toml
  - zircon_plugins/animation/dist/Cargo.toml
  - zircon_plugins/animation/dist/src/lib.rs
  - zircon_plugins/animation/runtime/Cargo.toml
  - zircon_plugins/animation/runtime/src/plugin.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/animation/runtime/src/tests.rs
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/physics/dist/Cargo.toml
  - zircon_plugins/physics/dist/src/lib.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/capability.rs
  - zircon_plugins/particles/plugin.toml
  - zircon_plugins/particles/dist/Cargo.toml
  - zircon_plugins/particles/dist/src/lib.rs
  - zircon_plugins/particles/runtime/Cargo.toml
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/capability.rs
  - zircon_plugins/particles/runtime/src/plugin.rs
  - zircon_plugins/particles/runtime/src/tests/package_manifest.rs
  - zircon_plugins/prefab_tools/runtime/Cargo.toml
  - zircon_plugins/prefab_tools/runtime/src/lib.rs
  - zircon_plugins/prefab_tools/runtime/src/capability.rs
  - zircon_plugins/rendering/runtime/Cargo.toml
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_plugins/rendering/runtime/src/capability.rs
  - zircon_plugins/solari/plugin.toml
  - zircon_plugins/solari/dist/Cargo.toml
  - zircon_plugins/solari/dist/src/lib.rs
  - zircon_plugins/solari/runtime/Cargo.toml
  - zircon_plugins/solari/runtime/src/lib.rs
  - zircon_plugins/solari/runtime/src/capability.rs
  - zircon_plugins/solari/runtime/src/plugin.rs
  - zircon_plugins/terrain/plugin.toml
  - zircon_plugins/terrain/dist/Cargo.toml
  - zircon_plugins/terrain/dist/src/lib.rs
  - zircon_plugins/terrain/runtime/Cargo.toml
  - zircon_plugins/terrain/runtime/src/lib.rs
  - zircon_plugins/terrain/runtime/src/capability.rs
  - zircon_plugins/texture/runtime/Cargo.toml
  - zircon_plugins/texture/runtime/src/lib.rs
  - zircon_plugins/texture/runtime/src/capability.rs
  - zircon_plugins/texture/plugin.toml
  - zircon_plugins/texture/dist/Cargo.toml
  - zircon_plugins/texture/dist/src/lib.rs
  - zircon_plugins/texture/runtime/src/plugin.rs
  - zircon_plugins/texture/runtime/src/tests.rs
  - zircon_plugins/tilemap_2d/runtime/Cargo.toml
  - zircon_plugins/tilemap_2d/runtime/src/lib.rs
  - zircon_plugins/tilemap_2d/runtime/src/capability.rs
  - zircon_plugins/virtual_geometry/runtime/Cargo.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/runtime/src/capability.rs
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/zr_vm_language/dist/Cargo.toml
  - zircon_plugins/zr_vm_language/dist/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/capability.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_plugins/native_window_hosting/editor/src/lib.rs
  - zircon_plugins/native_window_hosting/editor/src/capability.rs
  - zircon_plugins/native_window_hosting/editor/src/extension_ids.rs
  - zircon_plugins/native_window_hosting/editor/src/plugin.rs
  - zircon_plugins/native_window_hosting/editor/src/tests.rs
  - zircon_plugins/native_window_hosting/plugin.toml
  - zircon_plugins/native_window_hosting/dist/Cargo.toml
  - zircon_plugins/native_window_hosting/dist/src/lib.rs
  - zircon_plugins/runtime_diagnostics/editor/src/lib.rs
  - zircon_plugins/runtime_diagnostics/editor/src/capability.rs
  - zircon_plugins/runtime_diagnostics/editor/src/extension_ids.rs
- zircon_plugins/runtime_diagnostics/editor/src/plugin.rs
- zircon_plugins/runtime_diagnostics/editor/src/tests.rs
- zircon_plugins/runtime_diagnostics/plugin.toml
- zircon_plugins/runtime_diagnostics/dist/Cargo.toml
- zircon_plugins/runtime_diagnostics/dist/src/lib.rs
- zircon_plugins/ui_asset_authoring/editor/src/lib.rs
  - zircon_plugins/ui_asset_authoring/editor/src/capability.rs
  - zircon_plugins/ui_asset_authoring/editor/src/extension_ids.rs
  - zircon_plugins/ui_asset_authoring/editor/src/plugin.rs
  - zircon_plugins/ui_asset_authoring/editor/src/tests.rs
  - zircon_plugins/ui_asset_authoring/plugin.toml
  - zircon_plugins/ui_asset_authoring/dist/Cargo.toml
  - zircon_plugins/ui_asset_authoring/dist/src/lib.rs
  - zircon_plugins/material_editor/editor/src/lib.rs
  - zircon_plugins/material_editor/editor/src/capability.rs
  - zircon_plugins/material_editor/editor/src/extension_ids.rs
  - zircon_plugins/material_editor/editor/src/plugin.rs
  - zircon_plugins/material_editor/editor/src/tests.rs
  - zircon_plugins/material_editor/plugin.toml
  - zircon_plugins/material_editor/dist/Cargo.toml
  - zircon_plugins/material_editor/dist/src/lib.rs
  - zircon_plugins/animation_graph/editor/src/lib.rs
  - zircon_plugins/animation_graph/editor/src/capability.rs
  - zircon_plugins/animation_graph/editor/src/extension_ids.rs
  - zircon_plugins/animation_graph/editor/src/plugin.rs
  - zircon_plugins/animation_graph/editor/src/tests.rs
  - zircon_plugins/animation_graph/plugin.toml
  - zircon_plugins/animation_graph/dist/Cargo.toml
  - zircon_plugins/animation_graph/dist/src/lib.rs
  - zircon_plugins/prefab_tools/runtime/src/plugin.rs
  - zircon_plugins/prefab_tools/runtime/src/tests.rs
  - zircon_plugins/prefab_tools/editor/src/lib.rs
  - zircon_plugins/prefab_tools/editor/src/authoring.rs
  - zircon_plugins/prefab_tools/editor/src/capability.rs
  - zircon_plugins/prefab_tools/editor/src/extension_ids.rs
  - zircon_plugins/prefab_tools/editor/src/plugin.rs
  - zircon_plugins/prefab_tools/editor/src/tests.rs
  - zircon_plugins/terrain/runtime/src/plugin.rs
  - zircon_plugins/terrain/runtime/src/tests.rs
  - zircon_plugins/terrain/editor/src/lib.rs
  - zircon_plugins/terrain/editor/src/authoring.rs
  - zircon_plugins/terrain/editor/src/capability.rs
  - zircon_plugins/terrain/editor/src/extension_ids.rs
  - zircon_plugins/terrain/editor/src/plugin.rs
  - zircon_plugins/terrain/editor/src/tests.rs
  - zircon_plugins/tilemap_2d/runtime/src/plugin.rs
  - zircon_plugins/tilemap_2d/runtime/src/tests.rs
  - zircon_plugins/tilemap_2d/editor/src/lib.rs
  - zircon_plugins/tilemap_2d/editor/src/authoring.rs
  - zircon_plugins/tilemap_2d/editor/src/capability.rs
  - zircon_plugins/tilemap_2d/editor/src/extension_ids.rs
  - zircon_plugins/tilemap_2d/editor/src/plugin.rs
  - zircon_plugins/tilemap_2d/editor/src/tests.rs
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/capability.rs
  - zircon_runtime/src/ui/surface/input/editable_text.rs
  - zircon_plugins/native_dynamic_fixture/plugin.toml
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - zircon_plugins/native_dynamic_fixture/assets/shader.wgsl
  - tools/zircon_export/cli.py
  - tools/zircon_export/validate_stage.py
  - tools/zircon_export/plugin_command.py
  - tools/zircon_export/plugin_build.py
  - tools/zircon_export/plugin_build_command.py
  - tools/zircon_export/plugin_build_preflight.py
  - tools/zircon_export/plugin_build_package.py
  - tools/zircon_export/plugin_build_asset_pack.py
  - tools/zircon_export/plugin_build_signature.py
  - tools/zircon_export/plugin_package_source.py
  - tools/zircon_export/plugin_package_template.py
  - tools/zircon_export/plugin_package_identity.py
  - tools/zircon_export/plugin_validate.py
  - tools/zircon_export/plugin_validate_report.py
  - tools/zircon_export/plugin_validate_engine_version.py
  - tools/zircon_export/plugin_validate_distribution_assets.py
  - tools/zircon_export/plugin_validate_feature_provider.py
  - tools/zircon_export/plugin_validate_feature_provider_projection_compare.py
  - tools/zircon_export/tests/test_plugin_build.py
  - tools/zircon_export/tests/test_plugin_validate.py
  - tools/zircon_export/tests/test_plugin_validate_feature_provider.py
  - tools/zircon_export/tests/test_plugin_validate_distribution_modules.py
  - tools/zircon_export/tests/plugin_validate_support.py
  - tools/tests/test_zircon_build_plugin_carriers.py
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation/report.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation/schema.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/compatibility.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/registration_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/reports.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_failures.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/registration_replay.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_distribution_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/constructors.rs
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs
  - zircon_runtime/src/tests/plugin_extensions/plugin_workspace_shape.rs
  - zircon_plugins/animation/runtime/src/sequence/apply.rs
  - zircon_plugins/animation/runtime/src/sequence/target.rs
  - zircon_plugins/texture_importer/runtime/src/importers.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/p0_robustness.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/p0_robustness/priority_recommendation.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/diagnostics.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f12_dead_code.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/first_party_descriptors.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/scaffold.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/test_fixtures.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/constructor_retirement.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/private_fields.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/late_api_cleanup.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/source_inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/sources/paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/sources/reads.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/sources/budgets.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/sources/delegation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/sources/status_mirrors.rs
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/capability.rs
  - zircon_plugins/sound/runtime/src/plugin.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/descriptor.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/feature_manifest.rs
  - zircon_plugins/sound/editor/src/lib.rs
  - zircon_plugins/sound/editor/src/authoring_bindings.rs
  - zircon_plugins/sound/editor/src/capability.rs
  - zircon_plugins/sound/editor/src/extension_ids.rs
  - zircon_plugins/sound/editor/src/plugin.rs
  - zircon_plugins/sound/editor/src/tests.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/capability.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/plugin.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/tests.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/dist/Cargo.toml
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/dist/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/capability.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/plugin.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/tests.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/capability.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/plugin.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/tests.rs
  - zircon_plugins/sound/features/timeline_animation_track/dist/Cargo.toml
  - zircon_plugins/sound/features/timeline_animation_track/dist/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/capability.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/plugin.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/tests.rs
  - zircon_plugins/timeline_sequence/editor/src/lib.rs
  - zircon_plugins/timeline_sequence/editor/src/capability.rs
  - zircon_plugins/timeline_sequence/editor/src/extension_ids.rs
  - zircon_plugins/timeline_sequence/editor/src/plugin.rs
  - zircon_plugins/timeline_sequence/editor/src/tests.rs
  - zircon_plugins/timeline_sequence/dist/Cargo.toml
  - zircon_plugins/timeline_sequence/dist/src/lib.rs
  - zircon_plugins/editor_build_export_desktop/plugin.toml
  - zircon_plugins/editor_build_export_desktop/editor/src/plugin.rs
  - zircon_plugins/editor_build_export_desktop/editor/src/lib.rs
  - zircon_plugins/editor_build_export_desktop/editor/src/tests.rs
  - zircon_plugins/editor_build_export_desktop/dist/Cargo.toml
  - zircon_plugins/editor_build_export_desktop/dist/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/plugin.toml
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/tests.rs
  - zircon_plugins/plugin_sdk_examples/dist/Cargo.toml
  - zircon_plugins/plugin_sdk_examples/dist/src/lib.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/status_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/folder_backed_summary.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/typed_error.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/row_data_owner.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/row_data_owner.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/naming_guard_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/support_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/runtime_15_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/pre_runtime_15_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/line_budgets.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/status_and_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/sources.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/runtime_15_topics.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/runtime_15_expected_slice_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review_guard_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure_support_expected_slice.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/code_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/test_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/frontmatter_status.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/frontmatter_uniqueness.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/header_sections.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/plan_sources.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/child_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/status_sources.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync/source_ownership.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/nested_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_status_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_status/delegation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_status/source_ownership.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_status/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_status/budgets.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data_status/row_count.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/integrity_guards.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/layout_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/inventory_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/status_followups.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/row_data_owner.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/module_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/module_layout/guard_body.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_maps/guard_body.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/production_file_budget_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/evidence_anchor_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_row_data_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows/root_inventory_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows/owner_path_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows/root_path_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows/guard_inventory_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/base_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/code_review_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/direct_assertion_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/moved_row_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/row_data_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/status_doc_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status_docs/foundation_m2_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status_docs/child_group_status_doc_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status_docs/child_group_status_row_doc_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status_docs/child_group_moved_row_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/module_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/module_layout/base_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/module_layout/status_doc_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/module_layout/child_summary_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/module_layout/child_summary_status_doc_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/exports.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/exports/runtime_15_m3_parent.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/exports/runtime_15_parent.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/exports/top_level.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/production_guard_support.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/production_guard_runtime_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/production_guard_runtime_row_data/child_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/production_guard_runtime_row_data/export_chain.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/production_guard_runtime_row_data/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_child_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/owner_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/status_output_guard_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/status_row_base_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/production_guard_support_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/production_guard_core_and_evidence_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/production_guard_module_layout_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/production_guard_review_guard_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/production_guard_runtime_row_data_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/production_guard_status_docs_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/status_support_map_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/budgets.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_owner_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/owner_paths/m3_child_group_owner_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/owner_paths/production_guard_row_owner_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/owner_paths/production_guard_row_owner_paths/runtime_row_data_guard.rs
implementation_files:
  - docs/engine-architecture/large-file-ownership-m1.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - zircon_runtime/src/graphics/text/cache/mod.rs
  - zircon_runtime/src/graphics/text/cache/frame_dedup.rs
  - zircon_runtime/src/graphics/text/cache/layout_cache.rs
  - zircon_runtime/src/graphics/text/cache/measure_cache.rs
  - zircon_runtime/src/graphics/text/cache/shaped_cache.rs
  - zircon_runtime/src/graphics/text/shaping/mod.rs
  - zircon_runtime/src/graphics/text/layout/measure.rs
  - zircon_runtime/src/graphics/text/layout/line_break/mod.rs
  - zircon_runtime/src/graphics/text/layout/line_break/greedy.rs
  - zircon_runtime/src/graphics/text/layout/line_break/glyph_fallback.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/ellipsis.rs
  - zircon_runtime/src/ui/text/layout_engine/line_box.rs
  - zircon_runtime/src/ui/text/layout_engine/overflow_style.rs
  - zircon_runtime/src/ui/text/layout_engine/vertical.rs
  - zircon_runtime/src/ui/text/layout_engine/wrapping.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/graphics/text/cache/tests.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/surface/render/text_prewarm.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/native_bitmap_atlas/source_cache.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/tests/text_pipeline/mod.rs
  - zircon_runtime/src/ui/tests/text_pipeline/fixtures.rs
  - zircon_runtime/src/ui/tests/text_pipeline/font_registry.rs
  - zircon_runtime/src/ui/tests/text_pipeline/layout_request.rs
  - zircon_runtime/src/ui/tests/text_pipeline/measure_cache.rs
  - zircon_runtime/src/ui/tests/text_pipeline/render_extract_prewarm.rs
  - zircon_runtime/src/ui/tests/text_pipeline/surface_cache.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_chrome/fill.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_chrome/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows_tests/style.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows/root_inventory_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows/owner_path_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows/root_path_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows/guard_inventory_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/owner_paths/production_guard_row_owner_paths/core_and_evidence.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/production_guard_runtime_row_data.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/production_guard_runtime_row_data/child_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/production_guard_runtime_row_data/export_chain.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/production_guard_runtime_row_data/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/owner_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/status_output_guard_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/status_row_base_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/production_guard_support_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/production_guard_core_and_evidence_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/production_guard_module_layout_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/production_guard_review_guard_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/production_guard_runtime_row_data_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/production_guard_status_docs_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/root_paths/status_support_map_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/owner_paths/m3_child_group_owner_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/owner_paths/production_guard_row_owner_paths/runtime_row_data_guard.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate_markdown.py
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate/helpers.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate/module_doc_frontmatter.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate/output_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate/debt_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate/audit_status.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/support_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/runtime_15_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/pre_runtime_15_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/line_budgets.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/status_and_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/runtime_15_topics.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/runtime_15_expected_slice_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review_guard_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure_support_expected_slice.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/module_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/module_layout/guard_body.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_maps/guard_body.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/code_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/test_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/frontmatter_status.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/frontmatter_uniqueness.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/header_sections.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/plan_sources.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/child_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/status_sources.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync/source_ownership.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/nested_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_status_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_status/delegation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_status/source_ownership.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_status/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_status/budgets.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data_status/row_count.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/integrity_guards.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/layout_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/inventory_rows.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/status_followups.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/row_data_owner.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs
  - zircon_runtime/src/core/runtime/handle/registration/register_module.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/mod.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/types.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/multi.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/specialized.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/shutdown.rs
  - docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/zircon_plugins/13-standalone-plugin-build.md
  - docs/zircon_plugins/plugin-standalone-build.md
  - tools/zircon_export/cli.py
  - tools/zircon_export/validate_stage.py
  - tools/zircon_export/plugin_command.py
  - tools/zircon_export/plugin_build.py
  - tools/zircon_export/plugin_build_command.py
  - tools/zircon_export/plugin_build_preflight.py
  - tools/zircon_export/plugin_build_package.py
  - tools/zircon_export/plugin_build_asset_pack.py
  - tools/zircon_export/plugin_build_signature.py
  - tools/zircon_export/plugin_package_source.py
  - tools/zircon_export/plugin_package_template.py
  - tools/zircon_export/plugin_package_identity.py
  - tools/zircon_export/plugin_validate.py
  - tools/zircon_export/plugin_validate_report.py
  - tools/zircon_export/plugin_validate_engine_version.py
  - tools/zircon_export/tests/test_plugin_build.py
  - tools/zircon_export/tests/test_plugin_validate.py
  - tools/tests/test_plugin_standalone_ci_matrix.py
  - .github/workflows/ci.yml
  - zircon_runtime/src/plugin/native_plugin_loader/compatibility.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_distribution_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/constructors.rs
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs
  - zircon_runtime/src/tests/plugin_extensions/plugin_workspace_shape.rs
  - zircon_plugins/audio_importer/runtime/src/lib.rs
  - zircon_plugins/audio_importer/runtime/src/capability.rs
  - zircon_plugins/audio_importer/runtime/src/plugin.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/audio_importer/dist/Cargo.toml
  - zircon_plugins/audio_importer/dist/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/capability.rs
  - zircon_plugins/gltf_importer/runtime/src/plugin.rs
  - zircon_plugins/gltf_importer/plugin.toml
  - zircon_plugins/gltf_importer/dist/Cargo.toml
  - zircon_plugins/gltf_importer/dist/src/lib.rs
  - zircon_plugins/obj_importer/plugin.toml
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/capability.rs
  - zircon_plugins/obj_importer/runtime/src/plugin.rs
  - zircon_plugins/obj_importer/dist/Cargo.toml
  - zircon_plugins/obj_importer/dist/src/lib.rs
  - zircon_plugins/opus_importer/runtime/src/lib.rs
  - zircon_plugins/opus_importer/runtime/src/capability.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/capability.rs
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/texture_importer/runtime/src/lib.rs
  - zircon_plugins/texture_importer/runtime/src/capability.rs
  - zircon_plugins/texture_importer/runtime/src/plugin.rs
  - zircon_plugins/texture_importer/dist/Cargo.toml
  - zircon_plugins/texture_importer/dist/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/capability.rs
  - zircon_plugins/asset_importers/data/runtime/src/lib.rs
  - zircon_plugins/asset_importers/data/runtime/src/capability.rs
  - zircon_plugins/asset_importers/data/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/data/plugin.toml
  - zircon_plugins/asset_importers/data/dist/Cargo.toml
  - zircon_plugins/asset_importers/data/dist/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/capability.rs
  - zircon_plugins/asset_importers/shader/runtime/src/lib.rs
  - zircon_plugins/asset_importers/shader/runtime/src/capability.rs
  - zircon_plugins/asset_importers/audio/runtime/Cargo.toml
  - zircon_plugins/asset_importers/audio/runtime/src/lib.rs
  - zircon_plugins/asset_importers/audio/runtime/src/capability.rs
  - zircon_plugins/asset_importers/audio/runtime/src/plugin.rs
  - zircon_plugins/asset_importers/texture/runtime/Cargo.toml
  - zircon_plugins/asset_importers/texture/plugin.toml
  - zircon_plugins/asset_importers/texture/dist/Cargo.toml
  - zircon_plugins/asset_importers/texture/dist/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/lib.rs
  - zircon_plugins/asset_importers/texture/runtime/src/capability.rs
  - zircon_plugins/asset_importers/texture/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/solari/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/zr_vm_language/dist/Cargo.toml
  - zircon_plugins/zr_vm_language/dist/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/navigation/dist/Cargo.toml
  - zircon_plugins/navigation/dist/src/lib.rs
  - zircon_plugins/navigation/runtime/src/plugin.rs
  - zircon_plugins/navigation/runtime/src/tests/registration.rs
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/physics/dist/Cargo.toml
  - zircon_plugins/physics/dist/src/lib.rs
  - zircon_plugins/physics/runtime/src/plugin.rs
  - zircon_plugins/physics/runtime/src/tests.rs
  - zircon_plugins/particles/plugin.toml
  - zircon_plugins/particles/dist/Cargo.toml
  - zircon_plugins/particles/dist/src/lib.rs
  - zircon_plugins/particles/runtime/src/plugin.rs
  - zircon_plugins/particles/runtime/src/tests/package_manifest.rs
  - zircon_plugins/animation/plugin.toml
  - zircon_plugins/animation/dist/Cargo.toml
  - zircon_plugins/animation/dist/src/lib.rs
  - zircon_plugins/animation/runtime/src/plugin.rs
  - zircon_plugins/animation/runtime/src/tests.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/dist/Cargo.toml
  - zircon_plugins/hybrid_gi/dist/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/plugin.rs
  - zircon_plugins/hybrid_gi/runtime/src/tests.rs
  - zircon_plugins/native_window_hosting/editor/src/lib.rs
  - zircon_plugins/native_window_hosting/editor/src/capability.rs
  - zircon_plugins/native_window_hosting/editor/src/extension_ids.rs
  - zircon_plugins/native_window_hosting/editor/src/plugin.rs
  - zircon_plugins/native_window_hosting/editor/src/tests.rs
  - zircon_plugins/native_window_hosting/plugin.toml
  - zircon_plugins/native_window_hosting/dist/Cargo.toml
  - zircon_plugins/native_window_hosting/dist/src/lib.rs
  - zircon_plugins/runtime_diagnostics/editor/src/lib.rs
  - zircon_plugins/runtime_diagnostics/editor/src/capability.rs
  - zircon_plugins/runtime_diagnostics/editor/src/extension_ids.rs
- zircon_plugins/runtime_diagnostics/editor/src/plugin.rs
- zircon_plugins/runtime_diagnostics/editor/src/tests.rs
- zircon_plugins/runtime_diagnostics/plugin.toml
- zircon_plugins/runtime_diagnostics/dist/Cargo.toml
- zircon_plugins/runtime_diagnostics/dist/src/lib.rs
- zircon_plugins/ui_asset_authoring/editor/src/lib.rs
  - zircon_plugins/ui_asset_authoring/editor/src/capability.rs
  - zircon_plugins/ui_asset_authoring/editor/src/extension_ids.rs
  - zircon_plugins/ui_asset_authoring/editor/src/plugin.rs
  - zircon_plugins/ui_asset_authoring/editor/src/tests.rs
  - zircon_plugins/ui_asset_authoring/plugin.toml
  - zircon_plugins/ui_asset_authoring/dist/Cargo.toml
  - zircon_plugins/ui_asset_authoring/dist/src/lib.rs
  - zircon_plugins/material_editor/editor/src/lib.rs
  - zircon_plugins/material_editor/editor/src/capability.rs
  - zircon_plugins/material_editor/editor/src/extension_ids.rs
  - zircon_plugins/material_editor/editor/src/plugin.rs
  - zircon_plugins/material_editor/editor/src/tests.rs
  - zircon_plugins/material_editor/plugin.toml
  - zircon_plugins/material_editor/dist/Cargo.toml
  - zircon_plugins/material_editor/dist/src/lib.rs
  - zircon_plugins/animation_graph/editor/src/lib.rs
  - zircon_plugins/animation_graph/editor/src/capability.rs
  - zircon_plugins/animation_graph/editor/src/extension_ids.rs
  - zircon_plugins/animation_graph/editor/src/plugin.rs
  - zircon_plugins/animation_graph/editor/src/tests.rs
  - zircon_plugins/animation_graph/plugin.toml
  - zircon_plugins/animation_graph/dist/Cargo.toml
  - zircon_plugins/animation_graph/dist/src/lib.rs
  - zircon_plugins/prefab_tools/runtime/src/plugin.rs
  - zircon_plugins/prefab_tools/runtime/src/tests.rs
  - zircon_plugins/prefab_tools/editor/src/lib.rs
  - zircon_plugins/prefab_tools/editor/src/authoring.rs
  - zircon_plugins/prefab_tools/editor/src/capability.rs
  - zircon_plugins/prefab_tools/editor/src/extension_ids.rs
  - zircon_plugins/prefab_tools/editor/src/plugin.rs
  - zircon_plugins/prefab_tools/editor/src/tests.rs
  - zircon_plugins/terrain/runtime/src/plugin.rs
  - zircon_plugins/terrain/runtime/src/tests.rs
  - zircon_plugins/terrain/plugin.toml
  - zircon_plugins/terrain/dist/Cargo.toml
  - zircon_plugins/terrain/dist/src/lib.rs
  - zircon_plugins/terrain/editor/src/lib.rs
  - zircon_plugins/terrain/editor/src/authoring.rs
  - zircon_plugins/terrain/editor/src/capability.rs
  - zircon_plugins/terrain/editor/src/extension_ids.rs
  - zircon_plugins/terrain/editor/src/plugin.rs
  - zircon_plugins/terrain/editor/src/tests.rs
  - zircon_plugins/tilemap_2d/runtime/src/plugin.rs
  - zircon_plugins/tilemap_2d/runtime/src/tests.rs
  - zircon_plugins/tilemap_2d/editor/src/lib.rs
  - zircon_plugins/tilemap_2d/editor/src/authoring.rs
  - zircon_plugins/tilemap_2d/editor/src/capability.rs
  - zircon_plugins/tilemap_2d/editor/src/extension_ids.rs
  - zircon_plugins/tilemap_2d/editor/src/plugin.rs
  - zircon_plugins/tilemap_2d/editor/src/tests.rs
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/capability.rs
  - zircon_plugins/sound/runtime/src/plugin.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/descriptor.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/feature_manifest.rs
  - zircon_plugins/sound/editor/src/lib.rs
  - zircon_plugins/sound/editor/src/authoring_bindings.rs
  - zircon_plugins/sound/editor/src/capability.rs
  - zircon_plugins/sound/editor/src/extension_ids.rs
  - zircon_plugins/sound/editor/src/plugin.rs
  - zircon_plugins/sound/editor/src/tests.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/capability.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/plugin.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/tests.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/capability.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/plugin.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/tests.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/capability.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/plugin.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/tests.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/capability.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/plugin.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/tests.rs
  - zircon_plugins/timeline_sequence/editor/src/lib.rs
  - zircon_plugins/timeline_sequence/editor/src/capability.rs
  - zircon_plugins/timeline_sequence/editor/src/extension_ids.rs
  - zircon_plugins/timeline_sequence/editor/src/plugin.rs
  - zircon_plugins/timeline_sequence/editor/src/tests.rs
  - zircon_plugins/timeline_sequence/dist/Cargo.toml
  - zircon_plugins/timeline_sequence/dist/src/lib.rs
  - zircon_plugins/editor_build_export_desktop/plugin.toml
  - zircon_plugins/editor_build_export_desktop/editor/src/plugin.rs
  - zircon_plugins/editor_build_export_desktop/editor/src/lib.rs
  - zircon_plugins/editor_build_export_desktop/editor/src/tests.rs
  - zircon_plugins/editor_build_export_desktop/dist/Cargo.toml
  - zircon_plugins/editor_build_export_desktop/dist/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/plugin.toml
  - zircon_plugins/plugin_sdk_examples/editor/src/plugin.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/tests.rs
  - zircon_plugins/plugin_sdk_examples/dist/Cargo.toml
  - zircon_plugins/plugin_sdk_examples/dist/src/lib.rs
plan_sources:
  - user: 2026-06-22 优化 docs/plans editor/runtime/plugins 计划，统一代码结构与插件接口开发体验框架
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/engine-architecture/large-file-ownership-m1.md
  - docs/engine-architecture/runtime-interface-convergence.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/code_paths.rs::runtime_15_priority_plan_docs_code_paths_stay_current
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/test_paths.rs::runtime_15_priority_plan_docs_test_paths_stay_current
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/frontmatter_status.rs::runtime_15_priority_plan_docs_frontmatter_status_stays_current
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/frontmatter_uniqueness.rs::runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/header_sections.rs::runtime_15_priority_plan_docs_required_header_sections_stay_complete
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/plan_sources.rs::runtime_15_priority_plan_docs_plan_sources_stay_cross_linked
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs::runtime_15_priority_plan_docs_guard_tests_stay_listed
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/child_layout.rs::runtime_15_priority_plan_docs_guard_children_are_folder_backed
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_children_are_folder_backed
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_child_prose_names_full_inventory
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_guard_paths_stay_current
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory
  - tests/acceptance/runtime-priority-plan-output-archive-ownership.md
  - "请将产出记录放置在子计划中，子计划中记录超过10条则全部放到子目录，此处仅展示当前现状的概述"
  - "具体测试、验证与产出明细已迁入 docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"
doc_type: convention-authority
status: in_progress
---

# 引擎级代码结构与模块接口规范（Engine Code Structure & Module Interface Convention）

> 规范权威：跨域通用规则已统一收敛至 [Zircon 开发规范总纲](zircon_runtime/frameworks/development-conventions.md)；本文保留代码结构主题的细节论证与执行上下文，不再作为并列规则源。

## 产出记录迁移说明

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

具体结构补记、验证与修复记录已迁入 [`zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md`](zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md)。本文件仅保留结构规范、接口约定与当前现状概述。

当前 plan-status 结构同步（2026-07-10）：具体状态记录已硬切到 `zircon_runtime/runtime/01/` 至 `15/` 编号归档，父计划和总索引不再复制历史五列表格。测试支持按职责拆为 `plan_status/support/runtime_plan_archives.rs` 与 `plan_status/recent_static_guards/parent_routing.rs`，所有 owner 文件保持各自预算；Python boundary support 84/84、`risks = []`，standalone Rust plan-status 48/48。该结构同步没有恢复旧路径、兼容 facade、shim 或 re-export。

当前文本计划同步（2026-07-10）：rich/vertical prewarm、横向亚像素 cache identity、raster/upload report、backend face-ID authority、CompositeFont fallback span 与 run-language cache identity 均已收束到各自 child owner；renderer root 保持编排职责并低于软预算。真实 runtime UI → screen-space text → WGPU readback 的产品帧已关闭 CJK/RTL/彩色 Emoji/native/SDF、color-face RGBA、zh-Hans/ja 同码点、Arabic mark complex-cluster 单 face 与 VerticalRl CJK 单列首段。系统 `fontdb` face 的容器字节由 `FontDatabase` 根据权威 backend ID 物化，再由既有 source owner 提取独立 SFNT；诊断期 coverage→SDF 第二策略已删除。竖排继续复用 `graphics/text/shaping/vertical/orientation.rs`，SDF consumer 只承接 destination axes/UV/main-axis advance，没有在 renderer root 建第二套 Unicode 或字体策略；live typography 与真实 scroll 增量仍按子计划保持 open。具体命令、日志、哈希和 framebuffer 明细不在本规范复制。

## §0 适用范围与背景

适用 `zircon_runtime` / `zircon_editor` / `zircon_plugins` / `zircon_runtime_interface` / `zircon_app` / `zircon_hub`，含 render 子计划覆盖的 `graphics/**`。

引擎已非常庞大，分层方向合理，但模块内细节散乱，直接损害用户 code review / inspect 体验，集中表现在六类结构债：模块布局不统一、命名失序、巨型文件、公共 API 不友好、测试组织三套并行、插件 DX 割裂（证据见各落地子计划"现状缺口表"）。

本规范复用仓库既有的结构治理范式——`audit_runtime_structure.py` + `runtime_structure_audits/*.py` 审计脚本族、`tests/runtime_absorption/**` guard 测试、`docs/**` 镜像文档、`large_file_ownership_gate`（`m1_gate_status` / owner 分类 / 迁移债）——把"统一模块规范 + 插件 DX 框架"做成同形门禁，对**后续开发**与**存量重构**均可机器化强制、可验收、可防回归。

## §1 模块文件布局标准（owner-module 模式）

| 规则 | 定稿 | 反例（实证） |
|---|---|---|
| R1.1 根接线薄 façade | `lib.rs` / `mod.rs` 仅含 `mod` 声明 + 精选 `pub use` + 模块 doc，**零行为** | `plugin/mod.rs`（74 行扁平导出） |
| R1.2 `mod.rs` vs `module.rs` | 目录统一用 `mod.rs` 作 façade；引擎子系统的 `ModuleDescriptor` 单独放 `module.rs`，**且仅注册子系统**才有 `module.rs` | `input/mod.rs` 直塞注册 / `graphics/` 注册分散无 `module.rs` |
| R1.3 行为落具名 owner 叶子 | 行为进 `lifecycle.rs` / `dispatch.rs` / `validation.rs` / `diagnostics.rs` / `conversion.rs` / `extract.rs` 等具名模块，不堆胖单文件 | `dynamic_api/session.rs`（773 行协调 17 子模块） |
| R1.4 行数预算 | 生产文件软上限 800（review 警告）、硬上限 1000（gate 拦截）；测试文件 > 800 必须 folder-backed 拆分；豁免（vendored upstream、fixture、`@generated`）须在 gate `exempt` 字段登记 | `core/framework/tests.rs`（1848） |
| R1.5 嵌套深度 | 同一分类维度 ≤ 3 层；域重叠（如 `asset/assets/scene/animation/`）须拍平 | `asset/assets/scene/` 深嵌 |

**`module.rs` 存在判据**：当且仅当该目录对应一个会向 runtime/editor 注册的 `*Module`（拥有 `module_descriptor()`）。否则不得出现 `module.rs`。

owner 拆分纪律继承 `large-file-ownership-m1.md`：**按 ownership 拆，不按等行数切**；root 文件可作结构 façade，但不得为避免改调用方而保留行为；拆分时不留兼容 wrapper，消费方直接调用新 owner 路径或精选 façade。

## §2 命名规范

- **R2.1 复数 / 单数判定**：目录名是"其下每个文件都是它的一个实例"的**种类** → 复数（`components/`、`assets/`、`importers/`、`systems/`、`effects/`）；目录是"单一内聚子系统 / owner" → 单数（`manager/`、`dispatch/`、`pipeline/`、`backend/`、`layout/`）。判定测问句："该目录名是不是一类东西、其下每个文件是它的一个？"是则复数，否则单数。
- **R2.2 前缀允许词表**（其余前缀视为命名债）：
  - `runtime_`：**仅**当与 authoring/descriptor 孪生对比时（`runtime_asset_path` vs `asset_path`）。**禁止**当通用命名空间标签——已在 runtime crate / `*_runtime_provider` 目录内的模块不得再冠 `runtime_`（`hybrid_gi_runtime_provider/runtime_state.rs` → `state.rs`）。
  - `default_`（默认实现）、`builtin_`（内置目录 / 枚举）、`compiled_`（编译后产物）、`frozen_`（冻结表）。
- **R2.3 禁用无主名**：模块名禁用 `_inner` / `_impl` / `_helper` / `util(s)` / `misc` / `common`；改成描述其 owns 什么的名字（例如旧 `editor_event_runtime_inner.rs` 已按职责硬切为 `editor_event_runtime_state.rs`，旧 `core/runtime/state/runtime_inner.rs` 已按职责硬切为 `core/runtime/state/core_runtime_state.rs`，旧 `scene/ecs/observer/utils.rs` 已按职责硬切为 `scene/ecs/observer/callback_registry.rs`，旧 `scene/ecs/query/query_state/helpers.rs` 已按职责硬切为 `scene/ecs/query/query_state/many_item_array.rs`，旧 `scene/ecs/storage/component_storage/utils.rs` 已按职责硬切为 `scene/ecs/storage/component_storage/component_results.rs`，旧 `asset/watch/drop_impl.rs` 已按职责硬切为 `asset/watch/shutdown_on_drop.rs`，旧 `core/framework/camera_controller/common.rs` 已按职责硬切为 `core/framework/camera_controller/controller_output.rs`，旧 `asset/tests/assets/texture_upload_readiness/common.rs` 已按职责硬切为 `asset/tests/assets/texture_upload_readiness/container_fixtures.rs`）。
- **R2.4 文件名 snake_case**（已普遍满足，纳入审计兜底）。
- **R2.5 构造目录命名**：放构造逻辑的目录用 `construct` / `construction` / `builder`，**禁用 `*_new` 后缀和裸 `new` owner**；具体 hard-cutover 记录由 Runtime 15 产出目录维护。

## §3 公共 API 与"用户友好的模块化接口"

- **R3.1 精选 façade**：子系统 `mod.rs` 只 re-export 小而有意的公共集，**分组 + 每组注释**；façade 的 `pub use` 行数纳入审计（软阈值，超阈值要求改 prelude / 分组），禁止 100 符号扁平 dump。
- **R3.2 禁 glob 出口**：子系统 / crate façade 禁止 `pub use x::*`（隐藏 surface）；owner 组内小范围分组显式 re-export 允许。
- **R3.3 prelude 分层**：**子系统级 prelude 为主**——`<crate>::<subsystem>::prelude`（如 `zircon_runtime::asset::prelude`）是消费者主入口；**crate 级 `prelude` 仅聚合**——只 re-export 各子系统 prelude 的跨子系统高频集，不直接列符号。分工：façade(`mod.rs`)=完整公共面、prelude=高频常用面；prelude 也设符号预算，防退化成第二个 dump。插件 crate 经 `lib.rs` 暴露公共 API，体量大时才加 prelude。
- **R3.4 可见性纪律**：模块非 `pub`（公共 API）即 `pub(crate)` / `pub(super)`（实现）；同一 `mod.rs` 不得无规则混排 `pub mod` / `pub(crate) mod`——若混排，façade 注释须显式标注公共集边界；稳定公共项带 doc 注释。

### 范式：巨型扁平 façade → 分组 façade + prelude（`asset/mod.rs`）
**前**：`pub use assets::{ ...100+ 符号一坨... };`
**后**：
```rust
// asset/mod.rs —— 精选 façade（完整公共面，分组）
pub mod prelude;

// —— 资产类型 ——
pub use assets::{MeshAsset, MaterialAsset, TextureAsset, SceneAsset, ModelAsset,
    UiWidgetAsset, VirtualGeometryAsset /* … */};
// —— 导入 / 校验 ——
pub use assets::{asset_kind_for_imported_asset, validate_sprite_atlas_asset,
    validate_wgsl_captures /* … */};
```
```rust
// asset/prelude.rs —— 高频常用面（设符号预算，不得扩成第二个 dump）
pub use super::{Assets, AssetManager, ProjectAssetManager,
    MeshAsset, MaterialAsset, TextureAsset, SceneAsset};
```
crate 级 `zircon_runtime::prelude` 收窄为 `pub use crate::{asset::prelude::*, scene::prelude::*, ui::prelude::*, ...}`，不再直接列子系统符号。

## §4 测试组织（单一规则）

- **R4.1**：单文件小测（< ~150 行测试）→ 内联 `#[cfg(test)] mod tests`。
- **R4.2**：更大 / 行为测试 → folder-backed `tests/` 镜像源树、按行为族分文件。
- **R4.3**：禁止 > 800 行 `tests.rs`；禁止重复测试树（如 editor `src/tests/**` 镜像 `src/ui/**` 双写）——一个行为一个 owner。
- **R4.4**：跨 crate 集成测试归 crate `tests/`；测试命名按所属子系统过滤词前缀（沿用 `render_*`、`runtime_*` guard 命名惯例），便于 milestone 末按过滤词收窄。

## §5 资源 / 描述 / manifest 放置

- **R5.1**：出货资产 → crate `assets/`（staged build 已合并 editor / runtime 两 `assets/`）；测试 fixture → owner 模块 `tests/fixtures/` 或 `<module>/tests/assets/`，禁散落。
- **R5.2**：插件 manifest → crate 根 `plugin.toml`（强制、统一 schema，见 §6.2）。
- **R5.3**：每个 descriptor 家族（`.zui` / `.zmaterial` / `.zasset` / `plugin.toml`）有唯一 schema owner 文件，reviewer 可定位 schema 权威。`.zui` 是唯一 UI asset descriptor 家族；`.ui.toml` / `.v2.ui.toml` 已退役，不得作为当前 UI layout/asset schema owner 回流。`page_templates.toml`、`shell_regions.toml` 与 `presets.toml` 属 typed editor layout metadata，不是 UI asset descriptor family，只能引用 `.zui` UI 资产。

## §6 统一插件接口开发体验框架（Plugin DX）

### §6.1 唯一插件 crate 骨架（template）
```
<plugin>/
  plugin.toml          # 统一 schema（强制，见 §6.2）
  runtime/
    Cargo.toml
    src/
      lib.rs             # 薄：pub use 公共 API + 导出 Plugin struct + 常量
      plugin.rs          # 唯一注册 owner：impl RuntimePlugin + descriptor()
      capability.rs      # capability id pub const —— 单一来源
      contract/          # 该插件 ABI-safe DTO（纯消费 interface 则省略）
      backend/           # 实际算法 / 协议实现 owner（按 §1 拆叶子）
      systems/           # 注册进调度图的 ECS 系统
      tests/
  editor/                # 镜像同骨架（能力对称：plugin.rs / capability.rs / ...）
    Cargo.toml
    src/{lib.rs, plugin.rs, capability.rs, ...}
```
- 导入器类插件：`backend/` 即 importer 实现，`plugin.rs` 的 `register` 同时 `register_module` + 注册 importer descriptor；退役 `asset_importers/*/registration.rs` 的自由函数分离。
- native-dynamic 插件：`plugin.toml` 显式区分 runtime / editor 两 `[[modules]]` 的 `crate_name`，禁止两 module 指向同名 crate 却不以 `kind` 区分。

### §6.2 统一 `plugin.toml` schema（canonical）
唯一 schema owner：`docs/zircon_plugins/plugin-manifest-schema.md`（含校验器契约）。必选 / 可选段固定形状，使 30 行与 105 行插件共享骨架：

```toml
# —— 必选头 ——
id = "<plugin>"                       # 与 capability 前缀一致
version = "0.1.0"
sdk_api_version = "0.1.0"
display_name = "..."
category = "runtime|asset_importer|editor|..."
description = "..."
supported_targets = ["client_runtime", "editor_host", ...]
supported_platforms = ["windows", "linux", "macos"]
capabilities = ["runtime.plugin.<plugin>", ...]   # 与 capability.rs 单源核对
maturity = "stable|beta|experimental"

# —— 必选模块声明（每个 crate 一条）——
[[modules]]
name = "<plugin>.runtime"             # <plugin>.{runtime|editor}
kind = "runtime|editor"
crate_name = "zircon_plugin_<plugin>_runtime"
target_modes = [...]
capabilities = [...]
system_anchors = [...]                # 与实际注册的 system 源核对

# —— 可选段（按需，schema 固定形状）——
[[capability_statuses]]   capability = "..."  status = "partial|stable"
[[asset_importers]]       id = "..."  source_extensions = [...]  output_kind = "..."  ...
[[optional_features]]     id = "..."
[[dependencies]]          id = "..."
[[options]]               key = "..."  ...
[[event_catalogs]]        ...
```
规则：① 所有插件（含 `asset_importers/*`、native）必须有 `plugin.toml`；② `capabilities` 与 `capability.rs` 常量集双向一致（审计）；③ 可选段缺省即省略，不得改变必选段形状。

### §6.3 唯一注册入口
仅经 `impl RuntimePlugin::register(&self, registry)`（Plugins 01 已硬切运行时插件 trait 到唯一 `register`）；editor 侧经对称 editor plugin trait。自由函数注册收编进 `plugin.rs` 的 `register`。运行时模块与 system 注册优先走 `plugin_sdk::registration::RuntimePluginRegistrationBuilder`，由 SDK 封装 owner token 传递与注册顺序。标准 runtime crate helper 不得逐插件手写复制；新代码使用 `zircon_plugin_sdk::runtime_plugin_exports!(PluginType)`。

当前插件架构同步（2026-07-10，`plugins_01_m2_t2_t4_typed_extension_freeze_runtime_finalize`）：`TypedExtensionPoint` 冻结与 owner 撤销回归放在独立 `extension_registry_typed_points.rs` 测试 owner，生产实现继续由 `extension_registry/` 子模块承接；未向 `plugin/mod.rs` 堆入行为，也未增加兼容 facade、re-export shim 或双轨入口。详细完成项与验证归档到 Plugins 01 和 runtime extension registry 模块文档。插件架构整体状态：进行中。

> 请将产出记录放置在子计划中，子计划中记录超过10条则全部放到子目录，此处仅展示当前现状的概述

历史迁移批次、验证与状态记录已迁入 Runtime 15 产出目录。

- 迁入记录：[`zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md`](zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md)

### §6.4 capability 单源 + 四源一致性
capability id 为 `capability.rs` 的 `pub const`；guard 测试交叉核对四源：`capability.rs` 常量 ↔ `plugin.toml capabilities` ↔ runtime descriptor ↔ workspace member。扩展现有 `declared_system_anchors_are_registered` 同款模式到 capability。

> 请将产出记录放置在子计划中，子计划中记录超过10条则全部放到子目录，此处仅展示当前现状的概述

历史迁移批次、计数与验证记录已迁入 Runtime 15 产出目录。

- 迁入记录：[`zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md`](zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md)

### §6.5 `plugin_sdk` builder（祝福路径）
把 `plugin_sdk_examples` 固化为模板与 builder/test fixture API，新插件 ≈ 一文件声明（descriptor + capability + systems 注册），runtime system 注册通过 `plugin_sdk::registration` 隐藏 owner token 样板，runtime helper exports 通过 `plugin_sdk::runtime_plugin_exports!` 投影 trait-backed manifest/selection/registration，optional feature 能力通过 `PluginFeatureBundleBuilder` 同源投影 feature/module capabilities，editor/runtime 对称通过 `EditorPluginDeclaration::mirrors_runtime(...)` 显式声明，跨插件测试通过 `plugin_sdk::test::TestRuntime::builder()` 复用 runtime/scene/fixed-step 启动样板。

### §6.6 双形态独立构建（发行维扩展）
由 [Plugins 13](zircon_plugins/13-standalone-plugin-build.md) 落地、规范权威 [`docs/zircon_plugins/plugin-standalone-build.md`](../zircon_plugins/plugin-standalone-build.md)。在 §6.1 骨架上扩"发行维"：每个插件一份声明投影两形态——`embed`（`rlib`，静态链接、`impl RuntimePlugin::register`）与 `dist`（`cdylib`，ABI-only、`zircon_native_plugin_descriptor_v3` 导出），二者共享 `backend/` 纯逻辑不复制。**依赖边界铁律**：`dist` 产物依赖闭包禁含 `zircon_runtime`（与 §7.5 E8 同源），`backend/`/`capability.rs` 禁 `use zircon_runtime::*`，触碰 `zircon_runtime` 的代码一律 `#[cfg(feature = "embed")]`；由 `tools/plugin_structure_audits/dependency_boundary.py` 的 `dist_dependency_boundary_violations` 守卫。

> 请将产出记录放置在子计划中，子计划中记录超过10条则全部放到子目录，此处仅展示当前现状的概述

具体 rollout、构建、验证、计数与阻塞记录已迁入 Runtime 15 产出目录；本节仅保留双形态独立构建规范。

- 迁入记录：[`zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md`](zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md)

### 范式：插件 crate 骨架化（`asset_importers/model`）
**前**（无 `plugin.toml`，注册在自由函数）：`src/{lib.rs(re-export), registration.rs(161 行: descriptors + manifest + plugin_registration 自由函数), mesh_importer.rs, cad.rs, tests/}`
**后**：
```
asset_importers/model/runtime/
  plugin.toml          # 新增，统一 schema
  src/
    lib.rs             # 薄：pub use + ModelImporterRuntimePlugin
    plugin.rs          # impl RuntimePlugin::register（自由函数收编于此）
    capability.rs      # pub const RUNTIME_CAPABILITY = "runtime.plugin.asset_importer.model";
    backend/ mesh_importer.rs  cad.rs
    tests/
```

## §7 强制机制（对后续开发与存量重构同时生效）

1. **审计脚本族**：runtime 进现有 `runtime_structure_audits/`（`module_convention_gate.py` + `module_convention_gate_markdown.py`，由 `audit_runtime_structure.py` 聚合）；editor / plugins 各新建 owner 域同级目录 `editor_structure_audits/` / `tools/plugin_structure_audits/`（与 `runtime_structure_audits/` 平级），各带 `audit_editor_structure.py` / `tools/audit_plugin_structure.py` 聚合器。
2. **guard 测试**：runtime `tests/runtime_absorption/structure_convention.rs`、editor `zircon_editor/src/tests/structure_convention/`、plugins workspace guard——断言审计字段与镜像文档计数一致。
3. **owner-class gate**：`module_convention_gate` / `plugin_skeleton_gate` 报告 `m1_gate_status` ∈ {`migration-debt-present`, `classified-and-clear`}，含 `classification_counts` 与 `migration_debt_count`（目标 → 0），`exempt` 字段登记豁免。
4. **镜像文档**：`docs/**/structure/*.md` 计数须与审计一致，由 `*_mirror_docs_match_structure_audit_counts` 守卫锁定。
5. **硬切纪律**：新 owner 路径落地的同一变更内迁移调用方并删除旧路径，不留 re-export / shim / 双轨；grep 旧符号零命中。
6. **milestone-first 验收**：切片期轻量 `cargo check`，里程碑末进测试 + `cargo fmt --all --check` + 运行对应 `audit_*_structure.py --json`。

每条规则都映射到某审计字段（façade 行数→`oversized_facade_files`、前缀→`prefix_vocabulary_violations`、骨架→`skeleton_conformance`、capability 单源→`capability_source_mismatches`…），确保"可验收"非空话。

## §7.5 错误处理与反重复约定（2026-06 审查并入）

来自 [`engine-code-review-findings-2026-06.md`](engine-code-review-findings-2026-06.md) 的规范级结论。具体闭合状态、验证、计数与守卫记录已迁入 Runtime 15 产出目录。

> 请将产出记录放置在子计划中，子计划中记录超过10条则全部放到子目录，此处仅展示当前现状的概述

- 迁入记录：[`zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md`](zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md)

- **E1 typed error 优先**：跨模块公共 API 返回 typed error，不用裸 `String` 或 `format!()` 压扁 source。
- **E2 getter / resolver 命名**：`get_*` 表示 optional lookup，`resolve_*` 留给 fallible/result lookup。
- **E3 builder infallible**：链式 `with_*` 一律返 `Self`；校验移到 `build()` / `finish()`，可失败应用/解析入口使用非 builder 动词。
- **E4 镜像文档**：`docs/**/structure/*.md` 计数须与审计一致，并由镜像文档守卫锁定。
- **E5 反重复样板**：近似复制的并列模块必须抽泛型、宏或 derive；diagnostics 类结构统一 trait + 子域组合。
- **E6 `#[allow(dead_code)]` 限制**：生产代码禁止长期掩盖未接线脚手架或僵尸；要么接线，要么删除。
- **E7 FFI panic 边界**：所有 `extern "C"` 边界必须包裹 panic guard，panic 转状态码，不跨 FFI。
- **E8 边界依赖白名单**：`zircon_runtime_interface` 禁 wgpu/slint/winit/tokio；`zircon_editor`/`zircon_app` 允许窗口/事件循环依赖但禁 graphics backend 泄漏。
- **E9 生产锁 poison 处理**：运行时生产共享状态不得直接 `.lock().unwrap()`；infallible owner API 通过集中 helper 恢复，fallible API 返回 typed error。
- **E10 可失败 render submit 降级**：`submit_frame_extract` production paths must return `RenderFrameworkError`；viewport/provider 缺口不得用裸 `.unwrap(`/`.expect(` 维持不变量。

## §8 各计划集落地索引

| 计划集 | 落地 | 范围 |
|---|---|---|
| Runtime | `zircon_runtime/runtime/15-code-structure-and-module-conventions.md` | runtime 全模块 + graphics（render 子计划引用本文 §1/§5） |
| Editor UI | `zircon_editor/editor_ui/10-code-structure-and-module-conventions.md` | editor `core/scene/ui` |
| Plugins | `zircon_plugins/12-plugin-dx-and-structure-framework.md` | 全插件 DX + manifest/骨架/注册/capability |
| Plugins 发行 | `zircon_plugins/13-standalone-plugin-build.md` | 双形态独立构建 + 依赖边界 + 注册跨 ABI 编组 + per-plugin 动态包 |
| Render | `zircon_runtime/render/index.md`「代码结构规范」节 | graphics 热点纳入 Runtime 15 + `large_file_ownership_gate` |
| Hub | `zircon_hub/index.md`「代码结构规范」节 | Hub 巨型文件 + 前端组件化纳入本文 §1/§3/§4 |
| 审查发现目录 | `engine-code-review-findings-2026-06.md` | F1–F19 + D 系列 DX 发现，分派到各计划 |


## Runtime 15 M3 Review-Guard Row-Data Routing

Runtime 15 M3 review-guard row-data 的具体 cross-doc 与 supplemental anchors 已迁入 [`zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md`](zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md)，共享 current-owner inventory 由 [`zircon_runtime/runtime/15/2026-07-10-priority-plan-doc-current-owner-inventory.md`](zircon_runtime/runtime/15/2026-07-10-priority-plan-doc-current-owner-inventory.md) 持有；本文件继续只持有结构规范、接口约定与当前现状概述。

## 2026-07-13 M4 behavior postprocess tests owner split hard-cut note

- M4 behavior postprocess tests owner split 继续执行 §4 单一规则：`graphics/tests/m4_behavior_layers.rs` 只保留 shared fixture、offline-bake coverage 与 child mounts，`graphics/tests/m4_behavior_layers/postprocess.rs` 持有 bloom/color-grading 产品测试。
- 守卫 `runtime_15_m4_behavior_postprocess_tests_are_child_owner` 锁定父/子 800 行预算；旧测试或 helper 不得回流父文件。具体状态与验证证据只由 Plan09 / Render 编号归档持有，本总览不复制 machine status token。

## 2026-07-10 Runtime Text UAX#9 line-owner hard cut note

- The existing Runtime 15 visual-order child-owner split remains intact: `ui/text/layout_engine/visual_order.rs` is still the narrow adapter owned below `layout_engine.rs` and is now 178 lines.
- Algorithm authority is hard-cut to `graphics/text/shaping/bidi.rs`; the UI child no longer owns ASCII/RTL-block classification, neutral-span direction resolution, or a duplicate mirror table.
- `runtime_15_ui_text_layout_engine_visual_order_is_child_owner` now locks the shared `analyze_bidi_line` / `mirrored_bidi_char` calls and the parent call boundary; no compatibility facade or old algorithm path remains.
- Locale-specific cosmic state is isolated under `graphics/text/shaping/cosmic/font_system_cache.rs`; `cosmic.rs` remains the backend adapter/orchestrator instead of accumulating cache policy.
- The cache is explicitly bounded to four `FontSystem` instances and reuses one seed database, closing the review concern that arbitrary application-language values could grow persistent backend state without limit.
- The boundary is documented exactly: locale configures cosmic platform fallback selection, while per-run HarfRust `locl` remains open because cosmic-text 0.18.2 does not expose language on `Attrs`.
- SH-M3 vertical policy is child-owned under `graphics/text/shaping/vertical.rs` and `vertical/orientation.rs`; `cosmic.rs` only invokes the projection and owns third-party feature mapping, while `ui/text/layout_engine/vertical.rs` consumes the vertical provider instead of reimplementing Unicode orientation.
- The provider hard cut preserves the existing cache authority: vertical orientation/mode are part of `ShapedRunCacheKey`, and UI wrapping/ellipsis/measurement no longer create horizontal cache entries for VerticalRl content.
- Native `vmtx` advance remains isolated under `graphics/text/font/vertical_metrics.rs`. TTB/BTT shaping is now split into `shaping/vertical/backend.rs`, while `projection.rs` owns DTO projection and `orientation.rs` owns Unicode rotation policy; backend vertical-origin/VORG-side-bearing values reach the renderer without a compatibility wrapper.
- V1 normalization policy now has a narrow `graphics/text/shaping/normalize.rs` owner. Cosmic/fallback consume its identity view and source projection instead of embedding an unreviewable offset assumption in the backend adapter.
- Text 03 vertical column capacity, right-to-left frame placement, and cross/main axis extents moved to `graphics/text/layout/vertical_layout.rs`; the UI child consumes the shared result and retains only CandidateLine/rich/ellipsis/UiResolved DTO projection.
- The production SDF VerticalRl consumer calls the same shaping owner, while `render/text_advances.rs` projects source-cluster advances, `sdf_atlas/text_keys.rs` owns shaped glyph/face key collection, and `sdf_render/vertices.rs` maps vertical origin/rotation into destination frames and UVs. `render.rs` is 712 lines, `sdf_atlas.rs` 611, and no production file crosses the 800-line review warning; no old scalar-only vertical success path or compatibility shim remains.
- Native bitmap mixed-storage partitioning remains child-owned by `text/native_bitmap_atlas/storage.rs`. It now partitions contiguous storage runs rather than globally grouping equal formats, so repeated `R8 -> RGBA -> R8` order survives as three renderer passes without adding ordering policy to the 691-line parent or retaining the former glyphon fallback as a supported success path; the child is 141 lines.
- Mixed-BiDi hit source/affinity policy is isolated under `ui/text/hit_test/visual_source.rs`; the parent hit-test owner performs geometry selection and consumes the leaf result, while visual-order no longer merges descending logical clusters into lossy ranges.

## 2026-07-10 Runtime Text backend face-ID owner hard cut note

- Third-party identity stays isolated under `graphics/text/font/backend.rs`; `fontdb::ID` does not leak into core/framework DTOs, which continue to expose `FontFaceId`.
- Process sharing is child-owned by `graphics/text/font/shared.rs`, while locale cache refresh remains in `shaping/cosmic/font_system_cache.rs`; neither policy is stacked into the text renderer root.
- `shaping/font_id.rs` and its post-shape annotation path were physically deleted. Cosmic and native reporting consume actual `LayoutGlyph.font_id`; the structure guard now rejects the removed bridge symbols instead of preserving a facade or shim.
- The slice introduced no production `allow(dead_code)`: deleting the bridge also deleted its newly orphaned cluster resolver/source helper, restoring the existing 416-warning library baseline.
- Follow-up fallback/diagnostic reporting did not grow the renderer root past its soft budget: `text/prepare_report.rs` now owns prepare/raster/missing-glyph report DTO aggregation, and `text.rs` is back to 777 orchestration lines; the structure guard rejects moving those declarations back into the parent.
- Run language remains a backend-neutral field on `zircon_runtime_interface::UiResolvedStyle`; normalized layout/shaped cache identity stays in their cache owners, while SDF locale identity stays in `sdf_atlas.rs`. `render.rs` and `text.rs` only propagate the value and do not become a second locale-policy owner.

## 2026-07-11 Runtime Text screen-space font initialization note

- System-font policy remains owned by `graphics/text/font`: the screen-space renderer invokes the narrow `initialize_screen_space_ui_font_system(...)` boundary and does not duplicate `fontdb` enumeration, family tables, or platform-font constants.
- The fix adds no Editor-only font route, compatibility module, root facade, or test glyph injection. Runtime and Editor consume the same `FontDatabase -> glyphon FontSystem` synchronization path.
- The Windows lower regression remains in the existing folder-backed `scene_renderer/ui/text/tests.rs` owner, so production `text.rs` retains orchestration rather than accumulating test fixtures or platform assertions.
- The real HUD framebuffer gate passes after the same bounded 24-frame async-text settle policy used by the Runtime product test; waiting policy stays in test/product validation rather than becoming a production rendering bypass.

## 2026-07-11 Runtime Text rich parser owner split note

- Rich-text contracts stay backend-neutral under `core/framework/render/text/rich.rs`; parsing and security policy are not leaked into UI DTO roots or scene-renderer owners.
- `graphics/text/rich/parser.rs` remains the orchestration owner, while BBCode, decorator registration, and controlled HTML rules are separate folder children. `html_subset.rs` and `parser.rs` remain below 500 lines each (current parser 434), avoiding another oversized parser root.
- `ui/text/rich_text.rs` remains a narrow Markdown compatibility adapter over the shared parser; it does not retain a second parser or compatibility implementation.
- Grapheme boundary policy is applied once after markup stripping, and the three layout regressions were updated to assert whole-cluster runs rather than preserving a half-cluster legacy shape.
- Image/link parsing stays in the same HTML/BBCode leaf owners and uses the existing neutral `InlineObjectRef`/`LinkRef` contracts; no UI-local duplicate resource parser was added.
- No HTML/CSS crate, script bridge, network loader, root facade, re-export shim, or production `allow(dead_code)` was introduced. `LayoutItem::Inline` remains explicitly open rather than hiding an unused metric helper in production.

## 2026-07-11 Runtime Text rich inline-layout owner note

- `graphics/text/layout/rich.rs` is the narrow 03 owner for rich run-to-item projection and inline baseline metrics; parser policy remains under `graphics/text/rich`, and the UI/scene renderer roots do not duplicate its ascent/descent rules.
- Backend-neutral `LayoutItem`, `LaidOutLine`, and `LaidOutText` stay under `core/framework/render/text/rich.rs`. The owner records actual emitted item counts, so rejected source ranges cannot leave stale line indices.
- Text run origins are projected from the enlarged rich-line baseline instead of remaining pinned to `y=0`; Baseline/Center/Top/Bottom image modes share one metric conversion path.
- The child owner is under 300 lines and adds no compatibility facade, production `allow(dead_code)`, production panic/unwrap/expect, backend type, or duplicated renderer policy. UI resolved-layout/image-batch/link-hit integration remains explicitly open rather than being represented as a completed render path.
- The next integration cut keeps responsibilities leaf-owned: `ui/text/layout_engine/rich_inline.rs` projects the admitted single-line inline subset, while `scene_renderer/ui/render/rich_text.rs` owns renderer-side range/style/placement interpretation. The public style contract hard-cuts the Markdown-only boolean to `UiRichTextFormat`; no bool-to-format compatibility field or second parser survives.
- The renderer consumes the shared `LaidOutText` placement and never submits U+FFFC as a glyph batch. Image runs now route through folder-backed `scene_renderer/ui/image.rs`; the general color-quad pipeline and text renderer do not absorb texture bindings or WGSL sampling policy.
- UI texture preparation remains under the existing `ResourceStreamer`: `resources/ui_texture.rs` resolves locator-stable IDs against imported UUID-backed records, rejects non-D2/non-single-layer payloads, and returns the existing fallback on failure. No second GPU texture cache, asset loader, renderer-root resource map, or interface-level WGPU type was added.
- The leaf sizes remain bounded (`resources/ui_texture.rs` 139 lines, `scene_renderer/ui/image.rs` 259 lines). Rich run planning and placement now live in `scene_renderer/ui/render/rich_text.rs` (197 lines), which keeps the renderer root at 794 lines rather than the stale 866-line count; the real product framebuffer gate—not a policy diagram—proves both texture sampling and ellipsis-retained inline placement. Concrete evidence remains in the Text07 numbered archive.
- Vertical rich layout is split by responsibility: `graphics/text/layout/rich_vertical.rs` owns main/cross-axis metrics and wrap ranges, while `ui/text/layout_engine/rich_inline_vertical.rs` only projects those metrics through the shared VerticalRl column-capacity/placement and ellipsis owners. Object height advances y, object width expands the column; no second Unicode orientation, BiDi, texture loader, or renderer-local layout policy was introduced.
- The vertical addition keeps the production leaves bounded (`rich_vertical.rs` 322 lines, `rich_inline_vertical.rs` 239, `render/rich_text.rs` 236). Renderer rich tests moved to `render/tests/rich_inline.rs` (216 lines), leaving the test root at 779; product command builders moved to a 151-line child, leaving the integration root at 771. The renderer production root remains 794 lines.
- BBCode block alignment stays in the same leaf-owned chain: `graphics/text/rich/parser.rs` emits neutral `ParagraphOverride` ranges, while `ui/text/layout_engine.rs` only projects effective per-line alignment. No center/right parser or markup-range policy is duplicated in UI or renderer, and the rich parser remains below the oversized-file threshold.
- Rich-link input stays folder-backed: `ui/text/rich_text/link_hit.rs` owns caret-affinity/range resolution, `ui/surface/input/rich_link.rs` owns pointer admission, and `ui/surface/input/effect/link.rs` owns scheme/owner validation. `pointer.rs` only invokes the leaf after normal routing; the public interface carries neutral effect/host-request DTOs, with no browser backend or network dependency crossing E8.
