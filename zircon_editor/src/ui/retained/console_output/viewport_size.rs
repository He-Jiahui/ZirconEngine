use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::retained_host::host_contract::data::{
    FloatingWindowData, HostWindowPresentationData, PaneData,
};
use crate::ui::retained_host::primitives::ModelRc;

use super::ConsoleOutputPaintMetadata;

pub(in crate::ui::retained_host) fn console_output_viewport_size(
    presentation: &HostWindowPresentationData,
    source_window_id: Option<&str>,
) -> Option<UiSize> {
    let scene = &presentation.host_scene_data;
    if let Some(source_window_id) = source_window_id {
        if let Some(size) = floating_console_viewport_size(
            &scene.floating_layer.floating_windows,
            Some(source_window_id),
        )
        .or_else(|| {
            floating_console_viewport_size(
                &presentation.native_floating_surface_data.floating_windows,
                Some(source_window_id),
            )
        }) {
            return Some(size);
        }
    }

    [
        &scene.document_dock.pane,
        &scene.left_dock.pane,
        &scene.right_dock.pane,
        &scene.bottom_dock.pane,
    ]
    .into_iter()
    .find_map(console_pane_viewport_size)
    .or_else(|| floating_console_viewport_size(&scene.floating_layer.floating_windows, None))
    .or_else(|| {
        floating_console_viewport_size(
            &presentation.native_floating_surface_data.floating_windows,
            None,
        )
    })
}

fn floating_console_viewport_size(
    windows: &ModelRc<FloatingWindowData>,
    window_id: Option<&str>,
) -> Option<UiSize> {
    windows
        .iter()
        .filter(|window| window_id.is_none_or(|window_id| window.window_id == window_id))
        .find_map(|window| console_pane_viewport_size(&window.active_pane))
}

fn console_pane_viewport_size(pane: &PaneData) -> Option<UiSize> {
    if pane.kind != "Console" {
        return None;
    }
    let viewport = pane
        .console
        .nodes
        .metadata::<ConsoleOutputPaintMetadata>()?
        .viewport();
    (viewport.width > 0.0 && viewport.height > 0.0)
        .then_some(UiSize::new(viewport.width, viewport.height))
}
