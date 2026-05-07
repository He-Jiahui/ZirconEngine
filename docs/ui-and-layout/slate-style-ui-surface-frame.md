---
related_code:
  - zircon_runtime_interface/src/ui/surface/arranged.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/layout/metrics.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/layout/scroll.rs
  - zircon_runtime_interface/src/ui/layout/linear_sizing.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/ui_geometry_metrics.rs
  - zircon_runtime_interface/src/ui/tree/node/visibility.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/slot.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/surface/arranged.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/reflection_snapshot.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_runtime/src/ui/tree/node/interaction.rs
  - zircon_runtime/src/ui/tree/node/render_order.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/tests/template_grid_flow.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
  - zircon_runtime/src/ui/template/build/layout_contract.rs
  - zircon_runtime/src/ui/template/build/container_inference.rs
  - zircon_runtime/src/ui/template/build/parsers.rs
  - zircon_runtime/src/ui/template/build/slot_contract.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_root.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/surface_frame.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/viewport_toolbar.rs
  - zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/src/ui/slint_host/host_contract/redraw.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/layouts/views/preview_images.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/frame.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/diagnostics_overlay.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/debug_reflector_overlay.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/text.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/slint_host/host_contract/diagnostics.rs
  - zircon_editor/src/ui/slint_host/host_contract/presenter.rs
  - zircon_editor/src/ui/slint_host/app/invalidation.rs
  - zircon_editor/src/ui/slint_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/layouts/views/preview_images.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/shared_pointer/viewport_toolbar.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/route_for_control.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/cycle_display_mode_route.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/cycle_grid_mode_route.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/snap_routes.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/toggle_routes.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/frame_selection_route.rs
  - zircon_editor/src/ui/workbench/reflection/widget_reflector.rs
  - zircon_editor/src/ui/workbench/debug_reflector/model.rs
  - zircon_editor/src/ui/workbench/debug_reflector/overlay.rs
implementation_files:
  - zircon_runtime_interface/src/ui/surface/arranged.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/layout/metrics.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/layout/scroll.rs
  - zircon_runtime_interface/src/ui/layout/linear_sizing.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/ui_geometry_metrics.rs
  - zircon_runtime_interface/src/ui/tree/node/visibility.rs
  - zircon_runtime_interface/src/ui/tree/node/tree_node.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/slot.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/surface/arranged.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/reflection_snapshot.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
  - zircon_runtime/src/ui/tree/node/interaction.rs
  - zircon_runtime/src/ui/tree/node/render_order.rs
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/tests/template_grid_flow.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_runtime/src/ui/template/build/layout_contract.rs
  - zircon_runtime/src/ui/template/build/container_inference.rs
  - zircon_runtime/src/ui/template/build/parsers.rs
  - zircon_runtime/src/ui/template/build/slot_contract.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_root.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/surface_frame.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/viewport_toolbar.rs
  - zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/src/ui/slint_host/host_contract/redraw.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/frame.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/diagnostics_overlay.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/debug_reflector_overlay.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/text.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/slint_host/host_contract/diagnostics.rs
  - zircon_editor/src/ui/slint_host/host_contract/presenter.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/invalidation.rs
  - zircon_editor/src/ui/slint_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/runtime_diagnostics.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/shared_pointer/viewport_toolbar.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/cycle_display_mode_route.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/cycle_grid_mode_route.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/snap_routes.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/toggle_routes.rs
  - zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/frame_selection_route.rs
  - zircon_editor/src/ui/workbench/reflection/widget_reflector.rs
  - zircon_editor/src/ui/workbench/debug_reflector/model.rs
  - zircon_editor/src/ui/workbench/debug_reflector/overlay.rs
plan_sources:
  - Shared Slate-Style UI Layout, Render, And Hit Framework
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/ArrangedWidget.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/ArrangedChildren.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/Visibility.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Events.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/SViewport.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/Components/WidgetComponent.cpp
  - dev/bevy/examples/ui/render_ui_to_texture.rs
  - dev/bevy/examples/ui/widgets/viewport_node.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate/SlateInvalidationRoot.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - .codex/plans/Editor 绘制与鼠标事件优化计划.md
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
  - user: 2026-05-06 Zircon UI 与 Unreal Slate 差异审计及后续里程碑
  - user: 2026-05-06 完善命中测试，参照 dev 下虚幻源码
  - user: 2026-05-07 输入框无法输入文本、界面响应性能很差，需要 profile 并优化热点
  - docs/superpowers/specs/2026-05-06-ui-lifecycle-reflection-reflector-design.md
  - docs/superpowers/plans/2026-05-06-ui-lifecycle-reflection-reflector.md
  - docs/superpowers/plans/2026-05-06-ui-debug-reflector-full-closure.md
