use super::model::WorkbenchToastStyle;
use super::palette::{
    toast_normal_style_from_palette, workbench_toast_palette, WorkbenchToastPalette,
};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_state_style(
    state: UiPainterResolvedState,
) -> WorkbenchToastStyle {
    toast_state_style_from_palette(state, workbench_toast_palette())
}

fn toast_state_style_from_palette(
    state: UiPainterResolvedState,
    palette: WorkbenchToastPalette,
) -> WorkbenchToastStyle {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => WorkbenchToastStyle {
            surface: palette.disabled_surface,
            border: palette.disabled_border,
            text: palette.disabled_text,
            mark: palette.disabled_text,
            action: palette.disabled_text,
            close: palette.disabled_text,
            state,
        },
        UiPainterResolvedState::Pressed => {
            let mut style = toast_normal_style_from_palette(state, palette);
            style.surface = palette.pressed_surface;
            style.border = palette.action;
            style.action = palette.action;
            style
        }
        UiPainterResolvedState::Focused => {
            let mut style = toast_normal_style_from_palette(state, palette);
            style.border = palette.focus_border;
            style
        }
        UiPainterResolvedState::Open => {
            let mut style = toast_normal_style_from_palette(state, palette);
            style.border = palette.action;
            style.action = palette.action;
            style
        }
        UiPainterResolvedState::Hovered
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => {
            let mut style = toast_normal_style_from_palette(state, palette);
            style.surface = palette.hover_surface;
            style.border = palette.hover_border;
            style
        }
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => toast_normal_style_from_palette(state, palette),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_toast_state(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

#[cfg(test)]
mod tests {
    use super::super::palette::workbench_toast_palette_from_host;
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn toast_unavailable_state_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.surface_disabled = [10, 11, 12, 255];
        palette.border_disabled = [13, 14, 15, 255];
        palette.text_disabled = [16, 17, 18, 255];

        let style = toast_state_style_from_palette(
            UiPainterResolvedState::Loading,
            workbench_toast_palette_from_host(palette),
        );

        assert_eq!(style.surface, [10, 11, 12, 255]);
        assert_eq!(style.border, [13, 14, 15, 255]);
        assert_eq!(style.text, [16, 17, 18, 255]);
        assert_eq!(style.mark, [16, 17, 18, 255]);
        assert_eq!(style.action, [16, 17, 18, 255]);
        assert_eq!(style.close, [16, 17, 18, 255]);
    }

    #[test]
    fn toast_open_and_pressed_states_project_active_colors_from_host_palette() {
        let mut palette = PALETTE;
        palette.accent = [20, 21, 22, 255];
        palette.surface_pressed = [23, 24, 25, 255];

        let toast_palette = workbench_toast_palette_from_host(palette);
        let open = toast_state_style_from_palette(UiPainterResolvedState::Open, toast_palette);
        let pressed =
            toast_state_style_from_palette(UiPainterResolvedState::Pressed, toast_palette);

        assert_eq!(open.border, [20, 21, 22, 255]);
        assert_eq!(open.action, [20, 21, 22, 255]);
        assert_eq!(pressed.surface, [23, 24, 25, 255]);
        assert_eq!(pressed.border, [20, 21, 22, 255]);
        assert_eq!(pressed.action, [20, 21, 22, 255]);
    }

    #[test]
    fn toast_focused_state_keeps_normal_surface_with_focus_border() {
        let mut palette = PALETTE;
        palette.accent_soft = [10, 11, 12, 247];
        palette.focus_ring = [20, 21, 22, 255];

        let style = toast_state_style_from_palette(
            UiPainterResolvedState::Focused,
            workbench_toast_palette_from_host(palette),
        );

        assert_eq!(style.surface, [10, 11, 12, 247]);
        assert_eq!(style.border, [20, 21, 22, 255]);
    }

    #[test]
    fn toast_hover_state_projects_surface_and_border_from_host_palette() {
        let mut palette = PALETTE;
        palette.surface_selected = [30, 31, 32, 255];
        palette.accent_soft = [33, 34, 35, 247];

        let style = toast_state_style_from_palette(
            UiPainterResolvedState::Hovered,
            workbench_toast_palette_from_host(palette),
        );

        assert_eq!(style.surface, [30, 31, 32, 255]);
        assert_eq!(style.border, [33, 34, 35, 247]);
    }
}
