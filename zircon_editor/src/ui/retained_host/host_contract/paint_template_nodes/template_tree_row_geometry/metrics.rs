use super::super::template_row_metrics::{
    workbench_row_metrics, workbench_row_palette, WorkbenchRowMetrics,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_metrics(
) -> WorkbenchRowMetrics {
    workbench_row_metrics()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_font_size() -> f32 {
    tree_metrics().text_font_size
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_radius() -> f32 {
    tree_metrics().surface_radius
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_guide_color(
) -> [u8; 4] {
    workbench_row_palette().tree_guide
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_guide_opacity() -> f32
{
    tree_metrics().tree_guide_opacity
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_line_height() -> f32 {
    tree_metrics().text_line_height
}
