mod body;
mod content;
mod fallback;
mod template_nodes;

use self::body::draw_pane_shell_and_body;
use self::content::draw_pane_content_layers;
use super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, HostTextInputFocusData, HostViewportImageSet, PaneData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{intersect, is_visible_frame};

fn pane_intersects_damage(content: &FrameRect, paint_clip: Option<&FrameRect>) -> bool {
    is_visible_frame(content)
        && paint_clip.map_or(true, |damage| intersect(content, damage).is_some())
}

pub(in crate::ui::retained_host::host_contract) fn draw_pane(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    content: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    viewport_images: &HostViewportImageSet,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    draw_pane_for_surface(
        frame,
        pane,
        content,
        interaction,
        viewport_images,
        text_input_focus,
        None,
    );
}

pub(in crate::ui::retained_host::host_contract) fn draw_pane_for_surface(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    content: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    viewport_images: &HostViewportImageSet,
    text_input_focus: Option<&HostTextInputFocusData>,
    surface_key: Option<&str>,
) {
    if !pane_intersects_damage(content, frame.paint_clip()) {
        return;
    }
    let body = draw_pane_shell_and_body(frame, pane, content);
    draw_pane_content_layers(
        frame,
        pane,
        &body,
        content,
        interaction,
        viewport_images,
        text_input_focus,
        surface_key,
    );
}

#[cfg(test)]
#[path = "tests/pane.rs"]
mod tests;