tests:
  - cargo test --manifest-path E:\Git\ZirconEngine\Cargo.toml -p zircon_runtime_interface ui_surface_frame_contract_carries_arranged_render_and_hit_state --locked --target-dir E:\zircon-build\targets --jobs 1
  - cargo test -p zircon_runtime_interface --lib ui_visibility_contract_separates_layout_render_and_hit_policy --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime_interface --lib ui_surface_debug_snapshot_contract_serializes_reflector_and_batch_stats --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test --manifest-path E:\Git\ZirconEngine\Cargo.toml -p zircon_runtime surface_rebuild_derives_render_and_hit_from_same_arranged_geometry --locked --target-dir E:\zircon-build\targets --jobs 1
  - cargo test -p zircon_runtime --lib legacy_visible_false_is_normalized_into_hidden_visibility_for_surface_outputs --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime --lib focus_navigation_and_scroll_candidates_use_effective_visibility --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime --lib hit_grid_respects_slate_visibility_and_clip_semantics --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime --lib surface_rebuild_derives_render_and_hit_from_same_arranged_geometry --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime --lib hit_grid_omits_disabled_nodes_and_debug_dump_reports_why --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime --lib scrollable_virtualized_children_enter_hit_grid_only_when_arranged_visible --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime --lib hit_grid_uses_cursor_radius_as_slate_style_nearby_hit_fallback --locked --jobs 1 --target-dir E:\zircon-build\targets\hit-test-unreal-slate --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib exact_hit_wins_over_nearby_cursor_radius_candidates --locked --jobs 1 --target-dir E:\zircon-build\targets\hit-test-unreal-slate --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime_interface --lib ui_hit_metadata_contract_carries_scope_space_and_world_ray --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-hit-scope --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib hit_grid_scope_and_world_queries_require_the_shared_surface_frame --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-hit-scope --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib virtual_pointer_query_maps_custom_3d_hits_into_surface_local_hit_path --locked --jobs 1 --target-dir E:\zircon-build\targets\hit-test-unreal-slate --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime_interface --lib ui_hit_metadata_contract_carries_scope_space_and_world_ray --locked --jobs 1 --target-dir F:\cargo-targets\zircon-world-space-ui-interface --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir F:\cargo-targets\zircon-world-space-ui-interface --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib pointer_route_can_use_virtual_pointer_hit_from_custom_surface_mapper --locked --jobs 1 --target-dir E:\zircon-build\targets\hit-test-unreal-slate --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib pointer_dispatch_uses_virtual_pointer_query_for_component_events --locked --jobs 1 --target-dir E:\zircon-build\targets\hit-test-unreal-slate --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib surface_dirty_rebuild_keeps_render_only_changes_out_of_layout --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_runtime --lib surface_dirty_domains --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-dirty-domains --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib layout_slots --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib template_grid_flow --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib surface_frame --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib captured_pointer_dispatch_keeps_move_and_up_targeting_the_captured_node_outside_hit_bounds --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib surface_frame_hit_test_uses_borrowed_grid_with_index_parity --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-m7 --message-format short --color never
  - cargo test -p zircon_runtime --lib surface_dirty_rebuild_records_cached_counts_when_no_dirty_flags_exist --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-m7 --message-format short --color never
  - cargo test -p zircon_runtime --lib surface_frame_render_hit_and_pointer_dispatch_share_arranged_authority --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-shared-core --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime_interface --lib ui_geometry_metrics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-interface-geometry --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib layout_slots --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m1-slot-panel --message-format short --color never -- --nocapture
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
  - zircon_runtime/src/ui/tests/template_grid_flow.rs
  - zircon_runtime_interface/src/tests/ui_geometry_metrics.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test --manifest-path E:\Git\ZirconEngine\Cargo.toml -p zircon_editor native_host_viewport_toolbar_only_dispatches_primary_press --locked --target-dir E:\zircon-build\targets --jobs 1
  - cargo test -p zircon_editor --lib apply_presentation_resolves_splitters_from_shared_visible_drawer_projection --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_editor --lib viewport_toolbar_surface_frame_includes_projected_route_controls_without_action_list --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - cargo test -p zircon_editor --lib native_host_pointer_click_routes_viewport_toolbar_buttons_before_viewport_body --locked --target-dir E:\zircon-build\targets\slate-ui-framework --jobs 1
  - zircon_editor/src/tests/host/slint_viewport_toolbar_pointer/projection_fallback.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
  - zircon_editor/src/tests/host/slint_window/native_viewport_image.rs
  - zircon_editor/src/tests/host/slint_window/shell_window.rs
  - cargo test -p zircon_editor --lib native_host_welcome_material_text_field_accepts_keyboard_input --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib native_host_welcome_material_button_routes_welcome_callback --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib native_host_generic_template_text_field_routes_builtin_change_binding --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib native_host_document_tab_drag_releases_capture_and_forwards_drop --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib native_host_repeated_hierarchy_hover_moves_do_not_rebuild_presentation --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib native_host_viewport_button_and_scroll_wait_for_viewport_image_repaint --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib native_host_viewport --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib viewport_without_native_repaint --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib native_host_painter_draws_template_svg_image_pixels --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib rust_owned_host_painter_resolves_runtime_svg_image_assets --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui --message-format short --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib presenter::tests --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib host_contract::redraw::tests --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib host_invalidation --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib host_scene_projection_converts_host_owned_panes_to_host_contract_panes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-shared --message-format short --color never
  - cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir D:\cargo-targets\zircon-shared --message-format short --color never
  - cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-shared --message-format short --color never
  - cargo test -p zircon_runtime --lib repeated_same_target_mouse_moves_do_not_dirty_or_rebuild_surface --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never
  - cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m7-current --message-format short --color never
  - cargo test -p zircon_editor --lib builtin_template_compile_cache_is_reused_across_runtime_instances --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib tests::host::slint_window::native_host_contract --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib shared_core --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo test -p zircon_runtime --lib event_routing --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo test -p zircon_runtime --lib component_catalog --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo check -p zircon_runtime --lib --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo test -p zircon_editor --lib workbench_reflection --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo check -p zircon_editor --lib --locked --target-dir E:\zircon-build\targets\ui-lifecycle-reflection
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-input-perf --message-format short --color never
  - tests/acceptance/shared-slate-ui-surface-frame.md
