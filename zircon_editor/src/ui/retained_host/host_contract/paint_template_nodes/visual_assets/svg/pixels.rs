use resvg::{tiny_skia, usvg};
use std::path::Path;
use std::sync::Arc;

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
    let (source_target, supersample_scale) = target.vector_supersampled_source();
    let pixmap = {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_render_svg_raster");
        let svg_size = tree.size();
        let transform = tiny_skia::Transform::from_scale(
            source_target.width as f32 / svg_size.width(),
            source_target.height as f32 / svg_size.height(),
        );
        let mut pixmap = tiny_skia::Pixmap::new(source_target.width, source_target.height)?;
        resvg::render(tree.as_ref(), transform, &mut pixmap.as_mut());
        pixmap
    };

    let mut rgba = if supersample_scale == 1 {
        pixmap.take_demultiplied()
    } else {
        downsample_rgba_2x(
            &pixmap.take_demultiplied(),
            source_target.width,
            target.width,
            target.height,
        )
    };
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

fn downsample_rgba_2x(
    source: &[u8],
    source_width: u32,
    target_width: u32,
    target_height: u32,
) -> Vec<u8> {
    let mut target = vec![0_u8; target_width as usize * target_height as usize * 4];
    for y in 0..target_height {
        for x in 0..target_width {
            let mut alpha_sum = 0_u32;
            let mut premultiplied_sum = [0_u32; 3];
            for source_y in y * 2..y * 2 + 2 {
                for source_x in x * 2..x * 2 + 2 {
                    let source_offset =
                        ((source_y as usize * source_width as usize) + source_x as usize) * 4;
                    let alpha = source[source_offset + 3] as u32;
                    alpha_sum += alpha;
                    for channel in 0..3 {
                        premultiplied_sum[channel] +=
                            source[source_offset + channel] as u32 * alpha;
                    }
                }
            }
            let target_offset = ((y as usize * target_width as usize) + x as usize) * 4;
            target[target_offset + 3] = ((alpha_sum + 2) / 4).min(255) as u8;
            if alpha_sum > 0 {
                for channel in 0..3 {
                    target[target_offset + channel] =
                        ((premultiplied_sum[channel] + alpha_sum / 2) / alpha_sum).min(255) as u8;
                }
            }
        }
    }
    target
}

#[cfg(test)]
mod tests {
    use super::downsample_rgba_2x;

    #[test]
    fn two_x_resolve_averages_opaque_color_samples() {
        let source = [
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];

        assert_eq!(downsample_rgba_2x(&source, 2, 1, 1), [128, 128, 128, 255]);
    }

    #[test]
    fn two_x_resolve_ignores_rgb_hidden_by_zero_alpha() {
        let source = [255, 0, 0, 0, 0, 0, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0];

        assert_eq!(downsample_rgba_2x(&source, 2, 1, 1), [0, 0, 255, 64]);
    }
}
