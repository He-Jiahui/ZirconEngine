mod window;

use self::window::{draw_floating_window, floating_window_paint_bounds};
use super::super::super::data::{
    HostPaneInteractionStateData, HostTextInputFocusData, HostViewportImageData,
    HostWindowPresentationData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::intersect;

pub(in crate::ui::retained_host::host_contract) fn draw_floating_layer(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    let windows = &presentation.host_scene_data.floating_layer.floating_windows;
    for window in windows.iter() {
        if frame.paint_clip().is_some_and(|damage| {
            intersect(&floating_window_paint_bounds(&window.frame), damage).is_none()
        }) {
            continue;
        }
        draw_floating_window(frame, window, interaction, viewport_image, text_input_focus);
    }
}
