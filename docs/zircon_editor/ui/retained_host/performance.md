---
related_code:
  - zircon_editor/Cargo.toml
  - zircon_editor/src/ui/retained_host/app
  - zircon_editor/src/ui/retained_host/scroll_surface_host.rs
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/presentation.rs
  - zircon_editor/src/ui/retained_host/app/assets.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer.rs
  - zircon_editor/src/ui/retained_host/app/component_showcase_runtime.rs
  - zircon_editor/src/ui/retained_host/app/pane_payload_visibility.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/retained_host/app/helpers.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_sync.rs
  - zircon_editor/src/ui/host/startup/resolve_session.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/backend.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/error.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/factory.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/host_chrome_presenter.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/asset_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/welcome_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/close_prompt_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/chrome_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/drag_resize.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/pane_button_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/redraw_result.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/resize_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/tab_drag_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/template_hover_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/viewport_toolbar_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/skeleton.rs
  - zircon_runtime/src/rhi/mod.rs
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/pipeline.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_editor/src/ui/retained_host/ui_perf.rs
  - zircon_editor/src/ui/layouts/views/preview_images.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/template_runtime/host_nodes.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_viewport_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_tab_strip.zui
  - zircon_editor/assets/icons/editor_pages
  - docs/zircon_editor/assets/editor-page-function-icon-template-map.md
  - tools/ui-profile-capture.ps1
implementation_files:
  - zircon_editor/src/ui/retained_host/app
  - zircon_editor/src/ui/retained_host/scroll_surface_host.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/presentation.rs
  - zircon_editor/src/ui/retained_host/app/assets.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer.rs
  - zircon_editor/src/ui/retained_host/app/component_showcase_runtime.rs
  - zircon_editor/src/ui/retained_host/app/pane_payload_visibility.rs
  - zircon_editor/src/ui/retained_host/app/welcome_recent_pointer.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/retained_host/app/helpers.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_sync.rs
  - zircon_editor/src/ui/host/startup/resolve_session.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/backend.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/error.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/factory.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/host_chrome_presenter.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/asset_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/welcome_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/close_prompt_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/chrome_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/drag_resize.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/pane_button_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/redraw_result.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/resize_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/tab_drag_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/template_hover_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/viewport_toolbar_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/skeleton.rs
  - zircon_runtime/src/rhi/mod.rs
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/pipeline.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_editor/src/ui/retained_host/ui_perf.rs
  - zircon_editor/src/ui/layouts/views/preview_images.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/template_runtime/host_nodes.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_viewport_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_tab_strip.zui
  - tools/ui-profile-capture.ps1
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/01/2026-07-17-editor-retained-host-app-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-jobs-messaging-static-review.md
  - user: 2026-05-14 retained-host pointer-move and software painter CPU profile
  - user: 2026-05-15 GPU command stream should take over editor UI rendering
  - user: 2026-05-15 retained UI rendering needs poset depth batching
  - user: 2026-05-16 continue UI batching and interaction validation plan
  - .codex/plans/GPU Command Stream 接管 Editor UI 渲染计划.md
  - .codex/plans/UI 合批与交互完整校验计划.md
  - .codex/plans/Retained Host Chrome GPU 化与 Hover 卡顿根因修复计划.md
  - .codex/plans/Retained Host Chrome 性能根因修复计划.md
  - .codex/plans/Zircon UI .zui 组件资产与 Unreal 风格入口重构计划.md
  - docs/superpowers/plans/2026-05-18-editor-sprite-atlas-ui-batching.md
  - docs/superpowers/plans/2026-05-23-editor-pages-template-icon-wiring.md
tests:
  - zircon_editor retained-host performance_tests (2026-07-17 current-source Windows coordinator run pending)
  - zircon_editor/src/core/jobs/progress.rs::tests::primary_snapshot_clones_only_the_smallest_visible_job
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/atlas_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/pipeline.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_editor/src/ui/retained_host/ui_perf.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/host_chrome_presenter.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check; profiling artifact ownership scan; cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-18 after profiling artifact test responsibility split: passed with existing warning noise only; geometry regressions live in `profiling_artifacts/tests.rs`, `profiling_artifacts.rs` is 824 lines, and `profiling_artifacts/tests.rs` is 211 lines)
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check; profiling artifact geometry ownership scan; cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-18 after profiling artifact geometry responsibility split: passed with existing warning noise only; geometry extraction lives in `profiling_artifacts/geometry.rs`, `profiling_artifacts.rs` is 212 lines, and `profiling_artifacts/geometry.rs` is 651 lines)
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_sync.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - cargo test -p zircon_editor --lib host_chrome_presenter --locked
  - cargo test -p zircon_editor --lib render_framework_boundary --locked
  - cargo test -p zircon_editor --lib sync_pane_size_preserves_recent_project_paths --locked
  - cargo test -p zircon_editor --lib draw_rect_clipped --locked
  - cargo test -p zircon_editor --lib draw_rgba_image_clipped --locked --message-format=short
  - cargo test -p zircon_editor --lib template_nodes --locked --message-format=short
  - cargo test -p zircon_editor --lib redraw_region_can_request_frame_update_without_losing_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib redraw_merge_uses_latest_frame_update_scenario --locked --jobs 1 --message-format short
  - cargo test -p zircon_editor --lib redraw --locked --jobs 1 --message-format short
  - cargo test -p zircon_editor --lib native_host_viewport_toolbar_only_dispatches_primary_press --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_pointer_click_routes_late_viewport_toolbar_controls --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_resize_splitter_forwards_move_and_release_after_capture --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_document_tab_drag_releases_capture_and_forwards_drop --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_document_tab_drag_cross_dock_release_uses_center_status_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_document_tab_drag_document_edge_release_uses_center_status_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_document_tab_drag_floating_window_release_uses_floating_center_status_damage --offline --message-format=short
  - cargo test -p zircon_runtime --lib ui_hotspots_collect_gpu_presenter_counters --locked --message-format=short
  - cargo test -p zircon_runtime --lib ui_surface --locked
  - cargo test -p zircon_runtime --lib draw_list_stats_skip_commands_outside_damage --locked
  - cargo test -p zircon_runtime --lib draw_list_stats_do_not_count_cached_images_as_uploads --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_text_bounds_clip_to_damage_and_command_clip --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_text_skips_disjoint_damage --locked
  - cargo test -p zircon_runtime --lib batch_plan_batches_disjoint_quads_into_one_solid_draw --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_draw_items_sort_by_stable_z_order --locked
  - cargo test -p zircon_runtime --lib batch_plan_splits_text_when_overlapping_geometry_depends --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_presenter_uses_damage_for_patch_stats --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_image_cache_prune --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_uses_non_srgb_formats_for_byte_exact_editor_parity --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1
  - cargo test -p zircon_runtime --lib ui_hotspot --locked
  - cargo test -p zircon_editor --lib viewport_image_resource_key_tracks_same_size_content --locked
  - cargo test -p zircon_editor --lib draw_rgba_image_clipped_records_content_scoped_resource_keys --locked
  - cargo test -p zircon_editor --lib viewport_image_patch_can_carry_upload_bytes_for_gpu --locked
  - cargo test -p zircon_editor --lib command_stream --locked
  - cargo test -p zircon_editor --lib command_stream --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_editor --lib recorded_atlas_images_keep_shared_resource_key_and_distinct_uvs --locked
  - cargo test -p zircon_editor --lib recorded_atlas_image_uses_atlas_texture_payload_not_source_payload --locked
  - cargo test -p zircon_editor --lib resolver_reads_project_library_atlas_artifacts_for_template_icon --locked
  - cargo test -p zircon_editor --lib command_stream_replay_samples_atlas_uv_from_embedded_atlas_bytes --locked
  - cargo test -p zircon_runtime --lib batch_plan_batches_disjoint_atlas_images_with_same_key_and_distinct_uvs --locked
  - cargo test -p zircon_runtime --lib wgpu_ui_surface_headless_stats_batch_atlas_images_by_resource_key --locked
  - cargo test -p zircon_editor --lib painter --locked
  - cargo test -p zircon_editor --lib componentized_workbench_status_bar_skips_legacy_skeleton_fill --locked -- --nocapture
  - cargo test -p zircon_editor --lib status_bar --locked -- --nocapture
  - cargo test -p zircon_editor --lib componentized_workbench_activity_rail_skips_legacy_left_region_fill --locked -- --nocapture
  - cargo test -p zircon_editor --lib activity_rail --locked -- --nocapture
  - cargo test -p zircon_editor --lib componentized_workbench_main_band_skips_legacy_center_band_fill --locked -- --nocapture
  - cargo test -p zircon_editor --lib componentized_workbench_component_drawer_skips_legacy_bottom_region_fill --locked -- --nocapture
  - cargo test -p zircon_editor --lib document_tab --locked -- --nocapture
  - cargo test -p zircon_editor --lib componentized_workbench_window_template_bridge_exposes_document_tab_runtime_routes --locked -- --nocapture
  - cargo test -p zircon_editor --lib componentized_workbench_window_template_bridge_exports_surface_projection_frames_and_routes --locked -- --nocapture
  - cargo test -p zircon_editor --lib surface_backed_retained_projection_exposes_style_overrides_as_effective_properties --locked -- --nocapture
  - cargo test -p zircon_editor --lib patch_command_stream_matches_legacy_region_repaint_pixels --locked
  - cargo test -p zircon_editor --lib text_draw_skips_disjoint_active_and_explicit_clips --locked
  - cargo test -p zircon_editor editor_pages_template_icons_have_readable_16px_raster_footprints --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --nocapture
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\global-ui-m3-validation -- --ignored --nocapture
  - cargo test -p zircon_editor --lib draw_rect_clipped_skips_disjoint_active_and_explicit_clips --locked
  - cargo test -p zircon_editor --lib fill_rect_respects_active_paint_clip --locked
  - cargo test -p zircon_editor --lib gpu_presenter --locked
  - cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1 --message-format short --color never
  - cargo check -p zircon_app --features "target-editor-host" --locked
  - 2026-05-15 continuation: cargo check -p zircon_runtime --lib --locked
  - 2026-05-15 continuation: cargo check -p zircon_editor --lib --locked
  - 2026-05-15 continuation: cargo test -p zircon_runtime --lib ui_surface --locked
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib command_stream --locked
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib gpu_presenter --locked
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib render_framework_boundary --locked
  - 2026-05-15 continuation: cargo test -p zircon_runtime --lib ui_hotspot --locked
  - 2026-05-15 continuation: cargo check -p zircon_app --features target-editor-host --locked
  - 2026-05-15 M5: cargo check -p zircon_app --profile profiling --features "target-editor-host profiling profiling-chrome" --locked
  - 2026-05-15 M5: cargo build -p zircon_app --bin zircon_editor --profile profiling --features "target-editor-host profiling profiling-chrome" --locked
  - 2026-05-15 M5: cargo build -p zircon_runtime --lib --profile profiling --features "target-editor-host profiling profiling-chrome" --locked
  - 2026-05-15 M5: tools/ui-profile-capture.ps1 -ScenarioList startup,idle_hover,viewport_image,click,drag,drawer_resize,asset_refresh -AutoCloseSeconds 3 -SkipBuild (20260515-055615 through 20260515-055704)
  - 2026-05-15 workspace expansion: cargo build --workspace --locked --verbose
  - 2026-05-15 workspace expansion: cargo test --workspace --locked --verbose (blocked in zircon_editor template/demo-front tests)
  - 2026-05-15 workspace expansion: cargo test -p zircon_editor --lib --locked --message-format=short (1173 passed, 121 failed, 4 ignored; failure source outside GPU presenter/RHI surface)
  - 2026-05-15 closeout: cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1
  - 2026-05-15 closeout: cargo test -p zircon_editor --lib command_stream --locked --jobs 1
  - 2026-05-15 closeout: cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1
  - 2026-05-15 closeout: cargo test -p zircon_editor --lib render_framework_boundary --locked --jobs 1
  - 2026-05-15 closeout: git diff --check -- GPU command-stream plan/docs/session/touched startup module
  - 2026-05-15 continuation: cargo test -p zircon_runtime --lib app_editor_and_core_framework_sources_do_not_import_wgpu --locked --jobs 1 --message-format=short
  - 2026-05-15 continuation: cargo test -p zircon_runtime --lib production_ui_entry_assets_live_under_crate_assets_not_src --locked --jobs 1 --message-format=short
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib editor_retained_host_presenter_boundary_keeps_wgpu_inside_runtime_rhi --locked --jobs 1 --message-format=short
  - 2026-05-15 continuation: cargo test -p zircon_runtime --lib --locked --jobs 1 --message-format=short -- --test-threads=1 (1349 passed)
  - 2026-05-15 continuation: cargo test -p zircon_editor --lib --locked --jobs 1 --message-format=short -- --test-threads=1 (1298 passed, 4 ignored)
  - 2026-05-15 continuation: cargo test -p zircon_runtime_interface --lib --locked --jobs 1 --message-format=short -- --test-threads=1 (95 passed)
  - 2026-05-15 continuation: cargo test -p zircon_app --lib --locked --jobs 1 --message-format=short -- --test-threads=1 (42 passed)
  - 2026-05-15 continuation: cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 -- --test-threads=1 (8 passed)
  - 2026-05-15 continuation: cargo fmt --all -- --check
  - 2026-05-15 continuation: cargo test --workspace --locked --jobs 1 --message-format=short -- --test-threads=1 (attempted twice; final attempt timed out after 30 minutes without residual processes)
  - tools/ui-profile-capture.ps1 -Scenario startup -AutoCloseSeconds 3 -SkipBuild (20260515-013306-startup)
  - tools/ui-profile-capture.ps1 -ScenarioList startup,idle_hover,viewport_image,click,drag,drawer_resize,asset_refresh -AutoCloseSeconds 3 -SkipBuild (20260515-033851 through 20260515-033926)
  - cargo test -p zircon_editor --lib native_host_close_prompt_button_press_uses_overlay_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_pointer_click_routes_document_tab_with_document_region_origin --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_floating_document_tab_press_uses_floating_window_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_floating_window_header_press_uses_floating_layer_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_drawer_header_tab_press_uses_drawer_region_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_activity_rail_press_uses_center_band_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_pointer_click_routes_host_page_tabs_with_tab_local_point --offline --message-format=short
  - cargo test -p zircon_editor --lib native_host_hierarchy_press_uses_pane_center_status_damage --offline --message-format=short
  - cargo test -p zircon_editor --lib retained_window --offline --message-format=short
  - cargo check -p zircon_editor --lib --locked
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check; window event-loop/template-hover ownership scan; cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-18 after host window responsibility split)
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check; window module-local test ownership scan; cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-18 after host window test responsibility split: passed with existing warning noise only; `window.rs` is 817 lines, `window/tests.rs` is 83 lines)
  - cargo fmt -p zircon_editor; cargo fmt -p zircon_editor --check; native pointer drag/resize ownership scan; cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (2026-06-18 after native pointer drag/resize responsibility split: passed with existing warning noise only; `native_pointer.rs` is 732 lines and `native_pointer/drag_resize.rs` is 242 lines)
  - cargo check -p zircon_editor --lib --tests --locked --message-format=short
  - cargo test -p zircon_editor --lib preview_loader --locked -- --nocapture
  - cargo test -p zircon_editor --lib tests::host::retained_callback_dispatch::template_bridge::workbench_projection::builtin_host_window_template_bridge_recomputes_surface_backed_frames_with_shell_size --locked -- --exact --nocapture
  - cargo test -p zircon_editor --lib tests::host::retained_callback_dispatch::asset::template_bridge::builtin_asset_surface_open_browser_dispatches_static_binding_from_template --locked -- --exact --nocapture
  - cargo test -p zircon_editor --lib startup_session --locked -- --nocapture
  - cargo test -p zircon_editor --lib create_project_and_open_persists_recent_project_and_returns_project_session --locked -- --nocapture
  - cargo test -p zircon_editor --lib workbench_main_interface_entries_are_template_backed_and_reflected --locked -- --nocapture
  - cargo test -p zircon_editor --lib visual_assets --locked -- --nocapture
  - cargo test -p zircon_editor --lib builtin_asset_surface_minimal_bridge_dispatches_without_startup_runtime --locked -- --nocapture
  - cargo test -p zircon_editor --lib builtin_welcome_surface_minimal_bridge_dispatches_without_startup_runtime --locked -- --nocapture
  - cargo check -p zircon_app --features "target-editor-host" --locked --message-format=short
  - cargo check -p zircon_app --profile profiling --features "target-editor-host profiling profiling-chrome" --locked --message-format=short
  - cargo build -p zircon_app --bin zircon_editor --profile profiling --features "target-editor-host profiling profiling-chrome" --locked --message-format=short
  - 2026-05-15 hover patch closeout: cargo test -p zircon_editor --lib native_host_template_node_move_updates_hover_without_rebuilding_presentation --locked --jobs 1 --message-format=short
  - 2026-05-15 hover patch closeout: cargo test -p zircon_editor --lib native_host_hierarchy_move --locked --jobs 1 --message-format=short
  - 2026-05-15 hover patch closeout: cargo test -p zircon_editor --lib native_host_asset_template --locked --jobs 1 --message-format=short
  - 2026-05-15 hover patch closeout: cargo test -p zircon_editor --lib native_host_asset_tree_move_updates_visible_hover_state --locked --jobs 1 --message-format=short
  - 2026-05-15 hover patch closeout: tools/ui-profile-capture.ps1 -Scenario idle_hover -AutoCloseSeconds 3 -AutoInteract -RequireScenarioEvidence -SkipBuild (20260515-211644-idle_hover)
  - 2026-05-16 viewport closeout: cargo test -p zircon_editor --lib frame_update_region_queues_external_redraw_with_frame_update --locked --jobs 1 --message-format=short
  - 2026-05-16 viewport closeout: cargo test -p zircon_editor --lib close_requested_callback_can_mutate_host_state_without_reentrant_borrow --locked --jobs 1 --message-format=short
  - 2026-05-16 viewport closeout: cargo test -p zircon_runtime --lib ui_surface --locked --jobs 1 --message-format=short (35 passed)
  - 2026-05-16 viewport closeout: cargo test -p zircon_runtime --lib ui_hotspot --locked --jobs 1 --message-format=short (9 passed)
  - 2026-05-16 viewport closeout: cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1 --message-format=short (2 passed)
  - 2026-05-16 viewport closeout: cargo test -p zircon_editor --lib command_stream --locked --jobs 1 --message-format=short (7 passed)
  - 2026-05-16 viewport closeout: cargo check -p zircon_app --features target-editor-host --locked --message-format=short
  - 2026-05-16 viewport closeout: tools/ui-profile-capture.ps1 -Scenario startup -RequireScenarioEvidence -AutoCloseSeconds 3 -SkipBuild (20260516-001744-startup)
  - 2026-05-16 viewport closeout: tools/ui-profile-capture.ps1 -Scenario idle_hover -AutoCloseSeconds 3 -AutoInteract -RequireScenarioEvidence -SkipBuild (20260516-001914-idle_hover)
  - 2026-05-16 viewport closeout: tools/ui-profile-capture.ps1 -Scenario viewport_image -AutoCloseSeconds 3 -AutoInteract -RequireScenarioEvidence -SkipBuild (20260516-000208-viewport_image)
  - 2026-05-16 viewport closeout: tools/ui-profile-capture.ps1 -Scenario click -AutoCloseSeconds 3 -AutoInteract -RequireScenarioEvidence -SkipBuild (20260516-002011-click)
  - tools/ui-profile-capture.ps1 -SkipBuild -Scenario startup -AutoCloseSeconds 8
  - tools/ui-profile-capture.ps1 -Scenario startup -AutoCloseSeconds 3 -SkipBuild (20260514-223023-startup)
  - tools/ui-profile-capture.ps1 -Scenario startup -AutoCloseSeconds 3 -SkipBuild (20260514-225912-startup)
  - tools/ui-profile-capture.ps1 -Scenario startup -AutoCloseSeconds 3 -SkipBuild (20260514-232056-startup)
