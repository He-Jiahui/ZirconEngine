// Badge 入口使根节点同时拥有宿主表面和计数覆盖层；子槽由根节点统一吞掉回退。
#[path = "badge/commands.rs"]
mod commands;
#[path = "badge/geometry.rs"]
mod geometry;
#[path = "badge/identity.rs"]
mod identity;
#[path = "badge/labels.rs"]
mod labels;
#[path = "badge/style.rs"]
mod style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_badge_primitive_commands;
