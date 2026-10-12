//! 为选中项提供表面、边线与下划线配方；选择器总会返回这些通道，绘制端负责仅给选中项输出。
//! 边宽的存在标志保留显式零值；下划线高度只有有限正值覆盖主题默认值，非正值不表示关闭下划线。

use super::metrics::workbench_segmented_selector_metrics;
use super::palette::workbench_segmented_control_palette;
use super::state::is_unavailable_segmented_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_surface_color(
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_segmented_control_palette();
    if is_unavailable_segmented_state(state) {
        palette.disabled_background
    } else {
        palette.selected_background
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_border_color(
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_segmented_control_palette();
    if is_unavailable_segmented_state(state) {
        palette.disabled_border
    } else {
        palette.selected_border
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    if node.has_selected_segment_border_width {
        finite_non_negative(node.selected_segment_border_width).unwrap_or(0.0)
    } else {
        0.0
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_underline_height(
    node: &TemplatePaneNodeData,
) -> f32 {
    let height = finite_non_negative(node.selected_segment_underline_height).unwrap_or(0.0);
    if height > 0.0 {
        height
    } else {
        workbench_segmented_selector_metrics().selected_underline_height
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_underline_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_segmented_control_palette();
    if is_unavailable_segmented_state(state) {
        palette.disabled_text
    } else if node.selected_segment_underline_color.a > 0 {
        [
            node.selected_segment_underline_color.r,
            node.selected_segment_underline_color.g,
            node.selected_segment_underline_color.b,
            node.selected_segment_underline_color.a,
        ]
    } else {
        palette.selected_underline
    }
}

fn finite_non_negative(value: f32) -> Option<f32> {
    value.is_finite().then_some(value.max(0.0))
}
