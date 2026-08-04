mod content;
mod surface;

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::geometry::{avatar_corner_radius, avatar_frame};
use super::identity::is_avatar_node;
use super::image::avatar_image_pixels;
use super::style::{avatar_background_color, avatar_foreground_color};
use content::push_avatar_content;
use surface::{push_avatar_background, push_avatar_border};

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
    if !avatar_rect.x.is_finite()
        || !avatar_rect.y.is_finite()
        || avatar_rect.width <= 0.0
        || avatar_rect.height <= 0.0
    {
        return true;
    }

    let corner_radius = avatar_corner_radius(node, &avatar_rect);
    let avatar_image = avatar_image_pixels(node, &avatar_rect, corner_radius);
    let background = avatar_background_color(node, avatar_image.is_none());
    let foreground = avatar_foreground_color(node);

    push_avatar_background(
        commands,
        avatar_rect.clone(),
        clip,
        order,
        background,
        corner_radius,
        opacity,
    );
    push_avatar_content(
        commands,
        node,
        &avatar_rect,
        clip,
        order + 1,
        foreground,
        avatar_image,
        opacity,
    );
    push_avatar_border(
        commands,
        node,
        avatar_rect,
        clip,
        order + 2,
        corner_radius,
        opacity,
    );

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_finite_avatar_origins_do_not_emit_paint_commands() {
        let node = TemplatePaneNodeData {
            component_role: "avatar".to_owned(),
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: f32::INFINITY,
            y: 8.0,
            width: 24.0,
            height: 24.0,
        };
        let mut commands = Vec::new();

        assert!(push_avatar_primitive_commands(
            &mut commands,
            &node,
            &rect,
            &rect,
            0,
            1.0,
        ));
        assert!(commands.is_empty());
    }
}
