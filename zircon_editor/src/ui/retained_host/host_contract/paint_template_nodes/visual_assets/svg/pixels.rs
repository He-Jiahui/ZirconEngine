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
mod tests {
    use std::sync::Arc;

    use super::super::super::RasterTargetSize;
    use super::super::parse::parse_svg_tree_data;
    use super::{downsample_rgba, render_svg_tree_pixels};

    #[test]
    fn two_x_resolve_averages_opaque_color_samples_in_linear_light() {
        let source = [
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];

        assert_eq!(downsample_rgba(&source, 2, 1, 1, 2), [188, 188, 188, 255]);
    }

    #[test]
    fn two_x_resolve_ignores_rgb_hidden_by_zero_alpha() {
        let source = [255, 0, 0, 0, 0, 0, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0];

        assert_eq!(downsample_rgba(&source, 2, 1, 1, 2), [0, 0, 255, 64]);
    }

    #[test]
    fn four_x_resolve_uses_all_sixteen_coverage_samples() {
        let mut source = vec![0_u8; 4 * 4 * 4];
        source[0..4].copy_from_slice(&[255, 255, 255, 255]);

        assert_eq!(downsample_rgba(&source, 4, 1, 1, 4), [255, 255, 255, 16]);
    }

    #[test]
    fn vector_raster_resolves_fractional_edges_at_the_physical_target() {
        let tree = parse_svg_tree_data(
            br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">
                    <circle cx="5" cy="5" r="3.4" fill="#ffffff"/>
                </svg>"##,
            None,
        )
        .expect("parse vector fixture");
        let target = RasterTargetSize::new(9, 9).expect("physical target");

        let image = render_svg_tree_pixels(Arc::new(tree), target, None)
            .expect("render supersampled vector fixture");
        let alpha = image.rgba.chunks_exact(4).map(|pixel| pixel[3]);

        assert_eq!((image.width, image.height), (9, 9));
        assert!(alpha.clone().any(|value| value == 0));
        assert!(alpha.clone().any(|value| value == 255));
        assert!(alpha.into_iter().any(|value| (1..=254).contains(&value)));
    }

    #[test]
    fn vector_raster_centers_source_aspect_inside_the_full_requested_target() {
        let tree = parse_svg_tree_data(
            br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 10">
                    <rect width="20" height="10" fill="#ffffff"/>
                </svg>"##,
            None,
        )
        .expect("parse vector fixture");
        let target = RasterTargetSize::new(100, 100).expect("physical target");

        let image = render_svg_tree_pixels(Arc::new(tree), target, None)
            .expect("render aspect-preserving vector fixture");

        assert_eq!((image.width, image.height), (100, 100));
        let visible_rows = image
            .rgba
            .chunks_exact(100 * 4)
            .enumerate()
            .filter_map(|(row, pixels)| {
                pixels
                    .chunks_exact(4)
                    .any(|pixel| pixel[3] > 0)
                    .then_some(row)
            })
            .collect::<Vec<_>>();
        assert_eq!(visible_rows.first(), Some(&25));
        assert_eq!(visible_rows.last(), Some(&74));
    }
}
