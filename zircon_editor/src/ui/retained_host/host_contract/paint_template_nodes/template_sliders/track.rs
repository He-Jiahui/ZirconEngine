//! 轨道底色/范围填充与刻度绘制门面；调用方分配二者的相对层级。

mod rail;
mod ticks;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use rail::push_slider_track;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use ticks::push_slider_ticks;
