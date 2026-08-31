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
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn pane_without_damage_clip_keeps_visible_content() {
        assert!(pane_intersects_damage(&rect(10.0, 10.0, 100.0, 80.0), None));
    }

    #[test]
    fn pane_damage_gate_accepts_intersection_and_rejects_disjoint_or_collapsed_content() {
        let content = rect(10.0, 10.0, 100.0, 80.0);

        assert!(pane_intersects_damage(
            &content,
            Some(&rect(50.0, 20.0, 20.0, 20.0))
        ));
        assert!(!pane_intersects_damage(
            &content,
            Some(&rect(200.0, 200.0, 20.0, 20.0))
        ));
        assert!(!pane_intersects_damage(&rect(10.0, 10.0, 0.0, 80.0), None));
    }
}
