use super::super::super::data::TemplatePaneNodeData;
use super::super::template_style::resolved_style_color;
use super::super::theme::PALETTE;
use super::painter_state_for_node;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_TOAST_SURFACE: [u8; 4] =
    [21, 48, 53, 247];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_TOAST_BORDER: [u8; 4] =
    [53, 199, 208, 20];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_TOAST_TEXT: [u8; 4] =
    [206, 224, 226, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_TOAST_ACTION: [u8; 4] =
    [53, 199, 208, 255];

const WORKBENCH_TOAST_HOVER_SURFACE: [u8; 4] = [24, 58, 63, 247];
const WORKBENCH_TOAST_PRESSED_SURFACE: [u8; 4] = [16, 60, 74, 247];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::painter) struct WorkbenchToastStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub mark: [u8; 4],
    pub action: [u8; 4],
    pub close: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::painter) fn select_workbench_toast_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchToastStyle {
    let state = painter_state_for_node(node).resolved_state_for_family(UiPainterFamily::Toast);
    let mut style = toast_state_style(state);

    if let Some(surface) = resolved_style_color(node.button_style.element.background_color.as_ref())
    {
        style.surface = surface;
    }
    if let Some(border) = resolved_style_color(node.button_style.element.border_color.as_ref()) {
        style.border = border;
    }
    if let Some(text) = resolved_style_color(node.button_style.element.foreground_color.as_ref()) {
        style.text = text;
    }
    if state != UiPainterResolvedState::Disabled {
        if let Some(mark) = declared_color(node.label_color) {
            style.mark = mark;
        }
        if let Some(action) = declared_color(node.value_color) {
            style.action = action;
        }
    }

    style
}

fn toast_state_style(state: UiPainterResolvedState) -> WorkbenchToastStyle {
    match state {
        UiPainterResolvedState::Disabled => WorkbenchToastStyle {
            surface: PALETTE.surface_disabled,
            border: PALETTE.border_disabled,
            text: PALETTE.text_disabled,
            mark: PALETTE.text_disabled,
            action: PALETTE.text_disabled,
            close: PALETTE.text_disabled,
            state,
        },
        UiPainterResolvedState::Pressed => {
            let mut style = toast_normal_style(state);
            style.surface = WORKBENCH_TOAST_PRESSED_SURFACE;
            style.border = PALETTE.focus_ring;
            style.action = PALETTE.focus_ring;
            style
        }
        UiPainterResolvedState::Focused | UiPainterResolvedState::Open => {
            let mut style = toast_normal_style(state);
            style.border = PALETTE.focus_ring;
            style.action = PALETTE.focus_ring;
            style
        }
        UiPainterResolvedState::Hovered
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => {
            let mut style = toast_normal_style(state);
            style.surface = WORKBENCH_TOAST_HOVER_SURFACE;
            style.border = PALETTE.accent_soft;
            style
        }
        UiPainterResolvedState::Loading => {
            let mut style = toast_normal_style(state);
            style.text = PALETTE.text_muted;
            style.close = PALETTE.text_muted;
            style
        }
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => toast_normal_style(state),
    }
}

fn toast_normal_style(state: UiPainterResolvedState) -> WorkbenchToastStyle {
    WorkbenchToastStyle {
        surface: WORKBENCH_TOAST_SURFACE,
        border: WORKBENCH_TOAST_BORDER,
        text: WORKBENCH_TOAST_TEXT,
        mark: WORKBENCH_TOAST_ACTION,
        action: WORKBENCH_TOAST_ACTION,
        close: PALETTE.text_muted,
        state,
    }
}

fn declared_color(color: Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}
