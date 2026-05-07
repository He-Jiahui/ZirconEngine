use std::collections::BTreeMap;

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEvent, UiComponentEventError, UiComponentEventKind,
    UiComponentState, UiDragPayloadKind, UiValidationState, UiValue, UiValueKind,
};

pub fn apply_component_event(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    event: UiComponentEvent,
) -> Result<(), UiComponentEventError> {
    ensure_event_supported(descriptor, event.kind())?;

    let result = match event {
        UiComponentEvent::ValueChanged { property, value }
        | UiComponentEvent::Commit { property, value } => {
            apply_value(state, descriptor, property, value)
        }
        UiComponentEvent::Focus { focused } => {
            state.flags.focused = focused;
            Ok(())
        }
        UiComponentEvent::Hover { hovered } => {
            state.flags.hovered = hovered;
            Ok(())
        }
        UiComponentEvent::Press { pressed } => {
            state.flags.pressed = pressed;
            Ok(())
        }
        UiComponentEvent::BeginDrag { .. } => {
            state.flags.dragging = true;
            Ok(())
        }
        UiComponentEvent::DragDelta { property, delta } => {
            apply_numeric_drag(state, descriptor, property, delta, "step")
        }
        UiComponentEvent::LargeDragDelta { property, delta } => {
            apply_numeric_drag(state, descriptor, property, delta, "large_step")
        }
        UiComponentEvent::EndDrag { .. } => {
            state.flags.dragging = false;
            Ok(())
        }
        UiComponentEvent::DropHover { hovered } => {
            state.flags.drop_hovered = hovered;
            Ok(())
        }
        UiComponentEvent::ActiveDragTarget { active } => {
            state.flags.active_drag_target = active;
            Ok(())
        }
        UiComponentEvent::OpenPopup => {
            state.flags.popup_open = true;
            Ok(())
        }
        UiComponentEvent::OpenPopupAt { x, y } => {
            state.flags.popup_open = true;
            set_value(state, "popup_anchor_x".to_string(), UiValue::Float(x));
            set_value(state, "popup_anchor_y".to_string(), UiValue::Float(y));
            Ok(())
        }
        UiComponentEvent::ClosePopup => {
            state.flags.popup_open = false;
            Ok(())
        }
        UiComponentEvent::SelectOption {
            property,
            option_id,
            selected,
        } => apply_selection(state, descriptor, property, option_id, selected),
        UiComponentEvent::ToggleExpanded { expanded } => {
            state.flags.expanded = expanded;
            set_value(state, "expanded".to_string(), UiValue::Bool(expanded));
            Ok(())
        }
        UiComponentEvent::AddElement { property, value } => {
            clear_reference_source(state, &property);
            array_value_mut(state, &property).push(value);
            Ok(())
        }
        UiComponentEvent::SetElement {
            property,
            index,
            value,
        } => set_array_element(state, property, index, value),
        UiComponentEvent::RemoveElement { property, index } => {
            remove_array_element(state, property, index)
        }
        UiComponentEvent::MoveElement { property, from, to } => {
            move_array_element(state, property, from, to)
        }
        UiComponentEvent::AddMapEntry {
            property,
            key,
            value,
        } => add_map_entry(state, property, key, value),
        UiComponentEvent::SetMapEntry {
            property,
            key,
            value,
        } => set_map_entry(state, property, key, value),
        UiComponentEvent::RenameMapKey {
            property,
            from_key,
            to_key,
        } => rename_map_key(state, property, from_key, to_key),
        UiComponentEvent::RemoveMapEntry { property, key } => {
            remove_map_entry(state, property, key)
        }
        UiComponentEvent::DropReference { property, payload } => {
            if !descriptor.accepts_drag_payload(payload.kind) {
                state.validation = UiValidationState::error(format!(
                    "rejected drop payload `{}` for {}",
                    payload.kind.as_str(),
                    descriptor.id
                ));
                return Err(UiComponentEventError::RejectedDrop {
                    component_id: descriptor.id.clone(),
                    payload_kind: payload.kind.as_str().to_string(),
                });
            }
            let source = payload.source.clone();
            let value = match payload.kind {
                UiDragPayloadKind::Asset => UiValue::AssetRef(payload.reference),
                UiDragPayloadKind::SceneInstance | UiDragPayloadKind::Object => {
                    UiValue::InstanceRef(payload.reference)
                }
            };
            if let Some(source) = source {
                state.reference_sources.insert(property.clone(), source);
            } else {
                state.reference_sources.remove(&property);
            }
            state.values.insert(property, value);
            Ok(())
        }
        UiComponentEvent::ClearReference { property } => {
            state.reference_sources.remove(&property);
            state.values.insert(property, UiValue::Null);
            Ok(())
        }
        UiComponentEvent::LocateReference { property }
        | UiComponentEvent::OpenReference { property } => ensure_reference_value(state, property),
        UiComponentEvent::SetVisibleRange { start, count } => {
            apply_visible_range(state, start, count)
        }
        UiComponentEvent::SetPage {
            page_index,
            page_size,
        } => apply_page_window(state, page_index, page_size),
        UiComponentEvent::SetWorldTransform {
            position,
            rotation,
            scale,
        } => apply_world_transform(state, position, rotation, scale),
        UiComponentEvent::SetWorldSurface {
            size,
            pixels_per_meter,
            billboard,
            depth_test,
            render_order,
            camera_target,
        } => apply_world_surface(
            state,
            descriptor,
            size,
            pixels_per_meter,
            billboard,
            depth_test,
            render_order,
            camera_target,
        ),
    };

    if result.is_ok() {
        state.validation = UiValidationState::normal();
    }
    result
}

