use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::super::paint_geometry::intersect;
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::super::visual_assets::HostPaintImagePixels;
use super::super::geometry::avatar_fallback_child_frame;
use super::super::glyph::push_avatar_fallback_glyph;
use super::super::image::{avatar_icon_pixels, push_avatar_image};
use super::super::text::{avatar_label, push_avatar_text};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_avatar_content(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    avatar_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    foreground: [u8; 4],
    avatar_image: Option<HostPaintImagePixels>,
    opacity: f32,
) {
    if let Some(image) = avatar_image {
        push_avatar_image(commands, image, avatar_rect.clone(), clip, order, opacity);
    } else if !avatar_label(node).is_empty() {
        push_avatar_text(
            commands,
            node,
            avatar_rect,
            clip,
            order,
            foreground,
            opacity,
        );
    } else {
        let icon_rect = avatar_fallback_child_frame(avatar_rect);
        let icon = intersect(&icon_rect, clip).and_then(|damage_frame| {
            avatar_icon_pixels(node, &icon_rect, foreground, damage_frame)
        });
        match icon {
            Some(icon) => push_avatar_image(commands, icon, icon_rect, clip, order, opacity),
            None => {
                push_avatar_fallback_glyph(commands, avatar_rect, clip, order, foreground, opacity)
            }
        }
    }
}