doc_type: module-detail
---

# Retained Host Performance

The retained editor host now defaults native editor windows to the runtime-owned GPU presenter. `zircon_editor` still owns retained UI state, pointer damage, and command generation, but it does not own a raw `wgpu` swapchain. `ChromeCommandStream` is the backend-neutral draw list consumed by both `GpuChromePresenter` and `SoftbufferHostPresenter`; the GPU path converts it into `zircon_runtime::rhi::UiSurfaceDrawList`, while softbuffer repaints the same stream into a CPU `HostRgbaFrame` only for fallback, tests, and snapshots.

The softbuffer fallback presenter is now split by runtime cost owner. `presenter/softbuffer/diagnostics.rs` owns same-frame refresh overlay planning, full/region paint accounting, command-stream counters, and verbose summaries; `presenter/softbuffer/surface_io.rs` owns size clamp, resize, damage bounds, damage pixel counts, damage rects, and RGBA-to-softbuffer copy. `presenter/softbuffer.rs` remains the lifecycle and present orchestration owner.

The crate boundary is still intentional. `zircon_editor/Cargo.toml` may depend on `winit` and `softbuffer` for the host window and fallback presenter, but retained-host presenter sources must not reference `wgpu::` or name concrete `rhi_wgpu` providers. Runtime viewport rendering continues to route through the shared `RenderFramework`, and retained editor chrome reaches native GPU presentation through `zircon_runtime::rhi::UiSurfaceDescriptor::from_winit_window(...)` plus `zircon_runtime::rhi::create_default_ui_surface_presenter(...)`.

That means the retained host has two different performance surfaces:

1. Presentation data work before painting. Pointer callbacks must not rebuild full editor chrome snapshots unless the interaction truly needs fresh project or workbench data.
2. Command-stream and presentation work after retained data is ready. Normal native windows should record `gpu_draw_calls > 0` and `software_fallback_present_count == 0`; fallback software painting remains a bounded migration and failure path.

The command stream executor and the legacy painter must agree on damage semantics while the migration is active. Active paint clips are now authoritative even when a command also has its own local clip: if the active damage and explicit command clip do not intersect, the primitive, image, or text draw is skipped instead of falling back to unclipped painting. This protects both the softbuffer fallback and screenshot tests from repainting status/menu chrome outside a viewport-image patch. `patch_command_stream_matches_legacy_region_repaint_pixels` compares legacy region repaint and command-stream replay byte-for-byte after a viewport image upload, while the primitive/text clip tests guard the lower-level failure mode.

Image resource keys are part of that parity contract. The GPU runtime caches textures by `resource_key`, while softbuffer can still read the embedded RGBA bytes directly. Therefore recorded viewport, retained, SVG, preview, and generic RGBA image commands use asset/cache identity or content-derived hashes instead of dimension-only keys; otherwise two same-sized images in one frame could render as the last texture uploaded on GPU while still appearing correct in softbuffer. Because viewport keys can change with frame content, runtime also bounds its wgpu image cache with least-recently-used pruning instead of keeping every historical viewport texture alive.

The latest automated profile sweep on 2026-05-15 covered `startup`, `idle_hover`, `viewport_image`, `click`, `drag`, `drawer_resize`, and `asset_refresh` with auto-close. Each exported `ui_hotspots.json` reported zero UI alerts and `software_fallback_present_count=0`; startup recorded `gpu_draw_calls=243` and `gpu_upload_bytes=18288`. `20260515-033857-idle_hover` also produced two region patch presents with `chrome_command_patch_count=2`, `gpu_draw_calls=144`, `chrome_snapshot_count=0`, `workbench_model_build_count=0`, and `presentation_rebuild_count=0`. Other non-startup scenarios remain smoke captures unless the capture actually synthesizes interaction input.

The M5 follow-up sweep from `20260515-055615-startup` through `20260515-055704-asset_refresh` revalidated the profiling build after the demo-front `.zui` changes. Startup stayed on the GPU presenter with `software_fallback_present_count=0`, `gpu_draw_calls=366`, `gpu_upload_bytes=32968`, and zero UI hotspot alerts. The auto-close non-startup captures also had zero alerts, but they did not synthesize real pointer or viewport-image interaction patches, so they are smoke evidence rather than a replacement for manual interaction capture. Their CPU first-fix candidates are dominated by `load_component_showcase_templates`, which belongs to the active component-showcase startup plan rather than the GPU presenter path.

The later `-AutoInteract -RequireScenarioEvidence` path makes hover acceptance deterministic. The active startup document can be `UiComponentShowcase`, so idle-hover validation cannot rely only on hierarchy rows being visible. Template-node pointer move now stores `hovered_template_control_id` and its absolute frame in `HostPaneInteractionStateData`, overlays that state when `window.rs` publishes the presentation, and repaints only the old/new template-node frames through `native_pointer/template_hover_damage.rs`. The regression `native_host_template_node_move_updates_hover_without_rebuilding_presentation` proves the path changes visible hover state without a presentation rebuild and that repeated same-target hover stays idle. `20260515-211644-idle_hover` passed the tightened gate with `redraw_region_count=1`, `gpu_draw_calls=10`, `gpu_visible_draw_items=12`, `gpu_batch_layers=9`, `gpu_batch_dependencies=60`, and zero alerts/fallbacks.

## Welcome Recent Pointer Move

`HostPresentationCache` stores the workbench snapshot, recent-project paths, and console status text produced by the last committed presentation pass. Pointer callbacks read this cache instead of asking `EditorEventRuntime` for a fresh `EditorChromeSnapshot`. This keeps callback routing on the same retained data that was used to build the visible frame.

`RetainedEditorHost::welcome_recent_pointer_clicked`, `welcome_recent_pointer_moved`, and `welcome_recent_pointer_scrolled` now avoid `runtime.chrome_snapshot().welcome`. Click uses the cached recent-project list before dispatch, while move and scroll call `sync_welcome_recent_pointer_size()` and preserve `WelcomeRecentPointerLayout::recent_project_paths`. This removes the high-frequency path through `EditorEventRuntime::chrome_snapshot`, `EditorChromeSnapshot::build`, descriptor cloning, and presentation-data drop work shown in the 2026-05-14 profile.

The same rule applies to console/detail scrollers and floating-window size resolution: pointer-time helpers use committed cache and host frames first. If the required frame has not been committed yet, callback-provided dimensions or the existing cached size win; the pointer path does not rebuild the workbench model to recover it.

The invariant is that pointer move may update hover state after dispatch and publish that state to the UI, but it must not fetch a new chrome snapshot to perform hit testing. `pointer_handlers_do_not_force_slow_path_rebuilds` now rejects both `recompute_if_dirty(` and `chrome_snapshot(` inside high-frequency pointer modules.

Profiling exports enforce the same rule at runtime. Hover counters that report a chrome snapshot pull or workbench model build trigger `hover_rebuilt_chrome_snapshot_or_model`, while a region redraw that still falls back to a full-frame paint triggers `region_request_repainted_full_frame`. These two alerts separate presentation-data churn from software-painter damage fallback, which keeps the next optimization step grounded in captured evidence.

Pointer redraw classification is intentionally conservative. Presses that can invoke editor commands still request a frame update because the callback may dirty presentation or layout. Pure local interactions are allowed to stay regional: menu wheel scrolling repaints the menu chrome damage, template-node hover repaints only the previous/current node frames, and pane mouse release repaints the pane content frame to clear pressed state without asking the retained host to rebuild the editor tree.

Text input focus is also treated as paint-local state. A primary press first records the active edit frame before clearing focus; if the click lands on viewport or inert pane space, the host repaints only that old edit frame. Switching directly between two text fields unions the old and new input frames so both focus outlines update without a full-frame request. Decorative template nodes that have no action, binding, commit route, or dispatch metadata no longer request a repaint when clicked, because there is no visible or model state to update.

Menu clicks now follow the same retained-state rule. The dispatcher measures menu damage before and after the callback mutates `HostMenuStateData`, then repaints the union of those regions. This covers both closing a popup and opening a new popup without using a full host frame request; menu hover and wheel scroll already use the same menu damage area.

When a primary menu click also clears an active text input, the dispatcher unions the old edit frame into that same menu damage request. The result stays a regional repaint, but both the popup/menu change and the removed focus outline are covered by one damage region.

The menu hit-test and damage geometry lives in `native_pointer/menu_geometry.rs` so the pointer dispatcher remains responsible for routing and scenario counters, while popup placement and menu damage math stay isolated. This is deliberately a small ownership split: the module owns menu rectangles only, not command dispatch or editor state mutation.

The native pointer routing/redraw split keeps high-frequency dispatch timing in `native_pointer.rs` while moving hit-route construction into `native_pointer/routing.rs` and `NativePointerDispatchResult` damage wrapping into `native_pointer/redraw_result.rs`. This keeps profiling scenario counters and callback timing in the dispatcher, while route geometry and redraw-result composition can evolve without expanding the root pointer file.

The native pointer drag/resize split keeps capture-state progression in `native_pointer/drag_resize.rs`. Resize and tab-drag move/release paths still short-circuit before ordinary menu/pane routing, but the root dispatcher now delegates state updates, host callback forwarding, drag thresholding, source payload lookup, release redraw, and capture clearing to the child module.

The template-node hit-test test split keeps the high-frequency production route compact. `surface_hit_test/template_node.rs` now only owns pane/workbench template-node hit testing, popup row synthesis, dispatchable filtering, and surface-frame construction, while `surface_hit_test/template_node_tests.rs` owns the former inline Workbench dropdown/menu/text-input/decorative-layer regressions.

The host window split keeps native event-loop and transient hover presentation costs visible. `window/event_loop.rs` owns winit event matching, presenter fallback, external redraw draining, present scenario attribution, and profiling artifact export; `window/template_hover.rs` owns cloned presentation mutation for template-node, dropdown-option, and popup-menu hover state. `window.rs` therefore stays focused on host state and text-input dispatch instead of becoming another high-frequency event and model-rewrite owner.

The host-window test split moves private state regressions into `window/tests.rs`. Those tests cover diagnostics overlay text, close-request callback mutation outside active borrows, frame-update redraw requests that preserve damage regions, and one-shot completed scenario reads, so `window.rs` can keep the production state boundary without inline test bodies.

The profiling artifact geometry path is split into `profiling_artifacts/geometry.rs`. The production artifact writer now stays focused on export gates, schema DTOs, screenshot capture, and profile output paths, while geometry extraction owns splitter/tab/activity/template/viewport-toolbar frame collection, surface-frame clipping, route-hit sampling, and top-hit filtering.

The profiling artifact tests are split into `profiling_artifacts/tests.rs` because they exercise private geometry extraction helpers rather than a public crate surface. The module-local test owner covers geometry edge cases without expanding the hot profiling export file.

Viewport toolbar clicks now have a middle path between paint-only and full-frame invalidation. `HostRedrawRequest::Region` can request a frame update while preserving its damage rectangle, so common toolbar controls such as tool, projection, display, grid, snap, and preview toggles can apply their Rust callback state patch and then repaint only the toolbar frame. Commands that can affect camera, session, or status state broadly, such as play mode, frame selection, and view alignment, use `native_pointer/viewport_toolbar_damage.rs` to repaint the center band plus status bar instead of the full native host.

Chrome press handling has the same middle path for local top-level controls. `native_pointer/chrome_damage.rs` maps root document-tab presses to the document dock, floating document-tab presses to the owning floating-window frame, drawer-header tab presses to the owning drawer dock, and activity-rail presses to the center band because they can open, close, or swap side drawers and move the document area. Floating-window header focus uses the union of all floating-window frames because focusing can reorder the floating layer, while still avoiding a full host repaint. Host-page tab activation now damages the page chrome tab/project-path/template-node area plus the center band and status bar, so page switches can update presentation state without repainting the menu/title chrome.

Generic pane button presses now use `native_pointer/pane_button_damage.rs` instead of falling back to a full host frame. The callback can still refresh presentation state, but the damage is limited to the pane body plus the retained center band and status bar. This covers hierarchy/asset-style selection changes and status updates while avoiding menu/title chrome repaint on every pane click.

Close prompt button presses also preserve bounded damage. `native_pointer/close_prompt_damage.rs` returns the prompt overlay/dialog union, then the dispatcher requests a frame update with that region instead of `Full`. If the overlay intentionally covers the whole native window, the painted area can still be large, but profiling now records it as an explicit region request rather than an unclassified full-frame fallback.

Drawer resize uses the same middle path. Resize press, move, and release mutate transient layout state, so they still request a frame update, but their damage rectangle is the committed center band rather than the full native window. The center band is intentionally conservative: it covers drawer, document, splitter, and viewport layout shifts while leaving top menu chrome and the status bar out of the repaint. `native_pointer/resize_damage.rs` owns that geometry rule so later refinement can narrow left/right/bottom damage without expanding the main pointer dispatcher again.

When multiple frame-update redraw requests coalesce before a native `RedrawRequested`, the latest frame-update scenario owns the merged request. This matters for profile captures because a resize press can first queue a `click` frame update and the subsequent resize move/release queues `drawer_resize`; presenter counters must then be attributed to the interaction that produced the final retained frame. `HostRedrawRequest::merge` still unions region damage and preserves the frame-update bit, but region-region merges now replace the scenario with the later frame-update request instead of keeping the earlier one. `redraw_merge_uses_latest_frame_update_scenario` covers both region-region coalescing and a full-frame request followed by a later regional frame update, preventing drawer-resize or asset-refresh GPU presents from being hidden under an earlier click/startup label.

Tab drag release now follows the same measured-damage pattern for resolvable dock drops. The release callback is allowed to refresh the drag target first, then `native_pointer/tab_drag_damage.rs` compares the source group and active target group. Same-dock drops repaint the owning dock plus status damage; cross-dock drops between known local docks and document-edge splits repaint the retained center band plus status. Drops onto an existing floating window repaint the floating-window frame unioned with center/status when the source was a local dock. Detach targets intentionally remain full-frame because the newly created floating-window bounds are not present in the pre-dispatch presentation packet.

## Startup Chrome Projection

