---
related_code:
  - dev/bevy/Cargo.toml
  - dev/bevy/docs/cargo_features.md
  - dev/bevy/crates/bevy_internal/src/default_plugins.rs
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/bevy/crates/bevy_render/src/lib.rs
  - dev/bevy/crates/bevy_render/src/camera.rs
  - dev/bevy/crates/bevy_render/src/pipelined_rendering.rs
  - dev/bevy/crates/bevy_render/src/view/window/mod.rs
  - dev/bevy/crates/bevy_render/src/view/window/screenshot.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/mod.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/internal.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/render_asset_diagnostic_plugin.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/erased_render_asset_diagnostic_plugin.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/mesh_allocator_diagnostic_plugin.rs
  - dev/bevy/crates/bevy_image/src/lib.rs
  - dev/bevy/crates/bevy_image/src/image.rs
  - dev/bevy/crates/bevy_mesh/src/lib.rs
  - dev/bevy/crates/bevy_mesh/src/mesh.rs
  - dev/bevy/crates/bevy_shader/src/lib.rs
  - dev/bevy/crates/bevy_shader/src/shader.rs
  - dev/bevy/crates/bevy_shader/src/shader_cache.rs
  - dev/bevy/crates/bevy_core_pipeline/src/lib.rs
  - dev/bevy/crates/bevy_core_pipeline/src/schedule.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_render/src/render_resource/bind_group_layout.rs
  - dev/bevy/crates/bevy_camera/src/components.rs
  - dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs
  - dev/bevy/crates/bevy_light/src/lib.rs
  - dev/bevy/crates/bevy_light/src/ambient_light.rs
  - dev/bevy/crates/bevy_light/src/rect_light.rs
  - dev/bevy/crates/bevy_scene/src/lib.rs
  - dev/bevy/crates/bevy_scene/src/scene.rs
  - dev/bevy/crates/bevy_pbr/src/lib.rs
  - dev/bevy/crates/bevy_pbr/src/pbr_material.rs
  - dev/bevy/crates/bevy_pbr/src/material.rs
  - dev/bevy/crates/bevy_pbr/src/mesh_material.rs
  - dev/bevy/crates/bevy_pbr/src/material_bind_groups.rs
  - dev/bevy/crates/bevy_pbr/src/render/light.rs
  - dev/bevy/crates/bevy_pbr/src/render/pbr.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/pbr_lighting.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/mesh_view_types.wgsl
  - dev/bevy/crates/bevy_pbr/src/deferred/deferred_lighting.wgsl
  - dev/bevy/crates/bevy_pbr/src/cluster/cluster.wgsl
  - dev/bevy/crates/bevy_sprite/src/lib.rs
  - dev/bevy/crates/bevy_sprite/src/sprite.rs
  - dev/bevy/crates/bevy_sprite/src/texture_slice/mod.rs
  - dev/bevy/crates/bevy_sprite_render/src/lib.rs
  - dev/bevy/crates/bevy_sprite_render/src/render/mod.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
  - dev/bevy/crates/bevy_post_process/src/lib.rs
  - dev/bevy/crates/bevy_post_process/src/bloom/mod.rs
  - dev/bevy/crates/bevy_post_process/src/effect_stack/mod.rs
  - dev/bevy/crates/bevy_post_process/src/motion_blur/mod.rs
  - dev/bevy/crates/bevy_post_process/src/dof/mod.rs
  - dev/bevy/crates/bevy_post_process/src/msaa_writeback.rs
  - dev/bevy/crates/bevy_anti_alias/src/lib.rs
  - dev/bevy/crates/bevy_anti_alias/src/fxaa/mod.rs
  - dev/bevy/crates/bevy_anti_alias/src/smaa/mod.rs
  - dev/bevy/crates/bevy_anti_alias/src/taa/mod.rs
  - dev/bevy/crates/bevy_anti_alias/src/contrast_adaptive_sharpening/mod.rs
  - dev/bevy/crates/bevy_solari/src/lib.rs
  - dev/bevy/crates/bevy_solari/src/scene/mod.rs
  - dev/bevy/crates/bevy_solari/src/scene/extract.rs
  - dev/bevy/crates/bevy_solari/src/realtime/mod.rs
  - dev/bevy/crates/bevy_solari/src/realtime/node.rs
  - dev/bevy/crates/bevy_solari/src/pathtracer/mod.rs
  - zircon_app/src/entry/entry_profile.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/surface.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/image/mod.rs
  - zircon_runtime/src/core/framework/render/image/descriptor.rs
  - zircon_runtime/src/core/framework/render/image/sampler.rs
  - zircon_runtime/src/core/framework/render/image/usage.rs
  - zircon_runtime/src/core/framework/render/mesh/mod.rs
  - zircon_runtime/src/core/framework/render/mesh/bounds.rs
  - zircon_runtime/src/core/framework/render/mesh/descriptor.rs
  - zircon_runtime/src/core/framework/render/shader/mod.rs
  - zircon_runtime/src/core/framework/render/shader/stage.rs
  - zircon_runtime/src/core/framework/render/shader/entry_point.rs
  - zircon_runtime/src/core/framework/render/shader/dependency.rs
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs
  - zircon_runtime/src/core/framework/render/shader/pipeline_layout.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/mod.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_item.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/core/framework/render/post_process/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_node.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_graph.rs
  - zircon_runtime/src/core/framework/render/post_process/validation.rs
  - zircon_runtime/src/core/framework/render/anti_alias/mod.rs
  - zircon_runtime/src/core/framework/render/anti_alias/mode.rs
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs
  - zircon_runtime/src/core/framework/render/anti_alias/fallback.rs
  - zircon_runtime/src/core/framework/render/advanced/mod.rs
  - zircon_runtime/src/core/framework/render/advanced/feature.rs
  - zircon_runtime/src/core/framework/render/advanced/provider_report.rs
  - zircon_runtime/src/core/framework/render/advanced/runtime_plan.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/sprite/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/sprite.rs
  - zircon_runtime/src/core/framework/render/sprite/atlas.rs
  - zircon_runtime/src/core/framework/render/sprite/rect.rs
  - zircon_runtime/src/core/framework/render/sprite/anchor.rs
  - zircon_runtime/src/core/framework/render/sprite/bounds.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/reflect/fixed/lights.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/ui/workbench/model/menu/selection_menu.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_runtime/src/scene/components/render2d/mod.rs
  - zircon_runtime/src/scene/components/render2d/sprite.rs
  - zircon_runtime/src/scene/components/render2d/mesh2d.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/asset/assets/texture/mod.rs
  - zircon_runtime/src/asset/assets/model/mod.rs
  - zircon_runtime/src/asset/assets/shader/mod.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/shader/entry_point.rs
  - zircon_runtime/src/asset/assets/shader/dependency.rs
  - zircon_runtime/src/asset/assets/material/mod.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/query_stats/query_stats.rs
  - zircon_runtime/src/core/diagnostics/render.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_material.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_clustered_lighting/execute_clustered_lighting.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/sprite.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_runtime_feature_flags/scene_runtime_feature_flags.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