doc_type: module-detail
---

# Slate-Style UI Surface Frame

`UiSurfaceFrame` is the shared surface snapshot for editor and runtime UI. It carries one arranged tree, one render extract, one hit grid, and the current focus/capture/hover state. The arranged tree is the only spatial authority: render extraction and hit testing both consume `UiArrangedNode.frame` and `UiArrangedNode.clip_frame`.

The design follows Unreal Slate's `FArrangedWidget`/`FArrangedChildren` and `FHittestGrid` split. Layout produces arranged widgets; painting and hit grid insertion consume that arranged geometry; hit paths are reported as leaf-to-root bubble routes plus root-to-leaf paths.

## Visibility

`UiVisibility` replaces boolean-only visibility decisions for render and hit behavior:

- `Visible`: occupies layout, renders, self can be hit, children can be hit.
- `Hidden`: occupies layout, does not render, and blocks hit testing for self and children.
- `Collapsed`: does not occupy layout, does not render, and blocks hit testing.
- `HitTestInvisible`: renders but blocks hit testing for self and children.
- `SelfHitTestInvisible`: renders, skips self hit testing, and preserves child hit testing.

Legacy `state_flags.visible == false` is treated as effective `Hidden` before a node reaches arranged output unless the authored visibility is explicitly `Collapsed`, which keeps its layout-collapse semantics. New code should set `UiVisibility` explicitly, but the transitional bool is still normalized by `UiVisibility::effective(...)` and `UiTreeNode::effective_visibility()` so layout, render, focus, scroll, pointer, and hit-grid code cannot reinterpret the same node differently.

The shared helpers define the only allowed predicates for node participation:

- `UiTreeNode::is_render_visible()` and `UiArrangedNode::is_render_visible()` decide render extract inclusion.
- `UiTreeNode::is_focus_candidate()` keeps focus/navigation on enabled render-visible nodes, so `HitTestInvisible` controls can still take keyboard focus when authored as focusable.
- `UiTreeNode::supports_pointer()` and `UiTreeNode::allows_child_hit_test()` split self hit-test from descendant hit-test instead of overloading `state_flags.visible`.
- `UiStateFlags::visible_enabled()` remains only a legacy convenience for places that need the historical bool pair, not a policy source for layout/render/hit.

## Runtime Flow

`UiSurface::compute_layout()` runs layout, then `UiSurface::rebuild()` derives:

1. `UiArrangedTree` from `UiTree` layout cache, clip chain, z index, paint order, input policy, and control metadata.
2. `UiRenderExtract` from the arranged tree, not from a separate coordinate walk.
3. `UiHitTestGrid` from the arranged tree, filtered by visibility, enabled state, input policy, clip frame, and z/paint order.

Linear layout treats `Collapsed` as non-participating, matching Slate's collapsed semantics: collapsed children do not consume main-axis extent and do not create gaps between neighboring visible children. Template-authored `stretch = "Stretch"` axes are recorded on `UiTreeNode` so a node that explicitly asks to fill remaining linear space is not mistaken for an implicit content-sized leaf.