Startup profiling showed that hidden chrome work could dominate before the first usable frame. The root menu previously instantiated every popup template tree during `scene_menu_models`, even though only the menu bar itself is visible at startup. The host scene now stores menu item data plus popup dimensions and leaves root popup template nodes empty. When a menu is actually open, `draw_open_menu_popup` paints rows from retained menu item data; if a future path supplies real popup template nodes, the painter still honors them.

Dock headers use the existing procedural fallback chrome for the production first-frame path. This retains tab hit frames, close buttons, active-state metadata, and subtitle frames without paying a v2 surface layout pass for each dock band. The v2 dock/header assets remain in the repository for authoring tests and future GPU-backed chrome experiments, but they are no longer required to draw the first frame.

Icon and preview loading is cached in `preview_images.rs`. `load_preview_image` now caches by source and icon name so repeated tab, rail, and menu icons reuse the already-rasterized `Image` instead of re-reading SVG files and reloading font data. The 2026-05-14 startup captures show the effect: `scene_menu_popup_nodes` dropped from seconds to placeholder microseconds, `scene_menu_chrome` dropped to a few milliseconds, and the next dominant cost moved to software presenter repaint plus first-use icon/header work.

SVG tree parsing is also cached in `painter/visual_assets.rs`. The cache key uses canonical path, modification time, and file length, then stores the parsed `usvg::Tree` on the heap for reuse by subsequent raster/tint requests. The `20260514-215427-startup` trace shows the cache lookup itself in low microseconds; cold first-frame icon cost is now mostly unique SVG rasterization and template-node image pixel preparation rather than repeated tree parsing.

The 2026-05-25 editor-page icon validation adds a compact readability guard to that same retained-host path. `editor_pages_template_icons_have_readable_16px_raster_footprints` renders the unique wired `editor_pages` template icons at 16 x 16 px, using the production template image pipeline and current icon tint, then rejects missing, blank, collapsed, or full-slot footprints. This keeps compact toolbar/menu/dock icon regressions visible as a painter test instead of relying only on manual screenshots. The closeout rerun passed with 1 test, 0 failures, and 1510 filtered, and printed per-icon `ICON_16PX_READABILITY` footprints. The complementary ignored screenshot gate still writes the accepted M3 GUI artifact bundle under `target/visual-layout`, including small and large SVG icon scaling captures.

The later 2026-05-14 captures narrowed startup further. The component showcase runtime is now lazy: normal editor startup keeps a compact shared builtin runtime for bridge documents and first-screen pane bodies, and `component_showcase_runtime.rs` loads the showcase-only templates only when that view is visible or dispatching a showcase interaction. This removed the previous `new_load_builtin_templates` startup cliff.

Icon-only template preview calls also avoid startup raster work. When a template node has no image source and only names an icon, `preview_images.rs` returns fixed 24 by 24 metadata after confirming the icon exists. The painter still loads the real icon pixels when it needs to draw at the final size, but host-scene construction no longer parses and rasterizes SVGs just to compute preview metadata. The `20260514-201128-startup` report shows `apply_build_host_scene_data` dropping from roughly 142 ms to roughly 69 ms after this change.

Pane body projection now receives the same startup runtime instead of falling through to the static full builtin runtime on first use. The `20260514-201811-startup` report showed `convert_pane_hierarchy` spending about 203 ms in that fallback. `pane_data_conversion/mod.rs` now exposes runtime-aware hierarchy, inspector, console, and animation conversion paths, and `apply_presentation.rs` passes the shared runtime through the first-frame pane conversion. In `20260514-203032-startup`, that hierarchy fallback disappears from the top spans and `recompute_apply_presentation` drops to roughly 87 ms.

Hidden expensive panes are visibility-gated as well. `pane_payload_visibility.rs` only collects module-plugin and build-export payloads when the active view can show them. The latest `20260514-203843-startup` report has `retained_host:new` around 110 ms, `recompute_if_dirty` around 104 ms, `recompute_apply_presentation` around 86 ms, and `apply_build_host_scene_data` around 74 ms. The remaining visible retained-host costs are startup-session resolution, host-scene assembly, first full paint, and software presenter repaint/copy/present. `async_resolve_render_framework` can still dominate the aggregate trace, but it is on the asynchronous viewport/render-service path rather than the retained-host constructor path.

The following startup slice removes fixed-metric asset probing from host-scene construction. `surface_metrics_from_chrome_assets` keeps the public projection boundary but returns the fixed shell heights used by the authored v2 chrome controls. This avoids building menu, page, and dock header surfaces only to read `WorkbenchMenuTopBar`, `WorkbenchPageBar`, and `DockHeaderBar` heights. `20260514-212044-startup` shows `scene_surface_metrics` reduced to 0-1 us; `apply_build_host_scene_data` drops to roughly 51 ms, and the no-native-window case keeps `recompute_native_window_presenters` in single-digit microseconds because hidden native presenter payloads are no longer prepared when there are no target windows.

Startup session resolution is now split into its real work phases. `20260514-212822-startup` showed that project-mode startup still validated all recent projects before opening the last project. `resolve_session.rs` now leaves `recent_projects` empty while the last project is valid and only validates the recent list when startup falls back to Welcome, where the list is actually visible. `20260514-214204-startup` removes `validate_recent_projects` from that project-mode hot path and reduces `new_resolve_startup_session` to roughly 29 ms. The remaining session cost is `validate_last_project` plus `open_last_project`, which is real project/asset workspace setup rather than avoidable welcome-list work.

The next startup profile closed a repeated first-tick refresh loop. The host now drains asset, editor-asset, and resource events that were queued by bootstrap itself immediately after `sync_asset_workspace()`. Those events represented data already pulled into the startup snapshot and default scene; replaying them during the first event-loop tick reloaded the default scene and caused a second full presentation rebuild. `20260514-222022-startup` identified the loop with new asset-refresh counters: four asset changes, three editor-asset changes, and four resource changes drove `asset_refresh_reload_default_scene`. After the drain, `20260514-223023-startup` reports `asset_refresh` at roughly 0.02 ms, the retained-host tick at roughly 0.38 ms, startup slow-path rebuilds reduced from 2 to 1, presentation rebuilds from 4 to 2, and workbench model builds from 3 to 2. Initial queued event counts are still emitted as `ui.startup.drained_*_change_count`, so later regressions can distinguish bootstrap residue from real file-system or runtime asset updates.

The same profile pass added finer asset-refresh spans and counters for future captures. `refresh_project_assets` now records incoming asset/editor/resource event counts and plan flags for catalog sync, resource sync, selected-details refresh, visible-preview refresh, default-scene reload, and render/presentation/paint-only invalidation. This keeps the next asset-refresh optimization grounded in the exact trigger class instead of treating all refresh work as one opaque frame cost.

The next retained-host startup slice removed two more eager startup costs. `load_startup_builtin_template_runtime` now loads only first-screen shell, drawer, floating-window, viewport-toolbar, inspector, pane, hierarchy, inspector body, and console body documents. The hidden asset-surface and welcome-surface dispatch documents are compiled lazily through `BuiltinAssetSurfaceTemplateBridge::new_minimal` and `BuiltinWelcomeSurfaceTemplateBridge::new_minimal` when a click/change route actually needs them. The `20260514-225912-startup` capture proved that these bridges no longer appear during startup, and the follow-up capture `20260514-232056-startup` reports `new_load_shared_builtin_templates` at roughly 33 ms.

Startup last-project resolution also no longer pre-validates the same project document it is about to open. The valid auto-open path calls `open_project` once and falls back to Welcome validation only if that open fails. This removes the previous `validate_last_project` span from the hot path and drops `new_resolve_startup_session` from roughly 83 ms in `20260514-225912-startup` to roughly 29 ms in `20260514-232056-startup`. In that same capture, retained-host construction falls from roughly 157 ms to roughly 82 ms. The remaining visible retained-host startup work is now scene projection plus the software first paint; the large `async_resolve_render_framework` span remains on the asynchronous viewport/render-service side.

## Software Rect Painting

The softbuffer fallback command-stream executor still writes into a contiguous RGBA byte buffer for tests, screenshots, and no-GPU fallback. `draw_rect_clipped` now clips once, converts the target frame into a `PixelRect`, and fills each horizontal row span directly. Fully opaque colors copy four-byte pixels into each chunk, while translucent colors precompute alpha and inverse-alpha once per span and blend pixels in place.

This keeps the semantics of the old `write_pixel` path: transparent colors are ignored, opaque colors replace destination pixels, and translucent colors blend RGB over the destination while forcing alpha to 255. The difference is that clipping, row offset calculation, and alpha setup now happen outside the inner pixel loop. `draw_separator_line` uses the same span helper for one-row separators.

`HostRgbaFrame::filled` and `HostRgbaFrame::fill_rect` now use the same contiguous span replacement pattern, so full-frame initialization and region-damage clearing avoid an extra nested per-pixel API path. `write_pixel` remains available for image sampling and other per-pixel cases where each target pixel can map to a different source pixel. Rectangles and separators should prefer the span helpers because they have uniform color and contiguous memory access.

Image painting has the same fast/slow split. When a clipped RGBA image maps 1:1 to target pixels and all affected source pixels are opaque, the painter copies whole row spans into the host frame. Scaled or translucent images still sample per target pixel, but that path writes directly into the destination buffer instead of calling the older per-pixel frame API. The intent is to keep viewport and preview images from dominating profiles through tiny `copy_from_slice` calls and repeated offset recalculation.

Text drawing now caches `fontdue` glyph rasters by glyph id and pixel size for the process lifetime. Retained chrome redraws the same labels, tab titles, and diagnostic strings repeatedly; caching avoids re-rasterizing those glyph bitmaps during hover and region repaint. Opaque glyph pixels also write RGBA channels directly instead of going through a four-byte slice copy in the inner loop.

Template-node painting now applies the active region-damage clip before command generation. The pane clip is intersected with `HostRgbaFrame::paint_clip()`, nodes outside that effective damage region are skipped, and preview/icon image commands also check the effective clip before rasterizing pixels. This keeps region repaint from preparing text and image work for controls that the final primitive clip would discard anyway.

## Presenter And GPU Migration Boundary

The presenter boundary lives under `host_contract/presenter/`. `HostChromePresenter` is the object-safe seam used by `window.rs`; `HostPresenterBackend::default_native()` returns `Gpu`, and `HostPresenterBackend::fallback()` is the explicit softbuffer path used only when GPU presenter creation fails. Window startup logs the selected backend and exits only if both GPU and softbuffer construction fail.

`ChromeCommandStream` is now the retained-host command surface, not a parallel stub. The retained painter can run in record-only mode, so `record_host_frame_commands` traverses the same workbench, template-node, viewport image, close prompt, floating-window, menu, debug overlay, rect, border, image, and text paths that CPU painting uses. Full streams describe the complete retained UI; patch streams carry the same command vocabulary but clip generation to the requested damage region. Image commands preserve resource/content-derived keys through record, stream conversion, GPU upload, and softbuffer replay. Atlas-capable image commands additionally carry `atlas_uv: Option<ChromeImageUvRect>`: recorded viewport images set it to `None`, while untinted retained template/image sources can resolve generated SpriteAtlas manifests under `.zircon/cache/editor-sprite-atlases` into one atlas texture `resource_key` plus per-entry UV metadata. Full-opacity atlas images record the shared atlas texture key; opacity-baked images keep the non-atlas content key so tinted/translucent pixels do not mutate shared atlas semantics. This keeps GPU and softbuffer fallback on one UI expression instead of letting two painters drift.

The componentized Workbench status bar is a region-level hard cutover inside that command stream. When `workbench_window_nodes` exist, the Workbench template draws `WorkbenchStatusBar`; the root skeleton no longer records the legacy `STATUS_BAR` quad or `host_shell.status_secondary` marker for the same frame. The non-componentized retained path still keeps the old status bar fallback, but componentized Workbench frames now have one status-bar pixel source.

Activity rail follows the same single-source rule. When a componentized Workbench frame exposes `ActivityRailRoot`, `WorkbenchWindowActivityRail`, or `WorkbenchMainBandActivityRail`, the root skeleton no longer paints the old left-region `SIDE_PANEL` quad through that rail rectangle. The remaining left drawer area is still filled by the retained fallback until drawer migration is complete, but the rail pixels now come from the Workbench activity-rail template instead of being covered by the legacy shell background.

The main band has the same command-stream guard for the old center chrome fill. When componentized Workbench nodes expose `WorkbenchWindowMainBandRegion` or `WorkbenchMainBand`, the retained root skeleton splits the legacy `CENTER_BAND` quad around that frame. Document, viewport, drawer, tab-drag, and pointer-route fallbacks remain until their planned hard-cutover slices, but the main-band frame no longer receives the broad legacy center-band background underneath the runtime Workbench template.

The component drawer lower band follows the same single-source guard. When `workbench_window_nodes` expose `WorkbenchWindowComponentDrawerRegion` or `WorkbenchComponentDrawer`, the retained root skeleton splits the legacy bottom-region `SIDE_PANEL` quad around that frame. The component drawer therefore receives pixels from `WorkbenchComponentDrawer` instead of being covered by the old bottom-dock fill; drawer resize and header pointer bridges remain on the retained fallback path until the dedicated M2.S3 input cutover.

The componentized Workbench skeleton guards now live in `workbench_skeleton_regions.rs` instead of accumulating in the large `workbench.rs` painter. That module owns the control-id ownership markers, visible-frame filtering, and rectangle-splitting helper used by the root skeleton, while `workbench.rs` keeps only the draw order and fallback orchestration. The behavior is intentionally unchanged: status bar, activity rail, main band, and component drawer pixels still follow the same single-source rules, and retained drawer resize/header pointer bridges are still pending a later hard cutover.

The next Workbench surface-projection baseline moves document-tab routing and component-drawer samples onto the same materialized surface path. `workbench_viewport_panel.zui` mounts `DocumentTabsRoot` above the viewport toolbar and surface, with Change/Submit routes normalized to dock focus/close commands. The component drawer now projects `WorkbenchInputDisabled` and forwards `WorkbenchLabsTabOne/Two/Three` through `WorkbenchTabStrip`'s default slot, so component-library samples, disabled fields, selected list/table rows, and Labs tab state all enter the host contract instead of depending on legacy painter-only placeholders. Retained projection also folds surface `style_overrides` over metadata attributes before building effective properties, which lets instance-level Workbench gizmo colors and tab/list/table selected colors override neutral stylesheet defaults in native command-stream assertions.

`GpuChromePresenter<P: UiSurfacePresenter>` converts the chrome command stream into `zircon_runtime::rhi::UiSurfaceDrawList`. It records command full/patch counters, propagates runtime surface failures instead of hiding them, and emits `gpu_upload_bytes`, actual `gpu_draw_calls`, `gpu_visible_commands`, `gpu_visible_draw_items`, `gpu_batch_layers`, and `gpu_batch_dependencies`. It also maps `ChromeImageUvRect` directly into `UiSurfaceImageUvRect`, so atlas metadata crosses the editor/runtime boundary without exposing editor asset-manager or renderer-specific types. It never imports `wgpu`, and its factory no longer names `rhi_wgpu`; the concrete native surface, swapchain, offscreen retained UI target, quad/image pipelines, glyphon text atlas, texture uploads, batch planning, and surface present are selected by the runtime RHI factory.

Softbuffer replay samples atlas UV metadata when atlas-backed payloads embed full atlas RGBA bytes. The software command-stream executor extracts the atlas subimage described by `ChromeImageUvRect`, paints that subimage through the same clipped RGBA path, and falls back to `FALLBACK_IMAGE_COLOR` when bytes are absent or the atlas rect/byte length is invalid. Non-atlas viewport and recorded-image paths continue to set `atlas_uv: None`, preserving byte-for-byte software parity tests while keeping atlas replay deterministic.

The runtime presenter keeps a retained offscreen UI texture so damage patches can repaint only their region and still present a complete swapchain image. Full streams clear the offscreen target and rebuild it; patch streams load the previous target, clip command geometry to the command clip plus damage, upload changed image payloads, render UI geometry/text on the GPU, then blit the offscreen texture to the native surface. Runtime text preparation and image uploads use the same effective command-frame, clip, surface, and damage intersection as the quad/image clipping path from `rhi_wgpu/ui_surface/geometry.rs`; shader and blit setup lives in `rhi_wgpu/ui_surface/pipeline.rs`; glyphon buffer preparation, style mapping, and atlas rendering live in `rhi_wgpu/ui_surface/text.rs`. `rhi_wgpu/ui_surface/batching.rs` builds the draw plan from the clipped items: overlapping items keep the stable softbuffer z/index order through depth dependencies, while non-overlapping items are incomparable and can share a layer. Each layer batches all solid vertices into one solid draw, groups images by identical `resource_key`, and sends all layer text areas to one glyphon batch. This means `gpu_draw_calls` now measures actual planned GPU batch submissions instead of visible command count, and the visible command/item counters explain how much batching happened. Glyphs, texture uploads, and profile counters are therefore skipped or bounded exactly like the command stream patch. The headless constructor remains stats-only so runtime and editor tests can verify the contract without requiring a real window.