pub trait UiComponentStateRuntimeExt {
    fn apply_event(
        &mut self,
        descriptor: &UiComponentDescriptor,
        event: UiComponentEvent,
    ) -> Result<(), UiComponentEventError>;
}

impl UiComponentStateRuntimeExt for UiComponentState {
    fn apply_event(
        &mut self,
        descriptor: &UiComponentDescriptor,
        event: UiComponentEvent,
    ) -> Result<(), UiComponentEventError> {
        apply_component_event(self, descriptor, event)
    }
}

fn apply_value(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: String,
    value: UiValue,
) -> Result<(), UiComponentEventError> {
    let Some(schema) = descriptor.prop(&property) else {
        set_value(state, property, value);
        return Ok(());
    };
    let normalized = match schema.value_kind {
        UiValueKind::Float | UiValueKind::Int => {
            let numeric = value.as_f64().ok_or_else(|| {
                let message = value.display_text();
                state.validation =
                    UiValidationState::error(format!("invalid numeric value `{message}`"));
                UiComponentEventError::InvalidNumericValue {
                    property: property.clone(),
                    value: message,
                }
            })?;
            numeric_value(
                schema.value_kind,
                clamp_numeric(
                    numeric,
                    optional_numeric_setting(state, descriptor, "min", schema.min),
                    optional_numeric_setting(state, descriptor, "max", schema.max),
                ),
            )
        }
        _ if value_kind_matches(schema.value_kind, value.kind()) => value,
        _ => {
            let actual = value.kind();
            state.validation = UiValidationState::error(format!(
                "invalid value kind `{actual:?}` for `{property}`; expected `{:?}`",
                schema.value_kind
            ));
            return Err(UiComponentEventError::InvalidValueKind {
                property,
                expected: schema.value_kind,
                actual,
            });
        }
    };
    set_value(state, property, normalized);
    Ok(())
}

fn apply_numeric_drag(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: String,
    delta: f64,
    step_property: &str,
) -> Result<(), UiComponentEventError> {
    let Some(schema) = descriptor.prop(&property) else {
        return Err(UiComponentEventError::NonNumericProperty { property });
    };
    if !matches!(schema.value_kind, UiValueKind::Float | UiValueKind::Int) {
        return Err(UiComponentEventError::NonNumericProperty { property });
    }
    let current = state
        .values
        .get(&property)
        .or(schema.default_value.as_ref())
        .and_then(UiValue::as_f64)
        .unwrap_or(0.0);
    let step = numeric_setting(state, descriptor, step_property, schema.step, 1.0);
    let next = clamp_numeric(
        current + delta * step,
        optional_numeric_setting(state, descriptor, "min", schema.min),
        optional_numeric_setting(state, descriptor, "max", schema.max),
    );
    set_value(state, property, numeric_value(schema.value_kind, next));
    Ok(())
}