M1.3 closes the shared slot/panel geometry layer rather than leaving panel behavior in editor host code. `UiSlotSchema` remains the component authoring contract for named component content slots, while runtime layout uses parent-owned `UiSlot` records for child placement. The covered runtime slot fields are `padding`, `alignment`, `linear_sizing`, `canvas_placement`, `grid_placement`, `order`, `z_order`, and `dirty_revision`. The container-to-slot mapping is now: `Free -> Free`, `Container -> Container`, `Overlay -> Overlay`, `HorizontalBox`/`VerticalBox -> Linear`, `WrapBox`/`FlowBox -> Flow`, `GridBox -> Grid`, and `ScrollableBox -> Scrollable`.

The shared panel inventory is: Free/Container preserve node anchor, pivot, and position unless an explicit slot padding/alignment policy is present; Overlay consumes slot padding/alignment and preserves z/paint order through arranged, render, and hit outputs; Linear panels consume slot order, padding, alignment, and linear sizing; Flow/Wrap panels consume flow slot order, padding, alignment, gaps, and item minimum width; GridBox consumes fixed row/column counts, gaps, and per-child row/column/span placement; ScrollableBox preserves visible-range virtualization so only arranged visible children enter render and hit grids. The focused tests in `layout_slots.rs` and `template_grid_flow.rs` prove each accepted panel path feeds `UiSurfaceFrame.arranged_tree`, `render_extract`, and `hit_grid` from the same frames.

The hit grid stores spatial cells and entries sorted by paint priority. Querying a point through `hit_test_surface_frame(...)` returns `UiHitTestResult` with the top node, front-to-back stack, and `UiHitPath`. Editor adapters should use this runtime helper for submitted frames instead of rebuilding a local hit index around each host control family.

`UiHitTestQuery` extends the plain point query for the Slate custom-hit path slice. `cursor_radius` mirrors Unreal `FHittestGrid::GetBubblePath(... CursorRadius ...)`: exact point hits are considered first, and radius-only candidates are a fallback ordered by distance while still respecting z/paint order inside each class. `virtual_pointer` mirrors Unreal `FVirtualPointerPosition` and UMG `FWidget3DHitTester`: a host-side mapper or future 3D raycast backend converts a screen/world hit into surface-local current/previous coordinates, then the shared hit grid resolves the normal `UiHitPath`. This keeps custom and world-space UI from inventing a separate dispatch path; they supply a mapped query and still consume `UiSurfaceFrame.arranged_tree + hit_grid`.

The M1.4 hit metadata slice adds `UiHitTestScope` to both queries and hit grids. Scope carries optional user, window, surface, and pointer ids; unspecified fields are wildcards, while conflicting specified fields reject the borrowed grid before any node path is chosen. `UiHitCoordinateSpace` and `UiWorldHitRay` make world/screen/window hit sources explicit, but the shared hit grid still requires a surface-local point or `UiVirtualPointerPosition` before it can resolve widgets. Debug rejects now include scope mismatch, unsupported coordinate-space, and world-hit-unavailable reasons, so multi-window or 3D callers can diagnose missing projection without creating a second coordinate authority.

The first accepted custom/3D slice is intentionally a boundary contract, not a full raycaster. Zircon owns the surface-local DTO and route behavior; runtime rendering, physics, or editor viewport systems own the ray/UV/world-to-local mapping that produces `UiVirtualPointerPosition`. This matches the evidence split in Unreal, where `SViewport` registers an `ICustomHitTestPath`, `FWidget3DHitTester` maps scene hits into widget-local virtual cursor positions, and `UWidgetComponent::GetHitWidgetPath(...)` still delegates final widget path construction to the widget component's `FHittestGrid`. Bevy's render-to-texture UI example uses the same architectural split by raycasting into a texture, converting UV to 2D UI coordinates, and emitting virtual pointer input to the normal UI picking path.

The 2026-05-07 world-space interface follow-up makes that boundary executable. `UiHitTestQuery::with_projected_world_hit(...)` now requires a finite `UiWorldHitRay` plus a mapped `UiVirtualPointerPosition`; only then does a `World` query enter the shared surface-local hit grid. A world query with only a ray, or with a non-finite ray, is rejected before node resolution with `UiHitTestRejectReason::WorldHitUnavailable`. This keeps the contract ready for viewport/RHI raycast producers without letting them bypass `UiSurfaceFrame.arranged_tree + hit_grid`.

`UiSurface::rebuild_dirty(root_size)` is the shared invalidation entry point for retained surfaces. The M1.5 dirty-domain inventory is:

