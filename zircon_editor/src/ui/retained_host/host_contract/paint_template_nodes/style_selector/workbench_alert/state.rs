use super::model::{WorkbenchAlertStyle, WorkbenchAlertTone};
use super::palette::alert_tone_style;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_state_style(
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_alert_state(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}
