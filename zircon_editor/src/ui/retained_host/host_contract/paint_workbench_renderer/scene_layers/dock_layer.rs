use super::super::super::data::{
    paint_pane_interaction_state, paint_text_input_focus, paint_viewport_image,
    HostWindowPresentationData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::docks;

pub(super) fn draw_dock_layers(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let scene = &presentation.host_scene_data;
    let viewport_image = paint_viewport_image(presentation);
    let interaction = paint_pane_interaction_state(presentation);
    let text_input_focus = paint_text_input_focus(presentation);
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_left_dock");
        docks::draw_side_dock(
            frame,
            &scene.left_dock,
            &interaction,
            viewport_image.as_deref(),
            Some(&text_input_focus),
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_document_dock");
        docks::draw_document_dock(
            frame,
            &scene.document_dock,
            &interaction,
            viewport_image.as_deref(),
            Some(&text_input_focus),
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_right_dock");
        docks::draw_side_dock(
            frame,
            &scene.right_dock,
            &interaction,
            viewport_image.as_deref(),
            Some(&text_input_focus),
        );
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_bottom_dock");
        docks::draw_bottom_dock(
            frame,
            &scene.bottom_dock,
            &interaction,
            viewport_image.as_deref(),
            Some(&text_input_focus),
        );
    }
}

pub(super) fn draw_floating_layer(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    zircon_runtime::profile_scope!("editor", "host_painter", "painter_floating_layer");
    let interaction = paint_pane_interaction_state(presentation);
    let viewport_image = paint_viewport_image(presentation);
    let text_input_focus = paint_text_input_focus(presentation);
    docks::draw_floating_layer(
        frame,
        presentation,
        &interaction,
        viewport_image.as_deref(),
        Some(&text_input_focus),
    );
}
