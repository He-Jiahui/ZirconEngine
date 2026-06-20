mod commands;
mod identity;
mod text;
mod thumb;
mod track;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_slider_commands;

#[cfg(test)]
use identity::{is_workbench_slider, slider_style};

#[cfg(test)]
#[path = "template_sliders_tests.rs"]
mod tests;
