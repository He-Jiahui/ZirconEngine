//! 行尾标记图形的转发边界；所有图标最终统一走template_icon_assets解析与tint。

mod dispatch;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use dispatch::push_popup_row_adornment;