- `layout`, `style`, `text`, and `visible_range`: structural or measurement dirtiness; recompute layout and then regenerate arranged, render, and hit outputs together.
- `hit_test` and `input`: pointer/input policy dirtiness; rebuild the arranged tree and hit grid without recomputing layout or regenerating render commands.
- `render`: paint-only dirtiness; regenerate render extract from the existing arranged tree without rebuilding layout, arranged output, or hit grid.
- legacy `UiStateFlags::dirty`: transitional presentation dirtiness; normalize to `hit_test`, `input`, and `render`, then clear it with the structured flags. It must not imply layout dirtiness.

Editor host invalidation reasons sit above this runtime surface contract. Layout and window-metrics reasons may still enter the host slow path, while paint-only, viewport-image, pointer-hover, hit-test, and render reasons should preserve enough domain information to avoid accidental presentation/layout rebuilds. M7 reflector and overlay consumers should read `UiSurfaceRebuildReport` / `UiSurfaceFrame.last_rebuild` for dirty-domain and phase evidence instead of reclassifying nodes from local host state.

Every rebuild now records a `UiSurfaceRebuildReport` and publishes it through `UiSurfaceFrame.last_rebuild`. The report captures the dirty flag summary, dirty node count, rebuilt phase booleans, arranged node count, render command count, hit-grid entry/cell counts, and elapsed microseconds for layout, arranged-tree, hit-grid, and render extraction phases. A clean `rebuild_dirty(...)` call still refreshes cached counts in the report without marking any rebuild phase, so debug consumers can display current cache size even when no invalidation was needed.

Pointer routing carries the same hit result forward as `UiPointerRoute.hit_path`. `bubbled` remains the direct leaf-to-root dispatch route, while `hit_path.root_to_leaf` is available for Slate-style enter/leave, focus-path, and capture diagnostics without reconstructing ancestry from the tree after the hit query.

Milestone 1 now has an explicit shared-frame authority regression in `surface_frame_authority.rs`. The test builds overlapping controls once, captures `UiSurfaceFrame`, and proves render commands, hit-grid entries, direct surface hits, borrowed-frame hits, and pointer dispatch routes all carry the same arranged frame, clip, z order, hit stack, and bubble path. This is the bottom-layer guard before editor host and Material components can claim Slate-style behavior.

The M1.3 slot/panel geometry slice adds focused coverage for panel-owned layout policy without entering editor or Material visual templates. `layout_slots.rs` now proves overlay slot padding/alignment, virtualized `ScrollableBox` visible windows, Flow/Wrap slot ordering, and GridBox cell placement all feed `UiSurfaceFrame.arranged_tree`, render extract, hit-grid entries, and `hit_test_surface_frame(...)` from the same submitted frames. `template_grid_flow.rs` proves authored `.ui.toml` GridBox/FlowBox containers compile into the shared runtime container and slot contracts. Overlay slot `z_order` promotion and Canvas parent-owned placement remain explicit follow-ups in `docs/zircon_runtime/ui/layout/pass.md`.

M1.T closed the shared-core test gate with focused interface and runtime evidence. Runtime-interface contracts, geometry metrics, and hit-scope tests returned 0 failed; runtime surface authority, hit grid, dirty domains, layout slots, template Grid/Flow, and captured pointer route gates returned 0 failed. Broad `cargo check` validation also passed for `zircon_runtime_interface --tests` and `zircon_runtime --lib`; the runtime crate still emits existing unused-code warning noise.

The matching geometry slice keeps DPI and pixel snapping render-side. `UiGeometry::from_frame_with_metrics(...)` preserves `absolute_frame` as the arranged frame while snapping `render_bounds` and paint clip frames through `UiRenderCommand::to_paint_element_with_metrics(...)`. Hit testing and input still consume the unsnapped arranged tree; paint consumers receive crisp render bounds without inventing another hit coordinate table.

Milestone 1 currently accepts the effective visibility slice rather than the full editor-native cutover. The runtime arranged-tree builder writes the effective visibility into `UiArrangedNode`, render extract and hit grid consume that arranged output, and the retained-tree focus, scroll, pointer, and render-order helpers call the same shared predicates. This specifically prevents a drawer, toolbar, or pane control from being render-visible in one pass and hit-visible in another because a local path read `state_flags.visible` directly.

The focused regressions for this slice cover three boundaries: the interface contract separates `Hidden`, `Collapsed`, `HitTestInvisible`, and `SelfHitTestInvisible`; runtime surface outputs normalize legacy `visible=false` into `Hidden`; focus/navigation and scroll candidates use the same effective visibility helpers as render and hit testing.

## Surface Diagnostics And Reflector Baseline

`debug_surface_frame(...)` is the shared debug entry point for the Widget Reflector-style milestone. It consumes only `UiSurfaceFrame`, so editor and runtime debug tooling cannot drift into separate coordinate systems. The snapshot contains reflected arranged nodes, render counters, material batch groups, hit-grid occupancy, overdraw samples, and the focus/capture/hover state that was current when the frame was submitted.

