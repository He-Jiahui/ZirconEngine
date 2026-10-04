//! 菜单行组织边界；entry负责把投影菜单项与统一行几何组合，style负责状态选择。

mod entry;
mod style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_menu_row_commands;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use style::popup_menu_row_style;
