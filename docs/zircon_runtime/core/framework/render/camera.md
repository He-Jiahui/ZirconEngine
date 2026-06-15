---
related_code:
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/temporal_jitter.rs
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/temporal_frame_index.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_temporal_camera_history.rs
  - zircon_runtime/src/graphics/types/viewport_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/camera_matrices/view_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/write_scene_uniform.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_writeback_converter.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render/render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_target/finish_viewport_frame.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_runtime/src/graphics/tests/visibility.rs
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/bevy/crates/bevy_camera/src/components.rs
  - dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs
  - dev/bevy/crates/bevy_render/src/camera.rs
  - dev/bevy/crates/bevy_core_pipeline/src/schedule.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/temporal_jitter.rs
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/temporal_frame_index.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_temporal_camera_history.rs
  - zircon_runtime/src/graphics/types/viewport_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/camera_matrices/view_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/write_scene_uniform.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_writeback_converter.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_output_target_texture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render/render_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_target/finish_viewport_frame.rs
  - zircon_runtime/src/graphics/debug_markers.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs
plan_sources:
  - user: 2026-05-16 continue ZirconEngine Bevy-level rendering completion plan M2A/M2B
  - docs/superpowers/plans/2026-05-16-render-camera-target-routing-m2c.md
  - docs/superpowers/plans/2026-05-17-render-camera-ordering-m2d.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/bevy/crates/bevy_camera/src/components.rs
  - dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs
  - dev/bevy/crates/bevy_render/src/camera.rs
