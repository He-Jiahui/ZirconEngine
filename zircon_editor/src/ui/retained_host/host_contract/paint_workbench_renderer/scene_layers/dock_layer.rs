use super::super::super::data::HostWindowPresentationData;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::docks;

pub(super) fn draw_dock_layers(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let scene = &presentation.host_scene_data;
    let viewport_image = presentation.viewport_image.as_ref();
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_left_dock");
        docks::draw_side_dock(
            frame,
            &scene.left_dock,
            &presentation.pane_interaction_state,
            viewport_image,
            Some(&presentation.text_input_focus),
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_document_dock");
        docks::draw_document_dock(
            frame,
            &scene.document_dock,
            &presentation.pane_interaction_state,
            viewport_image,
            Some(&presentation.text_input_focus),
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_right_dock");
        docks::draw_side_dock(
            frame,
            &scene.right_dock,
            &presentation.pane_interaction_state,
            viewport_image,
            Some(&presentation.text_input_focus),
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_bottom_dock");
        docks::draw_bottom_dock(
            frame,
            &scene.bottom_dock,
            &presentation.pane_interaction_state,
            viewport_image,
            Some(&presentation.text_input_focus),
        );
    }
}

pub(super) fn draw_floating_layer(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    zircon_runtime::profile_scope!("editor", "host_painter", "painter_floating_layer");
    docks::draw_floating_layer(
        frame,
        presentation,
        &presentation.pane_interaction_state,
        presentation.viewport_image.as_ref(),
        Some(&presentation.text_input_focus),
    );
}
