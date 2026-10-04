mod entry;
mod fallback;
mod metrics;
mod runs;
mod shaped;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_text_paint_commands;

#[cfg(test)]
#[path = "tests/commands.rs"]
mod tests;
