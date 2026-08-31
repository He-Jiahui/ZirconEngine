mod window;

use self::window::{draw_floating_window, floating_window_paint_bounds};
use super::super::super::data::{
    paint_pane_interaction_state, paint_text_input_focus, paint_viewport_images,
    HostWindowPresentationData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::intersect;

pub(in crate::ui::retained_host::host_contract) fn draw_floating_layer(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let windows = &presentation.host_scene_data.floating_layer.floating_windows;
    let mut paint_state = None;
    for window in windows.iter() {
        if frame.paint_clip().is_some_and(|damage| {
            intersect(&floating_window_paint_bounds(&window.frame), damage).is_none()
        }) {
            continue;
        }
        let (interaction, viewport_images, text_input_focus) =
            paint_state.get_or_insert_with(|| {
                (
                    paint_pane_interaction_state(presentation),
                    paint_viewport_images(presentation),
                    paint_text_input_focus(presentation),
                )
            });
        draw_floating_window(
            frame,
            window,
            interaction.as_ref(),
            viewport_images,
            Some(text_input_focus.as_ref()),
        );
    }
}
