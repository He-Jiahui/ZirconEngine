use crate::ui::retained_host::host_contract::{
    HostPanePresentationLocation, HostPanePresentationPatch,
};
use crate::ui::retained_host::{callback_dispatch, HostWindowPresentationData, PaneData};

use super::pane_frame::{
    next_viewport_toolbar_surface_frame_pane, viewport_toolbar_size_for_width,
};

pub(super) fn append_docked_viewport_toolbar_surface_frame_patches(
    patch: &mut HostPanePresentationPatch,
    presentation: &HostWindowPresentationData,
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    document_viewport_toolbar_width: Option<f32>,
) {
    for leaf in &presentation.host_scene_data.document_leaves {
        append_docked_viewport_toolbar_surface_frame_patch(
            patch,
            HostPanePresentationLocation::DocumentLeaf {
                surface_key: leaf.surface_key.clone(),
            },
            viewport_toolbar_bridge,
            leaf.surface_key.as_str(),
            leaf.content_frame.width,
            &leaf.pane,
        );
    }
    let document_dock = &presentation.host_scene_data.document_dock;
    let document_width = document_viewport_toolbar_width
        .filter(|width| *width > f32::EPSILON)
        .unwrap_or(document_dock.content_frame.width);
    append_docked_viewport_toolbar_surface_frame_patch(
        patch,
        HostPanePresentationLocation::DocumentDock,
        viewport_toolbar_bridge,
        document_dock.surface_key.as_str(),
        document_width,
        &document_dock.pane,
    );

    let left_dock = &presentation.host_scene_data.left_dock;
    append_docked_viewport_toolbar_surface_frame_patch(
        patch,
        HostPanePresentationLocation::LeftDock,
        viewport_toolbar_bridge,
        left_dock.surface_key.as_str(),
        left_dock.content_frame.width,
        &left_dock.pane,
    );

    let right_dock = &presentation.host_scene_data.right_dock;
    append_docked_viewport_toolbar_surface_frame_patch(
        patch,
        HostPanePresentationLocation::RightDock,
        viewport_toolbar_bridge,
        right_dock.surface_key.as_str(),
        right_dock.content_frame.width,
        &right_dock.pane,
    );

    let bottom_dock = &presentation.host_scene_data.bottom_dock;
    append_docked_viewport_toolbar_surface_frame_patch(
        patch,
        HostPanePresentationLocation::BottomDock,
        viewport_toolbar_bridge,
        bottom_dock.surface_key.as_str(),
        bottom_dock.content_frame.width,
        &bottom_dock.pane,
    );
}

fn append_docked_viewport_toolbar_surface_frame_patch(
    patch: &mut HostPanePresentationPatch,
    location: HostPanePresentationLocation,
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    surface_key: &str,
    width: f32,
    pane: &PaneData,
) {
    let toolbar_size =
        viewport_toolbar_size_for_width(width, viewport_toolbar_bridge.scale_factor());
    let Some(next) = next_viewport_toolbar_surface_frame_pane(
        viewport_toolbar_bridge,
        surface_key,
        toolbar_size,
        pane,
    ) else {
        return;
    };
    patch.push(location, pane, next);
}