tests:
  - zircon_runtime/src/core/framework/tests.rs::render_camera_contracts_cover_viewports_and_bevy_layer_intersection
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::render_extract_filters_meshes_by_active_camera_layers
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::explicit_render_camera_snapshot_layers_override_scene_camera_layers
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::render_extract_projects_scene_camera_component_product_fields
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::inactive_render_camera_extracts_no_scene_renderables
  - zircon_runtime/src/scene/tests/render_extract.rs::inactive_camera_render_frame_extract_keeps_view_but_removes_scene_payload
  - zircon_runtime/src/scene/tests/render_extract.rs::hierarchy_inactive_camera_render_frame_extract_keeps_view_but_removes_scene_payload
  - zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_filters_meshes_sprites_and_visibility_by_camera_layers
  - zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_filters_lights_by_camera_layers
  - zircon_runtime/src/scene/tests/render_extract.rs::explicit_camera_request_layers_override_scene_camera_layers_for_direct_frame_extract
  - zircon_runtime/src/scene/tests/render_extract.rs::world_render_camera_order_report_projects_active_scene_cameras
  - zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_carries_scene_camera_order_report_for_scene_camera
  - zircon_runtime/src/scene/tests/render_extract.rs::explicit_camera_render_frame_extract_has_no_scene_camera_order_report
  - zircon_runtime/src/scene/tests/asset_scene.rs::scene_assets_roundtrip_camera_product_fields
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_camera_asset_roundtrip_preserves_bevy_style_camera_fields
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_camera_asset_defaults_bevy_camera_fields_when_omitted
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_surface_offscreen_submit_and_capture_survive_unbind_noop
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_headless_size_controls_offscreen_capture_size
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_missing_asset_reports_unsupported_without_primary_fallback_capture
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_requires_render_target_usage
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_requires_renderable_render_target_format
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_render_target_metadata_controls_offscreen_capture_size
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_srgb_target_imports_direct_graph_final_target
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_present_reports_unsupported_surface_fallback
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_headless_present_reports_unsupported_surface_fallback
  - zircon_runtime/src/core/framework/render/backend_types.rs::tests::camera_target_writeback_report_separates_copy_and_conversion_debug_markers
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_writeback_plan_ignores_non_texture_targets
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_writeback_plan_waits_for_target_descriptor
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_writeback_plan_accepts_matching_srgb_format
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_writeback_plan_accepts_linear_rgba_target_for_conversion
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_graph_import_plan_marks_srgb_texture_ready_for_direct_import
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_graph_import_plan_keeps_linear_texture_on_conversion_writeback_path
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs::tests::output_target_graph_import_plan_blocks_unsupported_target_format
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs::tests::build_runtime_frame_carries_prepared_sideband_and_output_target_into_viewport_frame
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs::tests::direct_runtime_frame_submit_projects_resolved_output_target
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_output_target_texture.rs::tests::output_target_texture_id_uses_resolved_texture_target_only
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_output_target_texture.rs::tests::output_target_texture_id_ignores_non_texture_targets
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::output_target_graph_import_report_marks_srgb_texture_ready_for_direct_import
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs::tests::output_target_graph_import_report_keeps_linear_texture_on_writeback_path
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_executes_ready_copy_and_conversion_plans
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_report_maps_ready_and_blocked_plans
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_extent_accepts_matching_source_and_destination
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_extent_rejects_source_size_mismatch
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_extent_rejects_destination_size_mismatch
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs::tests::output_target_texture_usages_prepare_render_target_only_without_sampled_binding
  - zircon_runtime/src/graphics/scene/resources/output_target_texture/output_target_texture_resource.rs::tests::output_target_texture_usages_preserve_copy_and_sampled_authoring_flags
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_texture_direct_graph_import_readiness
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_texture_direct_graph_import_execution
  - zircon_runtime/src/core/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_capture_source_report
  - zircon_runtime/src/core/framework/render/capture.rs::tests::captured_frame_new_defaults_to_primary_framework_offscreen_source
  - zircon_runtime/src/core/framework/render/capture.rs::tests::texture_capture_report_distinguishes_direct_import_and_conversion_sources
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs::tests::import_frame_targets_rebinds_final_aliases_to_imported_texture_target
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs::tests::compiled_scene_outputs_can_carry_output_target_graph_import_report
  - zircon_runtime/src/graphics/tests/render_debugger_and_history.rs::renderdoc_debug_marker_registry_covers_capture_timeline
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - zircon_runtime/src/core/framework/tests.rs::render_camera_ordering_sorts_by_order_then_target_and_tracks_target_hdr_index
  - zircon_runtime/src/core/framework/tests.rs::render_camera_ordering_reports_ambiguities_and_skips_inactive_cameras
  - zircon_runtime/src/graphics/tests/visibility.rs
  - zircon_runtime/src/core/framework/render/temporal_jitter.rs::tests::render_taa_halton_matches_reference_values
  - zircon_runtime/src/core/framework/render/temporal_jitter.rs::tests::render_taa_jitter_sequence_is_periodic_and_avoids_zero_index
  - zircon_runtime/src/core/framework/render/temporal_jitter.rs::tests::render_taa_jitter_sequence_clamps_zero_period
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs::tests::render_taa_matrix_pair_is_identical_without_jitter
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs::tests::render_taa_matrix_pair_applies_pixel_jitter_in_clip_space
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs::tests::render_taa_jitter_zero_when_taa_inactive
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_temporal_camera_history.rs::tests::successful_submit_records_camera_history_for_next_frame
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs::tests::scene_uniform_exposes_jittered_and_unjittered_current_matrices
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs::tests::scene_uniform_inverse_view_projection_is_unjittered
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/velocity_camera_params.rs::tests::render_velocity_camera_params_use_unjittered_camera_matrices
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs::tests::post_process_projection_params_ignore_temporal_jitter
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs::render_product_temporal_off_matches_anti_alias_feature_disabled_product
  - rustfmt --edition 2021 --check on TP-M2-S1a jitter contract files
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never (blocked by unrelated UI tree-view helper compile errors)
  - cargo fmt --package zircon_runtime
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s1b-0614 --message-format short --color never
  - cargo fmt --package zircon_runtime -- --check
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s2-0614 --message-format short --color never
  - cargo fmt --package zircon_runtime
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s3-0614 --message-format short --color never
doc_type: module-detail
---

# Runtime Render Camera Contracts

## Purpose

`zircon_runtime::core::framework::render::camera` owns the neutral camera data surface used by render extraction and graphics backends. M2A expands the earlier viewport-only snapshot into a Bevy-informed camera contract without moving concrete renderer execution out of `zircon_runtime::graphics`.

The contract is still data-oriented: it describes projection, target, viewport, render order, active state, HDR, exposure, clear color, MSAA sample count, and render layers. Scene and editor systems project their local state into this contract before render graph or RHI code consumes it.

