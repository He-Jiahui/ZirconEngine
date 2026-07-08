use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::visual_assets::{raster_size_from_frame, template_image_pixels};

pub(super) fn push_thumbnail_preview_image_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !thumbnail_has_real_preview(node) {
        return false;
    }
    let preview_rect = thumbnail_preview_image_rect(node, rect);
    let Some((target_width, target_height)) =
        raster_size_from_frame(preview_rect.width, preview_rect.height)
    else {
        return false;
    };
    let Some(image) = template_image_pixels(
        &node.preview_image,
        node.media_source.as_str(),
        "",
        target_width,
        target_height,
        None,
        true,
    ) else {
        return false;
    };

    commands.push(HostPaintCommand::image_pixels(
        preview_rect,
        Some(clip.clone()),
        order,
        image.resource_key,
        image.width,
        image.height,
        image.rgba,
        image.atlas,
        opacity,
    ));
    true
}

fn thumbnail_has_real_preview(node: &TemplatePaneNodeData) -> bool {
    node.has_preview_image || !node.media_source.trim().is_empty()
}

fn thumbnail_preview_image_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let size = node.preview_image.size();
    if size.width == 0 || size.height == 0 || rect.width <= 0.0 || rect.height <= 0.0 {
        return rect.clone();
    }
    let image_aspect = size.width as f32 / size.height as f32;
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
