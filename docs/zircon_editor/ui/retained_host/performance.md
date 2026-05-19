---
related_code:
  - zircon_editor/Cargo.toml
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/ui/retained_host/app/presentation_cache.rs
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
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/error.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/factory.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/host_chrome_presenter.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/asset_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/welcome_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/close_prompt_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/chrome_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/pane_button_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/resize_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/tab_drag_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/template_hover_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/viewport_toolbar_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs
  - zircon_runtime/src/rhi/mod.rs
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/pipeline.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_runtime/src/core/diagnostics/profiling/ui_hotspot.rs
  - zircon_runtime/src/core/diagnostics/profiling/export.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_editor/src/ui/retained_host/ui_perf.rs
  - zircon_editor/src/ui/layouts/views/preview_images.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - tools/ui-profile-capture.ps1
implementation_files:
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/ui/retained_host/app/presentation_cache.rs
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
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/error.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/factory.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/host_chrome_presenter.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/asset_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/welcome_surface/bridge.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/close_prompt_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/chrome_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/pane_button_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/resize_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/tab_drag_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/template_hover_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/viewport_toolbar_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs
  - zircon_runtime/src/rhi/mod.rs
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/pipeline.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_runtime/src/core/diagnostics/profiling/ui_hotspot.rs
  - zircon_runtime/src/core/diagnostics/profiling/export.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_editor/src/ui/retained_host/ui_perf.rs
  - zircon_editor/src/ui/layouts/views/preview_images.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - tools/ui-profile-capture.ps1
plan_sources:
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
tests:
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/geometry.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/pipeline.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/text.rs
  - zircon_runtime/src/core/diagnostics/profiling/ui_hotspot.rs
  - zircon_runtime/src/core/diagnostics/profiling/export.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_editor/src/ui/retained_host/ui_perf.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/host_chrome_presenter.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes.rs
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer/welcome_recent_pointer_bridge_sync.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
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
  - cargo test -p zircon_editor --lib painter --locked
  - cargo test -p zircon_editor --lib patch_command_stream_matches_legacy_region_repaint_pixels --locked
  - cargo test -p zircon_editor --lib text_draw_skips_disjoint_active_and_explicit_clips --locked
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

`ChromeCommandStream` is now the retained-host command surface, not a parallel stub. The retained painter can run in record-only mode, so `record_host_frame_commands` traverses the same workbench, template-node, viewport image, close prompt, floating-window, menu, debug overlay, rect, border, image, and text paths that CPU painting uses. Full streams describe the complete retained UI; patch streams carry the same command vocabulary but clip generation to the requested damage region. Image commands preserve resource/content-derived keys through record, stream conversion, GPU upload, and softbuffer replay. Atlas-capable image commands additionally carry `atlas_uv: Option<ChromeImageUvRect>`: recorded software/viewport images set it to `None`, while future SpriteAtlas consumers can use one atlas texture `resource_key` plus per-entry UV metadata. This keeps GPU and softbuffer fallback on one UI expression instead of letting two painters drift.

`GpuChromePresenter<P: UiSurfacePresenter>` converts the chrome command stream into `zircon_runtime::rhi::UiSurfaceDrawList`. It records command full/patch counters, propagates runtime surface failures instead of hiding them, and emits `gpu_upload_bytes`, actual `gpu_draw_calls`, `gpu_visible_commands`, `gpu_visible_draw_items`, `gpu_batch_layers`, and `gpu_batch_dependencies`. It also maps `ChromeImageUvRect` directly into `UiSurfaceImageUvRect`, so atlas metadata crosses the editor/runtime boundary without exposing editor asset-manager or renderer-specific types. It never imports `wgpu`, and its factory no longer names `rhi_wgpu`; the concrete native surface, swapchain, offscreen retained UI target, quad/image pipelines, glyphon text atlas, texture uploads, batch planning, and surface present are selected by the runtime RHI factory.

Softbuffer replay deliberately ignores atlas UV metadata. The software command-stream executor paints the embedded `rgba` bytes as the image payload it was given and falls back to `FALLBACK_IMAGE_COLOR` when bytes are absent, so atlas-backed producers must only depend on GPU atlas sampling when they provide the atlas texture to runtime. Non-atlas viewport and recorded-image paths continue to set `atlas_uv: None`, preserving byte-for-byte software parity tests.

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

The UI batching validation profile path now exports retained-host evidence beside the runtime trace. `profiling_artifacts.rs` writes `ui_profile_geometry.json` on each profiling present with the client size, selected backend, splitter frames, document/drawer/host tabs, activity-rail buttons, viewport frame, viewport-toolbar controls, dispatchable template controls, and sampled hit points. `profiling_hit_routes.rs` keeps the route-consistency checks out of the already-large artifact writer: each sample records whether the rendered frame contains the point and whether the shared retained-host route or surface hit-test would hit the same control. `tools/ui-profile-capture.ps1` consumes that file for `drag`, `drawer_resize`, `click`, and `idle_hover` auto-interaction before falling back to fixed client ratios, so resize and drag gates now target live splitter/tab geometry rather than approximate coordinates. The drawer-resize gate also writes `ui_interaction_evidence.json` with the selected splitter, pointer path, before/after layout deltas, and a `resize_changed_layout` assertion, which makes border-drag effectiveness testable instead of relying on a visual guess.