## Bevy Evidence

The M2A shape follows four local Bevy source areas:

- `dev/bevy/crates/bevy_core_pipeline/src/schedule.rs` treats rendering as camera-driven and chooses 2D/3D core schedules from camera setup.
- `dev/bevy/crates/bevy_camera/src/components.rs` defines `Camera2d`, `Camera3d`, and `Hdr` as explicit camera-side product signals.
- `dev/bevy/crates/bevy_camera/src/camera.rs` models camera viewport rectangles, render targets, order, active state, output mode, clear color, and target size calculations.
- `dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs` makes render layer intersection a first-class rule: default entities and cameras are on layer `0`, and an empty layer set is invisible.
- `dev/bevy/crates/bevy_render/src/camera.rs` removes extracted camera components when `Camera::is_active` is false, and `dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs` plus `core_3d/mod.rs` skip phase preparation for inactive cameras.

Zircon keeps the same product semantics but does not copy Bevy ECS components one-for-one. The stable boundary is `ViewportCameraSnapshot`, because current runtime render extraction already passes that snapshot through `RenderViewExtract`, `RenderFrameExtract`, picking, visibility, and renderer preparation.

## Data Model

`ViewportCameraSnapshot` now includes:

- projection data: `projection_mode`, `fov_y_radians`, `ortho_size`, `z_near`, `z_far`, and `aspect_ratio`;
- output data: `target`, optional `viewport`, `order`, `is_active`, `clear_color`, and `msaa_samples`;
- imaging data: `hdr` and `exposure_ev100`;
- visibility data: `render_layers`;
- temporal data: `temporal_jitter`, defaulting to a zero `TemporalJitterSample`.

`temporal_jitter.rs` owns the neutral jitter sample and sequence types for Plan 06. `TemporalJitterSequence::sample(...)` uses Halton base 2/3 with `(frame_index % period) + 1`, matching the URP convention of avoiding Halton index 0. The render-framework submit path now chooses the sample from the effective anti-aliasing state: effective TAA reads `ViewportRecord.temporal_frame_index`, while Off/Fxaa/Msaa and TAA fallback modes force a zero sample. The index advances only after successful submit/present paths, so failed or skipped frames do not consume the jitter sequence.

TP-M2-S3 adds a product-level guard for that neutral path: `render_product_temporal_off_matches_anti_alias_feature_disabled_product` submits the same world extract with `AntiAliasSettings::off()` through AA feature-enabled and feature-disabled WGPU viewports, then requires no AA/FXAA pass execution and byte-identical captured RGBA. The 2026-06-15 artifact audit found no in-repository pre-jitter hash or golden file to recover, so this Off-path product parity test is the accepted repository-local baseline; an external historical artifact can still be added later as supplemental evidence.

`view_matrix_pair.rs` owns `ViewProjectionMatrixPair::from_camera(...)`. It derives an unjittered projection/view matrix from `ViewportCameraSnapshot` and `viewport_size`, then builds the jittered variant as `translate(2*jx/width, 2*jy/height, 0) * unjittered`. `SceneUniform::from_frame(...)` now consumes this pair for the current jittered `view_proj`, explicit `view_proj_unjittered`, `inverse_view_proj` as unjittered world-from-clip, `previous_view_proj_unjittered`, and `jitter_params`; the previous-camera fallback uses the unjittered current matrix. Velocity camera/object paths and screen-space reconstruction use the unjittered matrices, while raster paths keep the jittered current matrix. Post-process SSR projection parameters and reflection-probe screen projection also derive from the unjittered pair or camera scalars, so enabling TAA jitter does not perturb view-space reconstruction.

`RenderViewportRect` stores physical position, physical size, and normalized depth range. `clamped_to_size(...)` mirrors Bevy's viewport containment rule by keeping the rectangle inside the target size before the camera recomputes aspect ratio.

`RenderCameraTarget` currently distinguishes the primary surface, texture targets, and headless targets. Backends can map those variants to native windows, offscreen textures, or no-color-output paths later without making framework consumers depend on WGPU.

`RenderCameraTargetKind` is the lightweight diagnostic projection of those variants. Submit-side stats use it for target-family reporting without exposing texture handles or backend surface objects.