implementation_files:
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - docs/assets-and-rendering/render-framework-architecture.md
  - docs/zircon_runtime/core/framework/render/profile.md
  - docs/zircon_runtime/core/framework/render/camera.md
  - docs/zircon_runtime/core/framework/render/image.md
  - docs/zircon_runtime/core/framework/render/mesh.md
  - docs/zircon_runtime/core/framework/render/shader.md
  - docs/zircon_runtime/core/framework/render/core_pipeline.md
  - docs/zircon_runtime/core/framework/render/light.md
  - docs/zircon_runtime/core/framework/render/post_process.md
  - docs/zircon_runtime/core/framework/render/anti_alias.md
  - docs/zircon_runtime/core/framework/render/advanced.md
  - docs/zircon_runtime/core/framework/render/sprite.md
  - docs/zircon_runtime/core/framework/render/solari.md
  - docs/zircon_runtime/graphics/render-product-submit.md
  - docs/zircon_runtime/asset/scene.md
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/image/mod.rs
  - zircon_runtime/src/core/framework/render/image/asset_usage.rs
  - zircon_runtime/src/core/framework/render/image/color_space.rs
  - zircon_runtime/src/core/framework/render/image/descriptor.rs
  - zircon_runtime/src/core/framework/render/image/dimension.rs
  - zircon_runtime/src/core/framework/render/image/fallback.rs
  - zircon_runtime/src/core/framework/render/image/sampler.rs
  - zircon_runtime/src/core/framework/render/image/usage.rs
  - zircon_runtime/src/core/framework/render/mesh/mod.rs
  - zircon_runtime/src/core/framework/render/mesh/bounds.rs
  - zircon_runtime/src/core/framework/render/mesh/descriptor.rs
  - zircon_runtime/src/core/framework/render/mesh/mesh_kind.rs
  - zircon_runtime/src/core/framework/render/mesh/topology.rs
  - zircon_runtime/src/core/framework/render/shader/mod.rs
  - zircon_runtime/src/core/framework/render/shader/stage.rs
  - zircon_runtime/src/core/framework/render/shader/entry_point.rs
  - zircon_runtime/src/core/framework/render/shader/dependency.rs
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs
  - zircon_runtime/src/core/framework/render/shader/pipeline_layout.rs
  - zircon_runtime/src/core/framework/render/material/mod.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/core/framework/render/sprite/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/sprite.rs
  - zircon_runtime/src/core/framework/render/sprite/atlas.rs
  - zircon_runtime/src/core/framework/render/sprite/rect.rs
  - zircon_runtime/src/core/framework/render/sprite/anchor.rs
  - zircon_runtime/src/core/framework/render/sprite/bounds.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/mod.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_item.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/pipeline_kind.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/render_phase.rs
  - zircon_runtime/src/core/framework/render/post_process/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_node.rs
  - zircon_runtime/src/core/framework/render/post_process/pass_graph.rs
  - zircon_runtime/src/core/framework/render/post_process/validation.rs
  - zircon_runtime/src/core/framework/render/anti_alias/mod.rs
  - zircon_runtime/src/core/framework/render/anti_alias/mode.rs
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs
  - zircon_runtime/src/core/framework/render/anti_alias/fallback.rs
  - zircon_runtime/src/core/framework/render/advanced/mod.rs
  - zircon_runtime/src/core/framework/render/advanced/feature.rs
  - zircon_runtime/src/core/framework/render/advanced/provider_report.rs
  - zircon_runtime/src/core/framework/render/advanced/runtime_plan.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/reflect/fixed/lights.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/ui/workbench/model/menu/selection_menu.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_runtime/src/scene/components/render2d/mod.rs
  - zircon_runtime/src/scene/components/render2d/sprite.rs
  - zircon_runtime/src/scene/components/render2d/mesh2d.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_material.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_clustered_lighting/execute_clustered_lighting.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/sprite.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_vertex.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/compile_options_for_profile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_post_process_resources/scene_post_process_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_ssao/execute_ssao.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - zircon_runtime/src/graphics/tests/visibility.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
plan_sources:
  - user: 2026-05-21 continue Bevy advanced/default render boundary evidence
  - user: 2026-05-21 continue Bevy default render profile completion gates
  - user: 2026-05-21 continue Bevy render schedule and submit pipeline evidence mapping
  - user: 2026-05-21 continue Bevy PBR material and lighting evidence mapping
  - user: 2026-05-08 implement ZirconEngine Bevy-Level Rendering Completion Plan M0
  - user: 2026-05-08 continue ZirconEngine Bevy-Level Rendering Completion Plan M1
  - user: 2026-05-21 continue Bevy-level render sprite evidence mapping
  - user: 2026-05-21 continue Bevy-level Solari experimental gating evidence
  - user: 2026-05-21 continue Bevy render diagnostics evidence mapping
  - user: 2026-05-21 continue Bevy presentation surface evidence mapping
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
  - docs/superpowers/plans/2026-05-17-render-camera-ordering-m2d.md
tests:
  - "M0 docs acceptance only: no runtime tests required by plan"
  - cargo test -p zircon_runtime render_profile --locked
  - cargo check -p zircon_app --locked --all-targets
  - cargo test -p zircon_runtime --locked render_product_assets
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts
  - cargo test -p zircon_runtime --locked render_product_pbr
  - cargo test -p zircon_runtime --lib render_product_pbr_world_frame_extract_exposes_authored_ambient_and_rect_light_slots --locked
  - cargo test -p zircon_runtime --lib ambient_and_rect_light_reflection_roundtrips_authoring_fields --locked
  - cargo test -p zircon_runtime --locked material
  - cargo check -p zircon_runtime --lib --locked
  - tests/acceptance/render-product-m5a-pbr-light.md
  - tests/acceptance/render-product-m6a-sprite-default-2d.md
  - zircon_runtime/src/core/framework/tests.rs::render_camera_contracts_cover_viewports_and_bevy_layer_intersection
  - zircon_runtime/src/core/framework/render/advanced/runtime_plan.rs::default_render_plan_does_not_request_advanced_providers
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/compile_options_for_profile.rs::compile_options_do_not_enable_advanced_capabilities_without_providers
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs::runtime_profile_bundle_for_quality_profile_defaults_without_advanced_flags
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::render_extract_filters_meshes_by_active_camera_layers
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::explicit_render_camera_snapshot_layers_override_scene_camera_layers
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::render_extract_projects_scene_camera_component_product_fields
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::inactive_render_camera_extracts_no_scene_renderables
  - zircon_runtime/src/scene/tests/asset_scene.rs::scene_assets_roundtrip_camera_product_fields
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_camera_asset_roundtrip_preserves_bevy_style_camera_fields
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_camera_asset_defaults_bevy_camera_fields_when_omitted
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_asset_toml_roundtrip_preserves_ambient_and_rect_lights
  - zircon_runtime/src/scene/tests/asset_scene.rs::scene_assets_roundtrip_ambient_and_rect_light_product_fields
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs::scene_uniform_uses_authored_ambient_light_when_lighting_is_enabled
  - zircon_runtime/src/core/framework/render/light/readiness.rs::light_status_counts_split_ready_and_degraded_slots
  - zircon_editor/src/tests/editing/editor_projection.rs::viewport_edit_mode_projection_exposes_ambient_and_rect_light_fields
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs::workbench_view_model_projects_menu_strip_drawers_and_status
  - zircon_editor/src/tests/editor_event/runtime.rs::operation_invocation_dispatches_rect_light_creation
  - zircon_runtime/src/graphics/tests/visibility.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_headless_size_controls_offscreen_capture_size
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_reports_unsupported_without_primary_fallback_capture
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_headless_present_reports_unsupported_surface_fallback
  - zircon_runtime/src/core/framework/tests.rs::render_camera_ordering_sorts_by_order_then_target_and_tracks_target_hdr_index
  - zircon_runtime/src/core/framework/tests.rs::render_camera_ordering_reports_ambiguities_and_skips_inactive_cameras
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs
  - zircon_runtime/src/scene/tests/world_basics.rs::render_product_sprite_world_frame_extract_exposes_runtime_sprite_components
  - zircon_runtime/src/scene/tests/world_basics.rs::render_product_sprite_world_frame_extract_filters_by_camera_layers
  - zircon_runtime/src/scene/tests/world_basics.rs::render_product_sprite_mesh2d_component_does_not_count_as_particle_sprite
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::default_core2d_pipeline_compiles_expected_stage_order_and_passes
  - cargo test -p zircon_runtime --locked render_product_sprite
  - cargo test -p zircon_runtime --locked render_product_pipeline
  - cargo test -p zircon_runtime --locked default_core2d_pipeline_compiles_expected_stage_order_and_passes
doc_type: milestone-detail
---

# Bevy Rendering Capability Matrix

## Scope

This is M0 acceptance evidence for `ZirconEngine Bevy-Level Rendering Completion Plan`. It maps local `dev/bevy` rendering feature collections and source modules to the current Zircon owners before M1 starts adding product profiles or code contracts.

The reference engine is the checked-in `dev/bevy` tree, target Bevy `0.19.0-dev` at commit `c040d7603`. Zircon intentionally maps Bevy Cargo features to runtime product profiles instead of copying Cargo-feature activation as the product surface.

## Ownership Rule

Neutral render contracts land under `zircon_runtime::core::framework::render`. Concrete rendering, resource preparation, render graph execution, WGPU, visibility, pipeline compilation, post-process, and runtime provider work stay under `zircon_runtime::graphics`, `zircon_runtime::rhi`, `zircon_runtime::rhi_wgpu`, and `zircon_runtime::render_graph`. Scene, asset, and runtime UI authoring data stay in their existing runtime owners and are projected into render contracts.

