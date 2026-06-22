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
    if badge_rect.width <= 0.0 || badge_rect.height <= 0.0 {
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
