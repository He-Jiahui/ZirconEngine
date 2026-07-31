use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::{HostWindowPresentationData, callback_dispatch};

use super::pane_frame::{
    attach_viewport_toolbar_surface_frame_to_pane, viewport_toolbar_size_for_width,
};

pub(super) fn attach_floating_viewport_toolbar_surface_frames(
    presentation: &mut HostWindowPresentationData,
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
) {
    let mut floating_windows = Vec::new();
    for row in 0..presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .row_count()
    {
        let Some(mut window) = presentation
            .host_scene_data
            .floating_layer
            .floating_windows
            .row_data(row)
        else {
            continue;
        };
        attach_viewport_toolbar_surface_frame_to_pane(
            viewport_toolbar_bridge,
            window.window_id.to_string(),
            viewport_toolbar_size_for_width(window.frame.width - 2.0),
            &mut window.active_pane,
        );
        floating_windows.push(window);
    }
    presentation.host_scene_data.floating_layer.floating_windows = model_rc(floating_windows);
}
