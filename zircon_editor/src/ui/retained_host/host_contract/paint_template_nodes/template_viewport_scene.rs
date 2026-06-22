mod commands;
mod geometry;
mod identity;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_viewport_scene_commands;

#[cfg(test)]
#[path = "template_viewport_scene_tests/mod.rs"]
mod tests;
