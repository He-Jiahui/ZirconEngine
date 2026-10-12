//! UI 测试入口按资源、输入、布局、渲染与运行时场景划分；这里注册测试模块并提供事件类型夹具。
#[path = "accessibility.rs"]
mod accessibility;
#[path = "accessibility_disabled_gate.rs"]
mod accessibility_disabled_gate;
#[path = "accessibility_state_values.rs"]
mod accessibility_state_values;
#[path = "accessibility_text_input_actions.rs"]
mod accessibility_text_input_actions;
#[path = "accessibility_widget_actions.rs"]
mod accessibility_widget_actions;
#[cfg(feature = "accessibility-accesskit")]
#[path = "accesskit.rs"]
mod accesskit;
#[path = "asset.rs"]
mod asset;
#[path = "asset_action_policy.rs"]
mod asset_action_policy;
#[path = "asset_binding.rs"]
mod asset_binding;
#[path = "asset_compile_cache.rs"]
mod asset_compile_cache;
#[path = "asset_component_contract.rs"]
mod asset_component_contract;
#[path = "asset_component_reference_layout.rs"]
mod asset_component_reference_layout;
#[path = "asset_contract_spine.rs"]
mod asset_contract_spine;
#[path = "asset_dependency_index.rs"]
mod asset_dependency_index;
#[path = "asset_hot_reload_executor.rs"]
mod asset_hot_reload_executor;
#[path = "asset_hot_reload_plan.rs"]
mod asset_hot_reload_plan;
#[path = "asset_invalidation.rs"]
mod asset_invalidation;
#[path = "asset_localization.rs"]
mod asset_localization;
#[path = "asset_mui_web_badge_style.rs"]
mod asset_mui_web_badge_style;
#[path = "asset_mui_web_collection_style.rs"]
mod asset_mui_web_collection_style;
#[path = "asset_mui_web_form_style.rs"]
mod asset_mui_web_form_style;
#[path = "asset_mui_web_lab_style.rs"]
mod asset_mui_web_lab_style;
#[path = "asset_mui_web_layout_style.rs"]
mod asset_mui_web_layout_style;
#[path = "asset_mui_web_mui_x_style.rs"]
mod asset_mui_web_mui_x_style;
#[path = "asset_mui_web_navigation_style.rs"]
mod asset_mui_web_navigation_style;
#[path = "asset_mui_web_style.rs"]
mod asset_mui_web_style;
#[path = "asset_mui_web_surface_child_style.rs"]
mod asset_mui_web_surface_child_style;
#[path = "asset_package_validation.rs"]
mod asset_package_validation;
#[path = "asset_prototype_store.rs"]
mod asset_prototype_store;
#[path = "asset_resource_refs.rs"]
mod asset_resource_refs;
#[path = "asset_resource_resolver.rs"]
mod asset_resource_resolver;
#[path = "asset_schema_migration.rs"]
mod asset_schema_migration;
#[path = "asset_surface_index.rs"]
mod asset_surface_index;
#[path = "binding.rs"]
mod binding;
#[path = "binding_conversion_registry.rs"]
mod binding_conversion_registry;
#[path = "binding_transaction.rs"]
mod binding_transaction;
#[path = "block_box_layout.rs"]
mod block_box_layout;
#[path = "boundary.rs"]
mod boundary;
#[path = "canvas_slot_layout.rs"]
mod canvas_slot_layout;
#[path = "canvas_slot_template.rs"]
mod canvas_slot_template;
#[path = "component_catalog.rs"]
mod component_catalog;
#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "ecs_projection.rs"]
mod ecs_projection;
#[path = "event_manager.rs"]
mod event_manager;
#[path = "event_routing.rs"]
mod event_routing;
#[path = "focus_navigation.rs"]
mod focus_navigation;
#[path = "hit_grid.rs"]
mod hit_grid;
#[path = "icon_atlas.rs"]
mod icon_atlas;
#[path = "layout_slots.rs"]
mod layout_slots;
#[path = "material_button_style.rs"]
mod material_button_style;
#[path = "material_layout.rs"]
mod material_layout;
#[path = "model_context_registry.rs"]
mod model_context_registry;
#[path = "model_schema_registry.rs"]
mod model_schema_registry;
#[path = "mui_responsive_layout.rs"]
mod mui_responsive_layout;
#[path = "pipeline_report.rs"]
mod pipeline_report;
#[path = "pointer_click_semantics.rs"]
mod pointer_click_semantics;
#[path = "popup_declarative_state.rs"]
mod popup_declarative_state;
#[path = "popup_test_support.rs"]
mod popup_test_support;
#[path = "popup_tooltip_state.rs"]
mod popup_tooltip_state;
#[path = "render_atoms.rs"]
mod render_atoms;
#[path = "render_buttons.rs"]
mod render_buttons;
#[path = "render_chrome.rs"]
mod render_chrome;
#[path = "render_collection_rows.rs"]
mod render_collection_rows;
#[path = "render_command_palette.rs"]
mod render_command_palette;
#[path = "render_dialog.rs"]
mod render_dialog;
#[path = "render_divider.rs"]
mod render_divider;
#[path = "render_drag_overlay.rs"]
mod render_drag_overlay;
#[path = "render_dropdowns.rs"]
mod render_dropdowns;
#[path = "render_feedback.rs"]
mod render_feedback;
#[path = "render_notification_center.rs"]
mod render_notification_center;
#[path = "render_painter_state.rs"]
mod render_painter_state;
#[path = "render_popup_menu.rs"]
mod render_popup_menu;
#[path = "render_popup_options.rs"]
mod render_popup_options;
#[path = "render_progress.rs"]
mod render_progress;
#[path = "render_segmented_controls.rs"]
mod render_segmented_controls;
#[path = "render_selection_controls.rs"]
mod render_selection_controls;
#[path = "render_skeleton.rs"]
mod render_skeleton;
#[path = "render_sliders.rs"]
mod render_sliders;
#[path = "render_text_fields.rs"]
mod render_text_fields;
#[path = "runtime_dispatch_effect_matrix.rs"]
mod runtime_dispatch_effect_matrix;
#[path = "runtime_drag_drop_component_state.rs"]
mod runtime_drag_drop_component_state;
#[path = "runtime_input_manager.rs"]
mod runtime_input_manager;
#[path = "runtime_input_ownership.rs"]
mod runtime_input_ownership;
#[path = "runtime_input_reply_routes.rs"]
mod runtime_input_reply_routes;
#[path = "runtime_loading_component_state.rs"]
mod runtime_loading_component_state;
#[path = "runtime_ui_asset_surface_index.rs"]
mod runtime_ui_asset_surface_index;
#[path = "runtime_ui_layout_routes.rs"]
mod runtime_ui_layout_routes;
#[path = "runtime_ui_window_event_routes/mod.rs"]
mod runtime_ui_window_event_routes;
#[path = "runtime_window_input_pump.rs"]
mod runtime_window_input_pump;
#[path = "scroll_virtualization.rs"]
mod scroll_virtualization;
#[path = "shared_core.rs"]
mod shared_core;
#[path = "style_mapping.rs"]
mod style_mapping;
#[path = "surface_dirty_domains.rs"]
mod surface_dirty_domains;
#[path = "surface_dirty_mui.rs"]
mod surface_dirty_mui;
#[path = "surface_frame_authority.rs"]
mod surface_frame_authority;
#[path = "surface_invalidation.rs"]
mod surface_invalidation;
#[path = "surface_node_pool.rs"]
mod surface_node_pool;
#[path = "surface_slot_dirty_domains.rs"]
mod surface_slot_dirty_domains;
#[path = "taffy_bridge.rs"]
mod taffy_bridge;
#[path = "taffy_layout_diagnostics.rs"]
mod taffy_layout_diagnostics;
#[path = "taffy_layout_pass.rs"]
mod taffy_layout_pass;
#[path = "taffy_visual_verification.rs"]
mod taffy_visual_verification;
#[path = "template.rs"]
mod template;
#[path = "template_grid_flow.rs"]
mod template_grid_flow;
#[path = "template_pipeline.rs"]
mod template_pipeline;
#[path = "template_tree_builder.rs"]
mod template_tree_builder;
#[path = "text_hit_testing.rs"]
mod text_hit_testing;
#[path = "text_layout/mod.rs"]
mod text_layout;
#[path = "text_pipeline/mod.rs"]
mod text_pipeline;
#[path = "text_shaper.rs"]
mod text_shaper;
#[path = "theme_registry.rs"]
mod theme_registry;
#[path = "timeline.rs"]
mod timeline;
#[path = "v2_asset.rs"]
mod v2_asset;
#[path = "widget_menu_behavior.rs"]
mod widget_menu_behavior;
#[path = "widget_menu_popup_activation.rs"]
mod widget_menu_popup_activation;
#[path = "widget_radio_behavior.rs"]
mod widget_radio_behavior;
#[path = "widget_range_navigation.rs"]
mod widget_range_navigation;
#[path = "widget_scrollbar_behavior.rs"]
mod widget_scrollbar_behavior;
#[path = "widget_text_input_ime_context.rs"]
mod widget_text_input_ime_context;
#[path = "widget_text_input_keyboard.rs"]
mod widget_text_input_keyboard;
#[path = "widget_text_input_keyboard_clipboard.rs"]
mod widget_text_input_keyboard_clipboard;
#[path = "widget_text_input_keyboard_hard_line.rs"]
mod widget_text_input_keyboard_hard_line;
#[path = "widget_text_input_keyboard_text.rs"]
mod widget_text_input_keyboard_text;
#[path = "widget_text_input_mui.rs"]
mod widget_text_input_mui;
#[path = "widget_text_input_pointer.rs"]
mod widget_text_input_pointer;

fn typed_component_event_kind_for_test(
    binding_id: &str,
) -> Option<zircon_runtime_interface::ui::component::UiComponentEventKind> {
    use zircon_runtime_interface::ui::component::UiComponentEventKind;

    let token = binding_id
        .rsplit(|character| character == '/' || character == '.')
        .next()?;
    UiComponentEventKind::from_schema_name(token)
}
