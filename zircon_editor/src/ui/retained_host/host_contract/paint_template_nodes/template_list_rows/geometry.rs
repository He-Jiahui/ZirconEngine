use super::super::super::data::FrameRect;
use super::super::template_row_metrics::workbench_row_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_list_row_extent(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_has_adornment_space(
    rect: &FrameRect,
) -> bool {
    let metrics = workbench_row_metrics();
    rect.width >= metrics.list_adornment_right_inset + metrics.list_adornment_size
        && rect.height >= metrics.list_adornment_size
}
