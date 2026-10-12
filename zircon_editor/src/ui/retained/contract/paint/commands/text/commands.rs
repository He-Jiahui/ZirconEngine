#[path = "commands/entry.rs"]
mod entry;
#[path = "commands/fallback.rs"]
mod fallback;
#[path = "commands/metrics.rs"]
mod metrics;
#[path = "commands/runs.rs"]
mod runs;
#[path = "commands/shaped.rs"]
mod shaped;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_text_paint_commands;

#[cfg(test)]
#[path = "tests/commands.rs"]
mod tests;
