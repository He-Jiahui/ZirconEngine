use super::super::super::data::TemplatePaneNodeData;
use super::metrics::SLIDER_THUMB_SIZE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_percent(
    node: &TemplatePaneNodeData,
) -> f32 {
    if node.value_percent.is_finite() {
        node.value_percent.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_range_min_percent(
    node: &TemplatePaneNodeData,
) -> Option<f32> {
    let is_range_row = node.control_id.as_str().contains("RangeSlider");
    if !is_range_row && node.layout_second_cell_offset_x <= 0.0 {
        return None;
    }
    Some(slider_declared_percent(node.layout_second_cell_offset_x))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_tick_count(
    node: &TemplatePaneNodeData,
) -> Option<usize> {
    let declared = node.layout_third_cell_offset_x.round() as usize;
    if declared >= 2 {
        Some(declared)
    } else if node.control_id.as_str().contains("StepsSlider") {
        Some(5)
    } else {
        None
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_fill_span(
    percent: f32,
    range_min_percent: Option<f32>,
) -> (f32, f32) {
    let end = percent.clamp(0.0, 1.0);
    let start = range_min_percent.unwrap_or(0.0).clamp(0.0, 1.0);
    if start <= end {
        (start, end)
    } else {
        (end, start)
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_thumb_size(
    node: &TemplatePaneNodeData,
) -> f32 {
    if node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        SLIDER_THUMB_SIZE
    }
}

fn slider_declared_percent(value: f32) -> f32 {
    if value > 1.0 {
        (value / 100.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}
