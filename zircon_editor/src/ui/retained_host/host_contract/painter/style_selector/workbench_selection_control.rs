use super::super::super::data::TemplatePaneNodeData;
use super::super::template_style::resolved_style_color;
use super::super::theme::PALETTE;
use super::painter_state_for_node;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_SELECTION_LABEL_MUTED:
    [u8; 4] = [130, 140, 147, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_SELECTION_MARK_IDLE_FILL:
    [u8; 4] = [20, 26, 30, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_SELECTION_MARK_IDLE_BORDER:
    [u8; 4] = [66, 78, 86, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHECKBOX_CHECKED_FILL:
    [u8; 4] = [32, 159, 168, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_RADIO_CHECKED_FILL: [u8;
    4] = [27, 39, 45, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_RADIO_CHECKED_BORDER:
    [u8; 4] = [76, 91, 99, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::painter) enum WorkbenchSelectionControlKind {
    Checkbox,
    Radio,
    Toggle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::painter) struct WorkbenchSelectionControlStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub thumb: [u8; 4],
    pub accent: [u8; 4],
    pub text: [u8; 4],
    pub label: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::painter) fn select_workbench_selection_control_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSelectionControlKind,
) -> WorkbenchSelectionControlStyle {
    let state = painter_state_for_node(node).resolved_state_for_family(family_for_kind(kind));
    let checked = node.checked || node.selected;
    WorkbenchSelectionControlStyle {
        surface: control_surface(node, kind, state, checked),
        border: control_border(node, kind, state, checked),
        thumb: toggle_thumb(node, state, checked),
        accent: control_accent(node, state),
        text: selection_text(node, state),
        label: mark_label(node, state),
        state,
    }
}

fn family_for_kind(kind: WorkbenchSelectionControlKind) -> UiPainterFamily {
    match kind {
        WorkbenchSelectionControlKind::Checkbox => UiPainterFamily::Checkbox,
        WorkbenchSelectionControlKind::Radio => UiPainterFamily::Radio,
        WorkbenchSelectionControlKind::Toggle => UiPainterFamily::Toggle,
    }
}

fn control_surface(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSelectionControlKind,
    state: UiPainterResolvedState,
    checked: bool,
) -> [u8; 4] {
    if is_disabled(state) {
        return PALETTE.surface_disabled;
    }
    match kind {
        WorkbenchSelectionControlKind::Checkbox => {
            if checked {
                declared_style_background(node).unwrap_or(WORKBENCH_CHECKBOX_CHECKED_FILL)
            } else {
                declared_style_background(node).unwrap_or(WORKBENCH_SELECTION_MARK_IDLE_FILL)
            }
        }
        WorkbenchSelectionControlKind::Radio => {
            if checked {
                declared_style_background(node).unwrap_or(WORKBENCH_RADIO_CHECKED_FILL)
            } else {
                declared_style_background(node).unwrap_or(WORKBENCH_SELECTION_MARK_IDLE_FILL)
            }
        }
        WorkbenchSelectionControlKind::Toggle => {
            if checked {
                declared_style_background(node).unwrap_or(PALETTE.accent)
            } else if state == UiPainterResolvedState::Pressed {
                PALETTE.surface_pressed
            } else if is_hot(state) {
                PALETTE.surface_hover
            } else {
                declared_style_background(node).unwrap_or(PALETTE.track)
            }
        }
    }
}

fn control_border(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSelectionControlKind,
    state: UiPainterResolvedState,
    checked: bool,
) -> [u8; 4] {
    if is_disabled(state) {
        return PALETTE.border_disabled;
    }
    match kind {
        WorkbenchSelectionControlKind::Checkbox => {
            if is_hot(state) {
                PALETTE.focus_ring
            } else if checked {
                declared_style_border(node)
                    .or_else(|| declared_style_background(node))
                    .unwrap_or(WORKBENCH_CHECKBOX_CHECKED_FILL)
            } else {
                declared_style_border(node).unwrap_or(WORKBENCH_SELECTION_MARK_IDLE_BORDER)
            }
        }
        WorkbenchSelectionControlKind::Radio => {
            if is_hot(state) {
                PALETTE.focus_ring
            } else if checked {
                declared_style_border(node).unwrap_or(WORKBENCH_RADIO_CHECKED_BORDER)
            } else {
                declared_style_border(node).unwrap_or(WORKBENCH_SELECTION_MARK_IDLE_BORDER)
            }
        }
        WorkbenchSelectionControlKind::Toggle => {
            if checked || is_hot(state) {
                declared_style_border(node).unwrap_or(PALETTE.accent)
            } else {
                declared_style_border(node).unwrap_or(PALETTE.border)
            }
        }
    }
}

fn toggle_thumb(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    checked: bool,
) -> [u8; 4] {
    if is_disabled(state) {
        PALETTE.text_disabled
    } else if checked {
        declared_style_foreground(node).unwrap_or(PALETTE.text)
    } else {
        declared_style_foreground(node).unwrap_or(PALETTE.text_muted)
    }
}

fn control_accent(node: &TemplatePaneNodeData, state: UiPainterResolvedState) -> [u8; 4] {
    if is_disabled(state) {
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

fn selection_text(_node: &TemplatePaneNodeData, state: UiPainterResolvedState) -> [u8; 4] {
    if is_disabled(state) {
        PALETTE.text_disabled
    } else {
        PALETTE.text
    }
}

fn mark_label(node: &TemplatePaneNodeData, state: UiPainterResolvedState) -> [u8; 4] {
    if is_disabled(state) {
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

fn is_disabled(state: UiPainterResolvedState) -> bool {
    state == UiPainterResolvedState::Disabled
}

pub(in crate::ui::retained_host::host_contract::painter) fn is_workbench_selection_state_hot(
    state: UiPainterResolvedState,
) -> bool {
    is_hot(state)
}

fn is_hot(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
}

fn declared_style_background(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .filter(|color| color[3] > 0)
}

fn declared_style_border(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .filter(|color| color[3] > 0)
}

fn declared_style_foreground(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.foreground_color.as_ref())
        .filter(|color| color[3] > 0)
}