Advanced Virtual Geometry and Hybrid GI remain explicit advanced capability/profile paths through `zircon_plugins/virtual_geometry` and `zircon_plugins/hybrid_gi`. They must not become dependencies for default 2D, 3D, or UI rendering.

## Bevy Profile Mapping

| Bevy collection or profile | Bevy source evidence | Zircon product profile | Zircon owner modules | M0 status |
| --- | --- | --- | --- | --- |
| `default` | `dev/bevy/Cargo.toml:133-151`, `dev/bevy/docs/cargo_features.md:22-30` combine `2d`, `3d`, `ui`, and `audio`; `dev/bevy/crates/bevy_internal/src/default_plugins.rs:43-77` orders the render-facing default plugins from render init through image/mesh/camera/light/core-pipeline/post-process/AA/sprite/UI/PBR. | `DefaultRender` for rendering scope only. Audio is out of this plan. | `zircon_app` profile selection plus `zircon_runtime::core::framework::render` bundles. | Accepted for M10A focused submit: DefaultRender covers asset readiness, Core2d/Core3d phase selection, postprocess, runtime PBR/light stats, non-particle Core2d sprites, runtime UI graph placement, and concrete FXAA fallback, while still excluding VG/HGI/Solari. M10B records the default render module-ordering contract in [Runtime Render Profile Contracts](../zircon_runtime/core/framework/render/profile.md). M10K adds the completion gate: common API, 2D, 3D, UI, presentation, and diagnostics each need their own evidence, and AdvancedRender/Solari evidence cannot close DefaultRender gaps. |
| `common_api` | `dev/bevy/Cargo.toml:198-211` includes camera, image, mesh, shader, material, text, color, and HDR without `bevy_render`. | `CommonRenderApi`. | `zircon_runtime/src/core/framework/render/*`, `zircon_runtime/src/scene/components/scene.rs`, `zircon_runtime/src/asset/assets/*`. | Partially present as viewport camera, scene components, texture/model/shader/material assets, and render extract DTOs. |
| `2d_api` | `dev/bevy/Cargo.toml:213-215` adds `bevy_sprite` without a renderer. | `Render2d` API slice. | `render::{sprite,mesh,material,camera,core_pipeline}` plus current asset and scene owners. | M6A adds first-class `Sprite2dComponent`, `Mesh2dComponent`, `RenderSpriteSnapshot`, and `SpriteExtract`. `Mesh2dComponent` is stored as 2D scene data but is not yet a mesh-rendered product path. |
| `2d_bevy_render` | `dev/bevy/Cargo.toml:216-224` adds `bevy_render`, `bevy_core_pipeline`, `bevy_post_process`, and `bevy_sprite_render`. | `Render2d` implementation bundle. | `zircon_runtime::graphics`, `zircon_runtime::render_graph`, `zircon_runtime::rhi`, `core_pipeline`, and `sprite` contracts. | Accepted for focused DefaultRender submit through camera-selected Core2d phases, default Core2d pipeline asset, sprite graph passes, sprite texture fallback stats, and concrete sprite quad drawing. Materialized Mesh2d drawing remains a later divergent gap. |
| `3d_api` | `dev/bevy/Cargo.toml:226-237` adds light, KTX2, morph, SMAA and tonemapping support around common API. | `Render3d` API slice. | `zircon_runtime/src/scene/components/scene.rs`, `zircon_runtime/src/asset/assets/material.rs`, `zircon_runtime/src/asset/assets/model.rs`, future `render::{light,pbr,mesh,material}`. | Partially present as `CameraComponent`, `MeshRenderer`, directional/point/spot lights, `MaterialAsset`, and `ModelAsset`. |
| `3d_bevy_render` | `dev/bevy/Cargo.toml:239-250` adds `bevy_render`, `bevy_core_pipeline`, `bevy_anti_alias`, `bevy_pbr`, and `bevy_post_process`. | `Render3d` implementation bundle. | `zircon_runtime/src/graphics/pipeline/*`, `zircon_runtime/src/graphics/scene/scene_renderer/*`, `core_pipeline`, `material`, `anti_alias`, and `post_process` contracts. | Accepted for focused DefaultRender submit through camera-selected Core3d, PBR/material fallback stats, light stats, postprocess graph execution, and FXAA fallback. Full Bevy PBR parity and additional AA modes remain intentional gaps. |
| `ui_api` | `dev/bevy/Cargo.toml:252-253` combines `default_app`, `common_api`, and `bevy_ui`. | `Ui` API slice. | `zircon_runtime/src/ui/*`, `zircon_runtime_interface/src/ui/surface/*`, `zircon_runtime/src/asset/assets/ui.rs`. | Existing runtime UI asset/layout/input/render-extract chain exists, but it is not yet a render product profile. |
| `ui_bevy_render` | `dev/bevy/Cargo.toml:255-261` adds `bevy_render`, `bevy_core_pipeline`, and `bevy_ui_render`. | `Ui` implementation bundle. | `zircon_runtime/src/graphics/scene/scene_renderer/ui/*` plus `render::UiRender` profile feature. | Accepted for focused DefaultRender submit: runtime UI is graph-executed after postprocess and before overlay, records target size/order stats, and no longer relies on a legacy renderer tail side path. |
| `bevy_solari` / `SolariPlugins` | `dev/bevy/crates/bevy_solari/src/lib.rs:29-57` defines the experimental plugin group, realtime Solari lighting, raytracing scene setup, validation-only pathtracer, and required ray-query/binding-array features; `scene/mod.rs:39-78`, `realtime/mod.rs:35-95`, and `pathtracer/mod.rs:23-60` show the scene, realtime, and validation paths. | `SolariExperimental`. | `zircon_runtime::core::framework::render::solari`, `zircon_runtime::graphics::solari_runtime_provider`, and `zircon_plugins/solari`. | Experimental and gated: neutral contracts, capability requirements, first-party plugin/provider registration, and unavailable-provider status are accepted. M10F documents that BLAS scene setup, Solari camera/prepass constraints, ReSTIR/world-cache compute pipelines, temporal history, DLSS RR integration, and validation pathtracing are not implemented yet. |

## Capability Matrix

