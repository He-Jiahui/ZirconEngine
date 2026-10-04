//! 不可用警报覆盖语义色调；其余状态沿用对应警报语义色，避免交互反馈改写警报级别。

use super::model::{WorkbenchAlertStyle, WorkbenchAlertTone};
use super::palette::{
    alert_tone_style_from_palette, workbench_alert_palette, WorkbenchAlertPalette,
};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_state_style(
    tone: WorkbenchAlertTone,
    state: UiPainterResolvedState,
) -> WorkbenchAlertStyle {
    alert_state_style_from_palette(tone, state, workbench_alert_palette())
}

fn alert_state_style_from_palette(
    tone: WorkbenchAlertTone,
    state: UiPainterResolvedState,
    palette: WorkbenchAlertPalette,
) -> WorkbenchAlertStyle {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => WorkbenchAlertStyle {
            surface: palette.disabled_surface,
            border: palette.disabled_border,
            mark: palette.disabled_text,
            text: palette.disabled_text,
            state,
        },
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Hovered
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => alert_tone_style_from_palette(tone, state, palette),
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

#[cfg(test)]
#[path = "tests/state.rs"]
mod tests;