Softbuffer fallback consumes the same command stream. It records `software_fallback_present_count`, command full/patch counters, painted pixels, and full/region paint counts, then executes the stream into its CPU backbuffer. That keeps screenshots and fallback parity tied to the GPU command stream instead of the retired whole-frame `paint_host_frame` path.

Profiling treats GPU takeover as an invariant. `ui_hotspot` alerts if any scenario records `software_fallback_present_count > 0`, if a command stream has no matching GPU draw calls, if viewport image command patches do not record GPU upload bytes, or if independent visible draw items produce no draw-call reduction. Existing hover and viewport-image rules still reject chrome snapshot rebuilds, workbench model rebuilds, presentation dirties, and full-frame redraw degeneration. The summary report now prints both command/item visibility and actual GPU batch draws so render efficiency can be judged from the same capture instead of inferring batching from command count.

Boundary tests keep the split explicit. Retained editor host presenter sources must not contain `wgpu::`, `rhi_wgpu`, or generated UI backend selectors. The factory now consumes only the neutral runtime RHI descriptor and presenter factory, while all raw GPU API usage and concrete backend names remain in `zircon_runtime`.

The 2026-05-15 M5 workspace expansion proved the build side of that split with `cargo build --workspace --locked --verbose`; the only warning was the existing Cargo output filename collision for `zircon_runtime.pdb`. The first broad workspace test failed in `zircon_editor` before later crates could run. The narrowed `cargo test -p zircon_editor --lib --locked --message-format=short` run reported 1173 passed, 121 failed, and 4 ignored. The lowest failure signals were active demo/template-front drift: the default active drawer was `editor.hierarchy#1` while older tests expected `editor.project#1`, builtin host template IDs differed between `template.ui.host_window` and `template.v2.ui.host_window`, pane body tests referenced stale or missing assets/bindings such as `runtime_diagnostics_body.ui.toml` and `PerformanceTimelinePaneBody/RefreshSnapshot`, and most callback failures were `PoisonError` cascades after the shared test lock was poisoned.

After recording that blocker, focused GPU-path closeout validation passed with `--locked`: runtime `ui_surface` passed 26/26, editor `command_stream` passed 6/6, editor `gpu_presenter` passed 2/2, and editor `render_framework_boundary` passed 3/3. The follow-up demo/template-front continuation fixed those stale v2 template and pane-body expectations enough for `zircon_editor --lib` to pass 1298 / 0 / 4 ignored, and the runtime RHI boundary convergence made `zircon_runtime --lib` pass 1349 / 0. `zircon_runtime_interface --lib` and `zircon_app --lib` also passed. Full workspace `cargo test --workspace` still is not a clean acceptance signal in this Windows debug target: one attempt hit the heavy `runtime_ui_text_render_contract` linker path before its focused rerun passed 8/8, and the final attempt timed out after 30 minutes with no remaining cargo/rustc/link processes.

The 2026-05-15 poset batching slice added runtime-side depth-layer batching without changing the editor command stream contract. Focused validation passed runtime `ui_surface` with 33 tests, runtime `ui_hotspot` with 9 tests, editor `command_stream`, `gpu_presenter`, and `render_framework_boundary`, runtime-interface lib tests, and `zircon_app --features target-editor-host` check under `D:\cargo-targets\zircon-shared\ui-poset-batching`. The profiling smoke captures `20260515-201453-startup`, `20260515-201501-idle_hover`, and `20260515-201507-viewport_image` had zero UI hotspot alerts. Startup recorded `software_fallback_present_count=0`, `gpu_upload_bytes=32968`, `gpu_visible_draw_items=250`, `gpu_draw_calls=37`, `gpu_batch_layers=21`, and `gpu_batch_dependencies=2143`, proving the profile surface now distinguishes command volume from real batch draws.

The hover evidence closeout then fixed the automated idle-hover gap. `tools/ui-profile-capture.ps1` now resolves profiling binaries from `$CARGO_TARGET_DIR\profiling`, can synthesize client-area pointer/click/drag interactions, and requires redraw plus GPU batch evidence for `idle_hover`. After template-node hover became retained state, `20260515-211644-idle_hover` recorded a real hover patch with `gpu_draw_calls=10` versus `gpu_visible_draw_items=12`, zero software fallback, and zero UI hotspot alerts. This complements the stronger click interaction profile `20260515-205945-click`, which recorded `redraw_region_count=3`, `gpu_draw_calls=87`, and `gpu_visible_draw_items=504`.

The viewport-image closeout fixed a startup retry hole in the retained host. The first frame can start the viewport render-framework resolver on a background thread and return before a viewport exists. In that case `RetainedEditorHost::tick()` keeps `render_dirty` and asks `window.rs` to queue a frame-update redraw for the viewport region, which lets the event loop retry extract submission after the backend is ready without invoking the frame callback reentrantly. The new `frame_update_region_queues_external_redraw_with_frame_update` test covers that queued-redraw contract, and the close-request regression keeps callback-owned state mutation outside the active `RefCell` borrow.

The 2026-05-16 strict captures use temporary `renderable-empty` projects for `idle_hover` and `viewport_image`, leaving `startup` on the default cold-start page. `20260516-001744-startup` reduced `188` visible draw items to `35` GPU draws, `20260516-001914-idle_hover` reduced `37` visible draw items to `32` GPU draws, `20260516-000208-viewport_image` recorded `dirty_paint_only_count=1`, `redraw_region_count=1`, `gpu_upload_bytes=1306792`, and `21` visible draw items to `16` GPU draws, and `20260516-002011-click` reduced `318` visible draw items to `84` GPU draws. All four finished with zero hotspot alerts and no software fallback.

The UI batching validation profile path now exports retained-host evidence beside the runtime trace. `profiling_artifacts.rs` writes `ui_profile_geometry.json` on each profiling present, and `profiling_artifacts/geometry.rs` builds the client size, selected backend, splitter frames, document/drawer/host tabs, activity-rail buttons, viewport frame, viewport-toolbar controls, dispatchable template controls, and sampled hit points. `profiling_artifacts/tests.rs` owns the module-local geometry regressions for absolute splitter/tab frames, clipped template controls, clipped viewport-toolbar controls, and top-hit filtering. `profiling_hit_routes.rs` keeps the route-consistency checks out of the artifact writer: each sample records whether the rendered frame contains the point and whether the shared retained-host route or surface hit-test would hit the same control. `tools/ui-profile-capture.ps1` consumes that file for `drag`, `drawer_resize`, `click`, and `idle_hover` auto-interaction before falling back to fixed client ratios, so resize and drag gates now target live splitter/tab geometry rather than approximate coordinates. The drawer-resize gate also writes `ui_interaction_evidence.json` with the selected splitter, pointer path, before/after layout deltas, and a `resize_changed_layout` assertion, which makes border-drag effectiveness testable instead of relying on a visual guess.

Material Component Lab uses the same evidence path through `--builtin-view editor.material_component_lab`. Its click gate is intentionally component-only: `material_lab_click` uses live `template_controls` instead of document tabs or host page tabs, so the measured click scenario represents Material prototype feedback rather than page activation. The capture script also accepts the comma-separated list used by the plan. The 2026-05-16 strict run produced `20260516-123734-material_lab_startup`, `20260516-123745-material_lab_hover`, and `20260516-123756-material_lab_click`; all three reported zero UI hotspot alerts and no software fallback. The click capture passed with `dirty_paint_only_count=1`, `redraw_region_count=2`, `presentation_rebuild_count=0`, `dirty_layout_count=0`, and `dirty_presentation_count=0`. The hover capture had no presentation churn, and its draw calls equaled visible items only because the patch was fully dependency-bound (`dependency_density=1.000`), so batch reduction was not expected.

The Material Lab structure is also guarded at the UI-asset boundary. `material_component_lab_shell_keeps_material_lab_layout_regions` freezes the AppBar, Drawer, scrollable component-family content area, right-side interaction legend, and the eight official family sections. It also keeps the MUI X subsection in the planned Tree View, Data Grid, Charts, chart subtypes, and AgentChat order. A direct structured TOML validation passed independently of Cargo, covering 73 prototypes/imports, 63 MUI Core docs rows, 10 MUI X prototype rows, and 48 authored `MaterialLab/*` interaction routes. Static binding validation also found all 48 authored event ids in the builtin template binding registry and confirmed the `--builtin-view editor.material_component_lab` descriptor/capture route; `material_component_lab_feedback_events_use_consistent_ids_routes_and_kinds` now checks each event id, dotted route, and event kind tail, while `material_component_lab_feedback_events_are_registered_as_builtin_bindings` encodes the source-level event-to-binding and `EditorUiEventKind` check in Rust. `MaterialLab/MuiXGauge/Hover` now gives Gauge the same chart hover feedback route as the other MUI X chart subtypes, and `material_component_lab_mui_x_prototypes_define_feedback_routes` locks that every MUI X prototype keeps a Material Lab feedback route. The design-matrix filename pass confirms 68 explicit `material_*.zui` references resolve to existing prototype files, and `material_ui_component_design_matrix_names_existing_zui_prototypes` now encodes that guard in Rust. The Rust guard was formatted successfully. The lower runtime compile blockers seen earlier were corrected by their owning sessions, and `cargo metadata --locked --no-deps --format-version 1` now passes; however, repeated focused Rust reruns on E: and D: target directories exit with process code `-1` in dependency compilation without a Rust diagnostic. The 2026-05-16 15:36 warm-target retry emitted only unrelated `zircon_runtime::core::framework::net::websocket` dead-code warnings before the same process exit. At that checkpoint, 23 Cargo/Rust processes from parallel sessions were active and E: had about 57 GB free, so the current Rust evidence gap remains classified as environment/compiler-process pressure, not as a Material Lab layout assertion failure.

The prototype input contract also separates visual state samples from real dispatch targets. Static and utility placeholders can show hovered/focused/selected styling in their props, but if they do not define a `MaterialLab/*` route then `input_interactive`, `input_clickable`, `input_hoverable`, and `input_focusable` stay false. `material_component_lab_non_route_prototypes_are_not_dispatchable_controls` keeps automated click captures from selecting no-feedback placeholder controls. The paired `material_component_lab_route_prototypes_are_dispatchable_controls` guard checks the other side of the contract: every prototype with a `MaterialLab/*` route must keep at least one dispatchable flag, so the visual evidence path cannot silently lose all click/hover/focus targets. `material_component_lab_feedback_routes_live_on_dispatchable_sample_nodes` then pins that route to the visible `material-lab-sample` node and requires all four input flags to be `true` on that node, while non-route prototypes must not hide a feedback node elsewhere. `material_component_lab_feedback_route_inventory_matches_expected_interactions` freezes the interaction kind for each route so specialized samples remain specialized: Slider stays `DragUpdate`, chart/Tooltip samples stay `Hover`, Chat samples stay `Submit`, toggle controls stay `Toggle`, selector controls stay `Change`, and button/surface controls stay `Click`. `material_component_lab_interactive_inventory_matches_route_bearing_prototypes` keeps the static interactive whitelist aligned with the route-bearing asset inventory, and `material_component_lab_places_every_prototype_once_in_visible_sections` verifies that every imported prototype appears once in a visible family section instead of staying as an unused import. `material_component_lab_prototype_nodes_match_material_file_stems` additionally keeps each `prototype_*` node id aligned with the `material_*.zui` file stem and exported component name. `material_component_lab_shell_keeps_material_style_contract` freezes the shared dark Material theme import plus shell/card classes, colors, border tone, and 12px panel radius. The shell order guard now freezes every family section's internal prototype order, not only the top-level section list and MUI X subsection.

The corresponding Rust boundary coverage is split by responsibility under `zircon_editor/src/tests/ui/boundary/material_component_lab/`: inventory, feedback, shell, projection, and support. The split is intentionally test-only and preserves the same retained evidence contracts while leaving room for additional component-specific guards without growing another monolithic test file. The inventory module now checks parsed sample-node props rather than only scanning source text: every `material_*.zui` prototype must expose one `material-lab-sample` node with shared Material classes, typed variant/tone/validation props, typed state and input flags, and numeric radius/border values. It also freezes the prototype card root as a fixed-height vertical Material card with stretch width, `6px` internal gap, and stable `title`, `meta`, `sample` children. That keeps profiler evidence tied to the actual visible prototype node the retained host will render and keeps the component grid from shifting as more samples are added.

The same capture script now writes sidecar evidence files after the runtime exporter creates `timeline.zrtrace.json`, `timeline.perfetto.json`, `hotspots.json`, `ui_hotspots.json`, and `summary.md`. `ui_batch_metrics.json` derives `batch_success_rate`, `draw_reduction_ratio`, `dependency_density`, and `layer_density` from `ui_hotspots.json`, and records the partial-order model, list-row batching interpretation, ideal case, worst-case degeneration, and rectangular clip/mask boundary. `ui_hit_consistency.json` stores the route/frame sample results and fails the strict evidence gate if any center or negative sample disagrees. `screenshot_reference.png` comes from the retained host software painter for the same presentation data; `screenshot_gpu.png` is a live client-area capture of the normal GPU run; optional `-CaptureSoftbufferScreenshot` launches a second profiling-only `ZIRCON_PROFILE_FORCE_SOFTBUFFER=1` window against the same temporary project, writes `screenshot_softbuffer.png`, compares both live captures to the reference plus direct GPU-vs-softbuffer in `screenshot_diff.json`, records the screenshot thresholds, and fails strict parity when direct GPU-vs-softbuffer exceeds the configured differing-sample ratio or average-channel-delta limit.

`ZIRCON_PROFILE_FORCE_SOFTBUFFER` is intentionally not a runtime/editor public DTO. `HostPresenterBackend::default_native()` only reads it under the profiling feature to select `Softbuffer` for screenshot parity, while normal native startup still attempts GPU first and records the actual fallback backend if GPU creation fails. Full strict profile acceptance requires zero normal-run `software_fallback_present_count`, zero `ui_hotspots` alerts, `gpu_draw_calls < gpu_visible_draw_items`, non-empty hit-consistency samples with no failures, drawer-resize layout movement for the border-drag scenario, and for `asset_refresh` an actual asset/editor/resource change counter from touching the temporary project asset file. Targeted evidence reruns may close one blocker, such as screenshot color-space parity or scenario counter attribution, only when the remaining alerts are stated explicitly. Rectangular clip validation remains the current mask boundary: solid/image command geometry and UVs are CPU-clipped before batching, text uses glyphon bounds, and non-rectangular masks would need a future explicit batch key/stencil layer or fallback rather than being inferred by this capture path.

The 2026-05-17 screenshot parity blocker was in the runtime WGPU surface color-space path rather than retained-host command generation. A pre-fix viewport-image run, `20260517-182730-viewport_image`, showed direct GPU-vs-softbuffer `differing_sample_ratio=0.6522`, matching the observed bright GPU capture. `zircon_runtime::rhi_wgpu::ui_surface` now keeps the retained UI target in `Rgba8Unorm`, prefers non-sRGB swapchain formats, clears new retained targets to opaque black, and prefers opaque swapchain alpha. After rebuilding the profiling binaries, `20260517-190736-viewport_image` reduced direct GPU-vs-softbuffer diff to `differing_sample_ratio=0.0165` and `average_channel_delta=0.9022`, below the configured `0.25` / `10.0` thresholds, while keeping `viewport_image` batch evidence at `21` visible draw items to `16` GPU draw calls and hit consistency at `93 failed=0`.

The same rebuild closed the earlier scenario-attribution gap for the requested interaction evidence. `HostRedrawRequest::merge` now lets the later frame-update scenario own a coalesced redraw request, so resize and asset-refresh presents are no longer hidden under earlier click/startup labels. `20260517-190840-drawer_resize` used live splitter geometry, moved the left drawer by `80px`, refreshed geometry, kept `87 failed=0` hit samples, and recorded `653` visible draw items to `124` GPU draw calls. `20260517-190851-asset_refresh` recorded real asset-refresh presenter work with `266` visible draw items to `42` GPU draw calls and `114 failed=0` hit samples. These captures are evidence for geometry-derived interaction, GPU counter attribution, screenshot parity, and no normal-run software fallback; they still leave UI hotspot cleanup work open for `click/non_structural_interaction_rebuilt_presentation`, `drawer_resize/resize_triggered_slow_path_rebuild`, `idle_hover/region_request_repainted_full_frame`, and `startup/gpu_presenter_recorded_no_draw_calls`.

