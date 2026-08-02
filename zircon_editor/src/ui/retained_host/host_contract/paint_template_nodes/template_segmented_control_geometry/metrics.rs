use crate::ui::retained_host::host_contract::paint_theme::{
    HostControlMetrics, current_host_metrics,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSegmentedControlMetrics
{
    pub segment_font_size: f32,
    pub segment_line_height: f32,
    pub segment_text_inset_x: f32,
    pub segment_text_inset_y: f32,
    pub segment_radius: f32,
    pub segment_group_label_font_size: f32,
    pub segment_group_label_line_height: f32,
    pub segment_group_label_height: f32,
    pub segment_group_label_gap: f32,
    pub segment_selected_inset: f32,
    pub segment_divider_width: f32,
    pub segment_divider_inset_y: f32,
    pub tab_font_size: f32,
    pub tab_line_height: f32,
    pub tab_underline_height: f32,
    pub tab_text_inset_x: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_segmented_control_metrics()
-> WorkbenchSegmentedControlMetrics {
    workbench_segmented_control_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_segmented_control_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchSegmentedControlMetrics {
    let segment_font_size = metrics.font_body;
    let segment_group_label_font_size = metrics.font_body;
    let tab_font_size = metrics.font_body;
    let segment_group_label_line_height = metrics.line_height(segment_group_label_font_size);
    WorkbenchSegmentedControlMetrics {
        segment_font_size,
        segment_line_height: metrics.line_height(segment_font_size),
        segment_text_inset_x: metrics.input_pad[0],
        segment_text_inset_y: metrics.segment_text_inset_y,
        segment_radius: metrics.radius_control,
        segment_group_label_font_size,
        segment_group_label_line_height,
        segment_group_label_height: segment_group_label_line_height + metrics.border_width * 2.0,
        segment_group_label_gap: metrics.gap_s,
        segment_selected_inset: metrics.segment_selected_inset,
        segment_divider_width: metrics.border_width,
        segment_divider_inset_y: metrics.gap_s,
        tab_font_size,
        tab_line_height: metrics.line_height(tab_font_size),
        tab_underline_height: metrics.tab_underline_height,
        tab_text_inset_x: metrics.button_pad_x,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_font_size() -> f32
{
    workbench_segmented_control_metrics().segment_font_size
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_text_inset_x()
-> f32 {
    workbench_segmented_control_metrics().segment_text_inset_x
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_text_inset_y()
-> f32 {
    workbench_segmented_control_metrics().segment_text_inset_y
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_radius() -> f32 {
    workbench_segmented_control_metrics().segment_radius
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_group_label_font_size()
-> f32 {
    workbench_segmented_control_metrics().segment_group_label_font_size
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_group_label_height()
-> f32 {
    workbench_segmented_control_metrics().segment_group_label_height
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_font_size() -> f32 {
    workbench_segmented_control_metrics().tab_font_size
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_underline_height()
-> f32 {
    workbench_segmented_control_metrics().tab_underline_height
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_group_label_gap()
-> f32 {
    workbench_segmented_control_metrics().segment_group_label_gap
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_selected_inset()
-> f32 {
    workbench_segmented_control_metrics().segment_selected_inset
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_divider_width()
-> f32 {
    workbench_segmented_control_metrics().segment_divider_width
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_divider_inset_y()
-> f32 {
    workbench_segmented_control_metrics().segment_divider_inset_y
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_text_inset_x() -> f32 {
    workbench_segmented_control_metrics().tab_text_inset_x
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_line_height() -> f32
{
    workbench_segmented_control_metrics().segment_line_height
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_group_label_line_height()
-> f32 {
    workbench_segmented_control_metrics().segment_group_label_line_height
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_line_height() -> f32 {
    workbench_segmented_control_metrics().tab_line_height
}
