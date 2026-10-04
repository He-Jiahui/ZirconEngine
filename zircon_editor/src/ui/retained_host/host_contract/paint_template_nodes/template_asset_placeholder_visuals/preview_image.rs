//! 资产缩略图先尝试真实预览，成功后停止占位图标；无资源时交还上层继续绘制后备视觉。
//! 返回 handled 还表示预览已被裁剪，不能被解释为一定追加了像素命令；图像按比例内收在缩略图井。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::visual_assets::{preview_artifact_image_pixels, raster_size_from_frame};
use crate::ui::retained_host::host_contract::paint_geometry::{
    intersect, inward_pixel_aligned_rect,
};

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
    let Some(damage_frame) = intersect(rect, clip) else {
        return true;
    };
    let Some((target_width, target_height)) = raster_size_from_frame(rect.width, rect.height)
    else {
        return false;
    };
    let Some(image) = preview_artifact_image_pixels(
        &node.preview_image,
        node.media_source.as_str(),
        target_width,
        target_height,
        Some(damage_frame),
    ) else {
        return false;
    };
    let preview_rect = fitted_thumbnail_preview_image_rect(rect, image.width, image.height)
        .unwrap_or_else(|| rect.clone());
    if intersect(&preview_rect, clip).is_none() {
        return true;
    }

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

fn fitted_thumbnail_preview_image_rect(
    rect: &FrameRect,
    image_width: u32,
    image_height: u32,
) -> Option<FrameRect> {
    if image_width == 0
        || image_height == 0
        || !rect.x.is_finite()
        || !rect.y.is_finite()
        || !rect.width.is_finite()
        || !rect.height.is_finite()
        || rect.width <= 0.0
        || rect.height <= 0.0
    {
        return None;
    }

    let image_aspect = image_width as f32 / image_height as f32;
    let rect_aspect = rect.width / rect.height;
    let fitted = if rect_aspect > image_aspect {
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
    };

    Some(inward_pixel_aligned_rect(&fitted))
}

#[cfg(test)]
#[path = "tests/preview_image.rs"]
mod tests;
