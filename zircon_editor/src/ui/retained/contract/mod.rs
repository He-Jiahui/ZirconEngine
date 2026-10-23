#[cfg(test)]
#[path = "tests/asset_deletion_blocker.rs"]
mod asset_deletion_blocker_tests;
#[path = "chrome_command_stream/mod.rs"]
pub(crate) mod chrome_command_stream;
#[path = "componentized_workbench_regions.rs"]
pub(in crate::ui::retained_host) mod componentized_workbench_regions;
#[path = "data/mod.rs"]
pub(crate) mod data;
#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "frame_geometry.rs"]
mod frame_geometry;
#[path = "globals.rs"]
pub(crate) mod globals;
#[path = "host_dock_overflow_menu.rs"]
mod host_dock_overflow_menu;
#[path = "host_page_overflow_menu.rs"]
mod host_page_overflow_menu;
#[path = "menu_popup_metrics.rs"]
mod menu_popup_metrics;
#[path = "native_keyboard.rs"]
mod native_keyboard;
#[path = "pointer.rs"]
mod native_pointer;
#[path = "native_popup_dismiss.rs"]
mod native_popup_dismiss;
#[path = "normal_capture_redraw_trace.rs"]
pub(in crate::ui::retained_host) mod normal_capture_redraw_trace;
#[path = "paint_asset_deletion_blocker.rs"]
mod paint_asset_deletion_blocker;
#[path = "paint_close_prompt.rs"]
mod paint_close_prompt;
#[path = "paint_color.rs"]
mod paint_color;
#[path = "paint_debug_reflector_overlay.rs"]
mod paint_debug_reflector_overlay;
#[path = "paint_diagnostics.rs"]
mod paint_diagnostics;
#[path = "paint_frame.rs"]
mod paint_frame;
#[path = "paint_geometry.rs"]
mod paint_geometry;
#[path = "paint_primitives.rs"]
mod paint_primitives;
#[path = "paint_recording.rs"]
mod paint_recording;
#[path = "paint/mod.rs"]
mod paint_template_nodes;
pub(in crate::ui::retained_host) use paint_template_nodes::{
    bind_visual_asset_loader, clear_svg_tree_cache, clear_visual_asset_pixels_cache,
    invalidate_editor_sprite_atlas_cache, invalidate_svg_tree_paths,
    invalidate_visual_asset_pixel_paths, reconcile_svg_tree_sources,
    reconcile_visual_asset_pixel_sources, take_visual_asset_completion, unbind_visual_asset_loader,
    VisualAssetLoadCompletion,
};
#[path = "paint_text.rs"]
mod paint_text;
#[path = "paint_theme.rs"]
pub(crate) mod paint_theme;
#[path = "paint_workbench.rs"]
mod paint_workbench;
pub(in crate::ui::retained_host) use paint_workbench::transition_panel_frame;
#[path = "workbench.rs"]
mod paint_workbench_renderer;
#[path = "presenter/mod.rs"]
mod presenter;
#[path = "profiling_artifacts.rs"]
mod profiling_artifacts;
#[path = "profiling_hit_routes.rs"]
mod profiling_hit_routes;
#[path = "redraw.rs"]
mod redraw;
#[path = "search_field_clear_action.rs"]
mod search_field_clear_action;
#[path = "settings_color_editor_geometry.rs"]
mod settings_color_editor_geometry;
#[path = "settings_window_geometry.rs"]
mod settings_window_geometry;
#[path = "surface_hit_test/mod.rs"]
mod surface_hit_test;
#[path = "activation_semantics.rs"]
mod template_activation_semantics;
#[path = "component_family.rs"]
mod template_component_family;
#[path = "geometry.rs"]
mod template_geometry;
#[path = "input_semantics.rs"]
mod template_input_semantics;
#[path = "popup_layout.rs"]
mod template_popup_layout;
#[path = "viewport_chrome_geometry.rs"]
mod viewport_chrome_geometry;
#[path = "viewport_chrome_state.rs"]
mod viewport_chrome_state;
#[path = "window.rs"]
mod window;
#[path = "context_menu.rs"]
mod workbench_context_menu;

pub(crate) use data::*;
pub(crate) use diagnostics::{
    HostInvalidationDiagnostics, HostWindowDiagnostic, HostWindowDiagnosticSeverity,
    STARTUP_REFRESH_DIAGNOSTICS_OVERLAY,
};
pub(crate) use globals::{HostAssetSurfaceInteractionState, PaneSurfaceHostContext, UiHostContext};
pub(crate) use menu_popup_metrics::menu_popup_text_width;
pub(crate) use native_pointer::{HostChromeTooltipTarget, WorkbenchTooltipPointerTarget};
pub(crate) use paint_text::TextPaintEvidenceScope;
pub(crate) use paint_text::{
    measure_runtime_text_width, measure_runtime_text_width_with_style,
    runtime_text_metrics_generation,
};
pub(in crate::ui::retained_host) use paint_theme::METRICS;
pub(crate) use paint_theme::{
    apply_host_appearance_from_tokens, apply_host_metrics_from_tokens,
    apply_host_paint_scale_factor, apply_host_palette_from_tokens, apply_host_text_preferences,
    current_host_paint_scale_factor, project_host_text_preferences,
};
pub(crate) use paint_theme::{current_host_metrics, HostControlMetrics};
#[cfg(test)]
pub(crate) use paint_theme::{
    enter_host_paint_theme_scope, host_paint_theme_snapshot_at_scale_for_test,
};
pub(crate) use presenter::{
    runtime_factory_error, HostPresenterResult, RuntimeUiSurfacePresenterFactory,
};
pub(crate) use search_field_clear_action::{
    search_field_clear_action_frame, search_field_clear_action_hit_test,
};
#[cfg(test)]
pub(crate) fn paint_host_frame_for_test(
    width: u32,
    height: u32,
    presentation: &HostWindowPresentationData,
) -> Vec<u8> {
    paint_workbench::paint_host_frame(width, height, presentation).into_bytes()
}
pub(crate) use paint_template_nodes::paint_template_nodes_for_evidence_with_background;
pub(crate) use paint_template_nodes::PaintEvidenceScope;
#[cfg(test)]
pub(crate) use paint_template_nodes::{
    paint_runtime_render_commands_for_test, paint_template_nodes_for_test,
    paint_template_nodes_for_test_with_background, template_node_command_summary_for_test,
};
pub(crate) use paint_workbench::paint_product_presentation_for_evidence;
#[cfg(test)]
pub(crate) use paint_workbench_renderer::paint_componentized_extension_workspace_for_test;
#[cfg(test)]
pub(crate) use paint_workbench_renderer::paint_scrollbar_component_for_test;
pub(crate) use surface_hit_test::{
    build_pane_template_surface_frame, rebuild_pane_template_hit_artifacts,
};
#[cfg(test)]
pub(in crate::ui::retained_host) use surface_hit_test::{
    hit_test_workbench_window_template_node_with_index, HostWorkbenchHitIndex,
};
#[cfg(test)]
pub(in crate::ui::retained_host) use template_popup_layout::menu_item_row_frame;
pub(crate) use window::{primary_host_window_id, HierarchyPointerSource, UiHostWindow};
