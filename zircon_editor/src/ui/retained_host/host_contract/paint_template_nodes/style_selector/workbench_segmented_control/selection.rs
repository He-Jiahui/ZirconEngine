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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_segmented_control_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSegmentedControlKind,
) -> WorkbenchSegmentedControlStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Tab);
    WorkbenchSegmentedControlStyle {
        background: control_background(node, kind, state),
        border: control_border(kind, state),
        border_width: control_border_width(kind),
        selected_surface: selected_segment_surface_color(state),
        selected_border: selected_segment_border_color(state),
        selected_border_width: selected_segment_border_width(node),
        selected_underline: selected_segment_underline_color(node, state),
        selected_underline_height: selected_segment_underline_height(node),
        selected_text: selected_text_color(state),
        idle_text: idle_text_color(state),
        group_label: group_label_color(node, state),
        state,
    }
}
