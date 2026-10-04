// 命令按根表面、根文字和覆盖层分组；sequencing 模块决定层级及组件接管语义。
mod overlay;
mod root_label;
mod root_surface;
mod sequencing;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use sequencing::push_badge_primitive_commands;