Material Component Lab uses the same evidence path through `--builtin-view editor.material_component_lab`. Its click gate is intentionally component-only: `material_lab_click` uses live `template_controls` instead of document tabs or host page tabs, so the measured click scenario represents Material prototype feedback rather than page activation. The capture script also accepts the comma-separated list used by the plan. The 2026-05-16 strict run produced `20260516-123734-material_lab_startup`, `20260516-123745-material_lab_hover`, and `20260516-123756-material_lab_click`; all three reported zero UI hotspot alerts and no software fallback. The click capture passed with `dirty_paint_only_count=1`, `redraw_region_count=2`, `presentation_rebuild_count=0`, `dirty_layout_count=0`, and `dirty_presentation_count=0`. The hover capture had no presentation churn, and its draw calls equaled visible items only because the patch was fully dependency-bound (`dependency_density=1.000`), so batch reduction was not expected.

The Material Lab structure is also guarded at the UI-asset boundary. `material_component_lab_shell_keeps_material_lab_layout_regions` freezes the AppBar, Drawer, scrollable component-family content area, right-side interaction legend, and the eight official family sections. It also keeps the MUI X subsection in the planned Tree View, Data Grid, Charts, chart subtypes, and AgentChat order. A direct structured TOML validation passed independently of Cargo, covering 73 prototypes/imports, 63 MUI Core docs rows, 10 MUI X prototype rows, and 48 authored `MaterialLab/*` interaction routes. Static binding validation also found all 48 authored event ids in the builtin template binding registry and confirmed the `--builtin-view editor.material_component_lab` descriptor/capture route; `material_component_lab_feedback_events_use_consistent_ids_routes_and_kinds` now checks each event id, dotted route, and event kind tail, while `material_component_lab_feedback_events_are_registered_as_builtin_bindings` encodes the source-level event-to-binding and `EditorUiEventKind` check in Rust. `MaterialLab/MuiXGauge/Hover` now gives Gauge the same chart hover feedback route as the other MUI X chart subtypes, and `material_component_lab_mui_x_prototypes_define_feedback_routes` locks that every MUI X prototype keeps a Material Lab feedback route. The design-matrix filename pass confirms 68 explicit `material_*.zui` references resolve to existing prototype files, and `material_ui_component_design_matrix_names_existing_zui_prototypes` now encodes that guard in Rust. The Rust guard was formatted successfully. The lower runtime compile blockers seen earlier were corrected by their owning sessions, and `cargo metadata --locked --no-deps --format-version 1` now passes; however, repeated focused Rust reruns on E: and D: target directories exit with process code `-1` in dependency compilation without a Rust diagnostic. The 2026-05-16 15:36 warm-target retry emitted only unrelated `zircon_runtime::core::framework::net::websocket` dead-code warnings before the same process exit. At that checkpoint, 23 Cargo/Rust processes from parallel sessions were active and E: had about 57 GB free, so the current Rust evidence gap remains classified as environment/compiler-process pressure, not as a Material Lab layout assertion failure.

The prototype input contract also separates visual state samples from real dispatch targets. Static and utility placeholders can show hovered/focused/selected styling in their props, but if they do not define a `MaterialLab/*` route then `input_interactive`, `input_clickable`, `input_hoverable`, and `input_focusable` stay false. `material_component_lab_non_route_prototypes_are_not_dispatchable_controls` keeps automated click captures from selecting no-feedback placeholder controls. The paired `material_component_lab_route_prototypes_are_dispatchable_controls` guard checks the other side of the contract: every prototype with a `MaterialLab/*` route must keep at least one dispatchable flag, so the visual evidence path cannot silently lose all click/hover/focus targets. `material_component_lab_feedback_routes_live_on_dispatchable_sample_nodes` then pins that route to the visible `material-lab-sample` node and requires all four input flags to be `true` on that node, while non-route prototypes must not hide a feedback node elsewhere. `material_component_lab_feedback_route_inventory_matches_expected_interactions` freezes the interaction kind for each route so specialized samples remain specialized: Slider stays `DragUpdate`, chart/Tooltip samples stay `Hover`, Chat samples stay `Submit`, toggle controls stay `Toggle`, selector controls stay `Change`, and button/surface controls stay `Click`. `material_component_lab_interactive_inventory_matches_route_bearing_prototypes` keeps the static interactive whitelist aligned with the route-bearing asset inventory, and `material_component_lab_places_every_prototype_once_in_visible_sections` verifies that every imported prototype appears once in a visible family section instead of staying as an unused import. `material_component_lab_prototype_nodes_match_material_file_stems` additionally keeps each `prototype_*` node id aligned with the `material_*.zui` file stem and exported component name. `material_component_lab_shell_keeps_material_style_contract` freezes the shared dark Material theme import plus shell/card classes, colors, border tone, and 12px panel radius. The shell order guard now freezes every family section's internal prototype order, not only the top-level section list and MUI X subsection.

