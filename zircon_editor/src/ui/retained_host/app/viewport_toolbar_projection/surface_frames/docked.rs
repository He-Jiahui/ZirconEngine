use crate::ui::retained_host::{HostWindowPresentationData, PaneData, callback_dispatch};

use super::pane_frame::{
    attach_viewport_toolbar_surface_frame_to_pane, viewport_toolbar_size_for_width,
};

pub(super) fn attach_docked_viewport_toolbar_surface_frames(
    presentation: &mut HostWindowPresentationData,
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    document_viewport_toolbar_width: Option<f32>,
) {
    let document_surface_key = presentation
        .host_scene_data
        .document_dock
        .surface_key
        .to_string();
    let document_width = document_viewport_toolbar_width
        .filter(|width| *width > f32::EPSILON)
        .unwrap_or(
            presentation
                .host_scene_data
                .document_dock
                .content_frame
                .width,
        );
    attach_docked_viewport_toolbar_surface_frame(
        viewport_toolbar_bridge,
        document_surface_key,
        document_width,
        &mut presentation.host_scene_data.document_dock.pane,
    );

    let left_surface_key = presentation
        .host_scene_data
        .left_dock
        .surface_key
        .to_string();
    let left_width = presentation.host_scene_data.left_dock.content_frame.width;
    attach_docked_viewport_toolbar_surface_frame(
        viewport_toolbar_bridge,
        left_surface_key,
        left_width,
        &mut presentation.host_scene_data.left_dock.pane,
    );

    let right_surface_key = presentation
        .host_scene_data
        .right_dock
        .surface_key
        .to_string();
    let right_width = presentation.host_scene_data.right_dock.content_frame.width;
    attach_docked_viewport_toolbar_surface_frame(
        viewport_toolbar_bridge,
        right_surface_key,
        right_width,
        &mut presentation.host_scene_data.right_dock.pane,
    );

    let bottom_surface_key = presentation
        .host_scene_data
        .bottom_dock
        .surface_key
        .to_string();
    let bottom_width = presentation.host_scene_data.bottom_dock.content_frame.width;
    attach_docked_viewport_toolbar_surface_frame(
        viewport_toolbar_bridge,
        bottom_surface_key,
        bottom_width,
        &mut presentation.host_scene_data.bottom_dock.pane,
    );
}

fn attach_docked_viewport_toolbar_surface_frame(
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    surface_key: String,
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