`RenderCameraClearColor` separates default clear policy, no clear, and explicit color.

## Render Layers

`RenderLayerSet` is the Bevy-style layer contract. Its default value is layer `0`; `RenderLayerSet::none()` belongs to no layers and does not intersect anything, including another empty set.

The current scene component still stores a legacy `u32` mask, so M2A provides `RenderLayerSet::from_legacy_mask(...)`, `to_legacy_mask_lossy(...)`, and `intersects_legacy_mask(...)`. This lets scene extraction enforce Bevy-style visibility immediately while leaving the broader scene serialization migration for a later deliberate cutover.

During scene render extraction, `World::build_render_camera(...)` projects the active camera entity's `RenderLayerMask` into `ViewportCameraSnapshot::render_layers`. Mesh, sprite, and scene-light extraction then filters scene entities against the effective camera layer set before building `GeometryExtract`, phase inputs, lighting extracts, and visibility input. Explicit `SceneViewportExtractRequest::camera` snapshots keep their own layer set and can override the scene camera. Request-only projection and viewport-size overrides are applied in this request-aware path, not in the shared scene-component camera snapshot helper.

Scene camera snapshots are active only when `CameraComponent::is_active` is true and the owning entity is active in hierarchy. When `ViewportCameraSnapshot::is_active` is false, scene extraction keeps the camera data available for diagnostics and editor state but emits no scene meshes, sprites, phase inputs, visibility renderables, or scene lights. This mirrors Bevy's inactive-camera path within Zircon's current single-camera extract DTOs.

## Scene And Asset Projection

M2B moves the product fields down into `CameraComponent` and `SceneCameraAsset`. Scene cameras now carry projection mode, orthographic size, target, viewport, render order, active state, HDR, exposure, clear color, and MSAA sample count. Serde defaults preserve older scene and project documents that only stored `fov_y_radians`, `z_near`, and `z_far`.

`SceneCameraTargetAsset` uses asset references for texture targets and contributes those texture references to `SceneAsset::direct_references()`. `World::from_scene_asset(...)` resolves texture targets into `RenderCameraTarget::Texture`, while `World::to_scene_asset(...)` writes component camera targets back to scene asset form. Headless camera targets round-trip through explicit physical sizes and can drive aspect-ratio calculation when no viewport size is supplied by the request.

## Concrete Target Routing

M2C starts concrete target routing at the graphics submission boundary. `RenderCameraTarget::PrimarySurface` still uses the active viewport record size. `RenderCameraTarget::Headless { size }` now resolves the submission size before visibility, history validation, runtime-frame construction, offscreen target allocation, and capture. The renderer clamps zero axes to at least one pixel through the same target-size path used by viewport records.

`RenderCameraTarget::Texture(_)` now resolves the referenced `TextureAsset` during submit preflight. The texture metadata must exist, expose a nonzero 2D single-layer single-mip descriptor, use a renderable RGBA8 format, and include `RenderImageUsage::RenderTarget`; otherwise submit reports `UnsupportedCapability` with `camera texture render target asset`, `camera texture render target extent`, `camera texture render target 2d single-layer`, `camera texture render target format`, or `camera texture render target usage`. A valid texture target uses the texture descriptor width and height as the offscreen submission size, capture size, history target size, and target-resolution diagnostic size.

Surface presentation is primary-surface-only in M2C. A headless camera submitted through `present_frame_extract` reports `UnsupportedCapability { capability: "headless camera surface present" }`, and a texture camera submitted through `present_frame_extract` reports `UnsupportedCapability { capability: "camera texture surface present" }`, so callers cannot accidentally blit a non-surface target into a bound surface.

The 2026-06-06 target-resolution diagnostics follow-up adds `RenderCameraTargetResolutionReport` as submit evidence. `build_frame_submission_context(...)` records the submitted camera target kind, primary viewport size, resolved target size, effective viewport size, and effective render size after target routing and dynamic-resolution application. `update_base_stats(...)` copies that report into `RenderStats.last_camera_target_resolution`, and core diagnostics mirrors it as `render.camera.target.*` bool/count rows. This is reporting only: primary-surface, headless, and valid texture-target submits expose their resolved sizes, invalid texture targets fail before submit stats advance, and WGPU still renders one effective camera.

