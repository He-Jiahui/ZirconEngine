#[path = "separators/commands.rs"]
mod commands;
#[path = "separators/lines.rs"]
mod lines;
#[path = "separators/metrics.rs"]
mod metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_shell_panel_separators;
