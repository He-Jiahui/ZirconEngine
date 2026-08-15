mod cache;
mod font;
mod image;
mod parse;
mod pixels;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use cache::load_svg_tree_with_parser;
pub(in crate::ui::retained_host) use cache::{
    clear_svg_tree_cache, invalidate_svg_tree_paths, reconcile_svg_tree_sources,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use font::svg_may_need_fonts;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use image::{
    render_svg_file_image, render_svg_tree_image,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use parse::parse_svg_tree_data;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use pixels::{
    render_svg_file_pixels, render_svg_tree_pixels,
};
