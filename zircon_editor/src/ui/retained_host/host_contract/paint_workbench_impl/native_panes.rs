mod assets;
mod content;
mod diagnostics;
mod hierarchy;
mod viewport;

use super::super::data::{
    FrameRect, HostPaneInteractionStateData, HostViewportImageData, PaneData,
};
use super::super::paint_frame::HostRgbaFrame;

pub(super) fn draw_pane_debug_overlay(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
) -> bool {
    diagnostics::draw_pane_debug_overlay(frame, pane, body, clip)
}

pub(super) fn draw_viewport_image(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    viewport_image: Option<&HostViewportImageData>,
) -> bool {
    viewport::draw_viewport_image(frame, pane, body, clip, viewport_image)
}

pub(super) fn draw_native_pane_content(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    content::draw_native_pane_content(frame, pane, body, clip, interaction)
}
