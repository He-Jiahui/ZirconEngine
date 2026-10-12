//! 状态栏文字、图标与语义信号共用宿主密度指标；坐标映射不负责决定节点认领。

#[path = "status_control_geometry/chips.rs"]
mod chips;
#[path = "status_control_geometry/common.rs"]
mod common;
#[path = "status_control_geometry/icons.rs"]
mod icons;
#[path = "status_control_geometry/metrics.rs"]
mod metrics;
#[path = "status_control_geometry/signals.rs"]
mod signals;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use chips::status_chip_text_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use common::{
    frame_is_within, has_paintable_status_control_extent, status_control_offset_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use icons::status_icon_button_glyph_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    status_chip_radius, status_font_size, status_icon_button_radius, status_icon_glyph_size,
    status_line_height, status_text_metrics_for_node, workbench_status_metrics,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use signals::status_signal_text_gap;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use signals::{
    status_signal_icon_paint_rect, status_signal_icon_rect, status_signal_text_rect,
};