fn apply_selection(
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
                .any(|value| value == &UiValue::Enum(option_id.clone()))
            {
                values.push(UiValue::Enum(option_id));
            }
        } else {
            values.retain(|value| value != &UiValue::Enum(option_id.clone()));
        }
    } else if selected {
        state.values.insert(property, UiValue::Enum(option_id));
    } else {
        state.values.insert(property, UiValue::Null);
    }
    state.flags.selected = selected;
    Ok(())
}

fn apply_visible_range(
    state: &mut UiComponentState,
    start: i64,
    count: i64,
) -> Result<(), UiComponentEventError> {
    let total_count = int_value(state, "total_count", 0).max(0);
    let viewport_start = start.clamp(0, total_count);
    let available_count = total_count.saturating_sub(viewport_start);
    let viewport_count = count.max(0).min(available_count);
    let visible_end = viewport_start
        .saturating_add(viewport_count)
        .min(total_count);
    let overscan = int_value(state, "overscan", 0).max(0);
    let requested_start = viewport_start.saturating_sub(overscan);
    let requested_end = visible_end.saturating_add(overscan).min(total_count);
    let requested_count = requested_end.saturating_sub(requested_start);
    let item_extent = float_value(state, "item_extent", 0.0).max(0.0);

    set_value(
        state,
        "viewport_start".to_string(),
        UiValue::Int(viewport_start),
    );
    set_value(
        state,
        "viewport_count".to_string(),
        UiValue::Int(viewport_count),
    );
    set_value(state, "visible_end".to_string(), UiValue::Int(visible_end));
    set_value(
        state,
        "requested_start".to_string(),
        UiValue::Int(requested_start),
    );
    set_value(
        state,
        "requested_count".to_string(),
        UiValue::Int(requested_count),
    );
    set_value(state, "overscan".to_string(), UiValue::Int(overscan));
    set_value(
        state,
        "scroll_offset".to_string(),
        UiValue::Float(viewport_start as f64 * item_extent),
    );
    Ok(())
}

fn apply_page_window(
    state: &mut UiComponentState,
    page_index: i64,
    page_size: i64,
) -> Result<(), UiComponentEventError> {
    let total_count = int_value(state, "total_count", 0).max(0);
    let page_size = page_size.max(1);
    let page_count = if total_count == 0 {
        0
    } else {
        ((total_count - 1) / page_size) + 1
    };
    let max_page_index = page_count.saturating_sub(1);
    let page_index = if page_count == 0 {
        0
    } else {
        page_index.clamp(0, max_page_index)
    };
    let page_start = page_index.saturating_mul(page_size).min(total_count);
    let page_end = page_start.saturating_add(page_size).min(total_count);

    set_value(state, "page_size".to_string(), UiValue::Int(page_size));
    set_value(state, "page_count".to_string(), UiValue::Int(page_count));
    set_value(state, "page_index".to_string(), UiValue::Int(page_index));
    set_value(state, "page_start".to_string(), UiValue::Int(page_start));
    set_value(state, "page_end".to_string(), UiValue::Int(page_end));
    set_value(state, "empty".to_string(), UiValue::Bool(total_count == 0));
    Ok(())
}

fn apply_world_transform(
    state: &mut UiComponentState,
    position: [f64; 3],
    rotation: [f64; 3],
    scale: [f64; 3],
) -> Result<(), UiComponentEventError> {
    if scale.iter().any(|value| *value <= 0.0) {
        state.validation = UiValidationState::error("world scale must be positive".to_string());
        return Err(UiComponentEventError::InvalidComplexValue {
            property: "world_scale".to_string(),
            value: format!("{scale:?}"),
        });
    }
    set_value(state, "world_position".to_string(), UiValue::Vec3(position));
    set_value(state, "world_rotation".to_string(), UiValue::Vec3(rotation));
    set_value(state, "world_scale".to_string(), UiValue::Vec3(scale));
    Ok(())
}

