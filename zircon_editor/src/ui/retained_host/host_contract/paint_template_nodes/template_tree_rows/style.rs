use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{select_workbench_tree_row_style, WorkbenchTreeRowStyle};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_background(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    tree_row_style(node).background
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_border(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    tree_row_style(node).border
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    tree_row_style(node).border_width
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    tree_row_style(node).text
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_icon_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    tree_row_style(node).icon
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_secondary_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    tree_row_style(node).secondary
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_action_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    tree_row_style(node).action
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_state(
    node: &TemplatePaneNodeData,
) -> UiPainterResolvedState {
    tree_row_style(node).state
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchTreeRowStyle {
    select_workbench_tree_row_style(node)
}