| Product capability | Bevy evidence | Current Zircon landing module | Target Zircon contract owner | Gap before later milestones |
| --- | --- | --- | --- | --- |
| Render sub-app and render stages | `dev/bevy/crates/bevy_render/src/lib.rs:120-128` defines `RenderPlugin` as a render `SubApp` that can run between main schedules or in parallel; `lib.rs:151-208` names `RenderSystems` from extract command application through prepare, queue, phase sort, render, cleanup, and post-cleanup; `pipelined_rendering.rs:68-105` documents render-thread overlap and `RenderExtractApp`; `bevy_core_pipeline/src/schedule.rs:111-170` runs per-camera schedules. | `submit_frame_extract/submit.rs:22-123`, `build_frame_submission_context/build.rs:19-150`, `prepare_runtime_submission/prepare.rs:8-31`, `render/render.rs:31-233`, `execute_graph_stage.rs:80-180`, `render_pipeline_asset/compile.rs:18-180`, and `render_pass_executor_registry.rs:19-140`. | `render::core_pipeline` for neutral schedule/phase names, concrete execution in `zircon_runtime::graphics`; see [Runtime Render Core Pipeline Contracts](../zircon_runtime/core/framework/render/core_pipeline.md). | M10J documents the boundary: Zircon has a synchronous submit pipeline, explicit context building, runtime preparation, compiled graph passes, executor validation, command submission, and stats, but no Bevy-like render app/render world, named `RenderSystems` schedule, render-thread overlap, `RenderExtractApp`, or true multi-camera schedule execution yet. |
| Render diagnostics and submit telemetry | `dev/bevy/crates/bevy_render/src/diagnostic/mod.rs:37-94` wires CPU/GPU pass timing and pipeline-statistic diagnostics into main/render worlds; `diagnostic/internal.rs:83-144` manages current/submitted/finished GPU query frames; `render_asset_diagnostic_plugin.rs:31-42`, `erased_render_asset_diagnostic_plugin.rs:35-46`, and `mesh_allocator_diagnostic_plugin.rs:36-52` expose render-asset and mesh allocator diagnostics. | `RenderStats`, `update_base_stats(...)`, `query_stats(...)`, render graph execution records, material/sprite/light/UI/product counters, and RenderDoc/debug-marker evidence. | `zircon_runtime::graphics::render_framework` stats plus runtime diagnostics bridge; see [Render Product Submit](../zircon_runtime/graphics/render-product-submit.md). | M10G documents the current boundary: Zircon has rich submit/product stats and external capture markers, but not Bevy-style CPU/GPU pass timing, pipeline-statistics queries, generic render-asset diagnostics, mesh allocator slab/byte diagnostics, or pipelined render-thread telemetry. |
| Presentation surfaces, camera targets, and screenshots | `dev/bevy/crates/bevy_camera/src/camera.rs:22-58` defines physical viewports; `camera.rs:814-855` defines `RenderTarget::{Window, Image, TextureView, None}`; `dev/bevy/crates/bevy_render/src/camera.rs:263-331` resolves target info or reports missing targets; `view/window/mod.rs:31-99` wires extracted windows, swapchain texture views, and `SurfaceTexture::present()`; `view/window/mod.rs:358-508` creates/configures surfaces and falls back present modes; `view/window/screenshot.rs:49-111`, `406-439`, and `596-682` define async target-aware screenshot capture. | `RenderCameraTarget::{PrimarySurface, Texture, Headless}`, `target_resolution.rs`, `present_frame_extract.rs`, raw Win32 `RenderViewportSurfaceDescriptor`, backend `ViewportSurface` present blit, `capture_frame(...)`, RenderDoc capture hooks, and `surface_targets.rs`. | `render::camera` owns neutral target vocabulary; `zircon_runtime::graphics::render_framework` owns submit/capture/present behavior; see [Render Product Submit](../zircon_runtime/graphics/render-product-submit.md). | M10H documents the current boundary: primary-surface present and headless offscreen capture are accepted, texture targets are explicitly unsupported, headless surface present is explicitly unsupported, and there is no Bevy-like image/texture-view render target, async screenshot component, broad platform surface lifecycle, or present-mode diagnostics yet. |
| Camera-driven core pipeline | `dev/bevy/crates/bevy_core_pipeline/src/schedule.rs:1-11` states rendering is camera driven, and `camera_driver` starts at `schedule.rs:119`; `dev/bevy/crates/bevy_render/src/camera.rs:527-530` removes extracted camera components when a camera is inactive; `dev/bevy/crates/bevy_render/src/camera.rs:663-722` sorts active cameras by order/target and reports ambiguities. | `zircon_runtime/src/core/framework/render/camera.rs` with `ViewportCameraSnapshot`; `zircon_runtime/src/core/framework/render/camera_ordering.rs` with neutral camera ordering; `zircon_runtime/src/scene/components/scene.rs` with `CameraComponent`; `RenderFrameExtract` in `frame_extract.rs`. | `render::camera` and `render::core_pipeline`. | M2A/M2B landed target, viewport, order, active state, inactive-camera extraction suppression, clear color, HDR, exposure, MSAA, render layers, scene component projection, and scene asset roundtrip. M2C routes headless targets into concrete offscreen submission size and rejects texture targets explicitly until GPU texture residency is ready. M2D adds Bevy-style active camera ordering, ambiguity reporting, and per-target/HDR sorted index contracts. |
| 2D pipeline and phases | `dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs:49-91` registers `Core2d`, `Opaque2d`, `AlphaMask2d`, and `Transparent2d`. | `CorePipelineKind::Core2d`, 2D render phases, `RenderPipelineAsset::default_core2d()`, and sprite phase queues. | `render::core_pipeline` plus `render::sprite` and `render::mesh`; see [Runtime Render Core Pipeline Contracts](../zircon_runtime/core/framework/render/core_pipeline.md). | M4A landed neutral Core2d phase names and pipeline matching. M6A adds sprite queueing into `Opaque2d`, `AlphaMask2d`, and `Transparent2d`; mesh2d draw execution remains future work. |
| 3D pipeline and phases | `dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs:94-157` registers `Core3d`, prepass, deferred, opaque, alpha mask, and transparent phases. | `default_forward_plus.rs`, `default_deferred.rs`, `render_pass_stage.rs`, and compiled scene renderer paths. | `render::core_pipeline` and `render::material`/future `render::pbr`; see [Runtime Render Core Pipeline Contracts](../zircon_runtime/core/framework/render/core_pipeline.md). | Zircon has explicit Core3d phase names and concrete forward/deferred pipeline assets. The remaining gap is per-camera Core3d schedule execution and complete PBR lighting parity. |
| Camera components | `dev/bevy/crates/bevy_camera/src/components.rs:8-89` defines `Camera2d`, `Camera3d`, and `Hdr`. | `zircon_runtime/src/core/framework/render/camera.rs`; `zircon_runtime/src/scene/components/scene.rs`; `zircon_runtime/src/asset/assets/scene.rs`. | `render::camera`, projected through scene component and scene asset owners. | M2B moved product fields into `CameraComponent` and `SceneCameraAsset`. Editor authoring and concrete graphics target routing remain. |
| Visibility layers | `dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs:10-20`, `45-50`, and `115-135` define default layer 0, empty invisible, and intersection semantics. | `zircon_runtime/src/core/framework/render/camera.rs` with `RenderLayerSet`; `zircon_runtime/src/scene/components/scene.rs` and `scene/world/render.rs` project legacy masks into the snapshot. | `render::camera`, with concrete scene extraction in `zircon_runtime::scene`. | M2A landed Bevy-style layer set semantics and mesh filtering during scene extraction. A later deliberate scene serialization cutover should replace or wrap the `u32` mask in the neutral contract. |
| Image / texture | Bevy `common_api` includes `bevy_image` at `dev/bevy/Cargo.toml:204`; `ImagePlugin` registers image assets and GPU preparation in `dev/bevy/crates/bevy_image/src/image.rs`. | `zircon_runtime/src/asset/assets/texture/mod.rs`; runtime GPU texture resources under `zircon_runtime/src/graphics/scene/resources/*`. | `render::image` plus asset-side texture projection; see [Runtime Render Image Contracts](../zircon_runtime/core/framework/render/image.md). | M3A landed `RenderImageDescriptor` with sampler, color space, usage, format, mip/layer counts, asset usage, and fallback class. Loader registration, container upload readiness, mip generation, and concrete texture fallback resource stats remain later renderer/asset work. |
| Mesh / model | Bevy `common_api` includes `bevy_mesh` at `dev/bevy/Cargo.toml:205`; `MeshPlugin` registers mesh assets and changed-asset tracking in `dev/bevy/crates/bevy_mesh/src/lib.rs`. | `zircon_runtime/src/asset/assets/model/mod.rs`; `MeshRenderer` in `zircon_runtime/src/scene/components/scene.rs:115-142`. | `render::mesh`; see [Runtime Render Mesh Contracts](../zircon_runtime/core/framework/render/mesh.md). | M3A landed `RenderMeshDescriptor` with topology, bounds, kind, 2D/3D suitability, counts, and VG payload presence. Vertex-layout readiness, Mesh2d/Mesh3d component parity, skinning/morph metadata, and materialized Mesh2d drawing remain later work. |
| Shader | Bevy `common_api` includes `bevy_shader` at `dev/bevy/Cargo.toml:206`; `dev/bevy/crates/bevy_shader/src/shader.rs:33-55` owns shader source/import/defs state, `dev/bevy/crates/bevy_shader/src/shader_cache.rs:59-66` owns import-aware shader cache state, and `dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs:190-217` owns queued GPU pipeline states. | `zircon_runtime/src/asset/assets/shader/mod.rs`; `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs`; mesh pipeline cache shader module creation. | `render::shader`; see [Runtime Render Shader Contracts](../zircon_runtime/core/framework/render/shader.md). | M3A landed runtime WGSL selection, entry-point descriptors, variant keys, explicit serialized shader dependencies, and serialized pipeline layout descriptors with bind groups/bindings. M10D now documents the Bevy evidence and current separation between DTOs, asset projection, and concrete graphics caching. Automatic WGSL import parsing, shader-def composition, dependency-driven pipeline requeue, async pipeline states, and deep bind-group reflection remain future work. |
| Material and PBR baseline | Bevy `common_api` includes `bevy_material` at `dev/bevy/Cargo.toml:207`; `dev/bevy/crates/bevy_pbr/src/lib.rs:130-156` defines `PbrPlugin` as the default PBR infrastructure; `pbr_material.rs:26-57` starts `StandardMaterial` with authored base-color and texture bindings; `pbr_material.rs:967-1003` and `1010-1056` define shader flags and GPU uniform fields. | `zircon_runtime/src/core/framework/render/material/standard_material.rs`, `readiness_report.rs`, `zircon_runtime/src/asset/assets/material/mod.rs`, `resource_streamer_ensure_material.rs`, `material_runtime.rs`, and `pipeline_key.rs`. | `render::material` and runtime-owned PBR pipeline keying. | M5A streams `StandardMaterialDescriptor` into `MaterialRuntime`, PBR texture-slot key bits, alpha/double-sided/unlit pipeline keys, readiness/fallback diagnostics, and renderer material stats. M10I documents the remaining Bevy material-surface gap: reflectance/specular, transmission/thickness/IOR, attenuation, clearcoat, anisotropy, parallax/depth maps, UV transform/channel controls, lightmap interaction, and shader-def/debug flag breadth are not yet accepted. |
| PBR renderer, material bind groups, and light integration | `dev/bevy/crates/bevy_pbr/src/lib.rs:179-244` loads PBR shader libraries and wires StandardMaterial, SSAO, fog, lightmaps, probes, volumetric fog, SSR, transmission, clustered decals, contact shadows, light sync, atmosphere, and GPU clustering; `material.rs:74-144` makes `Material` an `Asset + AsBindGroup`; `material.rs:289-342` registers specialized caches, phase draw commands, material queueing, bind-group preparation, and shadow queueing; `material_bind_groups.rs:36-115` tracks bindless/non-bindless material bind groups; `render/pbr.wgsl:65-89`, `deferred/deferred_lighting.wgsl:59-86`, `render/pbr_lighting.wgsl:34-122`, and `cluster/cluster.wgsl:4-24` show forward/deferred lighting and cluster surfaces. | `resource_streamer_ensure_material.rs:18-195`, `MaterialRuntime`, `PipelineKey`, deferred geometry/lighting passes, `SceneUniform`, `execute_clustered_lighting.rs`, material/sprite/light submit stats, and RenderDoc markers. | `render::material`, future `render::pbr`, `render::light`, and concrete `zircon_runtime::graphics` renderer passes. | M10I keeps this as a documented gap rather than pretending the existing deferred and clustered fragments equal Bevy PBR. Completion requires material bind-group reflection/residency, per-material pipeline specialization, prepass/deferred/shadow variants, point/spot clustered lighting, rect/area-light shading, shadows, probes/IBL, SSAO/SSR coupling, clearcoat/anisotropy/transmission lighting, and structured pipeline/render-asset diagnostics. |
| Lights | `dev/bevy/crates/bevy_light/src/lib.rs:159-245` wires light visibility, clusters, shadow maps, and directional/point/spot/rect light visibility; `ambient_light.rs` defines global ambient color/brightness/lightmap influence; `rect_light.rs` defines local-XY rectangular area lights facing local `-Z`; `dev/bevy/crates/bevy_pbr/src/render/light.rs:1316-1343` uploads point/spot lights as `GpuClusteredLight`, and `render/light.rs:1519-1624` builds per-view `GpuLights` with ambient, directional, cluster, and rect-light storage; `mesh_view_types.wgsl` carries directional, rect, and clustered light view data. | `AmbientLight`, `DirectionalLight`, `PointLight`, `RectLight`, and `SpotLight` in `zircon_runtime/src/scene/components/scene.rs`; scene persistence in `zircon_runtime/src/asset/assets/scene.rs`; render light snapshots and readiness reports in `zircon_runtime/src/core/framework/render/light`; fixed reflection in `scene/reflect/fixed/lights.rs`; editor creation/projection in `zircon_editor/src/ui/workbench/model/menu/selection_menu.rs` and `zircon_editor/src/scene/viewport/edit_mode_projection/build.rs`; basic ambient and single-directional uniform consumption in `scene_uniform/from_frame.rs`; submit stats in `update_stats/base_stats.rs`; limited directional clustered-light buffer in `execute_clustered_lighting.rs`. | `render::light` and runtime `LightingExtract`. | M5 light authoring now stores ambient/rect scene components, exposes them through property paths and reflection, persists them through `SceneAsset`, and projects them into `SceneViewportRenderPacket` plus `RenderFrameExtract`. The editor creation surface now lists ambient/directional/point/rect/spot lights, maps them to undoable create operations, and projects `AmbientLight` plus `RectLight` inspector fields including `RectLight.size` as `Vec2`. `render::light` owns the neutral light row vocabulary plus `RenderLightReadinessReport`; ambient light feeds the basic forward/deferred `SceneUniform::ambient_color` path, the first directional light is reported as ready through the single `SceneUniform` directional slot, and submit stats split directional/point/spot/ambient/rect ready/degraded counts from that shared report. Extra directional lights plus point, spot, and rect lights remain renderer-degraded until clustered/Forward+ and area-light shading land. Bevy-style shadows/clusters and full PBR lighting remain later work. |
| Sprite | `dev/bevy/crates/bevy_sprite/src/lib.rs:68-108` owns `SpritePlugin`, texture-atlas setup, bounds, and optional picking; `dev/bevy/crates/bevy_sprite/src/sprite.rs:19-41` defines the authored sprite fields; `dev/bevy/crates/bevy_sprite_render/src/lib.rs:54-125` wires extraction, queueing, bind groups, and phase sorting; `dev/bevy/crates/bevy_sprite_render/src/render/mod.rs:49-573` owns pipeline keys, specialization, visibility extraction, queueing, and sprite batches. | `Sprite2dComponent`, `RenderSpriteSnapshot`, `SpriteExtract`, Core2d sprite phase queueing, sprite renderer, graph executors, and sprite stats. | `render::sprite`; see [Render Sprite Contracts](../zircon_runtime/core/framework/render/sprite.md). | M6A lands non-particle sprite contracts, atlas/rect/flip/anchor/custom-size/tint/z/layer extraction, 2D queueing, fallback texture stats, and product submit evidence. M10E classifies remaining Bevy sprite gaps: image scaling/sliced/tiled modes, atlas import/layout workflow, Mesh2d/SpriteMesh products, binned batching, HDR/MSAA/tonemapping/dither/compositing/alpha-mask pipeline specialization, and separate picking/Text2d milestones. |
| Runtime UI render | `dev/bevy/crates/bevy_ui_render/src/lib.rs:192-270` registers UI extraction and inserts `ui_pass` after post-process and before upscaling for Core2d/Core3d. | `zircon_runtime/src/ui/*`, `zircon_runtime_interface/src/ui/surface/*`, `zircon_runtime/src/graphics/scene/scene_renderer/ui/*`, and the `ui.screen-space` graph executor. | `render::UiRender` plus existing runtime UI contracts. | Accepted for focused M10A submit: UI is graph-executed after postprocess and before overlay, with target-size and pass-order stats. Multi-camera UI targeting remains future work. |
| Post-process | `dev/bevy/crates/bevy_post_process/src/lib.rs:9-36` adds bloom, motion blur, depth of field, effect stack, and MSAA writeback; bloom schedules Core2d/Core3d before tonemapping at `bloom/mod.rs:44-83`; effect stack covers chromatic aberration and vignette at `effect_stack/mod.rs:3-6`. | `BuiltinRenderFeature::{Bloom, ColorGrading, HistoryResolve, PostProcess}`, `PostProcessStackDescriptor`, `PostProcessPassGraph`, and post-process renderer resources under `zircon_runtime/src/graphics/scene/scene_renderer/post_process/*`. | `render::post_process`; [Postprocess Pass Graph Contracts](../zircon_runtime/core/framework/render/post_process.md). | Accepted for focused DefaultRender submit with graph node/execution stats. M10C now classifies Bevy motion blur, depth of field, chromatic aberration, vignette, and MSAA writeback as explicit gaps instead of letting bloom/color-grading imply full post-process parity. |
| Anti-aliasing | `dev/bevy/crates/bevy_anti_alias/src/lib.rs:23-33` adds FXAA, SMAA, TAA, CAS, and optional DLSS; `fxaa/mod.rs:57-108`, `smaa/mod.rs:137-196`, `taa/mod.rs:47-115`, and `contrast_adaptive_sharpening/mod.rs:40-122` define the per-mode product surfaces. | `render::anti_alias`, `AntiAliasMode`, `AntiAliasSettings`, `AntiAliasFallbackReport`, FXAA renderer resources, capability validation, and AA graph stats. | `render::anti_alias`; [Anti-Alias Product Surface](../zircon_runtime/core/framework/render/anti_alias.md). | Accepted for focused DefaultRender submit through concrete FXAA fallback. M10C now records per-family gaps: MSAA writeback/resolve, SMAA three-pass resources/LUTs, TAA jitter/history/prepasses, CAS sharpening, and DLSS provider capability remain future work. |
| Solari | `dev/bevy/crates/bevy_solari/src/lib.rs:29-57` defines the experimental Solari plugin group and required WGPU features; `scene/mod.rs:39-78` gates BLAS/bind-group scene setup; `realtime/mod.rs:35-95` gates realtime lighting, deferred/prepass/HDR requirements, and Core3d scheduling; `realtime/node.rs:31-180` shows the concrete compute-pipeline/resource surface; `pathtracer/mod.rs:23-60` keeps reference pathtracing separate. | `render::solari`, Solari capability validation, Solari provider availability, and `zircon_plugins/solari`. | `render::solari` plus `SolariExperimental` profile; see [Solari Experimental Render Contract](../zircon_runtime/core/framework/render/solari.md). | Experimental gated path accepted: missing caps, missing provider, disabled gate, and unavailable provider all report explicit status. M10F records the remaining Bevy gaps: no BLAS scene build, raytracing scene bindings, SolariLighting camera/prepass validation, Core3d Solari node, temporal history, ReSTIR/world-cache pipeline family, DLSS RR integration, or validation pathtracer. |
| Virtual Geometry and Hybrid GI | Bevy's checked-in profile model keeps default 2D, 3D, and UI rendering in `default`, `2d`, `3d`, `ui`, and their renderer collections; no Bevy default profile row makes an advanced renderer a substitute for those defaults. | `zircon_plugins/virtual_geometry/runtime/src/lib.rs`, `zircon_plugins/hybrid_gi/runtime/src/lib.rs`, neutral DTOs in `render::advanced`, provider facades in `zircon_runtime::graphics`, and submit/runtime-plan gates. | `AdvancedRender` and `SolariExperimental` profile gates, not default profiles; see [Advanced Render Profile Runtime Plan](../zircon_runtime/core/framework/render/advanced.md). | Accepted for focused M10A AdvancedRender submit with provider-backed VG/HGI graph execution and authored payload stats. M10L records that this evidence is non-substitutable: it cannot close default `CommonRenderApi`, `Render2d`, `Render3d`, `Ui`, presentation, diagnostics, scheduling, or shader/material reflection gaps. |

