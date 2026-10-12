//! chip 文本区域预留左右缓冲，供标签和值左右分列；绘制器须再确认其有效且处于父框内。

use super::super::super::data::FrameRect;
use super::metrics::workbench_status_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_chip_text_rect(
    rect: &FrameRect,
) -> FrameRect {
    let metrics = workbench_status_metrics();
    let line_height = metrics.line_height;
    let inset = metrics.text_inset;
    FrameRect {
        x: rect.x + inset,
        y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.width - inset * 2.0).max(0.0),
        height: line_height,
    }
}
