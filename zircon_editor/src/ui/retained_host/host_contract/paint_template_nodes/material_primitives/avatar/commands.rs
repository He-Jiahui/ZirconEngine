use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::geometry::{avatar_corner_radius, avatar_fallback_child_frame, avatar_frame};
use super::glyph::push_avatar_fallback_glyph;
use super::identity::is_avatar_node;
use super::image::{avatar_icon_pixels, avatar_image_pixels, push_avatar_image};
use super::style::{
    avatar_background_color, avatar_border_color, avatar_border_width, avatar_foreground_color,
};
use super::text::{avatar_label, push_avatar_text};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_avatar_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_avatar_node(node) {
        return false;
    }

    let avatar_rect = avatar_frame(rect);
    if avatar_rect.width <= 0.0 || avatar_rect.height <= 0.0 {
        return true;
    }

    let corner_radius = avatar_corner_radius(node, &avatar_rect);
    let avatar_image = avatar_image_pixels(node, &avatar_rect, corner_radius);
    let background = avatar_background_color(node, avatar_image.is_none());
    let foreground = avatar_foreground_color(node);
    commands.push(HostPaintCommand::quad(
        avatar_rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        None,
        0.0,
        corner_radius,
        opacity,
    ));

    if let Some(image) = avatar_image {
        push_avatar_image(
            commands,
            image,
            avatar_rect.clone(),
            clip,
            order + 1,
            opacity,
        );
    } else if !avatar_label(node).is_empty() {
        push_avatar_text(
            commands,
            node,
            &avatar_rect,
            clip,
            order + 1,
            foreground,
            opacity,
        );
    } else if let Some(icon) = avatar_icon_pixels(node, &avatar_rect, foreground) {
        let icon_rect = avatar_fallback_child_frame(&avatar_rect);
        push_avatar_image(commands, icon, icon_rect, clip, order + 1, opacity);
    } else {
        push_avatar_fallback_glyph(commands, &avatar_rect, clip, order + 1, foreground, opacity);
    }

    if let Some(border_color) = avatar_border_color(node) {
        commands.push(HostPaintCommand::quad(
            avatar_rect,
            Some(clip.clone()),
            order + 2,
            None,
            Some(border_color),
            avatar_border_width(node),
            corner_radius,
            opacity,
        ));
    }

    true
}
