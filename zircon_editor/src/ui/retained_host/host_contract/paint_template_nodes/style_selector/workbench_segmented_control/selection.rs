//! 供分段与页签模板绘制前选择视觉通道；调用端提供控件类型，并决定何处绘制选中项标记。

use super::super::resolved_state_for_node;
use super::control::{control_background, control_border, control_border_width};
use super::model::{WorkbenchSegmentedControlKind, WorkbenchSegmentedControlStyle};
use super::segments::{
    selected_segment_border_color, selected_segment_border_width, selected_segment_surface_color,
    selected_segment_underline_color, selected_segment_underline_height,
};
use super::text::{group_label_color, idle_text_color, selected_text_color};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterFamily;

/// 接收节点与调用端识别的类别，生成外壳、选中项和组标签配方；不修改选择状态。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_segmented_control_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSegmentedControlKind,
) -> WorkbenchSegmentedControlStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Tab);
    let tab_text = if kind == WorkbenchSegmentedControlKind::Tab
        && !matches!(
            state,
            zircon_runtime_interface::ui::style::UiPainterResolvedState::Disabled
                | zircon_runtime_interface::ui::style::UiPainterResolvedState::Loading
        ) {
        Some(
            crate::ui::retained_host::host_contract::paint_theme::current_host_palette()
                .text_tab_active,
        )
    } else {
        None
    };
    let mut border = control_border(node, kind, state);
    let border_width = if kind == WorkbenchSegmentedControlKind::Tab && border.is_some() {
        node.authored_focus_border_width
            .filter(|value| value.is_finite() && *value >= 0.0)
            .unwrap_or_else(|| {
                crate::ui::retained_host::host_contract::paint_theme::current_host_metrics()
                    .border_width
            })
    } else {
        control_border_width(kind)
    };
    if kind == WorkbenchSegmentedControlKind::Tab && border_width <= 0.0 {
        border = None;
    }
    WorkbenchSegmentedControlStyle {
        background: control_background(node, kind, state),
        border,
        border_width,
        selected_surface: selected_segment_surface_color(state),
        selected_border: selected_segment_border_color(state),
        selected_border_width: selected_segment_border_width(node),
        selected_underline: selected_segment_underline_color(node, state),
        selected_underline_height: selected_segment_underline_height(node),
        selected_text: tab_text.unwrap_or_else(|| selected_text_color(state)),
        idle_text: if kind == WorkbenchSegmentedControlKind::Tab && node.hovered {
            tab_text.unwrap_or_else(|| idle_text_color(state))
        } else {
            idle_text_color(state)
        },
        group_label: group_label_color(node, state),
        state,
    }
}