## M1 Landing Rules Derived From M0

- M1 has landed `RenderProductProfile` and `RenderProfileBundle` under `zircon_runtime::core::framework::render::profile`, not under concrete `graphics` implementation modules.
- `CommonRenderApi`, `Render2d`, `Render3d`, `Ui`, `DefaultRender`, `AdvancedRender`, and `SolariExperimental` are runtime product choices rather than Cargo feature clones.
- `Headless` is a valid render bundle that carries no render product dependencies and can be selected by `EntryConfig`.
- `DefaultRender` validates as `CommonRenderApi + Render2d + Render3d + Ui` and does not include Virtual Geometry, Hybrid GI, or Solari.
- App-level selection now lives on `EntryConfig::render_profile` and is stored by `BuiltinEngineEntry` in `CoreRuntime` config under `RENDER_PROFILE_CONFIG_KEY` before module activation.

The module-detail doc for this M1 surface is [Runtime Render Profile Contracts](../zircon_runtime/core/framework/render/profile.md).

## M2-M3 Risk Notes

- M2A/M2B added neutral camera snapshots, scene camera product fields, scene asset projection, and `RenderLayerSet` semantics. `RenderLayerMask(u32)` still needs a later deliberate scene serialization cutover rather than long-lived compatibility aliases.
- Asset work must land before PBR and sprite rendering: `ImageAsset`, `MeshAsset`, shader descriptors, material contracts, and alpha behavior are prerequisites for stable 2D/3D products.
- M6A closes the largest basic sprite gap with `Sprite2dComponent` and `SpriteExtract`; remaining 2D gaps are materialized mesh2d drawing, batching, atlas/importer productization, and per-alpha-mode GPU variants.
- UI render already has strong runtime assets and SDF/text support, but it must become a profile-controlled render pass instead of an incidental compiled-scene side path.
- Advanced VG/HGI paths are powerful but must remain behind profile and backend capability gates so they do not mask missing default 2D/3D/UI behavior.

