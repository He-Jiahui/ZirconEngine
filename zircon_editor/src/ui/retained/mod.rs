#[path = "activity_rail_pointer/mod.rs"]
pub(crate) mod activity_rail_pointer;
#[path = "app.rs"]
mod app;
#[path = "asset_control_ids.rs"]
mod asset_control_ids;
#[path = "asset_pointer.rs"]
pub(crate) mod asset_pointer;
#[path = "dispatch/mod.rs"]
pub(crate) mod callback_dispatch;
#[path = "console_output.rs"]
mod console_output;
#[path = "detail_pointer/mod.rs"]
pub(crate) mod detail_pointer;
#[path = "document_tab_pointer/mod.rs"]
pub(crate) mod document_tab_pointer;
#[path = "drawer_header_pointer/mod.rs"]
pub(crate) mod drawer_header_pointer;
#[path = "drawer_resize.rs"]
pub(crate) mod drawer_resize;
#[path = "event_bridge.rs"]
pub(crate) mod event_bridge;
#[path = "floating_window_project.rs"]
pub(crate) mod floating_window_projection;
#[path = "hierarchy_pointer/mod.rs"]
pub(crate) mod hierarchy_pointer;
#[path = "contract/mod.rs"]
mod host_contract;
#[path = "host_page_pointer/mod.rs"]
pub(crate) mod host_page_pointer;
#[path = "input_policy.rs"]
mod input_policy;
#[path = "menu_pointer/mod.rs"]
pub(crate) mod menu_pointer;
#[path = "menu_popup_contract.rs"]
pub(crate) mod menu_popup_contract;
#[path = "option_spec.rs"]
mod option_spec;
#[path = "popup_anchor_metrics.rs"]
pub(crate) mod popup_anchor_metrics;
#[path = "primitives.rs"]
pub(crate) mod primitives;
#[path = "route_intent/mod.rs"]
pub(crate) mod route_intent;
#[path = "run_config.rs"]
mod run_config;
#[path = "scroll_surface_host.rs"]
pub(crate) mod scroll_surface_host;
#[path = "shell_pointer.rs"]
pub(crate) mod shell_pointer;
#[path = "tab_drag.rs"]
pub mod tab_drag;
#[path = "ui.rs"]
mod ui;
#[path = "ui_perf.rs"]
pub(crate) mod ui_perf;
#[path = "viewport/mod.rs"]
mod viewport;
#[path = "viewport_toolbar_pointer/mod.rs"]
pub(crate) mod viewport_toolbar_pointer;
#[path = "welcome_recent_geometry.rs"]
pub(crate) mod welcome_recent_geometry;
#[path = "welcome_recent_pointer/mod.rs"]
pub(crate) mod welcome_recent_pointer;
#[path = "popup_actions.rs"]
pub(crate) mod workbench_popup_actions;
#[path = "preview_actions.rs"]
pub(crate) mod workbench_preview_actions;
#[path = "zui_visual_evidence/mod.rs"]
mod zui_visual_evidence;

pub(crate) use app::build_startup_state;
pub(crate) use app::HostInvalidationMask;
pub(crate) use event_bridge::HostShellContentScope;
pub(crate) use host_contract::*;

#[cfg(test)]
pub(crate) use app::backend_refresh::{plan_asset_backend_refresh, AssetBackendRefreshPlan};
#[cfg(test)]
pub(crate) use app::{
    collect_native_floating_window_targets, configure_native_floating_window_presentation,
    NativeFloatingWindowTarget, NativeWindowPresenterStore,
};
pub use app::{
    run_editor, run_editor_with_config, run_editor_with_startup_request,
    run_retained_host_automation, RetainedHostAutomationResult,
};

pub use zui_visual_evidence::{
    export_zui_visual_evidence, export_zui_visual_evidence_with_context,
    export_zui_workbench_product_snapshots, export_zui_workbench_product_snapshots_with_context,
    ZuiVisualEvidenceSummary,
};
#[cfg(test)]
pub(crate) fn console_content_extent(text: &str) -> f32 {
    console_output::console_content_extent(text)
}
pub use run_config::EditorHostRunConfig;
#[cfg(test)]
pub(crate) use ui::apply_presentation;
#[cfg(test)]
pub(crate) use ui::refresh_runtime_diagnostics_debug_reflector_from_body_surface;
#[cfg(test)]
pub(crate) use ui::to_host_contract_animation_editor_pane_from_host_pane;
#[cfg(test)]
pub(crate) use ui::to_host_contract_component_showcase_pane_from_host_pane_with_runtime;
#[cfg(test)]
pub(crate) use ui::to_host_contract_console_pane_from_host_pane;
#[cfg(test)]
pub(crate) use ui::to_host_contract_generated_bottom_pane_from_host_pane;
#[cfg(test)]
pub(crate) use ui::to_host_contract_hierarchy_pane_from_host_pane;
#[cfg(test)]
pub(crate) use ui::to_host_contract_inspector_pane_from_host_pane;
#[cfg(test)]
pub(crate) use ui::to_host_contract_runtime_diagnostics_pane_from_host_pane;
#[cfg(test)]
pub(crate) use ui::to_host_contract_workbench_window_nodes;
