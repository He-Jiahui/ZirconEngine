//! 将节点数值声明映射为归一化进度、范围下限和安全刻度数量；范围值兼容百分数与 0..1 表示。

use super::super::super::data::TemplatePaneNodeData;
use super::metrics::workbench_slider_metrics;
use zircon_runtime_interface::ui::surface::bounded_ui_slider_tick_count;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_percent(
    node: &TemplatePaneNodeData,
) -> f32 {
    if node.value_percent.is_finite() {
        node.value_percent.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// RangeSlider 身份即使下限为 0 也保留双滑块语义；非范围滑块只有正的第二单元偏移才启用下限。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_range_min_percent(
    node: &TemplatePaneNodeData,
) -> Option<f32> {
    let is_range_row = node.control_id.as_str().contains("RangeSlider");
    if !is_range_row && node.layout_second_cell_offset_x <= 0.0 {
        return None;
    }
    Some(slider_declared_percent(node.layout_second_cell_offset_x))
}

/// 共享运行时接口限制声明刻度数量；StepsSlider 缺少声明时用固定五刻度回退。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_tick_count(
    node: &TemplatePaneNodeData,
) -> Option<usize> {
    bounded_ui_slider_tick_count(node.layout_third_cell_offset_x).or_else(|| {
        node.control_id
            .as_str()
            .contains("StepsSlider")
            .then_some(5)
    })
}

/// 将主值与可选下限排序为轨道填充区间；不改变节点原始两端值。
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
        workbench_slider_metrics().thumb_size
    }
}

fn slider_declared_percent(value: f32) -> f32 {
    if value > 1.0 {
        (value / 100.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}
