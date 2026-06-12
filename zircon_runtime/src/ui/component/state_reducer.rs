use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEvent, UiComponentEventError, UiComponentEventKind,
    UiComponentState, UiValidationState, UiValue, UiValueKind,
};

mod button;
mod collection;
mod disclosure;
mod interaction;
mod numeric;
mod overlay;
mod reference;
mod selection;
mod windowing;
mod world;

pub fn apply_component_event(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    event: UiComponentEvent,
) -> Result<(), UiComponentEventError> {
    ensure_event_supported(descriptor, event.kind())?;

    let event = if button::is_button_family(descriptor) {
        match button::reduce_button_event(state, event) {
            button::UiButtonReduceOutcome::Applied => {
                state.validation = UiValidationState::normal();
                return Ok(());
            }
            button::UiButtonReduceOutcome::UseGenericReducer(event) => event,
        }
    } else {
        event
    };

    let result = match event {
        UiComponentEvent::ValueChanged { property, value }
        | UiComponentEvent::Commit { property, value } => {
            apply_value(state, descriptor, property, value)
        }
        UiComponentEvent::Focus { focused } => {
            interaction::focus(state, focused);
            Ok(())
        }
        UiComponentEvent::Hover { hovered } => {
            interaction::hover(state, hovered);
            Ok(())
        }
        UiComponentEvent::Press { pressed } => {
            interaction::press(state, pressed);
            Ok(())
        }
        UiComponentEvent::BeginDrag { .. } => {
            interaction::begin_drag(state);
            Ok(())
        }
        UiComponentEvent::DragDelta { property, delta } => {
            numeric::apply_numeric_drag(state, descriptor, property, delta, "step")
        }
        UiComponentEvent::LargeDragDelta { property, delta } => {
            numeric::apply_numeric_drag(state, descriptor, property, delta, "large_step")
        }
        UiComponentEvent::EndDrag { .. } => {
            interaction::end_drag(state);
            Ok(())
        }
        UiComponentEvent::DropHover { hovered } => {
            interaction::drop_hover(state, hovered);
            Ok(())
        }
        UiComponentEvent::ActiveDragTarget { active } => {
            interaction::active_drag_target(state, active);
            Ok(())
        }
        UiComponentEvent::OpenPopup => overlay::open_popup(state),
        UiComponentEvent::OpenPopupAt { x, y } => overlay::open_popup_at(state, x, y),
        UiComponentEvent::ClosePopup => overlay::close_popup(state),
        UiComponentEvent::SelectOption {
            property,
            option_id,
            selected,
        } => selection::apply_selection(state, descriptor, property, option_id, selected),
        UiComponentEvent::ToggleExpanded { expanded } => {
            disclosure::toggle_expanded(state, expanded)
        }
        UiComponentEvent::AddElement { property, value } => {
            collection::add_element(state, property, value);
            Ok(())
        }
        UiComponentEvent::SetElement {
            property,
            index,
            value,
        } => collection::set_array_element(state, property, index, value),
        UiComponentEvent::RemoveElement { property, index } => {
            collection::remove_array_element(state, property, index)
        }
        UiComponentEvent::MoveElement { property, from, to } => {
            collection::move_array_element(state, property, from, to)
        }
        UiComponentEvent::AddMapEntry {
            property,
            key,
            value,
        } => collection::add_map_entry(state, property, key, value),
        UiComponentEvent::SetMapEntry {
            property,
            key,
            value,
        } => collection::set_map_entry(state, property, key, value),
        UiComponentEvent::RenameMapKey {
            property,
            from_key,
            to_key,
        } => collection::rename_map_key(state, property, from_key, to_key),
        UiComponentEvent::RemoveMapEntry { property, key } => {
            collection::remove_map_entry(state, property, key)
        }
        UiComponentEvent::DropReference { property, payload } => {
            reference::drop_reference(state, descriptor, property, payload)
        }
        UiComponentEvent::ClearReference { property } => {
            reference::clear_reference(state, property);
            Ok(())
        }
        UiComponentEvent::LocateReference { property }
        | UiComponentEvent::OpenReference { property } => {
            reference::ensure_reference_value(state, property)
        }
        UiComponentEvent::SetVisibleRange { start, count } => {
            windowing::apply_visible_range(state, start, count)
        }
        UiComponentEvent::SetPage {
            page_index,
            page_size,
        } => windowing::apply_page_window(state, page_index, page_size),
        UiComponentEvent::SetWorldTransform {
            position,
            rotation,
            scale,
        } => world::apply_world_transform(state, position, rotation, scale),
        UiComponentEvent::SetWorldSurface {
            size,
            pixels_per_meter,
            billboard,
            depth_test,
            render_order,
            camera_target,
        } => world::apply_world_surface(
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

fn set_value(state: &mut UiComponentState, property: String, value: UiValue) {
    clear_reference_source(state, &property);
    state.values.insert(property, value);
}

fn clear_reference_source(state: &mut UiComponentState, property: &str) {
    state.reference_sources.remove(property);
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

fn value_kind_matches(expected: UiValueKind, actual: UiValueKind) -> bool {
    expected == UiValueKind::Any || expected == actual
}
