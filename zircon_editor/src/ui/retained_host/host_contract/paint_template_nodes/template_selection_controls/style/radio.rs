use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::style_selector::WorkbenchSelectionControlKind as SelectionStyleKind;
use super::selector::selection_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn radio_background(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Radio).surface
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn radio_border_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Radio).border
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn control_accent_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Radio).accent
}
