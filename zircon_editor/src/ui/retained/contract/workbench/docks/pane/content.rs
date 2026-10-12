use super::super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, HostTextInputFocusData, HostViewportImageSet, PaneData,
};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::native_panes;

use super::fallback::draw_pane_fallback;
use super::template_nodes::draw_pane_template_nodes;

pub(super) fn draw_pane_content_layers(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    interaction: &HostPaneInteractionStateData,
    viewport_images: &HostViewportImageSet,
    text_input_focus: Option<&HostTextInputFocusData>,
    surface_key: Option<&str>,
) {
    let has_viewport_content = {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_viewport_image");
        native_panes::draw_viewport_image_for_surface(
            frame,
            pane,
            body,
            clip,
            viewport_images,
            surface_key,
        )
    };
    let native_before_template = native_content_precedes_template_nodes(pane.kind.as_str());
    let has_native_content_before = if native_before_template {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_native_content");
        native_panes::draw_native_pane_content(
            frame,
            pane,
            body,
            clip,
            interaction,
            text_input_focus,
        )
    } else {
        false
    };
    let has_template_content = {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_template_nodes");
        draw_pane_template_nodes(frame, pane, body, clip, interaction, text_input_focus)
    };
    let has_native_foreground = if native_before_template {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_native_foreground");
        super::super::super::welcome::draw_welcome_native_foreground(frame, pane, body, clip)
    } else {
        false
    };
    let has_native_content_after = if native_before_template {
        false
    } else {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_native_content");
        native_panes::draw_native_pane_content(
            frame,
            pane,
            body,
            clip,
            interaction,
            text_input_focus,
        )
    };
    let has_native_content =
        has_native_content_before || has_native_content_after || has_native_foreground;
    let has_debug_overlay_content = {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_debug_overlay");
        native_panes::draw_pane_debug_overlay(frame, pane, body, clip)
    };
    if !has_viewport_content
        && !has_template_content
        && !has_native_content
        && !has_debug_overlay_content
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_fallback");
        draw_pane_fallback(frame, pane, body, clip);
    }
}

fn native_content_precedes_template_nodes(pane_kind: &str) -> bool {
    pane_kind == "Welcome"
}

#[cfg(test)]
#[path = "tests/content.rs"]
mod tests;
