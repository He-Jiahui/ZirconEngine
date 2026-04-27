use std::collections::BTreeMap;

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use thiserror::Error;
use toml::Value as TomlValue;
use zircon_runtime::ui::component::{
    UiComponentDescriptorRegistry, UiComponentEvent, UiComponentEventError, UiComponentState,
    UiDragPayload, UiDragPayloadKind, UiValue,
};

use super::host_nodes::SlintUiHostModel;

pub(crate) const SHOWCASE_DOCUMENT_ID: &str = "editor.window.ui_component_showcase";

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UiComponentShowcaseDemoEventInput {
    None,
    Value(UiValue),
    Toggle(bool),
    DragDelta(f64),
    LargeDragDelta(f64),
    SelectOption {
        option_id: String,
        selected: bool,
    },
    DropReference {
        kind: UiDragPayloadKind,
        reference: String,
    },
    AddElement {
        value: UiValue,
    },
    SetElement {
        index: usize,
        value: UiValue,
    },
    RemoveElement {
        index: usize,
    },
    MoveElement {
        from: usize,
        to: usize,
    },
    AddMapEntry {
        key: String,
        value: UiValue,
    },
    SetMapEntry {
        key: String,
        value: UiValue,
    },
    RemoveMapEntry {
        key: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiComponentShowcaseDemoLogEntry {
    pub action: String,
    pub control_id: String,
    pub value_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiComponentShowcaseDemoState {
    selected_category: String,
    states: BTreeMap<String, UiComponentState>,
    event_log: Vec<UiComponentShowcaseDemoLogEntry>,
}

impl Default for UiComponentShowcaseDemoState {
    fn default() -> Self {
        Self {
            selected_category: "Visual".to_string(),
            states: BTreeMap::new(),
            event_log: Vec::new(),
        }
    }
}

impl UiComponentShowcaseDemoState {
    #[cfg(test)]
    pub(crate) fn selected_category(&self) -> &str {
        &self.selected_category
    }

    #[cfg(test)]
    pub(crate) fn event_log(&self) -> &[UiComponentShowcaseDemoLogEntry] {
        &self.event_log
    }

    #[cfg(test)]
    pub(crate) fn value_text(&self, control_id: &str, property: &str) -> Option<String> {
        if let Some(value) = self
            .states
            .get(control_id)
            .and_then(|state| state.value(property))
        {
            return Some(value.display_text());
        }
        self.default_state_for_control(control_id)
            .and_then(|state| state.value(property).map(UiValue::display_text))
    }

    pub(crate) fn apply_binding(
        &mut self,
        binding: &EditorUiBinding,
        input: UiComponentShowcaseDemoEventInput,
    ) -> Result<(), UiComponentShowcaseDemoError> {
        let (action, control_id) = showcase_action(binding)?;
        if let Some(category) = action.strip_prefix("SelectCategory.") {
            self.selected_category = category.to_string();
            self.push_log(action, control_id, Some(category.to_string()));
            return Ok(());
        }

        let component_id = component_id_for_control(control_id).ok_or_else(|| {
            UiComponentShowcaseDemoError::UnknownControl {
                control_id: control_id.to_string(),
            }
        })?;
        let registry = UiComponentDescriptorRegistry::editor_showcase();
        let descriptor = registry.descriptor(component_id).ok_or_else(|| {
            UiComponentShowcaseDemoError::MissingDescriptor {
                component_id: component_id.to_string(),
            }
        })?;
        let (event, changed_property) = component_event_for_action(action, input)?;
        let state = self
            .states
            .entry(control_id.to_string())
            .or_insert_with(|| default_state_for_control(control_id));
        state.apply_event(descriptor, event)?;
        let value_text = changed_property
            .as_deref()
            .and_then(|property| state.value(property))
            .map(UiValue::display_text);
        self.push_log(action, control_id, value_text);
        Ok(())
    }

    pub(crate) fn apply_to_host_model(&self, host_model: &mut SlintUiHostModel) {
        if host_model.document_id != SHOWCASE_DOCUMENT_ID {
            return;
        }

        for node in &mut host_model.nodes {
            let Some(control_id) = node.control_id.as_deref() else {
                continue;
            };
            if control_id == "ComponentShowcaseEventLog" {
                if let Some(text) = self.event_log_text() {
                    node.attributes
                        .insert("text".to_string(), TomlValue::String(text));
                }
                continue;
            }

            let Some(state) = self.state_for_control(control_id) else {
                continue;
            };
            if let Some(property) = primary_property_for_control(control_id) {
                if let Some(value) = state.value(property) {
                    node.attributes
                        .insert(property.to_string(), toml_value(value));
                    node.attributes.insert(
                        "value_text".to_string(),
                        TomlValue::String(value.display_text()),
                    );
                    match (property, value) {
                        ("expanded", UiValue::Bool(expanded)) => {
                            node.attributes
                                .insert("expanded".to_string(), TomlValue::Boolean(*expanded));
                        }
                        ("value", UiValue::Bool(checked)) => {
                            node.attributes
                                .insert("checked".to_string(), TomlValue::Boolean(*checked));
                        }
                        _ => {}
                    }
                }
            }
            if let Some(explicit_state) = self.states.get(control_id) {
                node.attributes.insert(
                    "popup_open".to_string(),
                    TomlValue::Boolean(explicit_state.flags().popup_open),
                );
            }
            let flags = state.flags();
            node.attributes
                .insert("focused".to_string(), TomlValue::Boolean(flags.focused));
            node.attributes
                .insert("dragging".to_string(), TomlValue::Boolean(flags.dragging));
            if flags.selected {
                node.attributes
                    .insert("selected".to_string(), TomlValue::Boolean(true));
            }
            if flags.checked {
                node.attributes
                    .insert("checked".to_string(), TomlValue::Boolean(true));
            }
            if flags.disabled {
                node.attributes
                    .insert("disabled".to_string(), TomlValue::Boolean(true));
            }
            if flags.focused && !node.attributes.contains_key("selection_state") {
                node.attributes.insert(
                    "selection_state".to_string(),
                    TomlValue::String("focused".to_string()),
                );
            }

            if control_id == "ArrayFieldDemo" {
                let element_type = node
                    .attributes
                    .get("element_type")
                    .and_then(TomlValue::as_str)
                    .unwrap_or("Element");
                node.attributes.insert(
                    "collection_items".to_string(),
                    TomlValue::Array(collection_items_for_array(
                        state.value("items"),
                        element_type,
                    )),
                );
            } else if control_id == "MapFieldDemo" {
                let key_type = node
                    .attributes
                    .get("key_type")
                    .and_then(TomlValue::as_str)
                    .unwrap_or("Key");
                let value_type = node
                    .attributes
                    .get("value_type")
                    .and_then(TomlValue::as_str)
                    .unwrap_or("Value");
                node.attributes.insert(
                    "collection_items".to_string(),
                    TomlValue::Array(collection_items_for_map(
                        state.value("entries"),
                        key_type,
                        value_type,
                    )),
                );
            }

            node.attributes.insert(
                "validation_level".to_string(),
                TomlValue::String(state.validation().level_name().to_string()),
            );
            if let Some(message) = &state.validation().message {
                node.attributes.insert(
                    "validation_message".to_string(),
                    TomlValue::String(message.clone()),
                );
            } else {
                node.attributes.remove("validation_message");
            }
        }
    }

    fn default_state_for_control(&self, control_id: &str) -> Option<UiComponentState> {
        component_id_for_control(control_id).map(|_| default_state_for_control(control_id))
    }

    fn state_for_control(&self, control_id: &str) -> Option<UiComponentState> {
        self.states
            .get(control_id)
            .cloned()
            .or_else(|| self.default_state_for_control(control_id))
    }

    fn event_log_text(&self) -> Option<String> {
        if self.event_log.is_empty() {
            return None;
        }

        Some(
            self.event_log
                .iter()
                .rev()
                .take(12)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(|entry| {
                    let value = entry
                        .value_text
                        .as_deref()
                        .filter(|value| !value.is_empty())
                        .map(|value| format!(" = {value}"))
                        .unwrap_or_default();
                    format!("{} -> {}{}", entry.control_id, entry.action, value)
                })
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }

    fn push_log(&mut self, action: &str, control_id: &str, value_text: Option<String>) {
        self.event_log.push(UiComponentShowcaseDemoLogEntry {
            action: action.to_string(),
            control_id: control_id.to_string(),
            value_text,
        });
    }
}

fn collection_items_for_array(value: Option<&UiValue>, element_type: &str) -> Vec<TomlValue> {
    let Some(UiValue::Array(values)) = value else {
        return vec![TomlValue::String(format!("Empty {element_type} list"))];
    };
    if values.is_empty() {
        return vec![TomlValue::String(format!("Empty {element_type} list"))];
    }
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            TomlValue::String(format!(
                "#{index} {element_type} = {}",
                value.display_text()
            ))
        })
        .collect()
}

fn collection_items_for_map(
    value: Option<&UiValue>,
    key_type: &str,
    value_type: &str,
) -> Vec<TomlValue> {
    let Some(UiValue::Map(values)) = value else {
        return vec![TomlValue::String(format!(
            "Empty {key_type} -> {value_type} map"
        ))];
    };
    if values.is_empty() {
        return vec![TomlValue::String(format!(
            "Empty {key_type} -> {value_type} map"
        ))];
    }
    values
        .iter()
        .map(|(key, value)| {
            TomlValue::String(format!(
                "{key}: {key_type} -> {value_type} = {}",
                value.display_text()
            ))
        })
        .collect()
}

fn primary_property_for_control(control_id: &str) -> Option<&'static str> {
    match control_id {
        "ArrayFieldDemo" => Some("items"),
        "MapFieldDemo" => Some("entries"),
        "GroupDemo" | "FoldoutDemo" | "TreeRowDemo" => Some("expanded"),
        control_id if component_id_for_control(control_id).is_some() => Some("value"),
        _ => None,
    }
}

fn toml_value(value: &UiValue) -> TomlValue {
    match value {
        UiValue::Bool(value) => TomlValue::Boolean(*value),
        UiValue::Int(value) => TomlValue::Integer(*value),
        UiValue::Float(value) => TomlValue::Float(*value),
        UiValue::String(value)
        | UiValue::Color(value)
        | UiValue::AssetRef(value)
        | UiValue::InstanceRef(value)
        | UiValue::Enum(value) => TomlValue::String(value.clone()),
        UiValue::Vec2(value) => {
            TomlValue::Array(value.iter().copied().map(TomlValue::Float).collect())
        }
        UiValue::Vec3(value) => {
            TomlValue::Array(value.iter().copied().map(TomlValue::Float).collect())
        }
        UiValue::Vec4(value) => {
            TomlValue::Array(value.iter().copied().map(TomlValue::Float).collect())
        }
        UiValue::Array(values) => TomlValue::Array(values.iter().map(toml_value).collect()),
        UiValue::Map(values) => {
            let mut table = toml::map::Map::new();
            for (key, value) in values {
                table.insert(key.clone(), toml_value(value));
            }
            TomlValue::Table(table)
        }
        UiValue::Flags(values) => TomlValue::Array(
            values
                .iter()
                .map(|value| TomlValue::String(value.clone()))
                .collect(),
        ),
        UiValue::Null => TomlValue::String(String::new()),
    }
}

#[derive(Debug, Error, PartialEq)]
pub(crate) enum UiComponentShowcaseDemoError {
    #[error("binding does not carry a UiComponentShowcase custom payload")]
    UnsupportedPayload,
    #[error("showcase payload is missing string argument {index}")]
    MissingPayloadArgument { index: usize },
    #[error("unknown showcase control {control_id}")]
    UnknownControl { control_id: String },
    #[error("missing runtime component descriptor {component_id}")]
    MissingDescriptor { component_id: String },
    #[error("action {action} does not accept the provided event input")]
    InputMismatch { action: String },
    #[error(transparent)]
    Component(#[from] UiComponentEventError),
}

fn showcase_action(
    binding: &EditorUiBinding,
) -> Result<(&str, &str), UiComponentShowcaseDemoError> {
    let EditorUiBindingPayload::Custom(call) = binding.payload() else {
        return Err(UiComponentShowcaseDemoError::UnsupportedPayload);
    };
    if call.symbol != "UiComponentShowcase" {
        return Err(UiComponentShowcaseDemoError::UnsupportedPayload);
    }
    let action = call
        .argument(0)
        .and_then(|value| value.as_str())
        .ok_or(UiComponentShowcaseDemoError::MissingPayloadArgument { index: 0 })?;
    let control_id = call
        .argument(1)
        .and_then(|value| value.as_str())
        .ok_or(UiComponentShowcaseDemoError::MissingPayloadArgument { index: 1 })?;
    Ok((action, control_id))
}

fn component_event_for_action(
    action: &str,
    input: UiComponentShowcaseDemoEventInput,
) -> Result<(UiComponentEvent, Option<String>), UiComponentShowcaseDemoError> {
    let mismatch = || UiComponentShowcaseDemoError::InputMismatch {
        action: action.to_string(),
    };
    match action
        .split_once('.')
        .map(|(kind, _)| kind)
        .unwrap_or(action)
    {
        "Commit" => match input {
            UiComponentShowcaseDemoEventInput::Value(value) => Ok((
                UiComponentEvent::Commit {
                    property: "value".to_string(),
                    value,
                },
                Some("value".to_string()),
            )),
            UiComponentShowcaseDemoEventInput::None => Ok((
                UiComponentEvent::Commit {
                    property: "value".to_string(),
                    value: UiValue::Null,
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "ValueChanged" | "Change" => match input {
            UiComponentShowcaseDemoEventInput::Value(value) => Ok((
                UiComponentEvent::ValueChanged {
                    property: "value".to_string(),
                    value,
                },
                Some("value".to_string()),
            )),
            UiComponentShowcaseDemoEventInput::Toggle(value) => Ok((
                UiComponentEvent::ValueChanged {
                    property: "value".to_string(),
                    value: UiValue::Bool(value),
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "BeginDrag" => Ok((
            UiComponentEvent::BeginDrag {
                property: "value".to_string(),
            },
            None,
        )),
        "DragDelta" => match input {
            UiComponentShowcaseDemoEventInput::DragDelta(delta) => Ok((
                UiComponentEvent::DragDelta {
                    property: "value".to_string(),
                    delta,
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "LargeDragDelta" => match input {
            UiComponentShowcaseDemoEventInput::LargeDragDelta(delta) => Ok((
                UiComponentEvent::LargeDragDelta {
                    property: "value".to_string(),
                    delta,
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "EndDrag" => Ok((
            UiComponentEvent::EndDrag {
                property: "value".to_string(),
            },
            Some("value".to_string()),
        )),
        "OpenPopup" => match input {
            UiComponentShowcaseDemoEventInput::None => Ok((UiComponentEvent::OpenPopup, None)),
            _ => Err(mismatch()),
        },
        "ClosePopup" => match input {
            UiComponentShowcaseDemoEventInput::None => Ok((UiComponentEvent::ClosePopup, None)),
            _ => Err(mismatch()),
        },
        "SelectOption" => match input {
            UiComponentShowcaseDemoEventInput::SelectOption {
                option_id,
                selected,
            } => {
                let option_id = if action.ends_with(".ContextActionMenu") {
                    context_action_menu_option_id(&option_id).ok_or_else(mismatch)?
                } else {
                    option_id
                };
                Ok((
                    UiComponentEvent::SelectOption {
                        property: "value".to_string(),
                        option_id,
                        selected,
                    },
                    Some("value".to_string()),
                ))
            }
            _ => Err(mismatch()),
        },
        "DropReference" => match input {
            UiComponentShowcaseDemoEventInput::DropReference { kind, reference } => Ok((
                UiComponentEvent::DropReference {
                    property: "value".to_string(),
                    payload: UiDragPayload::new(kind, reference),
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "ClearReference" => match input {
            UiComponentShowcaseDemoEventInput::None => Ok((
                UiComponentEvent::ClearReference {
                    property: "value".to_string(),
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "LocateReference" => match input {
            UiComponentShowcaseDemoEventInput::None => Ok((
                UiComponentEvent::LocateReference {
                    property: "value".to_string(),
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "OpenReference" => match input {
            UiComponentShowcaseDemoEventInput::None => Ok((
                UiComponentEvent::OpenReference {
                    property: "value".to_string(),
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "ToggleExpanded" => match input {
            UiComponentShowcaseDemoEventInput::Toggle(expanded) => Ok((
                UiComponentEvent::ToggleExpanded { expanded },
                Some("expanded".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "AddElement" => match input {
            UiComponentShowcaseDemoEventInput::AddElement { value } => Ok((
                UiComponentEvent::AddElement {
                    property: "items".to_string(),
                    value,
                },
                Some("items".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "SetElement" => match input {
            UiComponentShowcaseDemoEventInput::SetElement { index, value } => Ok((
                UiComponentEvent::SetElement {
                    property: "items".to_string(),
                    index,
                    value,
                },
                Some("items".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "RemoveElement" => match input {
            UiComponentShowcaseDemoEventInput::RemoveElement { index } => Ok((
                UiComponentEvent::RemoveElement {
                    property: "items".to_string(),
                    index,
                },
                Some("items".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "MoveElement" => match input {
            UiComponentShowcaseDemoEventInput::MoveElement { from, to } => Ok((
                UiComponentEvent::MoveElement {
                    property: "items".to_string(),
                    from,
                    to,
                },
                Some("items".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "AddMapEntry" => match input {
            UiComponentShowcaseDemoEventInput::AddMapEntry { key, value } => Ok((
                UiComponentEvent::AddMapEntry {
                    property: "entries".to_string(),
                    key,
                    value,
                },
                Some("entries".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "SetMapEntry" => match input {
            UiComponentShowcaseDemoEventInput::SetMapEntry { key, value } => Ok((
                UiComponentEvent::SetMapEntry {
                    property: "entries".to_string(),
                    key,
                    value,
                },
                Some("entries".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "RemoveMapEntry" => match input {
            UiComponentShowcaseDemoEventInput::RemoveMapEntry { key } => Ok((
                UiComponentEvent::RemoveMapEntry {
                    property: "entries".to_string(),
                    key,
                },
                Some("entries".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "Select" => Ok((
            UiComponentEvent::Focus { focused: true },
            Some("value".to_string()),
        )),
        _ => Err(mismatch()),
    }
}

fn context_action_menu_option_id(encoded: &str) -> Option<String> {
    if encoded == "---" {
        return None;
    }
    let mut parts = encoded.split('|');
    let label = parts.next()?.trim();
    let flags = parts.next().unwrap_or_default();
    if label.is_empty() || flags.split(',').any(|flag| flag.trim() == "disabled") {
        return None;
    }
    Some(label.to_string())
}

fn component_id_for_control(control_id: &str) -> Option<&'static str> {
    match control_id {
        "ButtonDemo" => Some("Button"),
        "IconButtonDemo" => Some("IconButton"),
        "ToggleButtonDemo" => Some("ToggleButton"),
        "CheckboxDemo" => Some("Checkbox"),
        "RadioDemo" => Some("Radio"),
        "SegmentedControlDemo" => Some("SegmentedControl"),
        "InputFieldDemo" => Some("InputField"),
        "TextFieldDemo" => Some("TextField"),
        "NumberFieldDemo" => Some("NumberField"),
        "RangeFieldDemo" => Some("RangeField"),
        "DropdownDemo" => Some("Dropdown"),
        "ComboBoxDemo" => Some("ComboBox"),
        "EnumFieldDemo" => Some("EnumField"),
        "FlagsFieldDemo" => Some("FlagsField"),
        "SearchSelectDemo" => Some("SearchSelect"),
        "AssetFieldDemo" => Some("AssetField"),
        "InstanceFieldDemo" => Some("InstanceField"),
        "ObjectFieldDemo" => Some("ObjectField"),
        "GroupDemo" => Some("Group"),
        "FoldoutDemo" => Some("Foldout"),
        "ArrayFieldDemo" => Some("ArrayField"),
        "MapFieldDemo" => Some("MapField"),
        "ListRowDemo" => Some("ListRow"),
        "TreeRowDemo" => Some("TreeRow"),
        "ContextActionMenuDemo" => Some("ContextActionMenu"),
        _ => None,
    }
}

fn default_state_for_control(control_id: &str) -> UiComponentState {
    match control_id {
        "NumberFieldDemo" => UiComponentState::new().with_value("value", UiValue::Float(42.0)),
        "RangeFieldDemo" => UiComponentState::new().with_value("value", UiValue::Float(68.0)),
        "DropdownDemo" => {
            UiComponentState::new().with_value("value", UiValue::Enum("runtime".to_string()))
        }
        "ComboBoxDemo" => {
            UiComponentState::new().with_value("value", UiValue::Enum("material".to_string()))
        }
        "EnumFieldDemo" => {
            UiComponentState::new().with_value("value", UiValue::Enum("RiderDocking".to_string()))
        }
        "FlagsFieldDemo" => UiComponentState::new().with_value(
            "value",
            UiValue::Flags(vec!["Selectable".to_string(), "Draggable".to_string()]),
        ),
        "SearchSelectDemo" => UiComponentState::new()
            .with_value("value", UiValue::Enum("runtime.ui.NumberField".to_string())),
        "AssetFieldDemo" => UiComponentState::new().with_value(
            "value",
            UiValue::AssetRef("res://textures/grid.albedo.png".to_string()),
        ),
        "InstanceFieldDemo" => UiComponentState::new().with_value(
            "value",
            UiValue::InstanceRef("scene://Root/CameraRig".to_string()),
        ),
        "ObjectFieldDemo" => UiComponentState::new().with_value(
            "value",
            UiValue::InstanceRef("object://Selection/MainCamera".to_string()),
        ),
        "GroupDemo" => UiComponentState::new().with_value("expanded", UiValue::Bool(true)),
        "FoldoutDemo" => UiComponentState::new().with_value("expanded", UiValue::Bool(false)),
        "ArrayFieldDemo" => UiComponentState::new().with_value(
            "items",
            UiValue::Array(vec![
                UiValue::String("Label".to_string()),
                UiValue::String("NumberField".to_string()),
                UiValue::String("AssetField".to_string()),
            ]),
        ),
        "MapFieldDemo" => {
            let mut entries = BTreeMap::new();
            entries.insert("speed".to_string(), UiValue::Float(1.0));
            entries.insert("visible".to_string(), UiValue::Bool(true));
            UiComponentState::new().with_value("entries", UiValue::Map(entries))
        }
        "ToggleButtonDemo" | "CheckboxDemo" => {
            UiComponentState::new().with_value("value", UiValue::Bool(true))
        }
        "RadioDemo" => UiComponentState::new().with_value("value", UiValue::Bool(false)),
        "ListRowDemo" => {
            UiComponentState::new().with_value("value", UiValue::String("selected".to_string()))
        }
        "TreeRowDemo" => UiComponentState::new().with_value("expanded", UiValue::Bool(true)),
        "ContextActionMenuDemo" => {
            UiComponentState::new().with_value("value", UiValue::String("Inspect".to_string()))
        }
        _ => UiComponentState::new(),
    }
}
