use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{
    select_workbench_selection_control_style, WorkbenchSelectionControlKind as SelectionStyleKind,
    WorkbenchSelectionControlStyle,
};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn checkbox_background(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Checkbox).surface
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn radio_background(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Radio).surface
}

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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn checkbox_border_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Checkbox).border
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selection_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Toggle).text
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selection_mark_label_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Checkbox).label
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selection_visual_state(
    node: &TemplatePaneNodeData,
) -> UiPainterResolvedState {
    selection_style(node, SelectionStyleKind::Checkbox).state
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selection_visual_unavailable(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        selection_visual_state(node),
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

fn selection_style(
    node: &TemplatePaneNodeData,
    kind: SelectionStyleKind,
) -> WorkbenchSelectionControlStyle {
    select_workbench_selection_control_style(node, kind)
}