`UiWidgetReflectorNode` mirrors the arranged geometry rather than the authoring tree alone. Each reflected node carries frame, clip frame, parent/children, z/paint order, visibility, input policy, state flags, control id, render command count, hit entry count, and hit cell count. This is the minimum data needed to build a live/snapshot tree like Unreal's Widget Reflector while still preserving Zircon's `.ui.toml` retained-tree source model.

The lifecycle/property slice adds a second, runtime-surface-oriented reflector DTO family beside the existing debug frame counters. `UiReflectorSnapshot` is produced by `UiSurface::reflector_snapshot(...)` from the retained tree, arranged tree, hit-test result, and focus/capture/hover state. It records `UiWidgetLifecycleState`, declared/effective visibility, state flags, input policy, dirty flags, reflected properties, template binding actions, and optional hit context without rebuilding layout or inventing a separate hit index.

`UiSurface::mutate_property(...)` is the matching runtime mutation seam for safe reflected edits. It accepts visibility, input policy, basic state flags, and template metadata attributes, then reports the dirty domains that a caller must rebuild. Authored `.ui.toml` remains the source document, and the editor Widget Reflector consumer remains read-only unless it deliberately calls this runtime mutation API.

The workbench consumer is `WorkbenchWidgetReflectorModel`. It projects rows and selected-node details from `UiReflectorSnapshot` while owning only local selection. It is not allowed to query Slint widget state or keep an alternate runtime UI tree.

Render diagnostics are intentionally named as estimates until the renderer exposes backend-confirmed counters. `UiRenderDebugStats` groups commands by a stable material signature and reports estimated draw calls from geometry and text-producing commands. The material batch list records a deterministic `break_reason` beside each stable key, so Widget Reflector-style panels can explain whether a group was separated by command kind, clipping, opacity, text/image resource use, or style/material identity. The runtime WGPU pass can later replace the estimate with real submitted pass counters without changing the snapshot boundary.

Overdraw diagnostics use a configurable sample grid over the visible render-command union. They report covered cells, overdrawn cells, max layers, and total layer samples. This is not a replacement for a GPU overdraw pass, but it gives the editor debug UI a deterministic CPU-side overlay source while the material batching and runtime renderer instrumentation mature.

`UiSurfaceDebugSnapshot.overlay_primitives` is the only source for editor reflector overlays. The editor Debug Reflector overlay state may filter those primitives by kind and may derive a missing `DamageRegion` primitive from `UiDamageDebugReport.damage_region`, but it does not rebuild frames from Slint nodes or host rectangles. Runtime Diagnostics carries the filtered primitives through its pane payload into `RuntimeDiagnosticsPaneData.overlay_primitives`; the native painter then draws them as clipped borders/fills over the pane content through `debug_reflector_overlay.rs`. This keeps selected frames, clip frames, hit cells, hit paths, rejected bounds, overdraw cells, material batch bounds, and damage regions on the same shared `UiSurfaceFrame` authority as render and hit diagnostics.

The M7 live Runtime Diagnostics bridge follows the same authority rule. During presentation conversion, the Runtime Diagnostics pane first builds its normal host-projected `body_surface_frame`; only that frame is converted into a diagnostics-only `UiSurfaceFrame` for the Debug Reflector model, detail rows, and overlay primitives. This lets the pane show widget path, focus/capture, hit path, reject reason, render/hit counts, batch breaks, and dirty flags without inventing a second coordinate table. Repeated same-target pointer moves are also locked at the runtime surface layer: after the first hover state is established, 100 identical moves keep the previous rebuild report, dirty flags, requested damage, and component event list unchanged.

## Editor Host Route

The native Slint host stores a toolbar `UiSurfaceFrame` in `SceneViewportChromeData`. That frame is built by iterating route-bearing projected controls from the `.ui.toml` host projection in `BuiltinViewportToolbarTemplateBridge`, so button hit rectangles match the component layout. Adding another toolbar button with a projected control id and binding makes it enter the surface frame without adding Rust coordinate rows or a toolbar action list. Root docks and floating-window active panes receive these frames before native pointer routing runs. `host_contract/surface_hit_test` calls the shared `hit_test_surface_frame(...)` helper and maps hit node `control_id` values to the existing toolbar and pane dispatch callbacks.

Toolbar hit control ids are separated from projected control ids only where the current editor state supplies a parameterized action or an existing semantic alias must be preserved. For example, the projected `SetTool` button maps to the current `tool.move`/`tool.rotate` action key, `FrameSelection` keeps the existing `frame.selection` semantic id, and direct no-argument buttons such as play-mode controls can use their projected control id. This keeps hit geometry owned by layout while preserving the existing viewport command semantics.