The corresponding Rust boundary coverage is split by responsibility under `zircon_editor/src/tests/ui/boundary/material_component_lab/`: inventory, feedback, shell, projection, and support. The split is intentionally test-only and preserves the same retained evidence contracts while leaving room for additional component-specific guards without growing another monolithic test file. The inventory module now checks parsed sample-node props rather than only scanning source text: every `material_*.zui` prototype must expose one `material-lab-sample` node with shared Material classes, typed variant/tone/validation props, typed state and input flags, and numeric radius/border values. It also freezes the prototype card root as a fixed-height vertical Material card with stretch width, `6px` internal gap, and stable `title`, `meta`, `sample` children. That keeps profiler evidence tied to the actual visible prototype node the retained host will render and keeps the component grid from shifting as more samples are added.

The same capture script now writes sidecar evidence files after the runtime exporter creates `timeline.zrtrace.json`, `timeline.perfetto.json`, `hotspots.json`, `ui_hotspots.json`, and `summary.md`. `ui_batch_metrics.json` derives `batch_success_rate`, `draw_reduction_ratio`, `dependency_density`, and `layer_density` from `ui_hotspots.json`, and records the partial-order model, list-row batching interpretation, ideal case, worst-case degeneration, and rectangular clip/mask boundary. `ui_hit_consistency.json` stores the route/frame sample results and fails the strict evidence gate if any center or negative sample disagrees. `screenshot_reference.png` comes from the retained host software painter for the same presentation data; `screenshot_gpu.png` is a live client-area capture of the normal GPU run; optional `-CaptureSoftbufferScreenshot` launches a second profiling-only `ZIRCON_PROFILE_FORCE_SOFTBUFFER=1` window against the same temporary project, writes `screenshot_softbuffer.png`, compares both live captures to the reference plus direct GPU-vs-softbuffer in `screenshot_diff.json`, records the screenshot thresholds, and fails strict parity when direct GPU-vs-softbuffer exceeds the configured differing-sample ratio or average-channel-delta limit.

`ZIRCON_PROFILE_FORCE_SOFTBUFFER` is intentionally not a runtime/editor public DTO. `HostPresenterBackend::default_native()` only reads it under the profiling feature to select `Softbuffer` for screenshot parity, while normal native startup still attempts GPU first and records the actual fallback backend if GPU creation fails. Full strict profile acceptance requires zero normal-run `software_fallback_present_count`, zero `ui_hotspots` alerts, `gpu_draw_calls < gpu_visible_draw_items`, non-empty hit-consistency samples with no failures, drawer-resize layout movement for the border-drag scenario, and for `asset_refresh` an actual asset/editor/resource change counter from touching the temporary project asset file. Targeted evidence reruns may close one blocker, such as screenshot color-space parity or scenario counter attribution, only when the remaining alerts are stated explicitly. Rectangular clip validation remains the current mask boundary: solid/image command geometry and UVs are CPU-clipped before batching, text uses glyphon bounds, and non-rectangular masks would need a future explicit batch key/stencil layer or fallback rather than being inferred by this capture path.

The 2026-05-17 screenshot parity blocker was in the runtime WGPU surface color-space path rather than retained-host command generation. A pre-fix viewport-image run, `20260517-182730-viewport_image`, showed direct GPU-vs-softbuffer `differing_sample_ratio=0.6522`, matching the observed bright GPU capture. `zircon_runtime::rhi_wgpu::ui_surface` now keeps the retained UI target in `Rgba8Unorm`, prefers non-sRGB swapchain formats, clears new retained targets to opaque black, and prefers opaque swapchain alpha. After rebuilding the profiling binaries, `20260517-190736-viewport_image` reduced direct GPU-vs-softbuffer diff to `differing_sample_ratio=0.0165` and `average_channel_delta=0.9022`, below the configured `0.25` / `10.0` thresholds, while keeping `viewport_image` batch evidence at `21` visible draw items to `16` GPU draw calls and hit consistency at `93 failed=0`.

The same rebuild closed the earlier scenario-attribution gap for the requested interaction evidence. `HostRedrawRequest::merge` now lets the later frame-update scenario own a coalesced redraw request, so resize and asset-refresh presents are no longer hidden under earlier click/startup labels. `20260517-190840-drawer_resize` used live splitter geometry, moved the left drawer by `80px`, refreshed geometry, kept `87 failed=0` hit samples, and recorded `653` visible draw items to `124` GPU draw calls. `20260517-190851-asset_refresh` recorded real asset-refresh presenter work with `266` visible draw items to `42` GPU draw calls and `114 failed=0` hit samples. These captures are evidence for geometry-derived interaction, GPU counter attribution, screenshot parity, and no normal-run software fallback; they still leave UI hotspot cleanup work open for `click/non_structural_interaction_rebuilt_presentation`, `drawer_resize/resize_triggered_slow_path_rebuild`, `idle_hover/region_request_repainted_full_frame`, and `startup/gpu_presenter_recorded_no_draw_calls`.
