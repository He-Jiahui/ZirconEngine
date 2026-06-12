use zircon_runtime_interface::ui::component::UiComponentState;

pub(super) fn focus(state: &mut UiComponentState, focused: bool) {
    state.flags.focused = focused;
}

pub(super) fn hover(state: &mut UiComponentState, hovered: bool) {
    state.flags.hovered = hovered;
}

pub(super) fn press(state: &mut UiComponentState, pressed: bool) {
    state.flags.pressed = pressed;
}

pub(super) fn begin_drag(state: &mut UiComponentState) {
    state.flags.dragging = true;
}

pub(super) fn end_drag(state: &mut UiComponentState) {
    state.flags.dragging = false;
}

pub(super) fn drop_hover(state: &mut UiComponentState, hovered: bool) {
    state.flags.drop_hovered = hovered;
}

pub(super) fn active_drag_target(state: &mut UiComponentState, active: bool) {
    state.flags.active_drag_target = active;
}
