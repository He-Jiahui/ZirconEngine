//! 滑块轨道、值框与居中标记的几何门面；命令上下文一次确定轨道后，后续绘制重用该框。

mod alignment;
mod range;
mod track;
mod value;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use alignment::centered_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use range::slider_range_min_value_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use track::slider_track_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use value::slider_value_rect;