## M0 Acceptance Evidence

This document maps every Bevy feature collection required by M0 to a Zircon owner module and records the implementation gaps that later milestones must close. Runtime tests are intentionally not run for M0 because the plan defines documentation evidence as the acceptance stage.

## M1 Acceptance Evidence

M1 product-profile validation is now recorded in the module-detail doc for [Runtime Render Profile Contracts](../zircon_runtime/core/framework/render/profile.md). The fresh 2026-05-08 gates were `cargo test -p zircon_runtime render_profile --locked` and `cargo check -p zircon_app --locked --all-targets`; both completed successfully with warning-only compile output outside the focused render-profile assertions.

## M3A Asset Product Contract Update

M3A gives the profile features for image, mesh, shader, and material a real asset-readiness surface. The owning docs are [Render Assets](../zircon_runtime/asset/render-assets.md), [Runtime Render Image Contracts](../zircon_runtime/core/framework/render/image.md), [Runtime Render Mesh Contracts](../zircon_runtime/core/framework/render/mesh.md), and [Render Material Contracts](../zircon_runtime/core/framework/render/material.md). Renderer phase scheduling, sprite rendering, anti-aliasing, Solari, and deep VG/HGI integration remain later milestones.

## M3B Basic Render Contract Explicitization Update

The foundational render product surface is now documented in Bevy-shaped pieces instead of relying on advanced renderer features to imply coverage. `render::image` owns texture/image descriptors and sampler/fallback vocabulary, `render::mesh` owns mesh metadata and bounds, and `render::core_pipeline` owns Core2d/Core3d phase classification and deterministic queue ordering. These docs make the Bevy `ImagePlugin`, `MeshPlugin`, and `CorePipelinePlugin` comparison explicit while keeping asset import, WGPU preparation, material/shader assetization, and multi-camera schedule execution in their owning milestones.

## M5A Runtime PBR And Light Baseline Update

M5A gives `Render3d` a runtime-only PBR material/light baseline without changing the coordinated shader/material assetization lane. StandardMaterial descriptor fields now reach runtime material preparation, pipeline variant keys, fallback readiness diagnostics, and renderer material stats. Ambient and rect-light DTO vectors now round-trip through render extract and submit stats; the follow-up M5 authoring slice adds world-authored ambient/rect components plus scene asset persistence. `render::light` now owns the neutral light snapshots and readiness report so the Bevy-style light API surface is explicit instead of being hidden inside `scene_extract.rs`; the owning module doc is [Runtime Render Light Contracts](../zircon_runtime/core/framework/render/light.md). Authored ambient lights now feed the basic forward/deferred `SceneUniform::ambient_color` path, the first directional light is counted as ready through the current single directional uniform slot, and `RenderStats` exposes ready/degraded split counts for directional, point, spot, ambient, and rect lights. Extra directional lights plus point, spot, and rect lights continue to report as degraded until clustered/Forward+ and concrete area-light shading land. Focused WSL evidence is recorded in [Render Product M5A PBR Light](../../tests/acceptance/render-product-m5a-pbr-light.md).

## M6A Sprite And Default 2D Renderer Update

M6A gives `Render2d` a non-particle sprite path. Runtime scene data can now store `Sprite2dComponent` and `Mesh2dComponent`; sprite components project into `RenderSpriteSnapshot` and `SpriteExtract` with image/material handles, atlas region, rect, flip flags, anchor, custom size, color tint, z order, alpha policy, and render layer mask. World extraction filters sprites by the active camera render layers and adds visible sprites to `VisibilityInput` as dynamic renderables.

