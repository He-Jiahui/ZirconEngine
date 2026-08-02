use std::path::Path;
use std::sync::Arc;

use resvg::{tiny_skia, usvg};

use super::super::{
    HostPaintImagePixels, RasterTargetSize, retained_image_resource_key,
    tint_non_transparent_pixels,
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
    let pixmap = {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_render_svg_raster");
        let svg_size = tree.size();
        let transform = tiny_skia::Transform::from_scale(
            target.width as f32 / svg_size.width(),
            target.height as f32 / svg_size.height(),
        );
        let mut pixmap = tiny_skia::Pixmap::new(target.width, target.height)?;
        resvg::render(tree.as_ref(), transform, &mut pixmap.as_mut());
        pixmap
    };

    let mut rgba = pixmap.take_demultiplied();
    if let Some(tint) = tint {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_render_svg_tint");
        tint_non_transparent_pixels(&mut rgba, tint);
    }
    let image = HostPaintImagePixels {
        resource_key: retained_image_resource_key(target.width, target.height, &rgba),
        width: target.width,
        height: target.height,
        rgba,
        atlas: None,
    };
    image.is_valid().then_some(image)
}