The 2026-05-22 SpriteAtlas M5 retained-host slice made atlas-backed image commands concrete without changing dynamic viewport-image behavior. `painter/sprite_atlas.rs` resolves untinted `template-icon:`, `template-image:`, `icon:`, and `image:` keys from source paths under an `assets` tree to generated `.zircon/cache/editor-sprite-atlases/*.toml` manifests and matching atlas PNG bytes. `visual_assets.rs` attaches that metadata to retained image pixels; `render_commands.rs` records atlas metadata only for full-opacity image draws; `command_stream.rs` converts recorded atlas images to the atlas texture `resource_key`, atlas dimensions/RGBA bytes, and `ChromeImageUvRect`; and softbuffer replay samples the subimage from those atlas bytes. The M5 debug pass fixed a payload bug where recorded atlas images kept the source key/dimensions instead of the atlas key/dimensions, then split `presenter/command_stream/tests.rs` out of the 1010-line command-stream file so the production implementation returned to 621 lines. Focused validation in `D:\cargo-targets\zircon-shared\sprite-atlas-ui` passed `cargo test -p zircon_editor --lib sprite_atlas --locked --jobs 1 --message-format short --color never` (`12` tests), `cargo test -p zircon_editor --lib command_stream --locked --jobs 1 --message-format short --color never` (`11` tests), `cargo test -p zircon_editor --lib gpu_presenter --locked --jobs 1 --message-format short --color never` (`5` tests), and `cargo check -p zircon_editor --lib --locked --message-format short --color never` with deferred unused SpriteAtlas producer warnings. Live atlas-heavy profiling remains pending because no `.zircon/cache/editor-sprite-atlases` artifacts currently exist in the workspace for an automated retained-host scenario.

## Job Status Projection

The 2026-07-17 performance slice keeps the full `job_progress_snapshot()` API for task-list consumers, but the retained status bar now calls `primary_job_progress_snapshot()`. The progress source is already ordered by `JobId`, so this query skips terminal entries and clones only the first visible job instead of cloning every active label and progress message on every host tick. Selection, fallback label, percentage, and terminal-visibility semantics are unchanged. The focused regression is implemented; coordinated Cargo and live active-job profiling remain pending.

## Retained App Projection And Pointer Hot Paths

The 2026-07-17 app audit covers all 451 Rust files under `ui/retained_host/app`. Pointer callbacks now consume the asset and hierarchy projections committed by the previous slow recompute. Asset surfaces retain an `Arc<AssetWorkspaceSnapshot>` per surface and hierarchy retains an `Arc<[SceneEntry]>`; move, scroll, press, and click no longer build a complete editor snapshot to obtain one list. When the callback resolves to the already committed surface size, asset tree/content/reference and hierarchy bridges keep their current layouts instead of rebuilding and comparing every row. Asset-details, console, and inspector scroll surfaces use the same changed-size rule.

This cache is a consumer of the existing chrome generation, not a second asset or scene authority. `sync_asset_pointer_layout` and `sync_hierarchy_pointer_layout` publish the projection alongside their bridge layouts; callbacks only clone the `Arc`. Project/catalog/selection changes continue to enter through the normal retained recompute. Dynamic acceptance must prove startup first-frame ordering, child-window fallback sizing, resize rebuilds, drag payload lookup, and hover/scroll routing before the app folder leaves the performance pending ledger.

Small no-op contracts were also tightened at the host boundary. UI asset and animation pane payloads share one visible-instance snapshot; main-window close reuses one snapshot for dirty and close-id projection; default-scene change matching parses one `ResourceLocator`; repeated drag-target groups do not republish UI state; palette hover dirties presentation only when the manager reports a changed target; and terminal export-wizard sessions are skipped during per-tick polling.

The remaining structural contract is generation-driven. A successful render must update a diagnostics counter without dirtying presentation. Asset/editor/resource event ingress must be bounded and coalesced before catalog/details/preview generations commit. Active-template and selected-inspector identity must be available without a full chrome/editor snapshot. A slow recompute must build each dirty domain at most once per frame, avoid the current viewport resize second chrome/model build, and patch only changed native-window/toolbar/pane payloads. Build/export and module/plugin panes must consume manifest, preset, job, and catalog generations rather than synchronously reopening files or rebuilding plans during UI projection. These responsibilities are tracked by PERF-MVP-103 through PERF-MVP-107 and their EditorUI08, Editor09, Editor12, and Editor15 handoffs.

The shared `asset_pointer` and `hierarchy_pointer` bridges remain a lower input-layer bottleneck after the app-side cache fix. Asset content, reference, tree, and scene-hierarchy scrolling currently rebuild the complete `UiSurface`, dispatcher registrations, route/target map, row paths, and frames whenever scroll offset changes; all four scrollable boxes declare no virtualization. Asset move dispatch also clones an owned target carrying the asset or folder string even when the consumer only needs hover state. PERF-MVP-109 assigns stable row identity, visible-range materialization, incremental scroll/hit updates, and a state-only move route to EditorUI01; both asset and scene rows must reuse that input authority, and the app must not work around it by creating another visible-row authority.

The detail scroll bridge has a smaller fixed tree and now follows a cheaper contract. Runtime default scroll handling already mutates the viewport's retained `scroll_state`; console, inspector, and asset-details callbacks retain that offset without recreating the root/viewport surface, route map, or formatted paths. Layout or externally supplied state changes still use `sync()` and rebuild the two-node surface. Asset-details extent calculation also sums its fixed section constants directly instead of allocating a temporary section vector. Focused Cargo and interaction acceptance remain pending under PERF-MVP-110.

Document-tab pointer measurement now has a steady-state fast path. Activate and close callbacks compare the host-reported tab frame with the committed measured frame and skip the full tab/close hit-tree rebuild when it is unchanged. When rebuilding is required, the bridge borrows the measured-frame vector rather than cloning every optional frame. First measurement, resize, floating-window projection, and document generation changes retain the existing full rebuild contract; PERF-MVP-111 requires click-storm and route-parity evidence before acceptance.

Menu pointer rebuild now borrows the committed `MenuItemSpec` tree through `Cow` rather than cloning root and submenu vectors. Popup metrics can read the same tree only to obtain its length, scroll handling no longer clones the owned route payload, repeated already-closed popup state is a no-op, and each visible popup layer locates its preorder route start once before advancing with subtree lengths. This removes the prior per-row double root traversal and makes route-index preparation linear within a layer. Scroll offset and submenu topology changes still recreate the whole surface and the root scroll box still lacks virtualization; PERF-MVP-112 assigns that shared pointer-surface work to EditorUI01.

Viewport-toolbar click measurement now upserts one control into the committed per-surface control set. Repeating the same action key and frame skips surface rebuild; a changed frame replaces only that control; a new control is appended without discarding previously synchronized controls. Surface removal also retains against borrowed layout keys instead of cloning a key set. Slow-recompute toolbar frame generation, temporary-surface rebuild, frame deep clone, and full presentation replacement remain the EditorUI08 responsibility tracked by PERF-MVP-113.

Activity-rail pointer dispatch itself is steady-state retained, but its layout producer is not generation-gated. Each slow recompute walks left/right drawer stacks and allocates new slot and instance strings before deep equality can skip the hit-surface rebuild. PERF-MVP-114 keeps this under the Workbench tool-window/layout generation authority in EditorUI08; the pointer bridge should consume one immutable projection per generation rather than adding a second cache of drawer state.

Drawer-header measured frames use the same steady-state fast path as document tabs. An unchanged host-reported tab frame no longer rebuilds every left/right/bottom header node, and required rebuilds borrow the optional-frame vector rather than cloning it. The producer still recreates drawer surface/items before equality on each slow recompute, so PERF-MVP-115 keeps generation ownership with EditorUI08.

Host-page pointer no longer maintains an unused measured-frame vector. Visible page clicks use the tab slot already committed by the shared overflow allocator; hidden/overflowed pages retain the direct typed-route fallback. This removes per-layout vector resize and per-click writes plus an unreachable empty-tab rebuild. Page id/title cloning and text-width allocation still belong behind the host-page/layout generation gate in EditorUI08 under PERF-MVP-116.

Welcome recent-project pointer remains another list-surface consumer of the shared input work. Every scroll offset currently recreates all recent rows plus open/remove nodes and owned path routes, with no virtualization; move dispatch can materialize a project path even when only hover state changes. PERF-MVP-117 extends EditorUI01's stable-row, visible-range, incremental-scroll, and state-only move acceptance to the Welcome page.

Shell drag frames are immutable after their surface generation, so drag dispatch now shares `Arc<DragTargetFrames>` without a mutex or poison recovery on every move. Resize geometry updates compare the root and three splitter targets and rebuild only when a frame/input/state actually changes. The drag surface itself still recreates floating-window nodes, dispatcher closures, and route intents on every host layout update; PERF-MVP-118 assigns an explicit layout/floating-window generation cache to EditorUI08/EditorUI02.

Tab-drop resolution currently reconstructs its target strip at drop time: it scans the Workbench layout, clones tab identity/title/host/path data, filters the dragged tab, and remeasures every target title before midpoint selection. PERF-MVP-119 requires drag start to capture the already committed document/drawer tab geometry generation and an instance-to-host index, so drop performs only bounded route resolution without a second model projection.

Viewport polling now publishes only a newly imported capture generation. Framework, capture, or import errors retain the last image for presentation continuity and expose the diagnostic through `take_error()`, but return `None` so the host does not re-upload and redraw stale pixels every tick. The successful path stores one lightweight image clone and returns the new image directly. Hybrid-GI environment profile parsing is cached once per process rather than reading and lowercasing the environment on every extract submission. These direct fixes are PERF-MVP-120; they do not close PERF-MVP-023's synchronous GPU readback boundary.

The remaining viewport controller path holds its shared state mutex across render-framework resolve, viewport destroy/create, `capture_frame`, RGBA import, world-space command construction, and frame submit. Normal editor presentation therefore serializes UI/controller access behind potentially blocking GPU/framework work. Resize changes destroy/recreate the viewport immediately, and world-space UI rebuilds owned commands/styles and clones full submissions for pointer capture. PERF-MVP-023 and PERF-MVP-121 require GPU texture or bounded async staging, short-lock generation snapshots, resize coalescing, immutable world-UI extracts, and lightweight capture handles.

Workbench preview-action classification now indexes the 1,191 unique static action ids once with a process-level `LazyLock<HashSet<&'static str>>`. Editor-event dispatch, common callback dispatch, and the componentized Workbench bridge no longer scan the root and extension slices linearly for every known or unknown action. The index preserves the existing static registry as the readable source of truth, performs no steady-state allocation, and keeps duplicate detection in the existing registry test. Coordinated Cargo and known/unknown action-storm evidence remain pending under PERF-MVP-122.

Retained route-intent node and route ids now use `HashMap` because the maps expose no ordered iteration and both keys are hashed `u64` newtypes. This removes ordered-tree comparisons from the double lookup performed for each pointer dispatch. The typed accessors still clone owned route payloads containing surface, slot, instance, action, path strings, and path vectors; PERF-MVP-123 assigns stable route handles or immutable shared payloads to EditorUI01 so move-only paths do not deep-clone data before the final consumer needs it.

## Callback Dispatch And Template Bridge

The 2026-07-17 callback audit covers all 135 Rust files under `ui/retained_host/callback_dispatch`. Builtin template bindings are now held in one process `LazyLock`; dispatch clones only the selected binding instead of rebuilding the full string-keyed registry. The shared viewport bridge owns one pointer dispatcher and skips surface rebuilds when the committed viewport frame is unchanged. Virtual-row growth indexes existing virtual row numbers and obtains the next node-id cursor once. Extension workspace navigation builds one action-route index from the static specs, so workspace, tab, row, command, and field queries no longer rescan all extension specs for every callback.

Floating-window source layout now records shell size and treats an unchanged size as a no-op. Responsive tier parsing compares ASCII case without allocating a lowercase string for every responsive node. These fixes preserve real resize, route, popup, and binding behavior; coordinated Cargo and interaction traces remain pending under PERF-MVP-124 through PERF-MVP-129.

The shared remaining problem is control identity. Popup, menu, property, component-row, and data-sync code repeatedly searches `surface.tree.nodes.values()` by control id, often several times per row or action. Dynamic virtual rows make bridge-local caches unsafe because insert, prune, reuse, and template reload can invalidate them. EditorUI01 owns a generation-maintained `control_id -> node id(s)` index at the `UiSurface` or template owner boundary. EditorUI08 owns typed row/property deltas and the rule that root/workbench/template layout runs at most once per dirty generation per frame. Bevy's entity-to-Taffy map and Slint's dirty repeated-row updates are the reference constraints for these owners.

## Retained Host Root Support

The 19 root Rust files were statically reviewed with their direct consumers. Floating-window projection now builds one borrowed native-host id index and stores frames in a hash map; it no longer scans the host list twice per floating window. Duplicate host ids retain the previous first-match behavior. `ModelRc::from(Rc<VecModel<T>>)` now takes ownership of a unique source with `Rc::try_unwrap`, eliminating the otherwise unconditional full row clone used by pane and template host projection; genuinely shared sources keep the clone fallback.

Drawer resize remains a layout-owner issue. A left or right group currently dispatches top and bottom `SetDrawerExtent` commands separately, and each command performs legacy sync plus complete session metadata recomputation. EditorUI08 must expose an atomic typed batch/delta so one resize takes one lock, applies both slots, recomputes affected metadata once, and publishes once. Notification history is already bounded to 64 entries; per-notification string encoding is not treated as an MVP blocker without storm evidence. Viewport RGBA copies remain tracked by PERF-MVP-023/120 rather than being hidden in the primitive wrapper.

## Presentation Conversion Core

`ModelRc` now exposes a borrowed iterator. Template-node mapping reads source rows without `row_data` clones, joins option text directly without a temporary string vector, and floating Welcome size detection borrows window rows rather than cloning complete window/pane DTOs. Combined with unique `VecModel` ownership transfer, the common projection path no longer performs the previous two consecutive full row clones.

Workbench-window node projection now uses a dedicated `node_index.rs`. One hash index memoizes ancestor render visibility and the nearest controlled ancestor for all nodes; visibility normalization compares ASCII while ignoring underscores without allocating, and parent cycles terminate as hidden/no-parent instead of looping. The 963-line converter was reduced to 906 lines under the repository large-file rule, with indexing isolated as one responsibility. Menu item values are parsed for structured rows before the original vector moves into the host model, removing another complete clone.

The unresolved owner boundary is still larger than these local fixes. Every visible workbench node recursively deep-converts its full retained property tree to TOML, while `apply_presentation` clones the complete prior presentation to preserve a few interaction fields, clones intermediate dock/pane DTOs, and replaces the complete host presentation. EditorUI05 owns typed property/document generations; EditorUI08 owns structural versus interaction generations and changed-pane/window patches. Unchanged generations must perform none of these conversions.

The shared preview-image cache used by Workbench, pane, and root-overlay projection now indexes source and icon name in two hash levels. Cache-hit lookup borrows both string slices, so a projected image node no longer allocates an owned composite key before discovering that the decoded/rasterized image already exists. Misses retain the existing lock-free decode/raster window followed by a locked insert; immutable image clones and source/icon pair isolation are unchanged. This direct fix is tracked by PERF-MVP-137.

## Retained Host UI Test Cost

The 10 Rust files under `ui/retained_host/ui/tests` are integration and source-contract coverage rather than product runtime. Their large component-showcase cases repeatedly load built-in templates, build runtime bindings, project a complete pane, and materialize host rows. Several fixtures also take the repository-wide environment lock because template/runtime setup can observe process environment. This raises validation wall-clock time and limits test parallelism, but it must not be reported as an editor frame bottleneck.

PERF-MVP-136 defers test-only caching until current-source F4 product paths have dynamic evidence. The acceptable follow-up is one immutable, suite-scoped built-in template/runtime fixture for read-only cases plus narrower lock ownership for tests that actually mutate or observe environment variables. The cache must not bypass production presentation generation, pointer-surface rebuilding, or clone counters used by performance acceptance.

## Pane Data Conversion

All 211 Rust files under `ui/retained_host/ui/pane_data_conversion` were statically reviewed on 2026-07-17. Generic `ModelRc` and template-node mapping now borrow source rows. Plugin status, performance timeline, Build/Export targets, hierarchy, and native template rows no longer materialize a complete source DTO solely to convert or format it. Build/Export duplicate-platform counters use hash maps, target key normalization uses one buffer, and the export wizard reuses its already resolved action id.

The shared pane template hit-surface builder also borrows both its dispatchability pass and its tree-build pass. It previously cloned every `TemplatePaneNodeData` twice for every pane generation. Only the strings required by `UiTreeNode` metadata and paths are now materialized. Option projection similarly performs borrowed ASCII-insensitive query matching and logarithmic normalized state-set lookup rather than allocating three lowercase strings and scanning every state value per option.

Command palette and notification center now parse their typed entry list once per specialized selection projection and derive plain and structured rows together. Filtered commands use a first-entry-preserving borrowed-id hash index, and query matching no longer allocates lowercase copies. Native animation/console/hierarchy/inspector fallback conversion borrows the pane DTO; button aliases clone an attribute map only when an alias must actually be inserted; virtual windows stop at the requested range; canvas/world read-only helpers avoid temporary strings and vectors. UI asset detail also borrows prop/state rows instead of cloning the full table.

Two owner-level bottlenecks remain. Runtime Diagnostics can build a generic hit surface, a second synthetic all-node surface plus snapshot, and a final hit surface in one pane conversion. UI asset detail still repeatedly scans and shifts the full node vector by section. PERF-MVP-143/145 define the generation-owned debug snapshot and changed-section contracts; PERF-MVP-135 remains the higher owner for unchanged property and pane generations. Current-source Cargo and scale counters are still required before these local changes are accepted.

## Template Surface Hit Testing

