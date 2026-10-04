mod candidates;
mod names;
mod parser;
mod rendering;
mod svg_document;

#[cfg(test)]
#[path = "mui_icons/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use candidates::{
    is_module_path, module_candidates,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use rendering::{
    render_module_image, render_module_pixels,
};
