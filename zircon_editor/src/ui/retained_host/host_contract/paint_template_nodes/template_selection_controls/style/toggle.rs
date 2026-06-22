use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::style_selector::WorkbenchSelectionControlKind as SelectionStyleKind;
use super::selector::selection_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toggle_track_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Toggle).surface
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toggle_thumb_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Toggle).thumb
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn control_border_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Toggle).border
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selection_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Toggle).text
}