The 16 Rust files under `host_contract/surface_hit_test` and 23 direct geometry/family/input/activation support files were statically reviewed. Generic surface build, bounds, popup-node discovery, and base-node lookup now borrow host rows. Menu and option popup hit testing computes a uniform candidate row from Y; it checks at most two rows only on an inclusive shared boundary, preserving the previous disabled/separator and earlier-row precedence. The base surface hit now returns only the node id its caller uses.

PERF-MVP-146 still requires the presentation generation to publish and retain the Workbench hit surface and open-popup z stack so pointer events never rebuild the surface or scan every host node merely to prove no popup is open. Disabled and separator rows must continue to block the underlay, and clip, z-order, above-control placement, and clamped bounds must preserve existing route behavior. The local borrowed/O(1) implementation remains pending current-source Cargo and 1/100/10k-row visited-count evidence.

## Host Contract Data Shape

All 77 Rust files under `host_contract/data` were statically reviewed on 2026-07-17. World-space extraction now borrows template and floating-window rows, and reuses one pane surface id across all candidate node collections. It no longer clones the complete wide node DTO before selecting the few world fields needed by the submission.

The remaining P0 cost is presentation and payload shape. `get_host_presentation()` deep-clones the full presentation for pointer, scroll, keyboard, present, and viewport-sync readers. The snapshot embeds every pane payload and may include a full RGBA viewport vector. `TemplatePaneNodeData` also stores roughly 160 fields for every node, including unrelated collection, world-space, timeline, heatmap, image, drag, ripple, and action data; `PaneData` similarly embeds all pane kinds.

PERF-MVP-147/148 require an immutable structural presentation handle with separate interaction/image generations, an active-pane tagged/shared payload, and a compact common node header plus component-family payloads. Slint's item-specific structs and Bevy's common `Node` plus sparse widget components are the reference shapes. Pointer consumers must share the same generation-owned hit/control indexes; no consumer-local cache or second DTO authority is allowed.

## Host Presenter

All 31 Rust files under `host_contract/presenter` were statically reviewed. GPU presentation tracks surface-cache bootstrap versus damage patches and exposes upload, draw, visibility, batch, and painted-pixel counters. Softbuffer retains a same-size backbuffer and limits repaint and surface copy to the damage region when its command stream is patchable.

PERF-MVP-149 covers the remaining fallback amplification. Softbuffer diagnostics currently clone the complete presentation each present only to replace one overlay string. Overlay/damage convergence can format diagnostics up to nine times, and verbose logging builds the full presentation summary before deciding that the frame is unchanged. The overlay must become a separate typed command/transient generation, and summary formatting must be keyed by presentation generation. Current-source GPU/Softbuffer traces and the adjacent command-stream audit remain pending.

## Chrome Command Stream And Image Recording

The 40 Rust files under `host_contract/chrome_command_stream` and the 9 direct `paint_primitives/image` files were statically reviewed on 2026-07-17. The normal GPU presenter now consumes its newly built command stream when projecting the runtime draw list, so text, resource keys, and RGBA allocations move into the RHI payload instead of being cloned immediately before the stream is dropped. Explicit borrowed stream presentation remains available for tests. Atlas recording no longer creates a second ordinary RGBA payload that extraction always discarded, unique upload statistics use a hash set, and monotonic software replay skips the former reference-vector allocation and stable sort.

These local fixes do not establish the final resource boundary. Atlas pixels are still cloned into commands, damage extraction still walks the complete presentation before primitive clipping, and scaled software images still perform floating-point source-coordinate work per target pixel. PERF-MVP-150 through PERF-MVP-153 require a generation-owned image registry, handle-only draw commands, dirty-section patch extraction, product-zero replay sort fallbacks, and resource-level opacity/scale metadata. Bevy's `AssetId<Image>` bind-group cache, Slint's `TextureCacheKey -> Rc<Texture>`, and Godot's canvas `RID texture` commands are the reference constraint: byte ownership belongs to the resource cache, not each draw item.

## Paint Frame And Primitive Recording

The 15 paint-frame files, 3 paint-recording files, and 26 paint-primitive files were statically reviewed on 2026-07-17. Recording-only square borders now emit one typed border command instead of four quads per pixel of border width. The existing replay already understands border width and square corners, so this reduces extraction, statistics, RHI conversion, and draw-list volume while retaining the software pixel path and clip semantics. One-pixel and three-pixel command-count tests cover the change.

The remaining software hotspot is rounded geometry. Rounded fills recompute frame/radius/clamp/distance for every target pixel, and rounded borders perform both outer and inner containment checks per pixel. PERF-MVP-155 follows Slint's line-based integer rounded-rectangle renderer: precompute the geometry once and emit row spans plus bounded edge coverage. A full-size mask cache is not acceptable without repeated-target trace evidence. Current-source Cargo, large-border command counters, and Softbuffer pixel/performance traces remain pending.

## Paint Geometry And Text

All 4 paint-geometry files and 30 paint-text/test files were statically reviewed on 2026-07-17. Geometry is a small pure-coordinate boundary and did not expose an independent hotspot. Text does: recording-only rendering computes and discards a complete glyph layout before preserving only display text, while the runtime path can invoke runtime layout, line shaping, and fontdue layout for one run. Cluster advance and origin helpers contain repeated glyph-by-grapheme scans and per-glyph temporary vectors.

Font state compounds that work. Text and glyph paths repeatedly acquire a global preference lock and clone font-family strings. A new request can rebuild and rescan the system-font database, resolved fonts are leaked into an unbounded map, glyph rasters live in a second unbounded global map, and a global Swash scale context serializes raster work. The fontdue fallback stores an 8x bitmap and repeats the downsampling loops on every draw instead of caching logical coverage.

PERF-MVP-156 through PERF-MVP-160 assign the convergence to EditorUI03 and the runtime text cache/font/atlas plans: use one resolved layout and linear cluster merge, publish a generation-owned immutable typography snapshot, retain one process font database, bound caches by entries or bytes, remove leaked ownership and global raster serialization, and perform optional supersample reduction once on raster miss. Slint's lifecycle-aware text layout cache, byte-weighted thread-local glyph CLRU, and bounded Skia font cache provide the minimum cache discipline. Current-source Cargo, product text traces, cache bounds, lock counters, and pixel parity remain pending.

## Paint Theme And Overlays

The 6 theme files, 3 workbench dispatch files, 5 close-prompt files, 4 debug-reflector files, and 6 diagnostics files were statically reviewed on 2026-07-17. The workbench and overlay modules are thin, clipped draw boundaries. Their remaining repeated text and full-presentation costs are already owned by PERF-MVP-149 and PERF-MVP-156.

Theme lookup is a separate frame-level bottleneck. Palette and metric access currently performs a global read lock at every style call. The retained painter has 199 palette call sites across 86 files and 83 metric call sites across 41 files; one material node can make several identical lookups. Appearance updates also publish palette, metrics, and typography through three independent locks, so a renderer can observe a mixed generation.

PERF-MVP-161 requires EditorUI08 to publish one immutable, generation-owned theme snapshot and acquire it once per frame or command build. Paint context and style helpers borrow that snapshot, while a theme change marks the appropriate style/presentation generation dirty. Godot's per-control theme cache and Slint's dirty item rendering provide the reference rule: project values at update time and consume stable values during draw. Current-source Cargo, 1/1k/10k node lock counters, theme-switch traces, and pixel parity remain pending.

## Host Globals, Diagnostics, And Redraw

The 16 host-global files, 9 diagnostics files, and 7 redraw files were statically reviewed on 2026-07-17. Callback invocation correctly releases the shared `RefCell` borrow before calling re-entrant user code. Diagnostics use fixed counters; repeated overlay formatting remains owned by PERF-MVP-149. UI performance scenarios are thread-local and compile to no-op counters without profiling.

PERF-MVP-162 covers a direct redundant path: every presentation called 22 empty pane-global setters and one empty mesh-path sink. Fourteen calls first built complete recent-project, project-overview, activity/browser folder, item, selection, reference, and used-by host models, then immediately dropped them. The 23 calls, 14 conversion functions, empty asset-data module, and unused welcome/mesh methods are now removed. A source guard keeps `ShellPresentation` and the host scene as the sole projection; current-source Cargo remains pending.

PERF-MVP-163 covers damage amplification. `HostRedrawRequest` retains one region, so every merge replaces separated rectangles with their bounding box. The redraw and GPU upload area can therefore grow with distance between changes rather than changed pixels. A fixed-capacity, containment-aware region set should follow Slint's three-rectangle `DirtyRegion`: retain separated damage, and when full merge the pair with minimum area growth or explicitly promote to full based on a measured threshold. Current-source Cargo, conversion counters, multi-region product traces, and pixel parity remain pending.

## Host Window And Event Loop

All 38 Rust files under `host_contract/window` were statically reviewed on 2026-07-17. The event loop waits rather than ticking continuously, distinguishes paint-only from frame-update redraw, changes IME native state only on transitions, and releases callback state borrows before re-entry.

PERF-MVP-164 covers transient hover injection. Every presentation read first clones the structural tree and, while a template hover is active, collects complete node vectors for the workbench, four docks, and every floating pane. Non-matching collections are still cloned and dropped; a matching popup also rebuilds all option or menu rows. Hover must remain a transient generation resolved through a stable control index, with only old/new damage, and must not mutate structural `ModelRc` values at present time.

PERF-MVP-165 now coalesces native redraw scheduling on the None-to-pending edge, following Slint's `pending_redraw` flag, and moves rather than clones the pending request. PERF-MVP-166 replaces per-`about_to_wait` native size/scale/maximize/position queries with event-driven window state and an explicit startup or measured reconciliation path. PERF-MVP-167 now shares one converted text value between state and callback and moves the focus snapshot once; EditorUI03 still owns the runtime edit-buffer/range-delta convergence so long typing is not quadratic in copied bytes. Current-source Cargo, 1k-event scheduling/window-query traces, text allocation traces, and behavior/pixel parity remain pending.

## Profiling Artifact And Hit-Route Cost

All 35 profiling-artifact files and 18 profiling-hit-route files were statically reviewed on 2026-07-17. The current export call is attached to every successful present. With capture disabled it still reads a process environment variable every frame; with capture enabled it rebuilds full geometry, pretty-serializes JSON, creates the directory, and synchronously overwrites the same file every frame. Optional reference capture also software-paints the complete presentation, PNG-encodes it, and writes it on the present thread. Interactive WPR and CPU scenarios therefore measure substantial profiler-generated projection, encoding, and file I/O.

PERF-MVP-168 makes capture an explicit, bounded generation request: configuration is parsed once at startup, normal presents perform no profile environment reads or artifact work, and one requested stable generation produces one geometry and one reference image. JSON/PNG encoding and file I/O belong on a bounded worker with queue, drop, completion-generation, and age counters. Bevy's short-lived `Screenshot` component and asynchronous completion event are the reference for request lifetime; the capture script must consume the final post-interaction generation rather than depend on continuous overwrite.

PERF-MVP-169 removes repeated verification scans. Existing frame groups should be iterated directly with stable route identity instead of cloned into another clickable-frame table, and center/outside samples should reuse the generation-owned hit/control index. Scale acceptance requires linear work for 1, 100, and 10,000 controls, at most one hit test per sample, no candidate-id formatting scans, and unchanged route and screenshot consistency semantics.

## Native Popup Keyboard And Dismiss

The 13 native-keyboard files, 3 popup-dismiss files, 6 Workbench-context-menu files, and the two overflow/menu-metric roots were statically reviewed on 2026-07-17. Key mapping and action semantics are narrow, but target discovery is not: every popup navigation key or typeahead character deep-clones the full host presentation, reverse-scans and clones wide Workbench node rows, then rebuilds every enabled option or menu row with multiple shared-string copies before selecting one row. Host-page overflow follows the same rebuild pattern, and outside primary press scans the tree again to rediscover the active popup.

PERF-MVP-170 assigns one stable active-popup stack and navigation model to the same surface/presentation generation that owns the control and hit indexes. Keys should only move the current index or search committed rows; dismiss should query the top popup. Popup/template/row generation changes invalidate the model atomically. Slint provides the reference boundary: its property-tracked menu shadow tree rebuilds only when dirty, and the window owns its active-popup collection. Consumer-local keyboard or dismiss caches are not acceptable.

The local typeahead lowercase allocation and linear uniform overflow-row hit loop are suitable direct fixes, with focused behavior tests and current-source Cargo still pending. Context-menu allocation is command-frequency work and remains below MVP priority without a right-click storm trace; menu text measurement is already covered by the shared text and theme snapshot plans.

## Native Pointer Move And Scroll

The 16 move-dispatch files and 18 scroll-dispatch files were statically reviewed on 2026-07-17. Both uncaptured move and scroll clone the full host presentation before routing. This is another direct consumer of PERF-MVP-147's immutable generation handle. A normal non-popup Workbench move also performs the Workbench hit route twice: the first result is discarded because it is not a popup, pane routing runs, and the same Workbench route runs again for the base hit. With the current event-time hit-surface construction this duplicates the largest local operation.

PERF-MVP-171 keeps one Workbench hit per move and reuses it across popup, pane-precedence, and base dispatch. Scroll must report damage only when a handler or retained state actually changes. Template-node, toolbar, UI-asset, other, and unmatched pane targets currently invoke no callback yet still request a full pane-region repaint; clamped menu scroll also lacks the move path's before/after state check. Those cases should return idle. Viewport input already follows the correct contract by waiting for a newly rendered image.

The final acceptance combines local 1,000-event hit/redraw counters with PERF-MVP-146/147 generation work. Slint's single item-tree mouse dispatch and explicit accepted/ignored result are the reference boundary: an ignored event should not manufacture paint work.

## Native Drag And Drawer Resize

All 21 native drag/resize files were statically reviewed on 2026-07-17. Tab drag already waits for a four-pixel threshold, avoids repainting stale content during active moves, and skips repeated target-group publication. Its full chrome/model route resolution occurs once on drop, not inside the move loop.

PERF-MVP-172 covers drawer resize ingress. Repeated identical points still write resize state, deep-clone the entire presentation to obtain one center-band frame, invoke the host callback, rewrite the same transient preferred extent, mark layout dirty, and request a frame update. Redraw scheduling coalesces the eventual recompute, but it does not remove those per-event main-thread operations. Identical points should return idle; resize damage should use a narrow frame snapshot or capture-time frame; changed points should publish a latest-wins transient generation consumed at most once per redraw drain. Release must flush the final point before PERF-MVP-131's atomic drawer batch.

## Native Pointer Routing And Shared Text Ownership

All 48 native-pointer routing files were statically reviewed on 2026-07-17. Floating windows, rail buttons, tab strips, and asset-panel discovery iterate with `ModelRc::row_data`, which clones each candidate before testing it. For floating windows and asset panels this can clone complete pane or wide node payloads for every miss. PERF-MVP-173 first converts these loops to borrowed iteration, then folds them into the generation-owned chrome/pane spatial and control indexes required by EditorUI01. Static route kinds and surface ids should be typed values, not freshly allocated strings per event.

The audit also found that retained `SharedString` is only a type alias for `String`. Across the editor it appears at 1,192 source locations, 816 inside retained host, so every supposedly shared presentation, node, route, and interaction clone still copies bytes. PERF-MVP-174 separates immutable shared text from editable owned buffers. IDs, labels, paths, and actions need a real copy-on-write or `Arc<str>` representation; text editing and formatting retain explicit `String` ownership. Slint's copy-on-write `SharedString` is the direct reference. A permanent global intern table is not an acceptable substitute because it removes the memory bound.

## Native Menu Geometry

All 27 native menu-geometry files were statically reviewed on 2026-07-17. Placement and damage work is bounded by the open submenu depth, but the complete root/nested stack is reconstructed repeatedly on every event. Popup move computes containment from the DTO and then recomputes stack damage after a state change; press computes before and after damage separately. Root and selected-level lookup use owned `row_data` clones, which are amplified by the current `SharedString = String` alias.

PERF-MVP-175 first converts menu/frame/branch access to borrowed iteration. The final menu-state and layout generation should publish popup stack frames, row ranges, blocking frame, and damage bounds once. Stable events query this projection; a changed submenu path updates only its suffix. The result must remain the same authority used by the menu pointer bridge, not a native-only cache.

## Native Pointer Button Dispatch

All 104 Rust files under `host_contract/native_pointer/button_dispatch` were statically reviewed on 2026-07-17. Route precedence is explicit and viewport input correctly waits for a later image rather than repainting immediately. The remaining ingress order is expensive: every press and release deep-clones the full host presentation before unsupported buttons are rejected and before an active resize or tab-drag capture consumes its release. Such early-consumed events therefore copy dock, pane, template, and potentially viewport image payloads they never inspect.

PERF-MVP-176 moves button-id validation and capture release before presentation acquisition as a local gate. The final path consumes PERF-MVP-147's immutable presentation handle and EditorUI01's generation-owned route. Pane callbacks need an explicit ignored/handled result with precise damage and frame-update intent; current `bool`/`()` callbacks force release to redraw the complete pane and pressed callbacks to use conservative frame-update or full-frame fallbacks even when no visible state changed. Acceptance counts snapshot bytes, route builds, callbacks, damage, frame updates, and full frames across unsupported, captured, passive, popup, Workbench, pane, toolbar, and viewport events while preserving focus, capture, pressed-state, callback-order, route, and pixel behavior.

