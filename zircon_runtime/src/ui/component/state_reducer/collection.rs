use std::collections::BTreeMap;

use zircon_runtime_interface::ui::component::{
    UiComponentEventError, UiComponentState, UiValidationState, UiValue,
};

pub(super) fn add_element(state: &mut UiComponentState, property: String, value: UiValue) {
    clear_reference_source(state, &property);
    array_value_mut(state, &property).push(value);
}

pub(super) fn set_array_element(
    state: &mut UiComponentState,
    property: String,
    index: usize,
    value: UiValue,
) -> Result<(), UiComponentEventError> {
    if index >= array_value_mut(state, &property).len() {
        state.validation = UiValidationState::error(format!(
            "array property `{property}` has no element at index {index}"
        ));
        return Err(UiComponentEventError::ArrayIndexOutOfBounds { property, index });
    }
    array_value_mut(state, &property)[index] = value;
    clear_reference_source(state, &property);
    Ok(())
}

pub(super) fn remove_array_element(
    state: &mut UiComponentState,
    property: String,
    index: usize,
) -> Result<(), UiComponentEventError> {
    if index >= array_value_mut(state, &property).len() {
        state.validation = UiValidationState::error(format!(
            "array property `{property}` has no element at index {index}"
        ));
        return Err(UiComponentEventError::ArrayIndexOutOfBounds { property, index });
    }
    array_value_mut(state, &property).remove(index);
    clear_reference_source(state, &property);
    Ok(())
}

pub(super) fn move_array_element(
    state: &mut UiComponentState,
    property: String,
    from: usize,
    to: usize,
) -> Result<(), UiComponentEventError> {
    if from >= array_value_mut(state, &property).len() {
        state.validation = UiValidationState::error(format!(
            "array property `{property}` has no element at index {from}"
        ));
        return Err(UiComponentEventError::ArrayIndexOutOfBounds {
            property,
            index: from,
        });
    }
    let values = array_value_mut(state, &property);
    let value = values.remove(from);
    values.insert(to.min(values.len()), value);
    clear_reference_source(state, &property);
    Ok(())
}

pub(super) fn add_map_entry(
    state: &mut UiComponentState,
    property: String,
    key: String,
    value: UiValue,
) -> Result<(), UiComponentEventError> {
    if map_value_mut(state, &property).contains_key(&key) {
        state.validation = UiValidationState::error(format!("map key `{key}` already exists"));
        return Err(UiComponentEventError::DuplicateMapKey { property, key });
    }
    map_value_mut(state, &property).insert(key, value);
    clear_reference_source(state, &property);
    Ok(())
}

pub(super) fn set_map_entry(
    state: &mut UiComponentState,
    property: String,
    key: String,
    value: UiValue,
) -> Result<(), UiComponentEventError> {
    if !map_value_mut(state, &property).contains_key(&key) {
        state.validation = UiValidationState::error(format!("map key `{key}` does not exist"));
        return Err(UiComponentEventError::MissingMapKey { property, key });
    }
    map_value_mut(state, &property).insert(key, value);
    clear_reference_source(state, &property);
    Ok(())
}

pub(super) fn rename_map_key(
    state: &mut UiComponentState,
    property: String,
    from_key: String,
    to_key: String,
) -> Result<(), UiComponentEventError> {
    if from_key == to_key {
        return Ok(());
    }
    if map_value_mut(state, &property).contains_key(&to_key) {
        state.validation = UiValidationState::error(format!("map key `{to_key}` already exists"));
        return Err(UiComponentEventError::DuplicateMapKey {
            property,
            key: to_key,
        });
    }
    if !map_value_mut(state, &property).contains_key(&from_key) {
        state.validation = UiValidationState::error(format!("map key `{from_key}` does not exist"));
        return Err(UiComponentEventError::MissingMapKey {
            property,
            key: from_key,
        });
    }
    let values = map_value_mut(state, &property);
    let value = values
        .remove(&from_key)
        .expect("map key was verified before rename");
    values.insert(to_key, value);
    clear_reference_source(state, &property);
    Ok(())
}

pub(super) fn remove_map_entry(
    state: &mut UiComponentState,
    property: String,
    key: String,
) -> Result<(), UiComponentEventError> {
    if !map_value_mut(state, &property).contains_key(&key) {
        state.validation = UiValidationState::error(format!("map key `{key}` does not exist"));
        return Err(UiComponentEventError::MissingMapKey { property, key });
    }
    map_value_mut(state, &property).remove(&key);
    clear_reference_source(state, &property);
    Ok(())
}

fn clear_reference_source(state: &mut UiComponentState, property: &str) {
    state.reference_sources.remove(property);
}

fn array_value_mut<'a>(state: &'a mut UiComponentState, property: &str) -> &'a mut Vec<UiValue> {
    if !matches!(state.values.get(property), Some(UiValue::Array(_))) {
        state
            .values
            .insert(property.to_string(), UiValue::Array(Vec::new()));
    }
    match state.values.get_mut(property) {
        Some(UiValue::Array(values)) => values,
        _ => unreachable!("array value was inserted before mutable access"),
    }
}

fn map_value_mut<'a>(
    state: &'a mut UiComponentState,
    property: &str,
) -> &'a mut BTreeMap<String, UiValue> {
    if !matches!(state.values.get(property), Some(UiValue::Map(_))) {
        state
            .values
            .insert(property.to_string(), UiValue::Map(BTreeMap::new()));
    }
    match state.values.get_mut(property) {
        Some(UiValue::Map(values)) => values,
        _ => unreachable!("map value was inserted before mutable access"),
    }
}
