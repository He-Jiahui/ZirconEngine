mod commands;
mod identity;
mod layout;
mod surface;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_tooltip_commands;

#[cfg(test)]
#[path = "template_tooltips_tests/mod.rs"]
mod tests;
