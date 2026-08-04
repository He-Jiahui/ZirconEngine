mod surface;
mod text;

use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::badge_overlay_frame;
use super::super::identity::{badge_is_dot, badge_is_invisible};
use super::super::labels::badge_display_text;
use surface::push_badge_overlay_surface;
use text::push_badge_overlay_text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_badge_overlay(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if badge_is_invisible(node) {
        return;
    }
    let display = badge_display_text(node);
    let dot = badge_is_dot(node);
    if !dot && display.is_empty() {
        return;
    }
    let badge_rect = badge_overlay_frame(node, rect, &display, dot);
    if !badge_rect.x.is_finite()
        || !badge_rect.y.is_finite()
        || badge_rect.width <= 0.0
        || badge_rect.height <= 0.0
    {
        return;
    }
    push_badge_overlay_surface(
        commands,
        node,
        badge_rect.clone(),
        clip,
        order,
        dot,
        opacity,
    );
    if !dot {
        push_badge_overlay_text(
            commands,
            node,
            &display,
            &badge_rect,
            clip,
            order + 1,
            opacity,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_finite_badge_anchor_does_not_emit_paint_commands() {
        let node = TemplatePaneNodeData {
            component_role: "badge".to_owned(),
            value_text: "1".to_owned(),
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: f32::INFINITY,
            y: 8.0,
            width: 24.0,
            height: 24.0,
        };
        let mut commands = Vec::new();

        push_badge_overlay(&mut commands, &node, &rect, &rect, 0, 1.0);

        assert!(commands.is_empty());
    }
}
