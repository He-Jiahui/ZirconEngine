use zircon_runtime_interface::ui::component::{UiComponentEventError, UiComponentState, UiValue};

pub(super) fn open_popup(state: &mut UiComponentState) -> Result<(), UiComponentEventError> {
    state.flags.popup_open = true;
    Ok(())
}

pub(super) fn open_popup_at(
    state: &mut UiComponentState,
    x: f64,
    y: f64,
) -> Result<(), UiComponentEventError> {
    state.flags.popup_open = true;
    super::set_value(state, "popup_anchor_x".to_string(), UiValue::Float(x));
    super::set_value(state, "popup_anchor_y".to_string(), UiValue::Float(y));
    Ok(())
}

pub(super) fn close_popup(state: &mut UiComponentState) -> Result<(), UiComponentEventError> {
    state.flags.popup_open = false;
    Ok(())
}
