use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::{intersect, is_visible_frame};
use super::super::render_commands::HostPaintCommand;
use super::super::template_style_color::resolved_style_color;
use super::super::visual_assets::{
    raster_size_from_frame, template_image_pixels, template_image_tint,
    template_vector_image_pixels,
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
    let materialization_rect =
        image_materialization_rect(node, rect, preview_size.width, preview_size.height);
    if !is_visible_frame(&materialization_rect) {
        return;
    }
    let Some(damage_frame) = intersect(&materialization_rect, clip) else {
        return;
    };
    let Some((target_width, target_height)) =
        raster_size_from_frame(materialization_rect.width, materialization_rect.height)
    else {
        return;
    };
    let tint = template_node_image_tint(node);
    let image = {
        zircon_runtime::profile_scope!("editor", "host_painter", "template_node_image_pixels");
        if node.role.as_str() == "SvgIcon" {
            template_vector_image_pixels(
                &node.preview_image,
                node.media_source.as_str(),
                node.icon_name.as_str(),
                target_width,
                target_height,
                tint,
                !is_icon_node(node),
                Some(damage_frame),
            )
        } else {
            template_image_pixels(
                &node.preview_image,
                node.media_source.as_str(),
                node.icon_name.as_str(),
                target_width,
                target_height,
                tint,
                !is_icon_node(node),
                Some(damage_frame),
            )
        }
    };
    let Some(image) = image else {
        return;
    };
    let image_rect = image_rect_for_node(node, rect, image.width, image.height);
    if !is_visible_frame(&image_rect) || intersect(&image_rect, clip).is_none() {
        return;
    }
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

fn image_materialization_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    preview_width: u32,
    preview_height: u32,
) -> FrameRect {
    if !is_icon_node(node) && (preview_width == 0 || preview_height == 0) {
        return rect.clone();
    }
    image_rect_for_node(node, rect, preview_width, preview_height)
}

fn template_node_image_tint(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    template_image_tint(
        is_icon_node(node),
        node.selected || node.checked || node.pressed || node.popup_open,
        node.disabled,
        node.text_tone.as_str(),
        node.validation_level.as_str(),
        resolved_style_color(node.button_style.element.foreground_color.as_ref()),
    )
}

#[cfg(test)]
#[path = "tests/command.rs"]
mod tests;
