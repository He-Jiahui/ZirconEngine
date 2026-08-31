use crate::ui::retained_host::{callback_dispatch, HostWindowPresentationData, PaneData};

use super::pane_frame::{
    attach_viewport_toolbar_surface_frame_to_pane, viewport_toolbar_size_for_width,
};

pub(super) fn attach_docked_viewport_toolbar_surface_frames(
    presentation: &mut HostWindowPresentationData,
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    document_viewport_toolbar_width: Option<f32>,
) {
    let document_dock = &mut presentation.host_scene_data.document_dock;
    let document_width = document_viewport_toolbar_width
        .filter(|width| *width > f32::EPSILON)
        .unwrap_or(document_dock.content_frame.width);
    attach_docked_viewport_toolbar_surface_frame(
        viewport_toolbar_bridge,
        document_dock.surface_key.as_str(),
        document_width,
        &mut document_dock.pane,
    );

    let left_dock = &mut presentation.host_scene_data.left_dock;
    attach_docked_viewport_toolbar_surface_frame(
        viewport_toolbar_bridge,
        left_dock.surface_key.as_str(),
        left_dock.content_frame.width,
        &mut left_dock.pane,
    );

    let right_dock = &mut presentation.host_scene_data.right_dock;
    attach_docked_viewport_toolbar_surface_frame(
        viewport_toolbar_bridge,
        right_dock.surface_key.as_str(),
        right_dock.content_frame.width,
        &mut right_dock.pane,
    );

    let bottom_dock = &mut presentation.host_scene_data.bottom_dock;
    attach_docked_viewport_toolbar_surface_frame(
        viewport_toolbar_bridge,
        bottom_dock.surface_key.as_str(),
        bottom_dock.content_frame.width,
        &mut bottom_dock.pane,
    );
}

fn attach_docked_viewport_toolbar_surface_frame(
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    surface_key: &str,
    width: f32,
    pane: &mut PaneData,
) {
    attach_viewport_toolbar_surface_frame_to_pane(
        viewport_toolbar_bridge,
        surface_key,
        viewport_toolbar_size_for_width(width),
        pane,
    );
}
