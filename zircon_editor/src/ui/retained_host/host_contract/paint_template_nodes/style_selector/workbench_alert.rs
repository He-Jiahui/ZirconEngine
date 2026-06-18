use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;
use super::super::template_style_color::resolved_style_color;
use super::resolved_state_for_node;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_ALERT_INFO_SURFACE: [u8;
    4] = [18, 46, 72, 255];
const WORKBENCH_ALERT_INFO_BORDER: [u8; 4] = [41, 101, 150, 255];
const WORKBENCH_ALERT_SUCCESS_SURFACE: [u8; 4] = [22, 57, 39, 255];
const WORKBENCH_ALERT_SUCCESS_BORDER: [u8; 4] = [53, 115, 72, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_ALERT_WARNING_SURFACE:
    [u8; 4] = [69, 50, 20, 255];
const WORKBENCH_ALERT_WARNING_BORDER: [u8; 4] = [132, 94, 35, 255];
const WORKBENCH_ALERT_ERROR_SURFACE: [u8; 4] = [72, 32, 36, 255];
const WORKBENCH_ALERT_ERROR_BORDER: [u8; 4] = [133, 61, 58, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchAlertTone {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchAlertStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub mark: [u8; 4],
    pub text: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_alert_style(
    node: &TemplatePaneNodeData,
    tone: WorkbenchAlertTone,
) -> WorkbenchAlertStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Alert);
    let mut style = alert_state_style(tone, state);

    if !is_unavailable_alert_state(state) {
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
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => WorkbenchAlertStyle {
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
        UiPainterResolvedState::Hovered
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => alert_tone_style(tone, state),
    }
}

fn is_unavailable_alert_state(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn alert_loading_state_uses_unavailable_visuals() {
        let mut node = TemplatePaneNodeData::default();
        node.hovered = true;
        node.focused = true;
        node.pressed = true;
        node.button_style.loading = true;
        node.icon_color = Color::from_rgb_u8(224, 163, 58);
        node.label_color = Color::from_rgb_u8(208, 217, 221);
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(69, 50, 20, 255)));
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(132, 94, 35, 255)));
        node.button_style.element.foreground_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(224, 163, 58, 255)));

        let style = select_workbench_alert_style(&node, WorkbenchAlertTone::Warning);

        assert_eq!(style.state, UiPainterResolvedState::Loading);
        assert_eq!(style.surface, PALETTE.surface_disabled);
        assert_eq!(style.border, PALETTE.border_disabled);
        assert_eq!(style.mark, PALETTE.text_disabled);
        assert_eq!(style.text, PALETTE.text_disabled);
    }
}
