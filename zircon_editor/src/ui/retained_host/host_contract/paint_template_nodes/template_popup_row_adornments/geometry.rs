use super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_row_adornment_rect(
    row_rect: &FrameRect,
) -> FrameRect {
    let metrics = super::super::template_popup_rows::metrics::workbench_popup_row_metrics();
    FrameRect {
        x: row_rect.x + row_rect.width - metrics.adornment_right - metrics.adornment_size,
        y: row_rect.y + (row_rect.height - metrics.adornment_size).max(0.0) * 0.5,
        width: metrics.adornment_size,
        height: metrics.adornment_size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn local_rect(
    origin: &FrameRect,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> FrameRect {
    FrameRect {
        x: origin.x + x,
        y: origin.y + y,
        width,
        height,
    }
}
