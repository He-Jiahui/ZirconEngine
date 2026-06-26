use super::super::super::template_style_color::resolved_style_color;
use super::palette::WORKBENCH_SELECTION_LABEL_MUTED;
use super::state::is_unavailable_selection_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toggle_thumb(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    checked: bool,
) -> [u8; 4] {
    if is_unavailable_selection_state(state) {
        PALETTE.text_disabled
    } else if checked {
        PALETTE.text
    } else {
        declared_style_foreground(node).unwrap_or(PALETTE.text_muted)
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn control_accent(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    if is_unavailable_selection_state(state) {
        PALETTE.text_disabled
    } else if node.value_color.a > 0 {
        [
            node.value_color.r,
            node.value_color.g,
            node.value_color.b,
            node.value_color.a,
        ]
    } else {
        PALETTE.accent
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selection_text(
    _node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    if is_unavailable_selection_state(state) {
        PALETTE.text_disabled
    } else {
        PALETTE.text
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn mark_label(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    if is_unavailable_selection_state(state) {
        PALETTE.text_disabled
    } else if node.label_color.a > 0 {
        [
            node.label_color.r,
            node.label_color.g,
            node.label_color.b,
            node.label_color.a,
        ]
    } else {
        WORKBENCH_SELECTION_LABEL_MUTED
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn declared_style_background(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .filter(|color| color[3] > 0)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn declared_style_border(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .filter(|color| color[3] > 0)
}

fn declared_style_foreground(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.foreground_color.as_ref())
        .filter(|color| color[3] > 0)
}
