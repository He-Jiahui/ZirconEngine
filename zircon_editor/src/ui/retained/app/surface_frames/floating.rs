use crate::ui::retained_host::host_contract::{
    HostPanePresentationLocation, HostPanePresentationPatch,
};
use crate::ui::retained_host::{callback_dispatch, HostWindowPresentationData};

use super::pane_frame::{
    next_viewport_toolbar_surface_frame_pane, viewport_toolbar_size_for_width,
};

pub(super) fn append_floating_viewport_toolbar_surface_frame_patches(
    patch: &mut HostPanePresentationPatch,
    presentation: &HostWindowPresentationData,
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
) {
    for (row, window) in presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .iter()
        .enumerate()
    {
        let toolbar_size = viewport_toolbar_size_for_width(
            window.frame.width - 2.0,
            viewport_toolbar_bridge.scale_factor(),
        );
        let Some(next) = next_viewport_toolbar_surface_frame_pane(
            viewport_toolbar_bridge,
            window.window_id.as_str(),
            toolbar_size,
            &window.active_pane,
        ) else {
            continue;
        };
        patch.push(
            HostPanePresentationLocation::Floating {
                row,
                window_id: window.window_id.clone(),
            },
            &window.active_pane,
            next,
        );
    }
}
