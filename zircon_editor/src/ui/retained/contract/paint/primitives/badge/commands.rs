// 命令按根表面、根文字和覆盖层分组；sequencing 模块决定层级及组件接管语义。
#[path = "../../badge/commands/overlay.rs"]
mod overlay;
#[path = "../../badge/commands/root_label.rs"]
mod root_label;
#[path = "../../badge/commands/root_surface.rs"]
mod root_surface;
#[path = "../../badge/commands/sequencing.rs"]
mod sequencing;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use sequencing::push_badge_primitive_commands;
