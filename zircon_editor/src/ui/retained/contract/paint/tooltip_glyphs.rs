//! 工作台 tooltip 的箭头和 info 图标复用资产入口；调用方决定其布局槽位与主题色。

#[path = "tooltip_glyphs/arrows.rs"]
mod arrows;
#[path = "tooltip_glyphs/icons.rs"]
mod icons;
#[path = "tooltip_glyphs/metrics.rs"]
mod metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use arrows::push_tooltip_arrow;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use icons::push_tooltip_info_icon;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    tooltip_arrow_size, tooltip_icon_size,
};
