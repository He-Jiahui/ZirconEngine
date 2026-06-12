use zircon_runtime_interface::ui::component::{UiComponentEventError, UiComponentState, UiValue};

pub(super) fn toggle_expanded(
    state: &mut UiComponentState,
    expanded: bool,
) -> Result<(), UiComponentEventError> {
    state.flags.expanded = expanded;
    super::set_value(state, "expanded".to_string(), UiValue::Bool(expanded));
    Ok(())
}
