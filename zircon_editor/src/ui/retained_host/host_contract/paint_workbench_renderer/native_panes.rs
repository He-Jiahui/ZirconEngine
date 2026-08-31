mod assets;
mod content;
mod diagnostics;
mod hierarchy;
mod scrollbar;
mod viewport;

use super::super::data::{
    FrameRect, HostPaneInteractionStateData, HostTextInputFocusData, HostViewportImageSet, PaneData,
};
use super::super::paint_frame::HostRgbaFrame;

pub(super) use scrollbar::draw_vertical_scrollbar;
#[cfg(test)]
pub(crate) use scrollbar::paint_scrollbar_component_for_test;

pub(in crate::ui::retained_host::host_contract) fn draw_pane_debug_overlay(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
) -> bool {
    diagnostics::draw_pane_debug_overlay(frame, pane, body, clip)
}

pub(in crate::ui::retained_host::host_contract) fn draw_viewport_image(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    viewport_images: &HostViewportImageSet,
) -> bool {
    viewport::draw_viewport_image(frame, pane, body, clip, viewport_images)
}

pub(in crate::ui::retained_host::host_contract) fn draw_native_pane_content(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    content::draw_native_pane_content(frame, pane, body, clip, interaction, text_input_focus)
}