The default Core2d pipeline now enables `BuiltinRenderFeature::Sprite` and compiles `sprite.opaque`, `sprite.alpha-mask`, and `sprite.transparent` graph passes for the 2D phase family. The concrete sprite renderer consumes `SpriteExtract.phase_queue`, draws texture-tinted quads through the existing texture streamer fallback path, and records sprite count/readiness/fallback plus sprite graph execution stats. Focused scoped evidence is recorded in [Render Product M6A Sprite Default 2D](../../tests/acceptance/render-product-m6a-sprite-default-2d.md).

## M10A Product Submit Acceptance Update

M10A adds focused product-profile acceptance over the accumulated M3A-M9B paths. `render_product_submit` now proves `DefaultRender` accepts the default Core3d material/light/postprocess/AA/runtime-UI path and the default Core2d sprite path while keeping VG/HGI/Solari disabled. The same runtime test file proves `Headless` carries no render product features, `AdvancedRender` executes provider-backed VG/HGI graph passes with authored payload sources, and `SolariExperimental` reports an explicit gated provider status without enabling VG/HGI when those quality flags are off.

App-side profile evidence now proves `RenderProfileBundle::solari_experimental()` flows into first-party plugin/provider planning: linked registrations include `virtual_geometry`, `hybrid_gi`, and `solari`, and diagnostics include `SolariPluginModule`. Focused evidence is recorded in [Render Product Default Profile Acceptance](../../tests/acceptance/render-product-default-profile.md) and [Render Product Advanced Profile Acceptance](../../tests/acceptance/render-product-advanced-profile.md). Workspace-wide and plugin-workspace all-target gates remain separate promotion checks.

## M10B Default Render Module Ordering Update

M10B makes the Bevy `DefaultPlugins` render ordering explicit in [Runtime Render Profile Contracts](../zircon_runtime/core/framework/render/profile.md). Zircon now documents how `RenderPlugin`, `ImagePlugin`, `MeshPlugin`, `CameraPlugin`, `LightPlugin`, `CorePipelinePlugin`, `PostProcessPlugin`, `AntiAliasPlugin`, `SpriteRenderPlugin`, `UiRenderPlugin`, and `PbrPlugin` map onto `RenderProductFeature` values and the current `render::{image,mesh,camera,light,core_pipeline,post_process,anti_alias,sprite,material}` contract modules.

This is a documentation and ownership slice only. It does not claim Bevy's render sub-app or pipelined rendering is complete. The point is to keep default 2D/3D/UI product coverage visible, and to keep Virtual Geometry, Hybrid GI, and Solari from masking missing baseline renderer work.

## M10C PostProcess And AntiAlias Evidence Update

M10C expands the Bevy evidence for [Postprocess Pass Graph Contracts](../zircon_runtime/core/framework/render/post_process.md) and [Anti-Alias Product Surface](../zircon_runtime/core/framework/render/anti_alias.md). The post-process row now distinguishes Zircon's accepted bloom/color-grading/history/final-composite graph evidence from Bevy features that remain incomplete: motion blur, depth of field, chromatic aberration, vignette, and MSAA writeback. The anti-alias row now distinguishes the concrete FXAA fallback path from named but degraded MSAA, SMAA, TAA, CAS, and DLSS modes.

This keeps M10 default rendering measurable at the product-contract layer without entering shader/material assetization or active zmeta/material implementation work.

## M10E Sprite Evidence Update

M10E expands [Render Sprite Contracts](../zircon_runtime/core/framework/render/sprite.md) from renderer-only evidence into Bevy's two-layer sprite shape: `bevy_sprite` owns the authored API/runtime fields, texture atlas setup, bounds, image modes, and slice/tiling DTOs, while `bevy_sprite_render` owns extraction, queueing, bind groups, phase sorting, pipeline specialization, and sprite batches.

M6A remains accepted for Zircon's non-particle default Core2d sprite product path. The new gap classification keeps the remaining Bevy parity work explicit: first-class image scaling/sliced/tiled modes, atlas import/layout/editor workflow, separate Mesh2d/SpriteMesh products, binned batching, per-view pipeline specialization, HDR/MSAA/tonemapping/dither/compositing and alpha-mask behavior, plus picking/Text2d routed to their own milestones instead of being hidden inside sprite rendering.

## M10F Solari Experimental Boundary Update

M10F expands [Solari Experimental Render Contract](../zircon_runtime/core/framework/render/solari.md) with the concrete Bevy Solari source shape. Bevy's `SolariPlugins` is not a default renderer feature; it combines raytracing scene setup with realtime Solari lighting, while the pathtracer is validation-oriented and separate. The source evidence now records the required ray-query/binding-array WGPU features, BLAS and bind-group scene setup, deferred/HDR/prepass/MSAA-off camera requirements, Core3d scheduling, temporal compute pipeline families, and validation pathtracing boundary.

Zircon remains intentionally status-only for Solari today: `SolariExperimental` can request the product feature, the backend capability and provider availability reports are explicit, and the first-party provider honestly reports unavailable. This keeps Solari from masking incomplete baseline 3D lighting/PBR work, default 2D sprite work, runtime UI render work, or presentation diagnostics.

## M10G Render Diagnostics Evidence Update

M10G expands [Render Product Submit](../zircon_runtime/graphics/render-product-submit.md) with Bevy render diagnostics evidence. Bevy's `RenderDiagnosticsPlugin` records CPU/GPU elapsed time per pass, pipeline statistics, and buffer-backed scalar diagnostics, syncs finished measurements into `DiagnosticsStore`, and exposes generic render-asset plus mesh allocator diagnostics. Bevy's pipelined rendering source also makes clear that render telemetry eventually needs to distinguish main-thread extraction, render-thread schedule execution, and thread handoff.

Zircon's accepted state remains narrower: `RenderStats` is a submit/product snapshot containing graph planning/execution counts, product fallback/readiness counts, UI/material/sprite/light stats, advanced-provider reports, Solari status, and VG/HGI counters. RenderDoc markers help external capture, but they are not CPU/GPU timing diagnostics. The remaining M10 diagnostics gap is a real diagnostics bridge for pass timings, pipeline-statistic queries, generic render-asset residency, mesh allocator memory, and future pipelined render-thread telemetry.

## M10H Presentation Surface Evidence Update

M10H expands [Render Product Submit](../zircon_runtime/graphics/render-product-submit.md) with Bevy camera-target, window-surface, and screenshot evidence. Bevy separates camera target vocabulary (`Window`, `Image`, `TextureView`, and no-color `None`) from render-app window surface preparation and screenshot readback; this keeps offscreen targets, external texture views, swapchain present, and CPU image capture visible as different product surfaces.

Zircon's accepted state is deliberately narrower: `PrimarySurface` presents through a bound viewport surface and backend blit, `Headless { size }` drives offscreen submit/capture size, and `Texture(handle)` plus headless surface-present fail with explicit unsupported-capability errors. The remaining presentation gaps are render-to-texture writeback, manual/external texture-view targets, a Bevy-like async screenshot request/result flow, broad platform surface lifecycle diagnostics, and present-mode fallback reporting.

## M10I PBR Material And Lighting Evidence Update

M10I expands [Render Product Submit](../zircon_runtime/graphics/render-product-submit.md) with Bevy PBR material, material pipeline, bind-group, shader, and light integration evidence. Bevy's `PbrPlugin` loads a PBR shader library family, registers `MaterialPlugin::<StandardMaterial>`, and wires SSAO, fog, lightmaps, light probes, volumetric fog, SSR, transmission, clustered decals, contact shadows, synchronized lights, atmosphere, GPU clustering, and deferred PBR lighting. This makes Bevy's baseline materially broader than a StandardMaterial struct plus one lighting shader.

Zircon's accepted state is a narrower runtime baseline: `StandardMaterialDescriptor` reaches `MaterialRuntime`, readiness/fallback reports, texture-slot residency attempts, and pipeline keys; the renderer has forward/deferred pipeline assets, a deferred geometry pass, a fullscreen deferred lighting pass, ambient/single-directional consumption through `SceneUniform`, and a limited directional clustered-light buffer. That state is useful and testable, but it is not full Bevy PBR parity.

The follow-up milestones are therefore explicit: add material bind-group reflection/residency and bindless/non-bindless policy; add per-material pipeline specialization for prepass/deferred/shadow/transparency variants; expand StandardMaterial to Bevy's reflectance/specular, transmission, clearcoat, anisotropy, parallax, UV, lightmap, and debug/shader-def families; implement point/spot clustered lighting, rect/area-light shading, shadow maps, probes/IBL, SSAO/SSR coupling, and advanced BRDF lobes; then connect `.zmaterial` and material editor authoring through the active asset/material lane rather than hiding those responsibilities in render submit.

