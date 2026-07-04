use super::model::WorkbenchTooltipStyle;
use super::palette::{tooltip_normal_style, tooltip_palette};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_state_style(
    state: UiPainterResolvedState,
) -> WorkbenchTooltipStyle {
    let palette = tooltip_palette();
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
        UiPainterResolvedState::Pressed | UiPainterResolvedState::Focused => {
            let mut style = tooltip_normal_style(state);
            style.border = palette.focused_border;
            style.icon = palette.focused_border;
            style.title = palette.title;
            style
        }
        UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => {
            let mut style = tooltip_normal_style(state);
            style.border = palette.border;
            style.icon = palette.hover_icon;
            style
        }
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => tooltip_normal_style(state),
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
