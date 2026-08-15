use super::model::WorkbenchTooltipStyle;
use super::palette::{tooltip_normal_style_from_palette, tooltip_palette, WorkbenchTooltipPalette};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_state_style(
    state: UiPainterResolvedState,
) -> WorkbenchTooltipStyle {
    tooltip_state_style_from_palette(state, tooltip_palette())
}

pub(super) fn tooltip_state_style_from_palette(
    state: UiPainterResolvedState,
    palette: WorkbenchTooltipPalette,
) -> WorkbenchTooltipStyle {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            WorkbenchTooltipStyle {
                surface: palette.disabled_surface,
                border: palette.disabled_border,
                title: palette.disabled_text,
                body: palette.disabled_text,
                arrow: palette.disabled_surface,
                icon: palette.disabled_text,
                shadow: palette.disabled_shadow,
                state,
            }
        }
        UiPainterResolvedState::Pressed => {
            let mut style = tooltip_normal_style_from_palette(state, palette);
            style.border = palette.focused_border;
            style.icon = palette.focused_border;
            style.title = palette.title;
            style
        }
        UiPainterResolvedState::Focused => {
            let mut style = tooltip_normal_style_from_palette(state, palette);
            style.border = palette.focused_border;
            style
        }
        UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => {
            let mut style = tooltip_normal_style_from_palette(state, palette);
            style.border = palette.border;
            style.icon = palette.hover_icon;
            style
        }
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => tooltip_normal_style_from_palette(state, palette),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_tooltip_state(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}