## M10J Render Schedule And Submit Pipeline Evidence Update

M10J expands [Runtime Render Core Pipeline Contracts](../zircon_runtime/core/framework/render/core_pipeline.md) with the Bevy render schedule evidence that sits below default 2D/3D/UI product features. Bevy separates the render `SubApp`, render schedule system sets, optional pipelined rendering, and per-camera Core2d/Core3d schedules. The relevant source points are `bevy_render/src/lib.rs:120-208`, `bevy_render/src/pipelined_rendering.rs:68-178`, and `bevy_core_pipeline/src/schedule.rs:1-170`.

Zircon's corresponding state is deliberately different: `submit_frame_extract(...)` is a synchronous runtime framework path that builds a submission context, prepares advanced sidebands, resolves history, builds a `ViewportRenderFrame`, renders through a compiled graph pipeline, records the submission, and updates stats. The concrete scene renderer then executes fixed early/late stage families plus declared graph executors. This is a coherent product pipeline, but it is not a Bevy render world or render-thread overlap model.

The follow-up milestone is to make this distinction observable rather than hidden. Add neutral diagnostics for extract/context-build, prepare, graph compile, queue/phase execution, render, post-process, present, and cleanup; add multi-camera schedule execution with per-target coverage; expose culled-pass/resource/executor decisions; and keep pipelined rendering as a separate scheduling milestone instead of folding it into synchronous submit stats.

## M10K Default Profile Completion Gate Update

M10K expands [Runtime Render Profile Contracts](../zircon_runtime/core/framework/render/profile.md) from ordering evidence into an explicit `DefaultRender` promotion gate. Bevy's source makes the rule concrete: `dev/bevy/Cargo.toml:134-151` defines default 2D, 3D, and UI profiles; `dev/bevy/Cargo.toml:198-261` separates common API collections from renderer collections; and `dev/bevy/crates/bevy_internal/src/default_plugins.rs:43-77` loads render infrastructure, image, mesh, camera, light, core pipeline, post-process, AA, sprite/UI render, and PBR as distinct default plugin slices.

The Zircon rule is now that `DefaultRender` is only accepted slice by slice. `CommonRenderApi`, `Render2d`, `Render3d`, `Ui`, presentation, and diagnostics each need their own source-backed evidence and gap statement. `AdvancedRender`, Virtual Geometry, Hybrid GI, and `SolariExperimental` remain valuable opt-in products, but they cannot be used as substitutes for missing default 2D, 3D, UI, presentation, or diagnostics behavior.

## M10L Advanced Render Boundary Evidence Update

M10L expands [Advanced Render Profile Runtime Plan](../zircon_runtime/core/framework/render/advanced.md) with the Bevy and Zircon evidence behind that non-substitution rule. Bevy's profiles and collections keep default 2D, 3D, and UI rendering separate from optional breadth; Zircon maps that into `DefaultRender` first, then layers `AdvancedRender` and `SolariExperimental` as opt-in profiles that depend on but do not complete the default slices.

The concrete runtime gates now have documented ownership: `AdvancedProfileRuntimePlan` reports VG/HGI as not requested under `DefaultRender`, profile compile options only enable advanced capabilities when a quality profile requests them and a provider is selected, and viewport state resolution falls back to `DefaultRender` when VG/HGI/Solari flags are absent. Advanced graph execution and authored payload stats therefore remain valid M9/M10 evidence for VG/HGI provider integration, but they cannot close Mesh2d/SpriteMesh, PBR lighting, UI target, presentation, diagnostics, scheduling, or shader/material reflection gaps.

## M2A Camera And Layer Contract Update

M2A expands the camera-facing render contract without taking ownership away from scene or graphics modules. `ViewportCameraSnapshot` now carries target, viewport rectangle, render order, active state, clear color, HDR, exposure, MSAA, and `RenderLayerSet`. `RenderViewportRect::clamped_to_size(...)` and `ViewportCameraSnapshot::effective_viewport_size(...)` mirror the Bevy camera viewport size path, while `RenderLayerSet` mirrors Bevy's default layer `0` and empty-set invisibility rule.

Scene render extraction now projects the active camera entity's legacy `RenderLayerMask` into `ViewportCameraSnapshot::render_layers` and filters mesh snapshots by camera/mesh layer intersection. Explicit camera snapshots supplied through `SceneViewportExtractRequest` keep their own layer set, which lets editor/runtime preview requests override the scene camera without changing scene state. Inactive cameras keep their camera snapshot for diagnostics but emit no scene meshes, phase inputs, visibility renderables, or scene lights.

The module-detail doc for this M2A surface is [Runtime Render Camera Contracts](../zircon_runtime/core/framework/render/camera.md).

## M2B Scene Camera Projection Update

M2B moves the camera product surface into scene-level data. `CameraComponent` and `SceneCameraAsset` now carry projection mode, orthographic size, render target, viewport, order, active state, HDR, exposure, clear color, and MSAA. `SceneCameraTargetAsset` uses asset references for texture targets and contributes those references to `SceneAsset::direct_references()`. `World::from_scene_asset(...)` and `World::to_scene_asset(...)` now round-trip those camera fields through project scene assets.

Remaining M2 work is editor authoring for these fields, true texture-target writeback, and a later hard cutover from legacy scene `RenderLayerMask(u32)` to the neutral render layer contract.

M2C entry evidence was captured on 2026-05-16 with `CARGO_TARGET_DIR=F:\cargo-targets\zircon-render-camera-m2-1819`: `cargo test -p zircon_runtime camera --locked --jobs 1 --message-format short --color never` passed 13 focused camera/layer/scene-asset tests, and `cargo check -p zircon_runtime --lib --locked --message-format short --color never` passed afterward. M2C can now begin concrete graphics routing for `RenderCameraTarget::{Texture,Headless}` while keeping the scene/editor authoring and legacy `RenderLayerMask(u32)` cutover as separate later work.

## M2C Camera Target Routing Update

M2C follows the Bevy target-size and missing-target precedent from `dev/bevy/crates/bevy_camera/src/camera.rs:459-483` and `dev/bevy/crates/bevy_render/src/camera.rs:268-328`. Zircon now resolves camera target size during graphics submission: `PrimarySurface` uses the viewport record size, `Headless { size }` drives offscreen submission/capture size, and `Texture(handle)` returns `RenderFrameworkError::UnsupportedCapability { capability: "camera texture render target" }` instead of rendering to the primary viewport.

Presentation remains primary-surface-only for this slice. `Headless` cameras submitted through the surface-present path return `UnsupportedCapability { capability: "headless camera surface present" }`, keeping headless/offscreen capture separate from window blitting until multi-target scheduling and texture residency are ready.

M2C acceptance evidence was captured on 2026-05-16 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-render-camera-m2c`: `cargo test -p zircon_runtime camera_target --locked --jobs 1 --message-format short --color never` passed 3 focused camera target tests, and `cargo check -p zircon_runtime --lib --locked --message-format short --color never` passed afterward.

## M2D Camera Ordering Update

M2D follows Bevy `SortedCameras` and `sort_cameras` in `dev/bevy/crates/bevy_render/src/camera.rs:663-722`. Zircon now has `sort_render_cameras(...)` under `zircon_runtime::core::framework::render`: inactive cameras are skipped, active cameras are sorted by render order and normalized target key, duplicate active `(order, target)` groups are reported through `RenderCameraOrderAmbiguity`, and `sorted_camera_index_for_target` is assigned per `(target, hdr)`.

This is deliberately a neutral contract rather than a concrete multi-camera render loop. Split-screen, render-to-texture scheduling, editor authoring, and texture writeback remain later slices because the current runtime extract is still single-camera and the asset/GPU texture residency lane is active separately.

M2D acceptance evidence was captured on 2026-05-17 in WSL/Linux with `CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-render-camera-m2d-wsl`: `cargo test -p zircon_runtime --lib render_camera_ordering --locked --jobs 1 --message-format short --color never` passed 2 focused ordering tests, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never` passed afterward with existing unused-function warnings only. Windows default-feature and core-min attempts both failed before Zircon source at `wgpu-hal 29.0.3` DX12/windows type mismatches.
