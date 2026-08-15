use std::{path::Path, sync::Arc};

use super::{
    super::{
        svg::{
            load_svg_tree_with_parser, parse_svg_tree_data, render_svg_tree_image,
            render_svg_tree_pixels,
        },
        HostPaintImagePixels, RasterTargetSize,
    },
    svg_document::module_svg,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn render_module_pixels(
    path: &Path,
    target: RasterTargetSize,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let tree = load_module_tree(path)?;
    render_svg_tree_pixels(tree, target, tint)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn render_module_image(
    path: &Path,
) -> Option<crate::ui::retained_host::primitives::Image> {
    let tree = load_module_tree(path)?;
    render_svg_tree_image(tree)
}

pub(super) fn load_module_tree(path: &Path) -> Option<Arc<resvg::usvg::Tree>> {
    load_svg_tree_with_parser(path, |source, _| {
        let source = std::str::from_utf8(source).ok()?;
        let svg = module_svg(source)?;
        parse_svg_tree_data(svg.as_bytes(), None)
    })
}
