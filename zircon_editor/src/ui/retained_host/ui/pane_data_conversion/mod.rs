mod animation_projection;
mod build_export;
mod build_export_wizard_panel;
mod component_showcase_projection;
mod console_projection;
mod generated_bottom_projection;
mod hierarchy_projection;
mod inspector_fields;
mod inspector_projection;
mod model_projection;
mod module_plugins;
mod native_template_node_panes;
mod pane_component_projection;
mod pane_menu_projection;
mod pane_option_projection;
mod pane_ui_asset_conversion;
#[cfg(test)]
mod pane_ui_asset_conversion_tests;
mod pane_value_conversion;
mod performance_timeline;
mod runtime_diagnostics;
mod template_node_projection;
mod template_runtime_projection;
mod ui_asset_detail_fields;

pub(crate) use self::animation_projection::{
    to_host_contract_animation_editor_pane_from_host_pane,
    to_host_contract_animation_editor_pane_from_host_pane_with_runtime,
};
pub(crate) use self::build_export::to_host_contract_build_export_pane_from_host_pane;
pub(crate) use self::component_showcase_projection::{
    to_host_contract_component_showcase_pane_from_host_pane,
    to_host_contract_component_showcase_pane_from_host_pane_with_runtime,
};
pub(crate) use self::console_projection::{
    to_host_contract_console_pane_from_host_pane,
    to_host_contract_console_pane_from_host_pane_with_runtime,
};
pub(crate) use self::generated_bottom_projection::to_host_contract_generated_bottom_pane_from_host_pane;
pub(crate) use self::hierarchy_projection::{
    to_host_contract_hierarchy_pane_from_host_pane,
    to_host_contract_hierarchy_pane_from_host_pane_with_runtime,
};
pub(crate) use self::inspector_projection::{
    to_host_contract_inspector_pane_from_host_pane,
    to_host_contract_inspector_pane_from_host_pane_with_runtime,
};
pub(crate) use self::module_plugins::to_host_contract_module_plugins_pane_from_host_pane;
pub(super) use self::native_template_node_panes::{
    to_host_contract_asset_browser_pane, to_host_contract_assets_activity_pane,
    to_host_contract_project_overview_pane,
};
pub(super) use self::pane_component_projection::{
    NotificationCenterMetadata, projected_command_palette_options,
    projected_command_palette_structured_options, projected_notification_center_metadata,
    projected_notification_center_metadata_from_host, projected_notification_center_option_rows,
    projected_notification_center_value_text, projected_sample_grid_data,
    projected_timeline_strip_data, projected_weight_heatmap_data,
};
pub(super) use self::pane_menu_projection::structured_menu_items;
pub(super) use self::pane_option_projection::structured_options_for_node;
pub(super) use self::pane_ui_asset_conversion::to_host_contract_ui_asset_pane;
pub(crate) use self::performance_timeline::to_host_contract_performance_timeline_pane_from_host_pane;
pub(crate) use self::runtime_diagnostics::{
    refresh_runtime_diagnostics_debug_reflector_from_body_surface,
    to_host_contract_runtime_diagnostics_pane_from_host_pane,
};
pub(crate) use self::template_runtime_projection::to_host_contract_template_v2_pane_from_host_pane_with_runtime;
use self::template_runtime_projection::{
    builtin_host_runtime, pane_template_runtime, project_pane_template_nodes,
    project_pane_template_nodes_with_runtime,
};

#[cfg(test)]
mod inspector_pane_tests;
