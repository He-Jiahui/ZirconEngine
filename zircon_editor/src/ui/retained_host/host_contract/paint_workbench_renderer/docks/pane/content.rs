use super::super::super::super::data::{
    FrameRect, HostPaneInteractionStateData, HostTextInputFocusData, HostViewportImageData,
    PaneData,
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
    viewport_image: Option<&HostViewportImageData>,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    let painted_viewport = {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_viewport_image");
        native_panes::draw_viewport_image(frame, pane, body, clip, viewport_image)
    };
    let native_before_template = native_content_precedes_template_nodes(pane.kind.as_str());
    let painted_native_before = if native_before_template {
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
    let painted_nodes = {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_template_nodes");
        draw_pane_template_nodes(frame, pane, body, clip, interaction, text_input_focus)
    };
    let painted_native_after = if native_before_template {
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
    let painted_native = painted_native_before || painted_native_after;
    let painted_debug_overlay = {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_debug_overlay");
        native_panes::draw_pane_debug_overlay(frame, pane, body, clip)
    };
    if !painted_viewport && !painted_nodes && !painted_native && !painted_debug_overlay {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_pane_fallback");
        draw_pane_fallback(frame, pane, body, clip);
    }
}

fn native_content_precedes_template_nodes(pane_kind: &str) -> bool {
    pane_kind == "Welcome"
}

#[cfg(test)]
mod tests {
    use super::native_content_precedes_template_nodes;

    #[test]
    fn welcome_native_foundation_precedes_shared_template_controls_only() {
        assert!(native_content_precedes_template_nodes("Welcome"));
        for pane_kind in ["Hierarchy", "Assets", "AssetBrowser", "Inspector"] {
            assert!(
                !native_content_precedes_template_nodes(pane_kind),
                "{pane_kind} native overlays must remain after template content"
            );
        }
    }
}
