//! 拖拽反馈的几何边界：预览与落点指示器共用密度指标，但位置来自两组独立投影数据。

mod indicator;
mod metrics;
mod preview;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use indicator::indicator_frame;
pub(super) use metrics::{drag_overlay_metrics, DragOverlayMetrics};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use preview::{
    preview_frame, preview_icon_frame, preview_text_frame,
};
