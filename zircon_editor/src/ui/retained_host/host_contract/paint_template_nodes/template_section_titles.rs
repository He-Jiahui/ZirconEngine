mod commands;
mod geometry;
mod identity;
mod style;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_section_title_commands;

#[cfg(test)]
use identity::is_workbench_section_title;
#[cfg(test)]
use style::{section_text_color, SECTION_MESH_TEXT};

#[cfg(test)]
#[path = "template_section_titles_tests.rs"]
mod tests;