The 2026-06-06 texture target metadata, residency, and writeback follow-ups extend that report path to valid texture targets. Registered render-target texture metadata with renderable RGBA8 format now advances submit stats with `RenderCameraTargetKind::Texture` and the texture descriptor extent, while unresolved, non-render-target, or non-renderable-format textures still fail before stats advance. This keeps Bevy-style target identity and size resolution separate from direct surface presentation while matching the GPU residency usage mapper that only grants WGPU render-attachment usage to renderable asset formats.

The 2026-06-06 output-target planning follow-up keeps that resolved target identity on renderer-bound frames. `FrameSubmissionContext` now stores a `ViewportRenderOutputTarget`, and both `build_runtime_frame(...)` and direct `submit_runtime_frame(...)` attach it to `ViewportRenderFrame` after preflight. Primary-surface frames use the default marker, headless frames carry the resolved size, and valid texture frames carry the texture handle plus resolved size. WGPU execution still submits one effective camera, but prepared sRGB texture targets can now become the graph final target directly; linear targets use the marker for renderer-owned conversion writeback after graph execution.

The same target marker exposes a `ViewportTextureWritebackPlan`. The plan is deliberately conservative: non-texture frames are `NotRequested`, unresolved target descriptors are `PendingTargetDescriptor`, texture targets whose descriptor format matches the framework offscreen output label `rgba8unorm_srgb` are `ReadyForSrgbCopy`, linear `rgba8unorm` targets are `ReadyForConversion`, and unsupported render-target labels remain `BlockedFormatMismatch`. This prevents every valid render-target texture from being treated as copy-compatible.

The resource streamer consumes the same marker at submit preparation time. When the resolved target is `Texture { handle, size }`, `ensure_scene_resources(...)` prepares that texture through a renderer-private `OutputTargetTextureResource` cache; primary and headless targets skip this path. This cache creates a renderable WGPU texture/view without creating a sampled material bind group, so camera targets can stay render-target-only. During graph execution, an eligible `rgba8unorm_srgb` prepared texture target is imported as the final graph aliases, including `final-color`, `viewport-output`, and final post-process outputs, so the graph writes the final image directly into the prepared target. Linear `rgba8unorm` targets stay on a renderer-owned full-screen conversion pass that samples the framework `rgba8unorm_srgb` final color and writes the prepared linear target. Conversion writes are reported separately and do not reuse the copy marker. Non-texture targets and unsupported descriptors still do not write.

The 2026-06-06 writeback telemetry follow-up adds `RenderCameraTargetWritebackReport` beside the target-resolution report. The streamer records `NotRequested`, `PendingTargetDescriptor`, `ReadyForCopy`, `ReadyForConversion`, `SkippedDirectImport`, `Copied`, `Converted`, or `BlockedFormatMismatch`, plus target extent, submitted copy count, submitted conversion count, and whether the `zircon::TextureWriteback` debug marker was emitted. `SceneRenderer` exposes that report and `update_base_stats(...)` stores it as `RenderStats.last_camera_target_writeback`. Core diagnostics mirrors the same state under `render.camera.target.writeback.*`, including one-hot status rows, `skipped_direct_import`, `copy_count`, `converted_count`, `debug_marker_emitted`, `width`, and `height`. This gives editor/runtime tooling direct-import skip, copy, conversion, and marker evidence without exposing WGPU texture handles.

The 2026-06-07 graph-import follow-ups add `ViewportTextureGraphImportPlan` and `RenderCameraTargetGraphImportReport` beside writeback telemetry. The same resolved output-target marker first reports whether a valid texture target is ready for direct graph import (`rgba8unorm_srgb`), must stay on the conversion writeback path (`rgba8unorm`), or is blocked by a format mismatch. During graph execution, successful sRGB imports are recorded as `DirectImported` with a nonzero direct-import count; readiness-only reports stay `ReadyForDirectImport` with zero direct-import count. `update_base_stats(...)` stores the neutral report as `RenderStats.last_camera_target_graph_import`, and diagnostics mirrors it under `render.camera.target.graph_import.*` with one-hot status rows, direct-import/conversion/block counts, and target extent.