Projection control ids now fall back through template bindings instead of re-entering the old toolbar alias table. The legacy pointer-route helpers recognize hit-grid action ids such as `display.cycle`, `snap.translate`, and `frame.selection`; TOML projection ids such as `SetDisplayMode` and `FrameSelection` are resolved by `dispatch_builtin_viewport_toolbar_control(...)` when no legacy route exists. This is the hard-cutover boundary that let the old toolbar alias list be deleted without keeping a compatibility shim.

Some real-host callback paths can still report a zero control rectangle while carrying an already-resolved hit-grid action id. In that case `dispatch_shared_viewport_toolbar_pointer_click(...)` first tries the projected template frame for TOML control ids, then uses the actual click point as a one-pixel active frame for legacy action ids that have no projected control frame. That fallback is not a coordinate table: it exists only so the existing shared `ViewportToolbarPointerBridge` can route the already-known action id through its retained `UiSurface + UiPointerDispatcher` path instead of failing before dispatch. Projection ids with no legacy route still go through template bindings.

Native template-node hit testing also routes through `PaneData.body_surface_frame`, which is built during host presentation conversion from projected template node frames. This keeps pane controls on the same arranged/render/hit model as toolbar controls: native pointer dispatch queries a submitted frame rather than rebuilding a local coordinate model at click time.

Pane component projection keeps a separate fallback label heuristic for bound or button-like controls that have no authored text. That helper only derives visible text such as `Apply Draft`; it does not define a toolbar action-control alias or a dispatch compatibility table.

The native pointer dispatch still gates toolbar activation to primary press only. Release, secondary, and middle button events do not dispatch toolbar commands.

The 2026-05-06 native GUI regression slice extends the same host route to text and drag state. Template-node hits now carry enough projected metadata for native text focus: component role, dispatch kind, edit/commit action ids, and current value text. `HostTextInputFocusData` lives beside pointer capture state in the host contract, so keyboard and IME events are routed to the focused `.ui.toml` control instead of depending on a hidden Slint widget. Enter, Escape, Backspace, and commit text are translated at the window boundary, then forwarded through the same callback envelopes as pointer-dispatched template actions.

Document, drawer, and floating tab drag also stay on the host route rather than using a painter-only overlay. Primary press over a chrome tab arms drag capture from the hit route, movement past the threshold emits the existing host drag callbacks, and release sends the matching drop/up callback before clearing the stored drag state. That cleanup is part of the hit/input contract: stale capture state must not survive into a later paint frame, or the native retained backbuffer can keep showing an obsolete drag marker.

## Editor Native Fast Path

The native host now has the first Slate-style fast path for editor repaint pressure. `SoftbufferHostPresenter` retains a `HostRgbaFrame` backbuffer. A full paint is still required for the first frame, resize, and full invalidation; damage redraws repaint only the clipped region into that retained frame and copy only the damaged pixel rows into softbuffer before calling `present_with_damage`. The painter enforces this through an active `HostRgbaFrame` paint clip, so root skeleton, template nodes, text, viewport images, and overlay primitives all consume the same damage rectangle instead of relying on each draw call to remember a local clip.

Pointer move routing also avoids repaint when the native pixels did not change. Viewport mouse moves still dispatch to the runtime pointer bridge, but they return idle to the native presenter; the host repaints when a new viewport image arrives. Viewport body press, release, and scroll now follow the same rule: they update runtime input state but do not repaint the retained native backbuffer before the renderer submits the next viewport image. Hierarchy hover compares the previous and current pointer-only pane state and damages only the affected row union. Repeating the same hierarchy or asset-tree hover target returns idle, which prevents high-frequency mouse motion from turning into repeated host presentation rebuilds or full-frame paints.

Text input uses the same retained-damage contract. `TemplateNodePointerHit` carries the hit node's host-space frame into `HostTextInputFocusData`, so focus, keyboard insert, backspace, and commit can request a region repaint for the edited control instead of `Full { frame_update: true }`. `HostWindowPresentationData` now also exposes the active text focus to the native painter; `template_nodes.rs` overlays `focus.value_text` for the matching control id and otherwise prefers `value_text` over stale label text for input-like nodes. This gives typed characters immediate visible feedback while keeping the runtime binding callback path intact for the eventual authoritative presentation update.

The 2026-05-08 validation attempt for this text-input fast path ran through `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-input-perf --message-format short --color never`, but the workspace stopped in `zircon_runtime` before editor checking because `zircon_runtime/src/ui/surface/surface.rs` imports the currently missing `crate::ui::layout::compute_incremental_layout_tree`. That failure is tracked as an external incremental-layout workspace blocker rather than evidence against the editor text-input path.

