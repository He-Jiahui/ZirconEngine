use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiValue,
};

#[cfg(test)]
mod borrowed_confirm_action_tests;

pub(super) fn open_popup(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentEventError> {
    state.flags.popup_open = true;
    set_open_values(state, descriptor, true);
    Ok(())
}

pub(super) fn open_popup_at(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    x: f64,
    y: f64,
) -> Result<(), UiComponentEventError> {
    open_popup(state, descriptor)?;
    super::set_value(state, "popup_anchor_x".to_string(), UiValue::Float(x));
    super::set_value(state, "popup_anchor_y".to_string(), UiValue::Float(y));
    Ok(())
}

pub(super) fn close_popup(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<(), UiComponentEventError> {
    if confirm_requires_explicit_action(state, descriptor) {
        return Ok(());
    }

    close_popup_unchecked(state, descriptor);
    Ok(())
}

pub(super) fn apply_cancel(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Result<bool, UiComponentEventError> {
    if !is_dialog_control(descriptor) {
        return Ok(false);
    }
    if bool_setting(state, descriptor, "disable_escape_key_down", false) {
        return Ok(true);
    }
    if confirm_requires_explicit_action(state, descriptor) {
        return Ok(true);
    }

    close_popup_unchecked(state, descriptor);
    Ok(true)
}

pub(super) fn apply_commit(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    value: &UiValue,
) -> Result<bool, UiComponentEventError> {
    if !is_dialog_control(descriptor) {
        return Ok(false);
    }

    let action_id = action_id_from_commit(state, descriptor, property, value);
    let confirmed = is_confirm_action(state, descriptor, &action_id);
    if confirmed && !bool_setting(state, descriptor, "confirm_enabled", true) {
        return Ok(true);
    }

    super::set_value(
        state,
        "dialog_action_id".to_string(),
        UiValue::String(action_id),
    );
    if descriptor.prop("confirmed").is_some() || state.values.contains_key("confirmed") {
        super::set_value(state, "confirmed".to_string(), UiValue::Bool(confirmed));
    }
    close_popup_unchecked(state, descriptor);
    Ok(true)
}

fn close_popup_unchecked(state: &mut UiComponentState, descriptor: &UiComponentDescriptor) {
    state.flags.popup_open = false;
    set_open_values(state, descriptor, false);
}

fn set_open_values(state: &mut UiComponentState, descriptor: &UiComponentDescriptor, open: bool) {
    for property in ["popup_open", "popupOpen", "open"] {
        if descriptor.prop(property).is_some() || state.values.contains_key(property) {
            super::set_value(state, property.to_string(), UiValue::Bool(open));
        }
    }
}

fn is_dialog_control(descriptor: &UiComponentDescriptor) -> bool {
    matches!(
        descriptor.role.as_str(),
        "dialog" | "modal" | "confirm-dialog"
    )
}

fn is_confirm_dialog(descriptor: &UiComponentDescriptor) -> bool {
    descriptor.role == "confirm-dialog"
}

fn confirm_requires_explicit_action(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> bool {
    is_confirm_dialog(descriptor)
        && bool_setting(state, descriptor, "requires_explicit_action", true)
}

fn action_id_from_commit(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    value: &UiValue,
) -> String {
    match value {
        UiValue::String(action_id) | UiValue::Enum(action_id) => action_id.clone(),
        UiValue::Bool(true) if property == "confirmed" => {
            string_setting_ref(state, descriptor, "confirm_action_id")
                .map(str::to_owned)
                .unwrap_or_else(|| "confirm".to_string())
        }
        UiValue::Bool(false) if property == "confirmed" => {
            string_setting_ref(state, descriptor, "cancel_action_id")
                .map(str::to_owned)
                .unwrap_or_else(|| "cancel".to_string())
        }
        _ => property.to_string(),
    }
}

fn is_confirm_action(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    action_id: &str,
) -> bool {
    let confirm_action =
        string_setting_ref(state, descriptor, "confirm_action_id").unwrap_or("confirm");
    action_id == confirm_action || action_id == "confirm"
}

fn bool_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    default_value: bool,
) -> bool {
    state
        .values
        .get(property)
        .and_then(|value| match value {
            UiValue::Bool(value) => Some(*value),
            _ => None,
        })
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(|value| match value {
                    UiValue::Bool(value) => Some(*value),
                    _ => None,
                })
        })
        .unwrap_or(default_value)
}

fn string_setting_ref<'a>(
    state: &'a UiComponentState,
    descriptor: &'a UiComponentDescriptor,
    property: &str,
) -> Option<&'a str> {
    state
        .values
        .get(property)
        .and_then(string_value_ref)
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(string_value_ref)
        })
}

fn string_value_ref(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.as_str()),
        _ => None,
    }
}
