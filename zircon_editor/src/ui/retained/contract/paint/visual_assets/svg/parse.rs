use std::path::PathBuf;

use resvg::usvg;

use super::font::{cached_svg_font_db, svg_may_need_fonts};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn parse_svg_tree_data(
    svg: &[u8],
    resources_dir: Option<PathBuf>,
) -> Option<usvg::Tree> {
    let mut options = usvg::Options {
        resources_dir,
        ..usvg::Options::default()
    };
    if svg_may_need_fonts(svg) {
        options.fontdb = cached_svg_font_db();
    }

    usvg::Tree::from_data(svg, &options).ok()
}
