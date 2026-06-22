use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::style_selector::WorkbenchSelectionControlKind as SelectionStyleKind;
use super::selector::selection_style;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn checkbox_background(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Checkbox).surface
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn checkbox_border_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Checkbox).border
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
