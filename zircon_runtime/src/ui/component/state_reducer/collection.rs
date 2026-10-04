use std::collections::{btree_map::Entry, BTreeMap};

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
    let updated = {
        let values = array_value_mut(state, &property);
        if let Some(existing) = values.get_mut(index) {
            *existing = value;
            true
        } else {
            false
        }
    };
    if !updated {
        state.validation = UiValidationState::error(format!(
            "array property `{property}` has no element at index {index}"
        ));
        return Err(UiComponentEventError::ArrayIndexOutOfBounds { property, index });
    }
    clear_reference_source(state, &property);
    Ok(())
}

pub(super) fn remove_array_element(
    state: &mut UiComponentState,
    property: String,
    index: usize,
) -> Result<(), UiComponentEventError> {
    let removed = {
        let values = array_value_mut(state, &property);
        if index < values.len() {
            values.remove(index);
            true
        } else {
            false
        }
    };
    if !removed {
        state.validation = UiValidationState::error(format!(
            "array property `{property}` has no element at index {index}"
        ));
        return Err(UiComponentEventError::ArrayIndexOutOfBounds { property, index });
    }
    clear_reference_source(state, &property);
    Ok(())
}

pub(super) fn move_array_element(
    state: &mut UiComponentState,
    property: String,
    from: usize,
    to: usize,
) -> Result<(), UiComponentEventError> {
    let moved = {
        let values = array_value_mut(state, &property);
        if from >= values.len() {
            false
        } else {
            let value = values.remove(from);
            values.insert(to.min(values.len()), value);
            true
        }
    };
    if !moved {
        state.validation = UiValidationState::error(format!(
            "array property `{property}` has no element at index {from}"
        ));
        return Err(UiComponentEventError::ArrayIndexOutOfBounds {
            property,
            index: from,
        });
    }
    clear_reference_source(state, &property);
    Ok(())
}

pub(super) fn add_map_entry(
    state: &mut UiComponentState,
    property: String,
    key: String,
    value: UiValue,
) -> Result<(), UiComponentEventError> {
    let duplicate_key = {
        let values = map_value_mut(state, &property);
        match values.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(value);
                None
            }
            Entry::Occupied(entry) => Some(entry.key().clone()),
        }
    };
    if let Some(key) = duplicate_key {
        state.validation = UiValidationState::error(format!("map key `{key}` already exists"));
        return Err(UiComponentEventError::DuplicateMapKey { property, key });
    }
    clear_reference_source(state, &property);
    Ok(())
}

pub(super) fn set_map_entry(
    state: &mut UiComponentState,
    property: String,
    key: String,
    value: UiValue,
) -> Result<(), UiComponentEventError> {
    let updated = {
        let values = map_value_mut(state, &property);
        if let Some(existing) = values.get_mut(&key) {
            *existing = value;
            true
        } else {
            false
        }
    };
    if !updated {
        state.validation = UiValidationState::error(format!("map key `{key}` does not exist"));
        return Err(UiComponentEventError::MissingMapKey { property, key });
    }
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
    let error = {
        let values = map_value_mut(state, &property);
        if values.contains_key(&to_key) {
            Some(UiComponentEventError::DuplicateMapKey {
                property: property.clone(),
                key: to_key,
            })
        } else if !values.contains_key(&from_key) {
            Some(UiComponentEventError::MissingMapKey {
                property: property.clone(),
                key: from_key,
            })
        } else {
            let value = values
                .remove(&from_key)
                .expect("map key was verified before rename");
            values.insert(to_key, value);
            None
        }
    };
    if let Some(error) = error {
        let message = match &error {
            UiComponentEventError::DuplicateMapKey { key, .. } => {
                format!("map key `{key}` already exists")
            }
            UiComponentEventError::MissingMapKey { key, .. } => {
                format!("map key `{key}` does not exist")
            }
            _ => unreachable!("rename only reports map-key validation errors"),
        };
        state.validation = UiValidationState::error(message);
        return Err(error);
    }
    clear_reference_source(state, &property);
    Ok(())
}

pub(super) fn remove_map_entry(
    state: &mut UiComponentState,
    property: String,
    key: String,
) -> Result<(), UiComponentEventError> {
    let removed = map_value_mut(state, &property).remove(&key).is_some();
    if !removed {
        state.validation = UiValidationState::error(format!("map key `{key}` does not exist"));
        return Err(UiComponentEventError::MissingMapKey { property, key });
    }
    clear_reference_source(state, &property);
    Ok(())
}

fn clear_reference_source(state: &mut UiComponentState, property: &str) {
    state.reference_sources.remove(property);
}

// 这里先把缺失或异型属性规范为空数组；后续索引失败仍可能留下该规范化结果，只有成功变更才清除引用来源。
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

#[cfg(test)]
#[path = "collection/tests/map_single_resolution_tests.rs"]
mod map_single_resolution_tests;

#[cfg(test)]
#[path = "collection/tests/mutation_single_resolution_tests.rs"]
mod mutation_single_resolution_tests;

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
