use super::super::super::data::FrameRect;
use super::super::template_row_metrics::workbench_row_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_adornment_rect(
    rect: &FrameRect,
) -> FrameRect {
    let metrics = workbench_row_metrics();
    FrameRect {
        x: rect.x + rect.width - metrics.list_adornment_right_inset - metrics.list_adornment_size,
        y: rect.y + (rect.height - metrics.list_adornment_size).max(0.0) * 0.5,
        width: metrics.list_adornment_size,
        height: metrics.list_adornment_size,
    }
}
