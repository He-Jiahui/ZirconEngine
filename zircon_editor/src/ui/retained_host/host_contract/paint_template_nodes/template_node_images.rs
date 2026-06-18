use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_geometry::{intersect, is_visible_frame};
use super::render_commands::HostPaintCommand;
use super::template_node_labels::template_node_label;
use super::template_style_color::resolved_style_color;
use super::visual_assets::{raster_size_from_frame, template_image_pixels, template_image_tint};

const ICON_TEXT_HORIZONTAL_INSET: f32 = 5.0;
const ICON_TEXT_VERTICAL_INSET: f32 = 5.0;

pub(super) fn push_template_image_command(
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

pub(super) fn is_icon_node(node: &TemplatePaneNodeData) -> bool {
    matches!(node.role.as_str(), "Icon" | "IconButton" | "SvgIcon") || !node.icon_name.is_empty()
}

pub(super) fn is_icon_only_node(node: &TemplatePaneNodeData) -> bool {
    matches!(node.role.as_str(), "Icon" | "IconButton" | "SvgIcon")
}

pub(super) fn leading_icon_size(rect: &FrameRect) -> f32 {
    (rect.height - ICON_TEXT_VERTICAL_INSET * 2.0)
        .min(rect.width * 0.28)
        .max(1.0)
}

fn template_node_has_image_source(node: &TemplatePaneNodeData) -> bool {
    node.has_preview_image || !node.media_source.is_empty() || !node.icon_name.is_empty()
}

fn image_rect_for_node(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    image_width: u32,
    image_height: u32,
) -> FrameRect {
    if is_icon_node(node) {
        let label = template_node_label(node, None);
        if !label.is_empty() && !is_icon_only_node(node) {
            let size = leading_icon_size(rect);
            return FrameRect {
                x: rect.x + ICON_TEXT_HORIZONTAL_INSET,
                y: rect.y + (rect.height - size) * 0.5,
                width: size,
                height: size,
            };
        }
        let inset = (rect.width.min(rect.height) * 0.16).min(4.0).max(0.0);
        let size = (rect.width.min(rect.height) - inset * 2.0).max(1.0);
        return FrameRect {
            x: rect.x + (rect.width - size) * 0.5,
            y: rect.y + (rect.height - size) * 0.5,
            width: size,
            height: size,
        };
    }
    fitted_image_rect(rect, image_width, image_height)
}

fn fitted_image_rect(rect: &FrameRect, image_width: u32, image_height: u32) -> FrameRect {
    if image_width == 0 || image_height == 0 || rect.width <= 0.0 || rect.height <= 0.0 {
        return rect.clone();
    }
    let image_aspect = image_width as f32 / image_height as f32;
    let rect_aspect = rect.width / rect.height;
    if rect_aspect > image_aspect {
        let height = rect.height;
        let width = height * image_aspect;
        FrameRect {
            x: rect.x + (rect.width - width) * 0.5,
            y: rect.y,
            width,
            height,
        }
    } else {
        let width = rect.width;
        let height = width / image_aspect;
        FrameRect {
            x: rect.x,
            y: rect.y + (rect.height - height) * 0.5,
            width,
            height,
        }
    }
}
