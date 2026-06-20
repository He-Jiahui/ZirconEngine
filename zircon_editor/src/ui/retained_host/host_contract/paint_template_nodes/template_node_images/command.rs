use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::{intersect, is_visible_frame};
use super::super::render_commands::HostPaintCommand;
use super::super::template_style_color::resolved_style_color;
use super::super::visual_assets::{
    raster_size_from_frame, template_image_pixels, template_image_tint,
};
use super::geometry::image_rect_for_node;
use super::identity::{is_icon_node, template_node_has_image_source};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_template_image_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !template_node_has_image_source(node) {
        return;
    }
    let preview_size = node.preview_image.size();
    let image_rect = image_rect_for_node(node, rect, preview_size.width, preview_size.height);
    if !is_visible_frame(&image_rect) {
        return;
    }
    if intersect(&image_rect, clip).is_none() {
        return;
    }
    let Some((target_width, target_height)) =
        raster_size_from_frame(image_rect.width, image_rect.height)
    else {
        return;
    };
    let tint = template_image_tint(
        is_icon_node(node),
        node.selected || node.focused || node.pressed,
        node.disabled,
        node.text_tone.as_str(),
        node.validation_level.as_str(),
        resolved_style_color(node.button_style.element.foreground_color.as_ref()),
    );
    let image = {
        zircon_runtime::profile_scope!("editor", "host_painter", "template_node_image_pixels");
        template_image_pixels(
            &node.preview_image,
            node.media_source.as_str(),
            node.icon_name.as_str(),
            target_width,
            target_height,
            tint,
            !is_icon_node(node),
        )
    };
    let Some(image) = image else {
        return;
    };
    commands.push(HostPaintCommand::image_pixels(
        image_rect,
        Some(clip.clone()),
        order,
        image.resource_key,
        image.width,
        image.height,
        image.rgba,
        image.atlas,
        opacity,
    ));
}
