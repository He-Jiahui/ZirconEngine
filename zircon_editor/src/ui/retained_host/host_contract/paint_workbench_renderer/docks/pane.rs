mod body;
mod content;
mod fallback;
mod template_nodes;

use self::body::draw_pane_shell_and_body;
use self::content::draw_pane_content_layers;
use super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, HostTextInputFocusData, HostViewportImageData,
    PaneData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::is_visible_frame;

pub(in crate::ui::retained_host::host_contract) fn draw_pane(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    content: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    if !is_visible_frame(content) {
        return;
    }
    let body = draw_pane_shell_and_body(frame, pane, content);
    draw_pane_content_layers(
        frame,
        pane,
        &body,
        content,
        interaction,
        viewport_image,
        text_input_focus,
    );
}
