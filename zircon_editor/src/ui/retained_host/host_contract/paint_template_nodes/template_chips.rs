mod commands;
mod geometry;
mod identity;
mod style;
mod surface;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_chip_commands;

#[cfg(test)]
#[path = "template_chips_tests/mod.rs"]
mod tests;
