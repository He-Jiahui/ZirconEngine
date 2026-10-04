//! 菜单与下拉弹层的底面、行面和分隔线组织边界；三者共享同一clip和popup层序。

mod background;
mod row;
mod separator;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use background::push_popup_background;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use row::push_popup_row_surface;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use separator::push_popup_separator;