Viewport image updates now use the same paint-only channel as pointer damage. `SlintEditorHost::poll_viewport_image_for_native_host()` accepts a fresh viewport image into `HostViewportImageData`, then queues an external `HostRedrawRequest::Region` for the current viewport content frame. The native event loop drains and coalesces those external redraw requests in `about_to_wait`, so multiple image/damage requests collapse before softbuffer presentation. This path does not set `presentation_dirty`, `layout_dirty`, or `render_dirty`, and a drained region redraw does not invoke `request_frame_update()`.

Repeated status messages are guarded before presentation invalidation. `EditorEventRuntime::status_line()` exposes the current status without constructing a full chrome snapshot; `SlintEditorHost::set_status_line()` now returns when the message is unchanged. This keeps recurring background errors from causing one presentation rebuild per timer tick while preserving normal status-line refresh when the text actually changes.

Builtin template loading is cached at the compiled-document and parsed-document level. The cache key includes the canonical path, modification timestamp, and file length, so multiple bridge/runtime instances can reuse the same builtin `.ui.toml` documents during one editor process without hiding file changes across process restarts. Diagnostic logs now distinguish cache hits from actual template loads, and host presentation logs include a rebuild count to make accidental full projection loops visible.

`apply_presentation` now projects only the active pane payload for each pane kind before building that pane's `body_surface_frame`. Scene panes do not build hierarchy/inspector/console/module/export payloads, and non-project panes do not project project overview data. This keeps presentation conversion aligned with the current visible pane role instead of rebuilding every possible pane body variant on each refresh. Host-owned native payload buckets are still preserved when they are already non-empty, so a synthetic host-scene pane kind cannot drop valid UI Asset Editor, animation, hierarchy, inspector, console, assets, asset-browser, or project-overview data before the surface frame is rebuilt.

`SlintEditorHost` also has a host-level invalidation root modeled after Unreal Slate's `FSlateInvalidationRoot`. The root records structured reasons instead of treating every change as the same dirty flag: layout, tree structure, presentation data, paint-only, pointer hover, viewport image, hit-test, window metrics, and render. Layout/window-metrics reasons still drive the current compatibility `layout_dirty`/`presentation_dirty` slow path, render reasons stay separate from presentation rebuilds, and viewport-image updates are counted as paint-only damage before queuing a regional redraw. The diagnostic channel `editor_host_invalidation` logs slow-path and render-path counts with the merged reasons, so repeated full refreshes can be traced to the source that requested them.

This invalidation root is intentionally a cutover layer, not a second layout system. Existing legacy dirty assignments can still force a slow path; when that happens the recompute log falls back to the observed legacy dirty flags. New editor host code should call `invalidate_host(...)` or `record_paint_only_invalidation(...)` so the reason survives coalescing and can later map directly onto retained arranged trees, hit grids, and cached paint output.

Preview image loading for template-projected nodes now lives in `ui/layouts/views/preview_images.rs` instead of a private Slint host adapter. This keeps layout-level `ViewTemplateNodeData` projection independent of native host conversion and lets both material-style template projection and host conversion share the same icon/media lookup behavior.

The retained painter fast path now also covers visual assets. Template-projected `Image`, `Icon`, and `SvgIcon` nodes carry loaded `preview_image` pixels into `TemplatePaneNodeData`; the native painter converts those pixels once per paint command and draws them through the same clipped `HostRgbaFrame` primitive as viewport images. Runtime-style `UiRenderCommandKind::Image` resolves `UiVisualAssetRef::Image` / `Icon` to cached decoded pixels before falling back to deterministic placeholders. Because these draw calls all use `HostRgbaFrame`'s active paint clip, local damage still limits the work to the dirty region and does not force a presentation rebuild just because an image command is present.

The top-right debug readout is likewise part of the native shell paint path, not a Slint UI replacement. `HostWindowShellData.debug_refresh_rate` is projected into the host contract and painted inside the top-bar clip in `workbench.rs`. The startup fallback uses the same field shape as the live overlay: `FPS 0.0 | present 0 | full 0 | region 0 | pixels 0 | slow 0 | render 0 | paint-only 0`. After the first native present the text is produced from `HostRefreshDiagnostics` plus `HostInvalidationRoot::diagnostics_snapshot()`.

That live overlay makes the retained-damage contract visible in the shell itself. Presenter counters report full paints, region paints, total presents, and total painted pixels, while invalidation counters report slow-path rebuilds, render-path rebuilds, and paint-only requests. Paint-only invalidations still do not set presentation or layout dirty flags; they update the invalidation snapshot and continue through region redraw where a caller provides damage. Presenter region expansion and painter clipping both derive the marker frame from `presentation_top_bar_frame(...)`, which follows the same scene-layout-first top-bar height as the workbench painter before using the startup fallback height. The overlay therefore observes the existing damage behavior without introducing a new coordinate table or a screen-specific Asset Browser branch.
