//! 滑块轨道、值框与居中标记的几何门面；命令上下文一次确定轨道后，后续绘制重用该框。

#[path = "rects/alignment.rs"]
mod alignment;
#[path = "rects/range.rs"]
mod range;
#[path = "rects/track.rs"]
mod track;
#[path = "rects/value.rs"]
mod value;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use alignment::centered_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use range::slider_range_min_value_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use track::slider_track_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use value::slider_value_rect;
