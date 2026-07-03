use super::super::super::super::data::FrameRect;
use super::super::metrics::workbench_slider_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_value_rect(
    rect: &FrameRect,
) -> Option<FrameRect> {
    let metrics = workbench_slider_metrics();
    if rect.width < metrics.value_min_width {
        return None;
    }
    let height = (rect.height - metrics.value_height_pad)
        .clamp(metrics.value_min_height, metrics.value_max_height);
    Some(FrameRect {
        x: rect.x + rect.width - metrics.horizontal_inset - metrics.value_width,
        y: rect.y + (rect.height - height).max(0.0) * 0.5,
        width: metrics.value_width,
        height,
    })
}
