use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiValidationState, UiValue,
    UiValueKind,
};

#[cfg(test)]
mod borrowed_option_id_tests;

pub(super) fn apply_selection(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: String,
    option_id: String,
    selected: bool,
) -> Result<(), UiComponentEventError> {
    if option_is_disabled(state, descriptor, &option_id) {
        state.validation =
            UiValidationState::error(format!("disabled option `{option_id}` cannot be selected"));
        return Err(UiComponentEventError::DisabledOption {
            component_id: descriptor.id.clone(),
            option_id,
        });
    }

    let is_flags = descriptor
        .prop(&property)
        .is_some_and(|schema| schema.value_kind == UiValueKind::Flags);
    let is_multiple = bool_setting(state, descriptor, "multiple", false);

    clear_reference_source(state, &property);
    if is_flags {
        let mut values = selection_flags_value(state, &property);
        if selected {
            if !values.iter().any(|value| value == &option_id) {
                values.push(option_id);
            }
        } else {
            values.retain(|value| value != &option_id);
        }
        state.values.insert(property, UiValue::Flags(values));
    } else if is_multiple {
        let values = selection_array_value_mut(state, &property);
        if selected {
            if !values
                .iter()
                .any(|value| enum_option_id_matches(value, &option_id))
            {
                values.push(UiValue::Enum(option_id));
            }
        } else {
            values.retain(|value| !enum_option_id_matches(value, &option_id));
        }
    } else if selected {
        state.values.insert(property, UiValue::Enum(option_id));
    } else {
        state.values.insert(property, UiValue::Null);
    }
    state.flags.selected = selected;
    Ok(())
}

fn clear_reference_source(state: &mut UiComponentState, property: &str) {
    state.reference_sources.remove(property);
}

fn selection_array_value_mut<'a>(
    state: &'a mut UiComponentState,
    property: &str,
) -> &'a mut Vec<UiValue> {
    if !matches!(state.values.get(property), Some(UiValue::Array(_))) {
        let values = match state.values.remove(property) {
            Some(UiValue::Array(values)) => values,
            Some(UiValue::Enum(value)) if !value.is_empty() => vec![UiValue::Enum(value)],
            Some(UiValue::String(value)) if !value.is_empty() => vec![UiValue::String(value)],
            Some(UiValue::Null) | None => Vec::new(),
            Some(value) => vec![value],
        };
        state
            .values
            .insert(property.to_string(), UiValue::Array(values));
    }
    match state.values.get_mut(property) {
        Some(UiValue::Array(values)) => values,
        _ => unreachable!("selection array value was inserted before mutable access"),
    }
}

fn selection_flags_value(state: &mut UiComponentState, property: &str) -> Vec<String> {
    match state.values.remove(property) {
        Some(UiValue::Flags(values)) => values,
        Some(UiValue::Array(values)) => values
            .into_iter()
            .filter_map(|value| match value {
                UiValue::Enum(value) | UiValue::String(value) if !value.is_empty() => Some(value),
                _ => None,
            })
            .collect(),
        Some(UiValue::Enum(value)) | Some(UiValue::String(value)) if !value.is_empty() => {
            vec![value]
        }
        _ => Vec::new(),
    }
}

fn bool_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    default_value: bool,
) -> bool {
    match state.values.get(property) {
        Some(UiValue::Bool(value)) => *value,
        _ => descriptor
            .prop(property)
            .and_then(|schema| schema.default_value.as_ref())
            .and_then(|value| match value {
                UiValue::Bool(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(default_value),
    }
}

fn enum_option_id_matches(value: &UiValue, option_id: &str) -> bool {
    matches!(value, UiValue::Enum(value) if value == option_id)
}

pub(super) fn option_is_disabled(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    option_id: &str,
) -> bool {
    descriptor
        .prop("options")
        .and_then(|schema| schema.options.iter().find(|option| option.id == option_id))
        .is_some_and(|option| option.disabled)
        || state
            .values
            .get("disabled_options")
            .is_some_and(|value| option_id_list_contains(value, option_id))
}

fn option_id_list_contains(value: &UiValue, option_id: &str) -> bool {
    match value {
        UiValue::Array(values) => values
            .iter()
            .any(|value| option_id_list_contains(value, option_id)),
        UiValue::String(value) | UiValue::Enum(value) => value == option_id,
        UiValue::Flags(values) => values.iter().any(|value| value == option_id),
        _ => false,
    }
}
