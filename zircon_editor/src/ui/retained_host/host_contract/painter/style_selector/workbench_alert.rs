use super::super::super::data::TemplatePaneNodeData;
use super::super::template_style::resolved_style_color;
use super::super::theme::PALETTE;
use super::painter_state_for_node;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_ALERT_INFO_SURFACE: [u8;
    4] = [18, 46, 72, 255];
const WORKBENCH_ALERT_INFO_BORDER: [u8; 4] = [41, 101, 150, 255];
const WORKBENCH_ALERT_SUCCESS_SURFACE: [u8; 4] = [22, 57, 39, 255];
const WORKBENCH_ALERT_SUCCESS_BORDER: [u8; 4] = [53, 115, 72, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_ALERT_WARNING_SURFACE:
    [u8; 4] = [69, 50, 20, 255];
const WORKBENCH_ALERT_WARNING_BORDER: [u8; 4] = [132, 94, 35, 255];
const WORKBENCH_ALERT_ERROR_SURFACE: [u8; 4] = [72, 32, 36, 255];
const WORKBENCH_ALERT_ERROR_BORDER: [u8; 4] = [133, 61, 58, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::painter) enum WorkbenchAlertTone {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::painter) struct WorkbenchAlertStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub mark: [u8; 4],
    pub text: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::painter) fn select_workbench_alert_style(
    node: &TemplatePaneNodeData,
    tone: WorkbenchAlertTone,
) -> WorkbenchAlertStyle {
    let state = painter_state_for_node(node).resolved_state_for_family(UiPainterFamily::Alert);
    let mut style = alert_state_style(tone, state);

    if state != UiPainterResolvedState::Disabled {
        if let Some(surface) =
            resolved_style_color(node.button_style.element.background_color.as_ref())
        {
            style.surface = surface;
        }
        if let Some(border) = resolved_style_color(node.button_style.element.border_color.as_ref())
        {
            style.border = border;
        }
        if let Some(text) =
            resolved_style_color(node.button_style.element.foreground_color.as_ref())
        {
            style.text = text;
        }
        if let Some(mark) =
            declared_color(node.icon_color).or_else(|| declared_color(node.label_color))
        {
            style.mark = mark;
        }
    }

    style
}

fn alert_state_style(
    tone: WorkbenchAlertTone,
    state: UiPainterResolvedState,
) -> WorkbenchAlertStyle {
    match state {
        UiPainterResolvedState::Disabled => WorkbenchAlertStyle {
            surface: PALETTE.surface_disabled,
            border: PALETTE.border_disabled,
            mark: PALETTE.text_disabled,
            text: PALETTE.text_disabled,
            state,
        },
        UiPainterResolvedState::Pressed | UiPainterResolvedState::Focused => {
            let mut style = alert_tone_style(tone, state);
            style.border = PALETTE.focus_ring;
            style
        }
        UiPainterResolvedState::Loading => {
            let mut style = alert_tone_style(tone, state);
            style.mark = PALETTE.text_muted;
            style.text = PALETTE.text_muted;
            style
        }
        UiPainterResolvedState::Hovered
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => alert_tone_style(tone, state),
    }
}

fn alert_tone_style(
    tone: WorkbenchAlertTone,
    state: UiPainterResolvedState,
) -> WorkbenchAlertStyle {
    let (surface, border, mark) = match tone {
        WorkbenchAlertTone::Info => (
            WORKBENCH_ALERT_INFO_SURFACE,
            WORKBENCH_ALERT_INFO_BORDER,
            PALETTE.info,
        ),
        WorkbenchAlertTone::Success => (
            WORKBENCH_ALERT_SUCCESS_SURFACE,
            WORKBENCH_ALERT_SUCCESS_BORDER,
            PALETTE.success,
        ),
        WorkbenchAlertTone::Warning => (
            WORKBENCH_ALERT_WARNING_SURFACE,
            WORKBENCH_ALERT_WARNING_BORDER,
            PALETTE.warning,
        ),
        WorkbenchAlertTone::Error => (
            WORKBENCH_ALERT_ERROR_SURFACE,
            WORKBENCH_ALERT_ERROR_BORDER,
            PALETTE.error,
        ),
    };
    WorkbenchAlertStyle {
        surface,
        border,
        mark,
        text: mark,
        state,
    }
}

fn declared_color(color: Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}