The capture report follow-up adds `RenderCaptureReport` and `RenderCaptureSource` as the neutral readback result for this target path. Primary-surface and headless frames report `FrameworkOffscreen`; sRGB texture targets that were graph-imported report `TextureDirectGraphImport`; linear `rgba8unorm` texture targets that use the fullscreen conversion pass report `TextureWritebackConversion`; and future direct copy readbacks can report `TextureWritebackCopy`. `CapturedFrame`, `ViewportFrame`, `RenderStats.last_capture_report`, and `render.capture.*` diagnostics carry the same target kind, source, output size, graph-import status, and writeback status without exposing WGPU texture handles.

M2C validation used `CARGO_TARGET_DIR=D:\cargo-targets\zircon-render-camera-m2c` on 2026-05-16. `cargo test -p zircon_runtime camera_target --locked --jobs 1 --message-format short --color never` passed the three focused target-routing tests, and `cargo check -p zircon_runtime --lib --locked --message-format short --color never` passed afterward.

## Camera Ordering

M2D adds the neutral ordering contract needed before Zircon expands from single-camera extracts into split-screen and multi-target camera schedules. `sort_render_cameras(...)` accepts camera snapshots paired with entity ids and returns active cameras sorted by render order, normalized target key, and deterministic entity tie-break.

`World::render_camera_order_report(...)` is the scene bridge for that neutral contract. It projects every scene `CameraComponent` through the same request-free `ViewportCameraSnapshot` builder used by direct render extraction, including hierarchy active state, target, HDR flag, order, render layers, and headless target aspect handling, then delegates to `sort_render_cameras(...)`. The report is scheduling evidence only; the current render frame path still extracts one effective camera.

Scene-backed `RenderFrameExtract` carries the selected scene camera entity plus the same ordering report on `RenderViewExtract`. Explicit camera snapshots supplied through `SceneViewportExtractRequest::camera` leave that metadata empty, because they are external view descriptions rather than scene-owned cameras.

The behavior follows Bevy's render-app `sort_cameras` path in `dev/bevy/crates/bevy_render/src/camera.rs:663-722`: active cameras are sorted by `(order, target)`, duplicate active `(order, target)` groups are reported as ambiguities, and each camera receives a `sorted_camera_index_for_target` counted per `(target, hdr)`. Inactive cameras are skipped because Bevy removes inactive cameras from extraction before sorting.

`RenderCameraTargetOrderKey` normalizes Zircon targets without depending on concrete WGPU objects: the primary surface is a single key, texture targets use the stable `ResourceId`, and headless targets use their physical size. This keeps the contract usable by later graphics and editor viewport scheduling while true texture residency remains owned by the asset/GPU resource lane.

M2D validation used WSL/Linux with `CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-render-camera-m2d-wsl` on 2026-05-17. `cargo test -p zircon_runtime --lib render_camera_ordering --locked --jobs 1 --message-format short --color never` passed the two focused ordering tests, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never` passed afterward with only existing unused-function warnings outside this module. Windows validation currently fails earlier in `wgpu-hal 29.0.3` DX12 dependency compilation, before Zircon source is checked.

## Scope Boundary

M2A/M2B/M2C/M2D plus the 2026-06-06 and 2026-06-07 follow-ups still leave editor authoring for the new fields, multi-camera submission, split-screen target routing, imported external texture views, broader output-format policy beyond RGBA8 sRGB/linear, and the later hard cutover from scene `RenderLayerMask(u32)` to `RenderLayerSet` for separate milestones. Texture targets now have two renderer-owned paths: matching `rgba8unorm_srgb` targets import the prepared texture as the graph final target and skip output-target writeback, while linear `rgba8unorm` targets use the fullscreen conversion writeback path. This is direct graph rendering only for the single effective camera's prepared sRGB texture target; it is not broad Bevy image/texture-view target parity.

The M2C entry gate was captured on 2026-05-16 with `CARGO_TARGET_DIR=F:\cargo-targets\zircon-render-camera-m2-1819`: `cargo test -p zircon_runtime camera --locked --jobs 1 --message-format short --color never` passed 13 focused camera/layer/scene-asset tests, and `cargo check -p zircon_runtime --lib --locked --message-format short --color never` passed afterward.
