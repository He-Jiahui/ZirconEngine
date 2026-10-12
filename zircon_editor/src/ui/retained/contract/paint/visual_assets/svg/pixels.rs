use resvg::{tiny_skia, usvg};
use std::path::Path;
use std::sync::Arc;

use crate::ui::retained_host::host_contract::paint_color::{
    linear_to_srgb_byte, srgb_byte_to_linear,
};

use super::super::{
    retained_image_resource_key, tint_non_transparent_pixels, HostPaintImagePixels,
    RasterTargetSize,
};
use super::cache::load_svg_tree;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn render_svg_file_pixels(
    path: &Path,
    target: RasterTargetSize,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let tree = load_svg_tree(path)?;
    render_svg_tree_pixels(tree, target, tint)
}

/// SVG 先按目标尺寸和超采样策略渲染，再降采样到绘制命令所需的共享 RGBA。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn render_svg_tree_pixels(
    tree: Arc<usvg::Tree>,
    target: RasterTargetSize,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let svg_size = tree.size();
    let content_target = target.fit_preserving_aspect(svg_size.width(), svg_size.height())?;
    let (source_target, supersample_scale) = content_target.vector_supersampled_source();
    let pixmap = {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_render_svg_raster");
        let transform = tiny_skia::Transform::from_scale(
            source_target.width as f32 / svg_size.width(),
            source_target.height as f32 / svg_size.height(),
        );
        let mut pixmap = tiny_skia::Pixmap::new(source_target.width, source_target.height)?;
        resvg::render(tree.as_ref(), transform, &mut pixmap.as_mut());
        pixmap
    };

    let content_rgba = if supersample_scale == 1 {
        pixmap.take_demultiplied()
    } else {
        downsample_rgba(
            &pixmap.take_demultiplied(),
            source_target.width,
            content_target.width,
            content_target.height,
            supersample_scale,
        )
    };
    let mut rgba = center_rgba_in_target(content_rgba, content_target, target)?;
    if let Some(tint) = tint {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_render_svg_tint");
        tint_non_transparent_pixels(&mut rgba, tint);
    }
    let image = HostPaintImagePixels {
        resource_key: retained_image_resource_key(target.width, target.height, &rgba),
        width: target.width,
        height: target.height,
        rgba: rgba.into(),
        atlas: None,
    };
    image.is_valid().then_some(image)
}

fn center_rgba_in_target(
    source: Vec<u8>,
    source_target: RasterTargetSize,
    target: RasterTargetSize,
) -> Option<Vec<u8>> {
    let source_stride = usize::try_from(source_target.width).ok()?.checked_mul(4)?;
    let source_len = source_stride.checked_mul(usize::try_from(source_target.height).ok()?)?;
    if source.len() != source_len
        || source_target.width > target.width
        || source_target.height > target.height
    {
        return None;
    }
    if source_target == target {
        return Some(source);
    }

    let target_stride = usize::try_from(target.width).ok()?.checked_mul(4)?;
    let target_len = target_stride.checked_mul(usize::try_from(target.height).ok()?)?;
    let offset_x = usize::try_from((target.width - source_target.width) / 2)
        .ok()?
        .checked_mul(4)?;
    let offset_y = usize::try_from((target.height - source_target.height) / 2).ok()?;
    let mut output = vec![0_u8; target_len];
    for source_y in 0..usize::try_from(source_target.height).ok()? {
        let source_start = source_y.checked_mul(source_stride)?;
        let target_start = offset_y
            .checked_add(source_y)?
            .checked_mul(target_stride)?
            .checked_add(offset_x)?;
        output[target_start..target_start.checked_add(source_stride)?]
            .copy_from_slice(&source[source_start..source_start.checked_add(source_stride)?]);
    }
    Some(output)
}

fn downsample_rgba(
    source: &[u8],
    source_width: u32,
    target_width: u32,
    target_height: u32,
    sample_axis: u32,
) -> Vec<u8> {
    debug_assert!(sample_axis > 1);
    let sample_count = (sample_axis * sample_axis) as f32;
    let mut target = vec![0_u8; target_width as usize * target_height as usize * 4];
    for y in 0..target_height {
        for x in 0..target_width {
            let mut alpha_sum = 0.0_f32;
            let mut premultiplied_linear_sum = [0.0_f32; 3];
            for source_y in y * sample_axis..y * sample_axis + sample_axis {
                for source_x in x * sample_axis..x * sample_axis + sample_axis {
                    let source_offset =
                        ((source_y as usize * source_width as usize) + source_x as usize) * 4;
                    let alpha = f32::from(source[source_offset + 3]) / 255.0;
                    alpha_sum += alpha;
                    for channel in 0..3 {
                        premultiplied_linear_sum[channel] +=
                            srgb_byte_to_linear(source[source_offset + channel]) * alpha;
                    }
                }
            }
            let target_offset = ((y as usize * target_width as usize) + x as usize) * 4;
            target[target_offset + 3] = encode_unorm8(alpha_sum / sample_count);
            if alpha_sum > 0.0 {
                for channel in 0..3 {
                    let straight_linear = premultiplied_linear_sum[channel] / alpha_sum;
                    target[target_offset + channel] = linear_to_srgb_byte(straight_linear);
                }
            }
        }
    }
    target
}

fn encode_unorm8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
#[path = "tests/pixels.rs"]
mod tests;
