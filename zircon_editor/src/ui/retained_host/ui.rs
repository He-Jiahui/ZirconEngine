#[path = "ui/apply_presentation.rs"]
mod apply_presentation_impl;
#[cfg(test)]
#[path = "ui/tests/asset_browser_icon_button_painter_tests.rs"]
mod asset_browser_icon_button_painter_tests;
#[path = "ui/component_contract_metadata.rs"]
mod component_contract_metadata;
#[path = "ui/floating_pane_geometry.rs"]
mod floating_pane_geometry;
mod pane_data_conversion;
#[cfg(test)]
#[path = "ui/tests/reference_component_tests.rs"]
mod reference_component_tests;
#[cfg(test)]
#[path = "ui/tests/reference_overlay_apply_tests.rs"]
mod reference_overlay_apply_tests;
#[path = "ui/root_template_overlay.rs"]
mod root_template_overlay;
#[path = "ui/scoped_presentation.rs"]
mod scoped_presentation;
#[path = "ui/shell_content_presentation.rs"]
mod shell_content_presentation;
#[cfg(test)]
#[path = "ui/tests/structure_component_tests.rs"]
mod structure_component_tests;
#[path = "ui/template_layout_context.rs"]
mod template_layout_context;
#[path = "ui/template_node_conversion.rs"]
mod template_node_conversion;
#[cfg(test)]
mod tests;
#[path = "ui/workbench_window_projection.rs"]
mod workbench_window_projection;

pub(crate) fn capture_zui_visual_evidence(
) -> Result<pane_data_conversion::zui_visual_acceptance::CaptureSummary, String> {
    pane_data_conversion::zui_visual_acceptance::capture_catalog()
}

pub(crate) fn capture_zui_visual_evidence_with_context(
    repo_root: &std::path::Path,
    core: &zircon_runtime::core::CoreHandle,
    build_set: &zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId,
) -> Result<pane_data_conversion::zui_visual_acceptance::CaptureSummary, String> {
    pane_data_conversion::zui_visual_acceptance::capture_catalog_with_context(
        repo_root, core, build_set,
    )
}

pub(crate) fn export_zui_workbench_product_snapshots(
    repo_root: &std::path::Path,
    output_path: &std::path::Path,
) -> Result<(), String> {
    pane_data_conversion::zui_visual_acceptance::export_product_workbench_case_snapshots(
        repo_root,
        output_path,
    )
}

pub(crate) fn export_zui_workbench_product_snapshots_with_context(
    repo_root: &std::path::Path,
    output_path: &std::path::Path,
    core: &zircon_runtime::core::CoreHandle,
    build_set: &zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId,
) -> Result<(), String> {
    pane_data_conversion::zui_visual_acceptance::export_product_workbench_case_snapshots_with_context(
        repo_root,
        output_path,
        core,
        build_set,
    )
}

pub(crate) use apply_presentation_impl::{
    apply_presentation, apply_presentation_with_template_v2_data,
    apply_window_metrics_geometry_presentation, to_host_contract_scene_viewport_chrome,
};
#[cfg(test)]
pub(crate) use pane_data_conversion::refresh_runtime_diagnostics_debug_reflector_from_body_surface;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_host_contract_animation_editor_pane_from_host_pane;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_host_contract_component_showcase_pane_from_host_pane_with_runtime;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_host_contract_console_pane_from_host_pane;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_host_contract_generated_bottom_pane_from_host_pane;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_host_contract_hierarchy_pane_from_host_pane;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_host_contract_inspector_pane_from_host_pane;
#[cfg(test)]
pub(crate) use pane_data_conversion::to_host_contract_runtime_diagnostics_pane_from_host_pane;
pub(crate) use pane_data_conversion::ConsolePaneProjectionCache;
pub(crate) use pane_data_conversion::ModulePluginsPaneProjectionCache;
pub(crate) use scoped_presentation::{
    build_ui_asset_presentation_patch, patch_ui_asset_presentation,
};
pub(crate) use shell_content_presentation::{
    patch_shell_content_presentation_from_state, shell_content_target, ShellContentTarget,
};
#[cfg(test)]
pub(crate) use workbench_window_projection::to_host_contract_workbench_window_nodes;
pub(crate) use workbench_window_projection::to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale;
pub(crate) use workbench_window_projection::{
    build_host_contract_workbench_window_node_patch_at_mount_and_scale,
    patch_host_contract_workbench_window_nodes_at_mount_and_scale, WorkbenchWindowNodePatch,
};
