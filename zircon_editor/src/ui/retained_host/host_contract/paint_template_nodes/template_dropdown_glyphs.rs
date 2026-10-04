//! 下拉框尾部箭头的资产边界；由下拉入口先校验尺寸再调用。

mod chevron;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use chevron::push_dropdown_chevron;