fn apply_world_surface(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    size: [f64; 2],
    pixels_per_meter: f64,
    billboard: bool,
    depth_test: bool,
    render_order: i64,
    camera_target: String,
) -> Result<(), UiComponentEventError> {
    if size.iter().any(|value| *value <= 0.0) {
        state.validation = UiValidationState::error("world size must be positive".to_string());
        return Err(UiComponentEventError::InvalidComplexValue {
            property: "world_size".to_string(),
            value: format!("{size:?}"),
        });
    }
    let pixels_per_meter = descriptor
        .prop("pixels_per_meter")
        .map(|schema| clamp_numeric(pixels_per_meter, schema.min, schema.max))
        .unwrap_or(pixels_per_meter);
    set_value(state, "world_size".to_string(), UiValue::Vec2(size));
    set_value(
        state,
        "pixels_per_meter".to_string(),
        UiValue::Float(pixels_per_meter),
    );
    set_value(state, "billboard".to_string(), UiValue::Bool(billboard));
    set_value(state, "depth_test".to_string(), UiValue::Bool(depth_test));
    set_value(
        state,
        "render_order".to_string(),
        UiValue::Int(render_order),
    );
    set_value(
        state,
        "camera_target".to_string(),
        UiValue::String(camera_target),
    );
    Ok(())
}

fn set_array_element(
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

fn remove_array_element(
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

fn move_array_element(
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

fn add_map_entry(
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

fn set_map_entry(
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

fn rename_map_key(
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

fn remove_map_entry(
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

fn ensure_reference_value(
    state: &mut UiComponentState,
    property: String,
) -> Result<(), UiComponentEventError> {
    match state.values.get(&property) {
        Some(UiValue::AssetRef(reference)) | Some(UiValue::InstanceRef(reference))
            if !reference.is_empty() =>
        {
            Ok(())
        }
        _ => {
            state.validation =
                UiValidationState::error(format!("reference property `{property}` is empty"));
            Err(UiComponentEventError::MissingReference { property })
        }
    }
}

fn set_value(state: &mut UiComponentState, property: String, value: UiValue) {
    clear_reference_source(state, &property);
    state.values.insert(property, value);
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

fn ensure_event_supported(
    descriptor: &UiComponentDescriptor,
    event_kind: UiComponentEventKind,
) -> Result<(), UiComponentEventError> {
    if descriptor.supports_event(event_kind) {
        Ok(())
    } else {
        Err(UiComponentEventError::UnsupportedEvent {
            component_id: descriptor.id.clone(),
            event_kind,
        })
    }
}

fn clamp_numeric(value: f64, min: Option<f64>, max: Option<f64>) -> f64 {
    value.clamp(
        min.unwrap_or(f64::NEG_INFINITY),
        max.unwrap_or(f64::INFINITY),
    )
}

fn numeric_value(kind: UiValueKind, value: f64) -> UiValue {
    match kind {
        UiValueKind::Int => UiValue::Int(value.round() as i64),
        _ => UiValue::Float(value),
    }
}

fn int_value(state: &UiComponentState, property: &str, default: i64) -> i64 {
    match state.values.get(property) {
        Some(UiValue::Int(value)) => *value,
        Some(value) => value
            .as_f64()
            .map(|value| value.round() as i64)
            .unwrap_or(default),
        None => default,
    }
}

fn float_value(state: &UiComponentState, property: &str, default: f64) -> f64 {
    state
        .values
        .get(property)
        .and_then(UiValue::as_f64)
        .unwrap_or(default)
}

fn numeric_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    schema_value: Option<f64>,
    default_value: f64,
) -> f64 {
    optional_numeric_setting(state, descriptor, property, schema_value).unwrap_or(default_value)
}

fn optional_numeric_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    schema_value: Option<f64>,
) -> Option<f64> {
    state
        .values
        .get(property)
        .and_then(UiValue::as_f64)
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(UiValue::as_f64)
        })
        .or(schema_value)
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

fn option_is_disabled(
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

fn value_kind_matches(expected: UiValueKind, actual: UiValueKind) -> bool {
    expected == UiValueKind::Any || expected == actual
}
