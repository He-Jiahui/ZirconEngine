use super::super::super::super::data::FrameRect;
use super::super::metrics::workbench_slider_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_range_min_value_rect(
    rect: &FrameRect,
    track_rect: &FrameRect,
) -> Option<FrameRect> {
    let metrics = workbench_slider_metrics();
    if rect.height < metrics.range_value_min_height || track_rect.width < metrics.value_width {
        return None;
    }
    Some(FrameRect {
        x: track_rect.x,
        y: track_rect.y + metrics.range_value_y_offset,
        width: metrics.value_width,
        height: metrics.range_value_height,
    })
}
