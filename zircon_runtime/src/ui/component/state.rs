use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    UiComponentDescriptor, UiComponentEvent, UiComponentEventError, UiComponentEventKind,
    UiDragPayloadKind, UiValidationState, UiValue, UiValueKind,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiComponentFlags {
    pub focused: bool,
    pub dragging: bool,
    pub popup_open: bool,
    pub expanded: bool,
    pub selected: bool,
    pub checked: bool,
    pub disabled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentState {
    values: BTreeMap<String, UiValue>,
    validation: UiValidationState,
    flags: UiComponentFlags,
}

impl Default for UiComponentState {
    fn default() -> Self {
        Self::new()
    }
}

impl UiComponentState {
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
            validation: UiValidationState::normal(),
            flags: UiComponentFlags::default(),
        }
    }

    pub fn with_value(mut self, property: impl Into<String>, value: UiValue) -> Self {
        self.values.insert(property.into(), value);
        self
    }

    pub fn value(&self, property: &str) -> Option<&UiValue> {
        self.values.get(property)
    }

    pub fn values(&self) -> &BTreeMap<String, UiValue> {
        &self.values
    }

    pub fn validation(&self) -> &UiValidationState {
        &self.validation
    }

    pub fn flags(&self) -> &UiComponentFlags {
        &self.flags
    }

    pub fn apply_event(
        &mut self,
        descriptor: &UiComponentDescriptor,
        event: UiComponentEvent,
    ) -> Result<(), UiComponentEventError> {
        ensure_event_supported(descriptor, event.kind())?;

        let result = match event {
            UiComponentEvent::ValueChanged { property, value }
            | UiComponentEvent::Commit { property, value } => {
                self.apply_value(descriptor, property, value)
            }
            UiComponentEvent::Focus { focused } => {
                self.flags.focused = focused;
                Ok(())
            }
            UiComponentEvent::BeginDrag { .. } => {
                self.flags.dragging = true;
                Ok(())
            }
            UiComponentEvent::DragDelta { property, delta } => {
                self.apply_numeric_drag(descriptor, property, delta, "step")
            }
            UiComponentEvent::LargeDragDelta { property, delta } => {
                self.apply_numeric_drag(descriptor, property, delta, "large_step")
            }
            UiComponentEvent::EndDrag { .. } => {
                self.flags.dragging = false;
                Ok(())
            }
            UiComponentEvent::OpenPopup => {
                self.flags.popup_open = true;
                Ok(())
            }
            UiComponentEvent::ClosePopup => {
                self.flags.popup_open = false;
                Ok(())
            }
            UiComponentEvent::SelectOption {
                property,
                option_id,
                selected,
            } => self.apply_selection(descriptor, property, option_id, selected),
            UiComponentEvent::ToggleExpanded { expanded } => {
                self.flags.expanded = expanded;
                self.values
                    .insert("expanded".to_string(), UiValue::Bool(expanded));
                Ok(())
            }
            UiComponentEvent::AddElement { property, value } => {
                self.array_value_mut(&property).push(value);
                Ok(())
            }
            UiComponentEvent::SetElement {
                property,
                index,
                value,
            } => {
                let values = self.array_value_mut(&property);
                let Some(slot) = values.get_mut(index) else {
                    return Err(UiComponentEventError::ArrayIndexOutOfBounds { property, index });
                };
                *slot = value;
                Ok(())
            }
            UiComponentEvent::RemoveElement { property, index } => {
                let values = self.array_value_mut(&property);
                if index >= values.len() {
                    return Err(UiComponentEventError::ArrayIndexOutOfBounds { property, index });
                }
                values.remove(index);
                Ok(())
            }
            UiComponentEvent::MoveElement { property, from, to } => {
                let values = self.array_value_mut(&property);
                if from >= values.len() {
                    return Err(UiComponentEventError::ArrayIndexOutOfBounds {
                        property,
                        index: from,
                    });
                }
                let value = values.remove(from);
                values.insert(to.min(values.len()), value);
                Ok(())
            }
            UiComponentEvent::AddMapEntry {
                property,
                key,
                value,
            } => {
                let values = self.map_value_mut(&property);
                if values.contains_key(&key) {
                    return Err(UiComponentEventError::DuplicateMapKey { property, key });
                }
                values.insert(key, value);
                Ok(())
            }
            UiComponentEvent::SetMapEntry {
                property,
                key,
                value,
            } => {
                let values = self.map_value_mut(&property);
                let Some(slot) = values.get_mut(&key) else {
                    return Err(UiComponentEventError::MissingMapKey { property, key });
                };
                *slot = value;
                Ok(())
            }
            UiComponentEvent::RemoveMapEntry { property, key } => {
                self.map_value_mut(&property).remove(&key);
                Ok(())
            }
            UiComponentEvent::DropReference { property, payload } => {
                if !descriptor.accepts_drag_payload(payload.kind) {
                    self.validation = UiValidationState::error(format!(
                        "rejected drop payload `{}` for {}",
                        payload.kind.as_str(),
                        descriptor.id
                    ));
                    return Err(UiComponentEventError::RejectedDrop {
                        component_id: descriptor.id.clone(),
                        payload_kind: payload.kind.as_str().to_string(),
                    });
                }
                let value = match payload.kind {
                    UiDragPayloadKind::Asset => UiValue::AssetRef(payload.reference),
                    UiDragPayloadKind::SceneInstance | UiDragPayloadKind::Object => {
                        UiValue::InstanceRef(payload.reference)
                    }
                };
                self.values.insert(property, value);
                Ok(())
            }
            UiComponentEvent::ClearReference { property } => {
                self.values.insert(property, UiValue::Null);
                Ok(())
            }
            UiComponentEvent::LocateReference { property }
            | UiComponentEvent::OpenReference { property } => self.ensure_reference_value(property),
        };

        if result.is_ok() {
            self.validation = UiValidationState::normal();
        }
        result
    }

    fn apply_value(
        &mut self,
        descriptor: &UiComponentDescriptor,
        property: String,
        value: UiValue,
    ) -> Result<(), UiComponentEventError> {
        let Some(schema) = descriptor.prop(&property) else {
            self.values.insert(property, value);
            return Ok(());
        };
        let normalized = match schema.value_kind {
            UiValueKind::Float | UiValueKind::Int => {
                let numeric = value.as_f64().ok_or_else(|| {
                    let message = value.display_text();
                    self.validation =
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
                        self.optional_numeric_setting(descriptor, "min", schema.min),
                        self.optional_numeric_setting(descriptor, "max", schema.max),
                    ),
                )
            }
            _ => value,
        };
        self.values.insert(property, normalized);
        Ok(())
    }

    fn apply_numeric_drag(
        &mut self,
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
        let current = self
            .values
            .get(&property)
            .or(schema.default_value.as_ref())
            .and_then(UiValue::as_f64)
            .unwrap_or(0.0);
        let step = self.numeric_setting(descriptor, step_property, schema.step, 1.0);
        let next = clamp_numeric(
            current + delta * step,
            self.optional_numeric_setting(descriptor, "min", schema.min),
            self.optional_numeric_setting(descriptor, "max", schema.max),
        );
        self.values
            .insert(property, numeric_value(schema.value_kind, next));
        Ok(())
    }

    fn apply_selection(
        &mut self,
        descriptor: &UiComponentDescriptor,
        property: String,
        option_id: String,
        selected: bool,
    ) -> Result<(), UiComponentEventError> {
        if descriptor
            .prop("options")
            .and_then(|schema| schema.options.iter().find(|option| option.id == option_id))
            .is_some_and(|option| option.disabled)
        {
            self.validation = UiValidationState::error(format!(
                "disabled option `{option_id}` cannot be selected"
            ));
            return Err(UiComponentEventError::DisabledOption {
                component_id: descriptor.id.clone(),
                option_id,
            });
        }

        let is_flags = descriptor
            .prop(&property)
            .is_some_and(|schema| schema.value_kind == UiValueKind::Flags);
        let is_multiple = self.bool_setting(descriptor, "multiple", false);

        match self.values.get_mut(&property) {
            Some(UiValue::Flags(values)) => {
                if selected {
                    if !values.iter().any(|value| value == &option_id) {
                        values.push(option_id);
                    }
                } else {
                    values.retain(|value| value != &option_id);
                }
            }
            _ if is_flags => {
                if selected {
                    self.values
                        .insert(property, UiValue::Flags(vec![option_id]));
                } else {
                    self.values.insert(property, UiValue::Flags(Vec::new()));
                }
            }
            _ if is_multiple => {
                let values = self.selection_array_value_mut(&property);
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
            }
            _ if selected => {
                self.values.insert(property, UiValue::Enum(option_id));
            }
            _ => {
                self.values.insert(property, UiValue::Null);
            }
        }
        self.flags.selected = selected;
        Ok(())
    }

    fn array_value_mut(&mut self, property: &str) -> &mut Vec<UiValue> {
        if !matches!(self.values.get(property), Some(UiValue::Array(_))) {
            self.values
                .insert(property.to_string(), UiValue::Array(Vec::new()));
        }
        match self.values.get_mut(property) {
            Some(UiValue::Array(values)) => values,
            _ => unreachable!("array value was inserted before mutable access"),
        }
    }

    fn map_value_mut(&mut self, property: &str) -> &mut BTreeMap<String, UiValue> {
        if !matches!(self.values.get(property), Some(UiValue::Map(_))) {
            self.values
                .insert(property.to_string(), UiValue::Map(BTreeMap::new()));
        }
        match self.values.get_mut(property) {
            Some(UiValue::Map(values)) => values,
            _ => unreachable!("map value was inserted before mutable access"),
        }
    }

    fn ensure_reference_value(&mut self, property: String) -> Result<(), UiComponentEventError> {
        match self.values.get(&property) {
            Some(UiValue::AssetRef(reference)) | Some(UiValue::InstanceRef(reference))
                if !reference.is_empty() =>
            {
                Ok(())
            }
            _ => {
                self.validation =
                    UiValidationState::error(format!("reference property `{property}` is empty"));
                Err(UiComponentEventError::MissingReference { property })
            }
        }
    }

    fn selection_array_value_mut(&mut self, property: &str) -> &mut Vec<UiValue> {
        if !matches!(self.values.get(property), Some(UiValue::Array(_))) {
            let values = match self.values.remove(property) {
                Some(UiValue::Array(values)) => values,
                Some(UiValue::Enum(value)) if !value.is_empty() => vec![UiValue::Enum(value)],
                Some(UiValue::String(value)) if !value.is_empty() => vec![UiValue::String(value)],
                Some(UiValue::Null) | None => Vec::new(),
                Some(value) => vec![value],
            };
            self.values
                .insert(property.to_string(), UiValue::Array(values));
        }
        match self.values.get_mut(property) {
            Some(UiValue::Array(values)) => values,
            _ => unreachable!("selection array value was inserted before mutable access"),
        }
    }

    fn numeric_setting(
        &self,
        descriptor: &UiComponentDescriptor,
        property: &str,
        schema_value: Option<f64>,
        default_value: f64,
    ) -> f64 {
        self.optional_numeric_setting(descriptor, property, schema_value)
            .unwrap_or(default_value)
    }

    fn optional_numeric_setting(
        &self,
        descriptor: &UiComponentDescriptor,
        property: &str,
        schema_value: Option<f64>,
    ) -> Option<f64> {
        self.values
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
        &self,
        descriptor: &UiComponentDescriptor,
        property: &str,
        default_value: bool,
    ) -> bool {
        match self.values.get(property) {
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
}

impl UiComponentEvent {
    pub fn kind(&self) -> UiComponentEventKind {
        match self {
            Self::ValueChanged { .. } => UiComponentEventKind::ValueChanged,
            Self::Commit { .. } => UiComponentEventKind::Commit,
            Self::Focus { .. } => UiComponentEventKind::Focus,
            Self::BeginDrag { .. } => UiComponentEventKind::BeginDrag,
            Self::DragDelta { .. } => UiComponentEventKind::DragDelta,
            Self::LargeDragDelta { .. } => UiComponentEventKind::LargeDragDelta,
            Self::EndDrag { .. } => UiComponentEventKind::EndDrag,
            Self::OpenPopup => UiComponentEventKind::OpenPopup,
            Self::ClosePopup => UiComponentEventKind::ClosePopup,
            Self::SelectOption { .. } => UiComponentEventKind::SelectOption,
            Self::ToggleExpanded { .. } => UiComponentEventKind::ToggleExpanded,
            Self::AddElement { .. } => UiComponentEventKind::AddElement,
            Self::SetElement { .. } => UiComponentEventKind::SetElement,
            Self::RemoveElement { .. } => UiComponentEventKind::RemoveElement,
            Self::MoveElement { .. } => UiComponentEventKind::MoveElement,
            Self::AddMapEntry { .. } => UiComponentEventKind::AddMapEntry,
            Self::SetMapEntry { .. } => UiComponentEventKind::SetMapEntry,
            Self::RemoveMapEntry { .. } => UiComponentEventKind::RemoveMapEntry,
            Self::DropReference { .. } => UiComponentEventKind::DropReference,
            Self::ClearReference { .. } => UiComponentEventKind::ClearReference,
            Self::LocateReference { .. } => UiComponentEventKind::LocateReference,
            Self::OpenReference { .. } => UiComponentEventKind::OpenReference,
        }
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