## Native Pointer Damage And Redraw

The 57 files across native chrome, close-prompt, pane-button, redraw-result, resize, tab-drag, template-hover, and viewport-toolbar damage were statically reviewed on 2026-07-17. Their geometry is bounded and move correctly returns idle when interaction state is unchanged, but they expose three already-owned amplification points. Candidate discovery still clones `ModelRc::row_data` under PERF-MVP-173. Callback-specific helpers conservatively include complete center-band, status, sibling-pane, or floating-window bounds because the mutation boundary does not return precise dirty state; PERF-MVP-176 replaces that void/bool contract with typed handled damage.

All separated frames are still collapsed into one bounding rectangle, so distant focus, hover, status, floating-window, or tab-drop updates repaint and upload unchanged pixels between them. PERF-MVP-163's fixed-capacity region set is therefore also the native pointer damage contract. Dynamic acceptance compares changed, requested, painted, and uploaded areas, preserves separated regions, counts every full-frame promotion reason, and retains overlap, z-order, tab-drop, focus, hover, clipping, and pixel behavior.

The three remaining native-pointer root files only expose the module, fixed host/viewport pointer ABI values, and the pressed/released state enum. They contain no independent runtime work; later pointer optimizations must preserve those ABI values and mappings.

## Workbench Paint Projection

All 102 files under `host_contract/paint_workbench_renderer` were statically reviewed on 2026-07-17. Invisible docks and panes exit early and renderer scopes are already available, but stable frames still reconstruct structural paint information. The componentized path scans the full Workbench node model for top and status clips, clones it to discover the extension region, builds string parent maps and a per-node ancestor visited set, then scans the model again to paint the selected subtree. Welcome resolves roughly thirteen known controls with separate linear scans. Asset projectors, hover overlays, and scrollbars repeatedly rediscover the same frames and extents. Hierarchy and long menus clone every offscreen row before clipping, and diagnostics materializes a second primitive vector.

PERF-MVP-177 now has two direct local fixes. Hierarchy derives a conservative visible index range from row stride, scroll, and the final clip before calling `row_data`, bounding a 10,000-row viewport to visible rows plus at most two edges. RuntimeDiagnostics passes its `ModelRc` row iterator into a `Borrow<Primitive>` overlay painter, removing the second primitive Vec while preserving the existing slice API. PERF-MVP-219 separately combines Activity/Browser projector discovery. Current-source Cargo and counters remain pending. The final presentation generation still must own immutable parent/subtree membership, control-to-frame, clip-section paint ranges, and the active extension root. EditorUI01 supplies the same virtual-row visible range used by input; the painter does not maintain another row authority. Stable generations perform no topology/map/set/string rebuild, damage visits only intersecting paint segments, and all large lists visit visible rows plus overscan. Slint's dependency-tracked per-item cache and pre-render item filtering are the reference lifecycle. Pixel acceptance covers extension selection, clips, z-order, menus, scroll, hover, focus, floating windows, and fallbacks.

## Template Command Compilation

The 61 core files covering the template-node dispatcher, node pipeline tests, runtime-command conversion, and host-command draw model were statically reviewed together with the two `zircon_runtime_interface` command/list conversion owners. The current path has useful clipping before specialized template handlers, but it still reconstructs several transient representations. Each runtime command first creates a paint-element vector. A command with background, image, text, and border can emit four elements, and every element recomputes the same cache generation by serializing the complete command into a fresh JSON byte vector. The retained host then creates another command vector, clones owned text and resource payloads, and allocates and stable-sorts a reference vector for every draw. Visible template nodes also fall through a linear chain of five primary, one dropdown, and twenty-two secondary handlers before generic material/MUI/fallback handlers.

PERF-MVP-178 first computes the command generation once and hashes the deterministic serialization without a temporary byte vector. Runtime09 then owns typed render elements and role identity at extract generation, while EditorUI08 retains compiled, already ordered paint segments at presentation generation. A changed node replaces only its segment; a stable generation performs no element conversion, handler probing, host-command construction, payload cloning, or sorting. Resource bytes, text layout, and immutable string ownership remain with PERF-MVP-150, PERF-MVP-156, and PERF-MVP-174 rather than being duplicated in a host-local cache. Slint's dependency-tracked per-item cache and Bevy's typed extracted UI items plus image batching are the reference boundaries. Current-source Cargo, deterministic generation parity, 1/100/10,000-node counters, product traces, and GPU/Softbuffer pixel parity remain pending.

The six standalone viewport/icon/property/dropdown/row-metric leaves were also reviewed. Surface and segment geometry are bounded. Icon pixel loading remains a resource-generation concern under PERF-MVP-150, and repeated metric/palette acquisition remains under PERF-MVP-161. Property-axis parsing is paint-frequency work: it allocates an axis string and one string per value token, then joins those tokens into another string before the command owns a final text copy. PERF-MVP-178's compiled segment must perform that parsing only when the node generation changes; any earlier local cleanup must preserve the normalized axis/value text and property-row pixels.

The asset-placeholder, icon-kind/glyph, generic node-surface, and style-color family adds twelve reviewed files. Geometry and surface eligibility are bounded, but asset tiles acquire metric and palette snapshots per node before resource lookup. Missing icon assets enter a manual fallback that formats `control_id` plus `icon_name`, allocates a lowercase copy, and probes a long substring chain to rediscover one typed glyph. Presentation/template projection should publish that role once; PERF-MVP-178 then compiles it into the node segment, while PERF-MVP-150 and PERF-MVP-161 own icon pixels and theme values. Stable asset grids must not repeat theme acquisition, resource rasterization, string normalization, glyph classification, or surface-command construction.

All thirty-four manual glyph shape/dispatch files and their shared segment leaf were reviewed as PERF-MVP-179. They are bounded per glyph but expand one missing icon into two to eight quad commands; this cost survives command compilation because every segment remains a separate software primitive and RHI command. Product evidence decides the fix. Shipped MVP icons should resolve through real assets and report zero fallback. Any required fallback should become one bounded, generation-cached mask or atlas resource owned by Render13, with one draw command and unchanged tint, scale, clip, opacity, and pixels.

The nine sprite-atlas resolver/cache/test files expose PERF-MVP-180. Atlas resolution currently performs directory enumeration and candidate sorting for every image request, canonicalizes and stats every manifest before a cache lookup, clones the complete manifest on a hit, and opens and decodes the complete atlas image after an entry match. The cache grows by path/mtime/length keys without evicting stale generations, and the first valid manifest without the requested entry aborts later candidate search. Editor10 must publish a file-watcher-owned immutable source-to-atlas index; Render13 must own decoded CPU and GPU texture generations. Paint consumes only a handle and UV and performs no filesystem, parse, decode, or full-manifest clone work.

The forty-one visual-asset candidate/loading/SVG/MUI/test files extend the issue as PERF-MVP-181. Candidate paths and filesystem existence checks occur before the pixel cache can hit. The global pixel cache is unbounded and clones complete RGBA and atlas payloads on every hit. SVG tree lookup stats the source before locking another unbounded generation map. Retained previews copy and hash all pixels during paint, missing icons rerasterize, and a MUI miss reads and parses JavaScript from the repository `dev/material-ui` tree. Editor10 resolves canonical resources and change generations; Render13 owns bounded raster variants and decoded/uploaded resources. Stable paint clones only a lightweight handle and performs no candidate building, filesystem access, full-pixel hashing/copying, parsing, or rasterization.

The fourteen template-node label/text/test files were reviewed against PERF-MVP-156, PERF-MVP-161, PERF-MVP-174, and PERF-MVP-178. Label selection is bounded but always returns an owned `String`. A fallback text command builds that label once, then leading-icon geometry calls the same label builder again; image geometry for an icon-plus-text node can call it a third time. Default font sizing also acquires the global metrics snapshot per node. A local cleanup may pass one resolved label or `has_label` fact through image/text geometry, but the final owner remains the generation-compiled segment and frame theme snapshot. Stable nodes perform no label formatting/copy, repeated identity probing, metric locking, or text-command construction.

The eighteen template-style/state/color/test files are bounded match and arithmetic code, but callers repeatedly resolve the same interaction state across surface, border, width, text, and elevation decisions. Typed button color helpers can also reacquire the palette independently. PERF-MVP-161 supplies one frame theme snapshot; PERF-MVP-178 compiles one resolved node style containing interaction, surface, border, text, dimensions, and elevation. Stable nodes perform no repeated state classification or palette access, while changed state preserves the tested disabled/loading/pressed/focused/hover priority and asset-surface exceptions.

## Template Style Selector

All 157 Rust files in `paint_template_nodes/style_selector` were statically reviewed. Several selectors already establish the desired boundary: slider, selection control, status control, and chrome acquire one palette snapshot and pass it through their leaf helpers. Dropdown, text field, popup/list/tree/table rows, segmented control, button, and icon button do not. They reacquire the global `RwLock` for individual surface, border, text, glyph, and width decisions. A segmented control can read the palette about nine times for one node; table rows reacquire it again for every cell through `text_for_cell`; composed button states and role overrides can read it about three to eight times.

PERF-MVP-182 first applies the existing one-snapshot selector pattern consistently, then lets EditorUI08 compile changed nodes against the frame's immutable theme generation. Stable generations perform no selector or theme lookup. PERF-MVP-183 removes paint-time role reconstruction: danger/glyph classification currently allocates joined and lowercased strings, while command and tab-like button paths classify the same identifiers repeatedly. The local path uses allocation-free matching and one classification per changed node; the final presentation/template projection publishes a typed role consumed by PERF-MVP-178's compiled segment. Current-source Cargo, lock/allocation counters at 1/100/10,000 nodes, theme-switch atomicity, and pixel parity remain pending.

## Material Primitives

All 150 Rust files in `paint_template_nodes/material_primitives` were statically reviewed. Generic nodes probe eight primitive families in sequence, then matching alert, chip, badge, divider, and related helpers repeatedly rescan the same delimited variant string. Alert tone resolution additionally allocates `colorX` candidates inside its token loop. Palette reads, owned label creation, and text measurement are split across property helpers; badge overlay measures the same display twice, and several label paths copy an already owned string again. PERF-MVP-184 compiles one typed material spec per changed node with one theme snapshot, borrowed/shared label, and one text layout. Stable generations perform none of this work.

Avatar exposes a separate resource-lifecycle failure under PERF-MVP-185. After the visual cache supplies owned pixels, every paint scans the full target image to apply the same rounded alpha mask and formats a derived resource key. Render13 must own a bounded `(resource, generation, size, radius)` variant or express the radius in the draw command; the compiled node only carries a handle and generation.

PERF-MVP-186 covers final command amplification that survives compilation. Alert close and chip delete each emit ten dot quads, alert icons emit three, avatar and chip fallback glyphs emit two, and paper shadow emits three layers. Product counters decide which paths matter in the MVP. Required glyphs become one real or cached resource/typed compound command; paper retains its layer semantics through a typed, batchable effect. Current-source Cargo, primitive hit/build/lock/allocation/masked-pixel/command counters, product traces, cache bounds, and GPU/Softbuffer pixel parity remain pending.

## MUI X Primitives

All 53 Rust files in `paint_template_nodes/mui_x_primitives` were statically reviewed. Line, pie, sparkline, and gauge charts allocate and clear an RGBA image on every paint, up to 192 by 192 pixels, then execute hand-written per-pixel line, disc, arc, or angle rasterization. The resource key contains only chart kind and dimensions, omitting theme, gauge value, pie hole state, and data generation. PERF-MVP-187 moves chart identity to EditorUI06 and submits typed geometry to Render13 where possible; any required raster becomes a bounded generation-owned worker/cache result, never paint-thread work.

The shared quad helper also acquires the global palette lock even when border width is zero. Callers acquire their own color snapshot before calling it, producing about seven reads for the sample DataGrid, eight for an open picker, and fourteen for the three-row TreeView. PERF-MVP-188 first avoids the zero-border lookup and passes one theme snapshot through each changed component; EditorUI08 ultimately compiles stable component commands with no paint-time theme or geometry work. Current-source Cargo, 1/100/10,000 component counters, chart cache identity/bounds, product traces, and GPU/Softbuffer parity remain pending.

## Material State Layer

The nine material-state-layer and test files were statically reviewed. Interaction resolution and ripple geometry are bounded, but the command entry resolves its palette-backed color before checking whether either an overlay or a ripple will be emitted. Idle buttons and generic nodes therefore take a global theme read and then produce no command. PERF-MVP-189 is a focused direct fix: compute overlay and ripple eligibility first, return when both are empty, and resolve one shared color only for real work. The final compiled-style path removes the call entirely for stable generations. An idle zero-theme-read regression test, current-source Cargo, scale counters, and pixel parity remain pending.

## Template Buttons

The 29 template-button, glyph, and test files were statically reviewed. Button kind and glyph each build the same six-field key with `format!` and lowercase it. Surface and content each run the full style selector, duplicating state, command/tab role, and palette work. Content layout then reacquires metrics for font, line height, clip guard, padding, icon size and gap, chevron reserve, trailing inset, pressed offset, and radius, plus text preferences. PERF-MVP-190 introduces one changed-node `ButtonPaintSpec` carrying typed identity, resolved style, one theme snapshot, label layout, and surface/content/indicator geometry; stable nodes reuse the compiled segment. The existing extensive state, geometry, text, asset-glyph, ordering, and pixel tests form the parity gate. Current-source Cargo is running; scale counters and GPU/Softbuffer parity remain pending.

## Template Fields

The 21 template-field, stepper, and test files were statically reviewed. Search identity is recomputed by identity, geometry, glyph, text-inset, and placeholder paths; one probe can allocate lowercase copies of five candidate strings. Label construction occurs once during placeholder/style resolution and again for the text command. Metrics and theme reads are split across geometry, surface, search, text, stepper, and selector helpers. PERF-MVP-191 introduces one changed-node `FieldPaintSpec` carrying typed normal/search/stepper identity, one label, resolved state/style, one theme/metrics/text snapshot, and resolved geometry; stable fields reuse the compiled segment. Current-source Cargo, allocation/lock/build counters, scale traces, and GPU/Softbuffer parity remain pending.

## Template Icon Buttons

The 18 template-icon-button and test files were statically reviewed. Their entry correctly resolves context and style once, but a stable frame still repeats component identity, string-prefix context classification, style/theme selection, one or two metrics reads, and resource/fallback glyph command creation. No new root item is needed: PERF-MVP-178/179/181/182/183 already own typed role, compiled segment, glyph amplification, resource lifetime, and immutable theme work. Changed icon buttons may perform each operation at most once; stable generations perform none. Current-source Cargo, scale counters, resource traces, and GPU/Softbuffer parity remain pending.

## Template Axis Controls

The 43 axis-label, axis-value-field-style, axis-value-field, and test files were statically reviewed. One Transform value field acquires metrics independently in geometry, surface, and text, then acquires palette independently for background, border, and text. Axis labels rederive five scaled RGB tones for every node. PERF-MVP-192 compiles one typed `AxisControlPaintSpec` per changed node and derives the axis palette once per unified theme generation; stable nodes reuse PERF-MVP-178's compiled segment. Current-source Cargo, scale lock/allocation/build counters, and GPU/Softbuffer parity remain pending.

## Template Inspector Rows

The 38 inspector geometry/glyph/kind/row and test files were statically reviewed. Resource rows copy two or three text strings and resolve two icon assets per stable paint; missing icons expand to three quads. PERF-MVP-174/178/179/181 own shared text, compiled rows, fallback amplification, and resource handles. PERF-MVP-193 is a direct local fix for shadow bool parsing, which currently allocates a lowercase String for every paint; an allocation-free case-insensitive candidate comparison preserves semantics. Current-source Cargo, the focused fix/test, scale counters, and GPU/Softbuffer parity remain pending.

## Template Property Rows

The 21 property-axis parser, property-row, row-metrics, and test files were statically reviewed. A three-axis value with units builds dynamic vectors plus axis/token/join strings before copying seven more strings into commands. Its helper graph acquires row metrics about 28 times and palette about four times. PERF-MVP-194 moves typed scalar/axis values to the projection and compiles one `PropertyRowPaintSpec` from one theme snapshot per changed row; stable rows perform no parse, allocation, theme access, or command build. Current-source Cargo, scale counters, and GPU/Softbuffer parity remain pending.

## Template Selection Controls

The 26 checkbox/radio/toggle geometry, paint, and test files were statically reviewed. Checkbox executes the complete selector three times, radio three to four times, and toggle four times; geometry and label helpers reacquire metrics up to about five times. PERF-MVP-195 compiles one `SelectionControlPaintSpec` per changed node with typed identity, one resolved style/theme/metrics snapshot, label, geometry, and resource handle. Stable controls reuse the compiled segment; tick assets and fallback amplification remain owned by PERF-MVP-179/181. Current-source Cargo, scale counters, and GPU/Softbuffer parity remain pending.

