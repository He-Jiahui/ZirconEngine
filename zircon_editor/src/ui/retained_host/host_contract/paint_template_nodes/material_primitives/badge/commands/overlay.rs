use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::{badge_overlay_frame, badge_overlay_radius, badge_overlay_text_frame};
use super::super::identity::{badge_is_dot, badge_is_invisible};
use super::super::labels::badge_display_text;
use super::super::style::{
    badge_overlay_background_color, badge_overlay_border_color, badge_overlay_border_width,
    badge_overlay_text_color,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

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
    let background = badge_overlay_background_color(node);
    let foreground = badge_overlay_text_color(node);
    let border_width = badge_overlay_border_width(node);
    commands.push(HostPaintCommand::quad(
        badge_rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        Some(badge_overlay_border_color(node, background)),
        border_width,
        badge_overlay_radius(dot),
        opacity,
    ));
    if !dot {
        push_badge_overlay_text(
            commands,
            &display,
            &badge_rect,
            clip,
            order + 1,
            foreground,
            opacity,
        );
    }
}

fn push_badge_overlay_text(
    commands: &mut Vec<HostPaintCommand>,
    display: &str,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let text_frame = badge_overlay_text_frame(display, rect);
    commands.push(HostPaintCommand::text(
        text_frame.rect,
        Some(clip.clone()),
        order,
        display.to_string(),
        color,
        text_frame.font_size,
        text_frame.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
