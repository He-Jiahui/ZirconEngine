mod window;

use self::window::draw_floating_window;
use super::super::super::data::{
    HostPaneInteractionStateData, HostTextInputFocusData, HostViewportImageData,
    HostWindowPresentationData,
};
use super::super::super::paint_frame::HostRgbaFrame;

pub(in crate::ui::retained_host::host_contract) fn draw_floating_layer(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    let windows = &presentation.host_scene_data.floating_layer.floating_windows;
    for row in 0..windows.row_count() {
        let Some(window) = windows.row_data(row) else {
            continue;
        };
        draw_floating_window(
            frame,
            &window,
            interaction,
            viewport_image,
            text_input_focus,
        );
    }
}
