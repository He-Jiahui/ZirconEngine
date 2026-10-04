//! 滑块三类文字的绘制门面；标签、主值和范围下限来自不同数据来源。

mod label;
mod range_min;
mod value;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use label::push_slider_label;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use range_min::push_slider_range_min_value;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use value::push_slider_value;
