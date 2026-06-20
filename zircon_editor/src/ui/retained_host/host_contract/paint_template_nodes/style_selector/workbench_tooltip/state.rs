use super::model::WorkbenchTooltipStyle;
use super::palette::tooltip_normal_style;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_state_style(
    state: UiPainterResolvedState,
) -> WorkbenchTooltipStyle {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            WorkbenchTooltipStyle {
                surface: PALETTE.surface_disabled,
                border: PALETTE.border_disabled,
                title: PALETTE.text_disabled,
                body: PALETTE.text_disabled,
                arrow: PALETTE.surface_disabled,
                icon: PALETTE.text_disabled,
                shadow: [0, 0, 0, 48],
                state,
            }
        }
        UiPainterResolvedState::Pressed | UiPainterResolvedState::Focused => {
            let mut style = tooltip_normal_style(state);
            style.border = PALETTE.focus_ring;
            style.icon = PALETTE.focus_ring;
            style.title = PALETTE.text;
            style
        }
        UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => {
            let mut style = tooltip_normal_style(state);
            style.border = PALETTE.border;
            style.icon = PALETTE.accent;
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
