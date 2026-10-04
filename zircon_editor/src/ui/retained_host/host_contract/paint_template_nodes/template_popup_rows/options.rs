//! 下拉选项行组织边界；投影选项与菜单共享容器、文字和尾部标记绘制组件。

mod entry;
mod style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_option_row_commands;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use style::popup_option_row_style;
