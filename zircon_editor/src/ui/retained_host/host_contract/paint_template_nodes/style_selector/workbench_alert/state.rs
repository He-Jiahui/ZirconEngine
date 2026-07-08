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
        UiPainterResolvedState::Pressed => {
            let mut style = alert_tone_style_from_palette(tone, state, palette);
            style.border = palette.active_border;
            style
        }
        UiPainterResolvedState::Focused => alert_tone_style_from_palette(tone, state, palette),
        UiPainterResolvedState::Hovered
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
mod tests {
    use super::super::palette::workbench_alert_palette_from_host;
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn alert_unavailable_state_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.surface_disabled = [10, 11, 12, 255];
        palette.border_disabled = [13, 14, 15, 255];
        palette.text_disabled = [16, 17, 18, 255];

        let style = alert_state_style_from_palette(
            WorkbenchAlertTone::Warning,
            UiPainterResolvedState::Loading,
            workbench_alert_palette_from_host(palette),
        );

        assert_eq!(style.surface, [10, 11, 12, 255]);
        assert_eq!(style.border, [13, 14, 15, 255]);
        assert_eq!(style.mark, [16, 17, 18, 255]);
        assert_eq!(style.text, [16, 17, 18, 255]);
    }

    #[test]
    fn alert_pressed_state_projects_active_border_from_host_palette() {
        let mut palette = PALETTE;
        palette.warning_container = [20, 21, 22, 255];
        palette.warning = [23, 24, 25, 255];
        palette.focus_ring = [26, 27, 28, 255];

        let style = alert_state_style_from_palette(
            WorkbenchAlertTone::Warning,
            UiPainterResolvedState::Pressed,
            workbench_alert_palette_from_host(palette),
        );

        assert_eq!(style.surface, [20, 21, 22, 255]);
        assert_eq!(style.border, [26, 27, 28, 255]);
        assert_eq!(style.mark, [23, 24, 25, 255]);
        assert_eq!(style.text, [23, 24, 25, 255]);
    }

    #[test]
    fn alert_focused_state_keeps_tone_border_from_host_palette() {
        let mut palette = PALETTE;
        palette.warning_container = [30, 31, 32, 255];
        palette.warning = [33, 34, 35, 255];
        palette.focus_ring = [36, 37, 38, 255];

        let style = alert_state_style_from_palette(
            WorkbenchAlertTone::Warning,
            UiPainterResolvedState::Focused,
            workbench_alert_palette_from_host(palette),
        );

        assert_eq!(style.surface, [30, 31, 32, 255]);
        assert_eq!(style.border, [33, 34, 35, 255]);
        assert_ne!(style.border, [36, 37, 38, 255]);
    }
}