## Template Sliders

The 35 editor slider geometry/paint/test files were statically reviewed, with a focused cross-layer read of the runtime UI tick path. Both editor and runtime accept an external tick count, cast it to `usize`, and emit one quad per tick without a bound. PERF-MVP-196 adds one shared interface-owned UI tick budget plus pixel-column clamping and tests both consumers. Normal sliders otherwise resolve style once, but reacquire a 21-field metrics projection about seven times, or about twelve for range/tick/double-thumb variants, while rebuilding label/value strings. PERF-MVP-197 extends the existing command context into one changed-node `SliderPaintSpec`; stable/static parts compile once and percent changes patch only dynamic segments. Current-source Cargo, budget fixes, scale counters, and GPU/Softbuffer parity remain pending.

## Template Segmented Controls

The 28 segmented/tab geometry, paint, and test files were statically reviewed. Every option is cloned by `row_data` and copied again, selected text is lowercased into a new String, and each label allocates. Group, selected, and per-option label paths execute the complete selector up to N+2 times; per-field geometry helpers similarly reacquire a 16-field metrics projection. PERF-MVP-198 first removes the redundant option copy and selected lowercase allocation, then compiles one `SegmentedControlPaintSpec` per changed node with shared options, typed selection, one style/theme/metrics snapshot, and geometry. Stable controls reuse the compiled segment. Current-source Cargo, direct fixes, scale counters, and GPU/Softbuffer parity remain pending.

## Template Alerts

The 29 alert, toast, glyph, and test files were statically reviewed. Toast identity copies and lowercases the label before the text command copies it again; generic alert tone discovery formats six string fields and lowercases the result. Wide toasts also allocate the fixed action label each paint. PERF-MVP-199 compiles one typed `AlertPaintSpec` per changed node with resolved kind, tone, state, theme, shared text, geometry, and ordered commands. Stable generations perform no identity String allocation, selector/theme lookup, label copy, or command build. Warning and close fallback glyphs currently expand to eight quads each; product counters must prove shipped assets avoid fallback or converge each retained fallback to one mask/atlas command under PERF-MVP-179.

## Template Chips

The 18 chip, chevron-glyph, and test files were statically reviewed. A chip with a chevron projects the full palette about four times and the full host metrics about nine times, copies its label, and probes chevron identity twice per paint. PERF-MVP-200 builds one typed `ChipPaintSpec` for each changed node with identity, chevron presence, state, one palette/metrics snapshot, shared label, colors, geometry, and ordered commands. Stable generations reuse the compiled segment. The three-quad chevron fallback remains a product asset/command-budget decision under PERF-MVP-179.

## Template Status Controls

The 34 status geometry, signal/chip/icon painter, glyph, and test files were statically reviewed. A `label:value` chip copies the complete label, allocates both final text fragments, remeasures the value, and projects status metrics about ten times per paint; signals project them about six times and icon buttons about three. PERF-MVP-201 compiles one typed `StatusControlPaintSpec` per changed node with kind, state, one theme/metrics snapshot, exact shared text fragments, cached measurement/geometry, and separate static/dynamic commands. Stable status generations perform no split, format, copy, measurement, theme/metrics lookup, or command build. Snap, World, and Target manual glyphs still emit five to six quads and remain under PERF-MVP-179's product fallback budget.

## Template List Rows

The 20 list-row, adornment-glyph, and test files were statically reviewed. Each row executes the full style selector about six times for surface, border, text, adornment color, and adornment kind, projects metrics about three times, copies its label, and enters resource resolution for its trailing asset. PERF-MVP-202 compiles one `ListRowPaintSpec` per changed visible row and combines it with PERF-MVP-177's shared visible range and PERF-MVP-181's resource generation. Stable visible rows perform no selector, label, resource resolve, or command build; offscreen rows perform no paint work. Existing asset tests make shipped check, chevron, and disabled fallback count zero an explicit product contract.

## Template Table Rows

The 28 table-row, cell, action, and test files were statically reviewed. Every cell recomputes the complete row column allocation and reacquires cell, action, and column metrics; four cells therefore repeat the same layout four times and remeasure four fixed header samples sixteen times. Cell, surface, and action paths can run the row selector about eight times. Option rows also copy owned `row_data` again and allocate normalized Strings per cell. PERF-MVP-203 compiles one `TableRowPaintSpec` per changed visible row with typed cells, one column layout/theme/metrics/style snapshot, cached cell measurement/geometry/color, and an action resource handle. Stable visible rows build nothing, offscreen rows do no work, and fixed header widths are cached by font/metrics generation. Existing action tests require shipped settings and kebab assets to avoid fallback.

## Template Tree Rows

The 32 tree-row geometry, glyph, painter, and test files were statically reviewed. One row can execute the full style selector about nine times and project theme/metrics roughly fifteen to seventeen times before guides; every indent guide then reacquires metrics in both its rect and x-coordinate helpers. Normal rows also resolve disclosure, object, eye, and secondary action assets and copy their label. PERF-MVP-204 compiles one `TreeRowPaintSpec` per changed visible row with typed icon/action kinds, one state/theme/metrics/style snapshot, shared label, depth-guide geometry, and four resource handles. Stable visible rows build nothing, offscreen rows do no work, and metrics acquisition does not grow with depth. Existing tests make all four shipped glyph fallback counts zero.

## Template Popup Rows

The 45 popup menu/option, adornment, and test files were statically reviewed. Both loops clone row data and resolve style before leaf-level clipping; menu adornment is classified twice per row, repeatedly scanning flags and allocating lowercase icon or label Strings. Visible rows also project metrics three to four times and copy labels and shortcuts. PERF-MVP-205 first gates by row frame and clip before clone/style, classifies one typed adornment with allocation-free ASCII matching, and then compiles one `PopupRowPaintSpec` per changed visible row. Stable rows build nothing and offscreen rows perform no clone, flag, style, text, or command work.

## Template Section Titles

The 23 section-title, icon-glyph, and test files were statically reviewed. An icon-bearing strong title projects metrics about six times and the palette about four times, copies the source label, then copies it again for both strong text layers. Manual icons emit four to six quads. PERF-MVP-206 is P1 because title counts are much smaller than list, table, and tree row counts; it still compiles one `SectionTitlePaintSpec` per changed node with typed icon/strong state, one theme/metrics snapshot, shared label, geometry, and commands. Stable titles build nothing.

## Template Tooltips

The 19 tooltip, arrow/info-glyph, and test files were statically reviewed. A tooltip projects metrics about seven times and copies its title and body. Its border and fill arrows are expanded into one quad per scanline, producing roughly twice the requested size in commands before surface, text, and info-glyph commands. PERF-MVP-207 is P1 because few tooltips are visible, but same-target hover bursts must reuse one `TooltipPaintSpec` with a single theme/metrics snapshot and shared text. Render13 or the icon resource path should own one bounded arrow mask command; the scanline fallback requires explicit zero-product-hit or bounded-budget evidence.

## Template Notification Center

The 11 notification-center files were statically reviewed. Header construction clones every option to count unread rows, then the paint loop clones all options again before leaf clipping; visible rows copy title and description again. PERF-MVP-208 assigns the lowest data fix to EditorLayout09: publish a bounded immutable notification generation with unread count and explicit overflow semantics. EditorUI08 then computes the visible+overscan range before `row_data` and compiles shared text commands. Stable generations do not rescan unread rows, closed centers do no paint work, and offscreen rows are neither cloned nor built.

## Template Shell Panels

The 16 shell-panel, separator, and test files were statically reviewed without a new independent hotspot. Each panel runs one chrome selector; non-content panels do not read frame metrics, while content panels read border width and radius separately. Command count is fixed by kind at one surface plus at most two separators. PERF-MVP-161 supplies one frame theme/metrics snapshot and PERF-MVP-178 reuses the stable compiled segment; dynamic stable-shell counters and pixel parity remain required before acceptance.

## Template Dialogs

The 22 dialog, confirm/alert, action-layout, and test files were statically reviewed. Closed dialogs exit before theme or text work. A typical open confirm dialog can project metrics about fourteen times, the palette eight to ten times, repeatedly scan variant fields for state and severity, clone and measure two action labels, and copy title/body text. PERF-MVP-209 is P1 because only one modal is normally open; each changed dialog still compiles one `DialogPaintSpec` with resolved state/severity, one theme/metrics/palette snapshot, shared text/actions, one action measurement/layout, and ordered commands. Stable-open dialogs build nothing and closed dialogs retain zero work.

## Template Drag Overlay

The nine drag-overlay files were statically reviewed. Inactive overlays exit before paint work; active overlays use bounded O(1) geometry, no theme lock, and at most four commands. PERF-MVP-210 addresses the remaining high-frequency cost: each pointer move copies an unchanged payload label and rebuilds surface, icon, and text even when only coordinates change. Drag start or payload generation compiles shared text/style/static sizes once, while same-payload moves patch only frames and indicator geometry. Dedicated tests and pointer-move counters are required because this module currently has no local test suite.

## Template Command Palette

The 39 command-palette painter files were statically reviewed and the already-reviewed registry/open-state entry points were rechecked. Opening first owns a complete entry Vec, then materializes a complete commands UiValue and clones every id into filtered commands. Paint clones every row before leaf clipping; each visible row projects metrics about five to six times and copies label/detail text. PERF-MVP-211 assigns immutable catalog generation and typed search/index results to Editor08. EditorUI08 consumes shared handles, computes visible+overscan before `row_data`, and compiles changed row segments with one panel/search theme/metrics snapshot. Stable catalog opens avoid full deep copies and offscreen rows do no paint work.

## Template Material Feedback

The 21 material-feedback and test files were statically reviewed. Linear progress and backdrop have bounded command counts, but circular progress allocates a `size * size * 4` RGBA buffer and runs `sqrt`, `atan2`, and modular angle work for every pixel on every paint. It then formats a key that records only the red channel of track and fill colors, and indeterminate pixels use a different percent source from that key. Stable retained frames therefore repeat CPU rasterization and allocation while exposing unsafe cache identity.

PERF-MVP-212 now has a transitional direct mitigation. The resolved and normalized percent is shared by raster and key; the key includes exact progress bits and complete track/fill RGBA. A four-entry thread-local size LRU stores only ring pixel offsets and turns, so squared-distance membership, `atan2`, and modular angle work occur on first use of a size rather than every stable paint. Owned RGBA allocation/recolor and key formatting still occur. The final direction follows Slint's retained `Path`/`ArcTo`, Material UI's SVG circle/dash parameters, and Godot's retained texture/geometry submission: Render13 or the host backend owns typed ring/arc geometry, while the UI updates only progress or indeterminate angle. Stable determinate frames then perform no raster, key formatting, allocation, or upload; indeterminate animation changes typed dynamic parameters rather than rebuilding pixels.

## Template Dropdowns

The 19 dropdown, glyph, metrics, and test files were statically reviewed. The path emits a bounded surface, text, and shared icon, and the option popup is correctly owned elsewhere. However, placeholder style selection builds the label once and text emission builds it again. Surface, text, chevron reserve, chevron size, and chevron inset each project the same eight-field dropdown metrics, for about five projections per control.

PERF-MVP-213 now resolves one `(label, placeholder)` pair and one metrics snapshot in the dropdown entry, then lends them through style, surface, text, and glyph layers. The redundant glyph metrics forwarding module was deleted; child layers no longer read the global metrics snapshot. Source guards lock the one-label/one-metrics budget and a behavioral test preserves first-option fallback. Current-source Cargo and pixels remain pending. The follow-up folds the result into the shared compiled segment so stable dropdown generations perform no label, theme/metrics, resource-resolution, or command work. This follows Slint's retained current-value/text/icon bindings and Godot's selected-text and minimum-size caches. Popup row clipping and virtualization remain under PERF-MVP-205.

## Template Sample Grid

The 11 sample-grid and test files were statically reviewed. Every dashed grid line is expanded into one quad per three-pixel dash, so command count grows with tick count and panel pixel dimensions. Each diamond marker is expanded into nine or thirteen scanline quads; a selected marker can emit about 25 marker commands before its label. Tick values are formatted into new Strings every paint, while static grid, text, and points rebuild together with selection changes.

PERF-MVP-214 assigns immutable tick/label/point generations to Editor07, static/dynamic segment compilation to EditorUI08, and batched dashed-line/marker primitives to Render13 or the host backend. This follows Godot's blend-space canvas invalidation plus line/circle primitives and Fyrox's retained point widgets. Stable frames perform no static build or formatting; point drags patch only the affected dynamic marker/label segment.

## Template Timeline Strip

The 11 timeline-strip and test files were statically reviewed. Tick generation uses an unbounded cumulative-float `while` loop. Tiny intervals can allocate enormous vectors, and an interval that no longer changes `time` can make the UI thread loop forever. Surface and text independently allocate the same tick list each paint, text reformats every tick, and playhead changes rebuild static ticks, labels, and all scanline-expanded key markers.

PERF-MVP-215 first lands a direct integer-indexed, pixel-column and hard-cap bounded tick generator, called once per paint and borrowed by surface/text. Editor07 later publishes a preformatted timeline generation; EditorUI08 separates static ticks/labels/keys from dynamic playhead/selection, and marker batching follows PERF-MVP-214. Stable frames do no tick/label/static command work, while scrubbing patches only dynamic segments.

## Template Weight Heatmap

The 10 weight-heatmap and test files were statically reviewed. Authored rows and columns have no upper bound. Every cell revisits the complete source model, clones each source DTO, and evaluates an exponential influence, making CPU and clone work O(columns * rows * sources) while host commands grow O(columns * rows). Markers traverse sources again and expand each point into scanline quads.

PERF-MVP-216 now projects source DTOs once per paint and lends the same slice to field and markers. Field dimensions are bounded by plot pixels, 4,096 cells, and a 65,536 influence-evaluation budget that lowers grid resolution as source count rises without dropping sources. Legend strips are bounded to 64 and their pixel height. Each marker is one rounded quad rather than seven or eleven scanline commands. Extreme/non-positive dimensions, adaptive budgets, source preservation, and marker command count have unit guards; current-source Cargo and pixel evidence remain pending. Editor07 then owns immutable heat generations and worker scheduling; Render13 should retain a texture or compute result. Stable generations perform no heat compute, source clone, command build, or upload.

## Template Viewport Scene

The 105 viewport-scene painter and test files were statically reviewed against the 89 `WorkbenchViewport*` controls declared by the Workbench viewport ZUI. The dense fallback runs multi-stage string classification and rebuilds layered surface, lighting, floor, prop, and gizmo commands per node. Floor-grate, rack, and cargo details expand command count with pixel dimensions. This is static layout/theme work, and it is wholly redundant underneath a fully covering live viewport image.

PERF-MVP-217 now has a direct guard for the exceptional componentized missing-layout path: when the presentation carries a valid live viewport image, a typed-kind transform drops decorative viewport scene nodes before command construction while retaining toolbar, selection, axis, gizmo, and unrelated nodes. Missing or invalid images preserve the complete fallback. This is a local mitigation; it still visits and classifies nodes. EditorUI08 must still publish a typed live/fallback viewport generation so every live path skips the decorative subtree and submits the Render16/13 texture handle plus typed overlays. No-frame mode compiles or rasterizes the fallback once per layout/theme generation and reuses it through damage tracking. This matches Unreal's single viewport draw element or fallback box and Godot's retained SubViewport plus explicitly invalidated overlays.

## Template Node Transform Ownership

PERF-MVP-218 removes an independent DTO-copy amplifier in the shared template pipeline. `ModelRc::row_data` already returns an owned `TemplatePaneNodeData`; the transform branch cloned that full value again before passing it to a consumer. The branch now consumes the owned row through an explicit `match`, while the no-transform branch also moves it once. Existing move/clip/suppress and identity-path tests retain source-model and pixel semantics, and a source guard prohibits `source_node.clone()` from returning. Current-source Cargo and allocation counters remain pending.

## Pane Content And Asset Projection

The 10 `paint_workbench_renderer/docks/pane.rs` and `docks/pane/**` files were statically reviewed. Pane visibility, shell/background, viewport, template, native, diagnostics, and fallback selection are bounded constant dispatches. The asset-content projector initialization was not: Activity scanned and cloned the complete node model once to find its panel and again to count folder rows; Browser independently searched for grid, table, header, and preview, reaching four full scans before the template pipeline's own traversal.

PERF-MVP-219 now collects Activity and Browser geometry summaries in one model traversal per projector, with early exit when a thumbnail grid establishes Browser mode. The native Browser content scrollbar uses the same single-pass grid-or-list discovery instead of four independent lookups. Source guards fix the one-loop budgets and remove the old search helpers; existing tests retain list/thumbnail, fixed header/grid, preview boundary, scroll, clip, hover, empty, and stale-state behavior. Unreal's `SListView` owns an item source and generated visible row widgets, while Godot's `ItemList` retains `rect_cache` and invalidates shape only on change. Editor09 should likewise publish generation-owned asset identity/geometry and visible ranges so stable paint performs zero projector/scrollbar initialization scans. Current-source Cargo and product counters remain pending.
