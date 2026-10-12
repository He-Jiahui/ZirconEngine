use std::path::Path;
use std::sync::Arc;

use resvg::{tiny_skia, usvg};

use super::super::MAX_VECTOR_RASTER_EDGE;
use super::cache::load_svg_tree;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn render_svg_file_image(
    path: &Path,
) -> Option<crate::ui::retained_host::primitives::Image> {
    let tree = load_svg_tree(path)?;
    render_svg_tree_image(tree)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn render_svg_tree_image(
    tree: Arc<usvg::Tree>,
) -> Option<crate::ui::retained_host::primitives::Image> {
    let size = tree.size();
    let width = size
        .width()
        .ceil()
        .clamp(1.0, MAX_VECTOR_RASTER_EDGE as f32) as u32;
    let height = size
        .height()
        .ceil()
        .clamp(1.0, MAX_VECTOR_RASTER_EDGE as f32) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
    resvg::render(
        tree.as_ref(),
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    let pixels = pixmap.take_demultiplied();
    Some(crate::ui::retained_host::primitives::Image::from_rgba8(
        crate::ui::retained_host::primitives::SharedPixelBuffer::<
            crate::ui::retained_host::primitives::Rgba8Pixel,
        >::clone_from_slice(&pixels, width, height),
    ))
}
