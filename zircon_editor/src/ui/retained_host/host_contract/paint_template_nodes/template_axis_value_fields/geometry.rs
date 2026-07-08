use super::super::super::data::FrameRect;
use super::metrics::axis_value_field_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_rect(
    rect: &FrameRect,
) -> FrameRect {
    let metrics = axis_value_field_metrics();
    let height = rect.height.min(metrics.max_height).round().max(0.0);
    FrameRect {
        x: rect.x.round(),
        y: (rect.y + (rect.height - height).max(0.0) * 0.5).round(),
        width: rect.width.round().max(0.0),
        height,
    }
}
