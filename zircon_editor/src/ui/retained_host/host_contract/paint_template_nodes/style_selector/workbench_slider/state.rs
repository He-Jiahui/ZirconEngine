use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_slider_state_hot(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
}

pub(super) fn slider_state_shows_thumb_halo(state: UiPainterResolvedState) -> bool {
    state == UiPainterResolvedState::Focused || is_workbench_slider_state_hot(state)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_slider_state(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}
