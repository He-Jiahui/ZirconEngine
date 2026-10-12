#[path = "../mui_icons/candidates.rs"]
mod candidates;
#[path = "mui_icons/names.rs"]
mod names;
#[path = "mui_icons/parser.rs"]
mod parser;
#[path = "../mui_icons/rendering.rs"]
mod rendering;
#[path = "../mui_icons/svg_document.rs"]
mod svg_document;

#[cfg(test)]
#[path = "../mui_icons/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use candidates::{
    is_module_path, module_candidates,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use rendering::{
    render_module_image, render_module_pixels,
};
