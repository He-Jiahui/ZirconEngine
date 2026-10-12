#[path = "divider/commands.rs"]
mod commands;
#[path = "divider/geometry.rs"]
mod geometry;
#[path = "divider/horizontal.rs"]
mod horizontal;
#[path = "divider/identity.rs"]
mod identity;
#[path = "divider/labels.rs"]
mod labels;
#[path = "divider/lines.rs"]
mod lines;
#[path = "divider/style.rs"]
mod style;
#[path = "divider/vertical.rs"]
mod vertical;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_divider_primitive_commands;
