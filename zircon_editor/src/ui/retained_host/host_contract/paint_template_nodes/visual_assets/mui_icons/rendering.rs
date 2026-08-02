use std::{path::Path, sync::Arc};

use super::{
    super::{
        HostPaintImagePixels, RasterTargetSize,
        svg::{parse_svg_tree_data, render_svg_tree_image, render_svg_tree_pixels},
    },
    svg_document::module_svg,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn render_module_pixels(
    path: &Path,
    target: RasterTargetSize,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let svg = module_svg(path)?;
    let tree = parse_svg_tree_data(svg.as_bytes(), None).map(Arc::new)?;
    render_svg_tree_pixels(tree, target, tint)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn render_module_image(
    path: &Path,
) -> Option<crate::ui::retained_host::primitives::Image> {
    let svg = module_svg(path)?;
    let tree = parse_svg_tree_data(svg.as_bytes(), None).map(Arc::new)?;
    render_svg_tree_image(tree)
}
