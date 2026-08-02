mod chrome_command_stream;
mod data;
mod diagnostics;
mod frame_geometry;
mod globals;
mod host_page_overflow_menu;
mod menu_popup_metrics;
mod native_keyboard;
mod native_pointer;
mod native_popup_dismiss;
mod paint_close_prompt;
mod paint_debug_reflector_overlay;
mod paint_diagnostics;
mod paint_frame;
mod paint_geometry;
mod paint_primitives;
mod paint_recording;
mod paint_template_nodes;
pub(in crate::ui::retained_host) use paint_template_nodes::{
    clear_visual_asset_pixels_cache, invalidate_editor_sprite_atlas_cache,
};
mod paint_text;
mod paint_theme;
mod paint_workbench;
mod paint_workbench_renderer;
mod presenter;
mod profiling_artifacts;
mod profiling_hit_routes;
mod redraw;
mod surface_hit_test;
mod template_activation_semantics;
mod template_component_family;
mod template_geometry;
mod template_input_semantics;
mod template_popup_layout;
mod window;
mod workbench_context_menu;

pub(crate) use data::*;
pub(crate) use diagnostics::{HostInvalidationDiagnostics, STARTUP_REFRESH_DIAGNOSTICS_OVERLAY};
pub(crate) use globals::{PaneSurfaceHostContext, UiHostContext};
pub(crate) use menu_popup_metrics::menu_popup_text_width;
pub(crate) use paint_text::measure_runtime_text_width;
pub(crate) use paint_theme::{
    HostControlMetrics, METRICS, apply_host_appearance_from_tokens, apply_host_metrics_from_tokens,
    apply_host_palette_from_tokens, apply_host_text_preferences, current_host_metrics,
    project_host_text_preferences,
};
#[cfg(test)]
pub(crate) fn paint_host_frame_for_test(
    width: u32,
    height: u32,
    presentation: &HostWindowPresentationData,
) -> Vec<u8> {
    paint_workbench::paint_host_frame(width, height, presentation).into_bytes()
}
#[cfg(test)]
pub(crate) use paint_template_nodes::{
    paint_runtime_render_commands_for_test, paint_template_nodes_for_test,
    paint_template_nodes_for_test_with_background,
};
#[cfg(test)]
pub(crate) use paint_workbench_renderer::paint_componentized_extension_workspace_for_test;
#[cfg(test)]
pub(crate) use paint_workbench_renderer::paint_scrollbar_component_for_test;
pub(crate) use surface_hit_test::build_pane_template_surface_frame;
pub(crate) use window::UiHostWindow;
