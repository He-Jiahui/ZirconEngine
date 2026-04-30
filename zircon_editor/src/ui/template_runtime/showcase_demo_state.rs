use std::collections::BTreeMap;

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use thiserror::Error;
use toml::Value as TomlValue;
use zircon_runtime::ui::component::{
    UiComponentDescriptorRegistry, UiComponentEvent, UiComponentEventError, UiComponentState,
    UiDragPayload, UiDragSourceMetadata, UiValue,
};

use super::host_nodes::SlintUiHostModel;

mod categories;
mod defaults;
mod state_panel;

use categories::{project_selected_category_state, should_keep_for_selected_category};
use defaults::{component_id_for_control, default_state_for_control};

pub(crate) const SHOWCASE_DOCUMENT_ID: &str = "editor.window.ui_component_showcase";

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UiComponentShowcaseDemoEventInput {
    None,
    Value(UiValue),
    Toggle(bool),
    Hover(bool),
    Press(bool),
    DragDelta(f64),
    LargeDragDelta(f64),
    DropHover(bool),
    ActiveDragTarget(bool),
    OpenPopupAt { x: f64, y: f64 },
    SelectOption { option_id: String, selected: bool },
    DropReference { payload: UiDragPayload },
    AddElement { value: UiValue },
    SetElement { index: usize, value: UiValue },
    RemoveElement { index: usize },
    MoveElement { from: usize, to: usize },
    AddMapEntry { key: String, value: UiValue },
    SetMapEntry { key: String, value: UiValue },
    RenameMapEntry { from_key: String, to_key: String },
    RemoveMapEntry { key: String },
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
            selected_category: "All".to_string(),
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
        let (result, value_text) = {
            let state = self
                .states
                .entry(control_id.to_string())
                .or_insert_with(|| default_state_for_control(control_id));
            let result = state.apply_event(descriptor, event);
            let value_text = if result.is_ok() {
                changed_property
                    .as_deref()
                    .and_then(|property| state.value(property))
                    .map(UiValue::display_text)
            } else {
                None
            };
            (result, value_text)
        };
        self.push_log(action, control_id, value_text);
        result?;
        Ok(())
    }

    pub(crate) fn apply_to_host_model(&self, host_model: &mut SlintUiHostModel) {
        if host_model.document_id != SHOWCASE_DOCUMENT_ID {
            return;
        }

        project_selected_category_state(&mut host_model.nodes, &self.selected_category);
        host_model
            .nodes
            .retain(|node| should_keep_for_selected_category(node, &self.selected_category));

        for node in &mut host_model.nodes {
            let Some(control_id) = node.control_id.as_deref() else {
                continue;
            };
            if state_panel::project_state_panel_node(self, control_id, &mut node.attributes) {
                continue;
            }
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
            if let Some(source_summary) = state
                .reference_source("value")
                .and_then(UiDragSourceMetadata::summary)
            {
                node.attributes.insert(
                    "drop_source_summary".to_string(),
                    TomlValue::String(source_summary),
                );
            } else {
                node.attributes.remove("drop_source_summary");
            }
            if let Some(explicit_state) = self.states.get(control_id) {
                node.attributes.insert(
                    "popup_open".to_string(),
                    TomlValue::Boolean(explicit_state.flags().popup_open),
                );
            }
            project_state_value_attribute(&mut node.attributes, &state, "popup_anchor_x");
            project_state_value_attribute(&mut node.attributes, &state, "popup_anchor_y");
            project_state_value_attribute(&mut node.attributes, &state, "query");
            let flags = state.flags();
            let force_transient_flags = self.states.contains_key(control_id);
            project_bool_attribute(
                &mut node.attributes,
                "focused",
                flags.focused,
                force_transient_flags,
            );
            project_bool_attribute(
                &mut node.attributes,
                "dragging",
                flags.dragging,
                force_transient_flags,
            );
            project_bool_attribute(
                &mut node.attributes,
                "hovered",
                flags.hovered,
                force_transient_flags,
            );
            project_bool_attribute(
                &mut node.attributes,
                "pressed",
                flags.pressed,
                force_transient_flags,
            );
            project_bool_attribute(
                &mut node.attributes,
                "drop_hovered",
                flags.drop_hovered,
                force_transient_flags,
            );
            project_bool_attribute(
                &mut node.attributes,
                "active_drag_target",
                flags.active_drag_target,
                force_transient_flags,
            );
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

fn project_bool_attribute(
    attributes: &mut BTreeMap<String, TomlValue>,
    key: &str,
    value: bool,
    force: bool,
) {
    if value || force {
        attributes.insert(key.to_string(), TomlValue::Boolean(value));
    }
}

fn project_state_value_attribute(
    attributes: &mut BTreeMap<String, TomlValue>,
    state: &UiComponentState,
    key: &str,
) {
    if let Some(value) = state.value(key) {
        attributes.insert(key.to_string(), toml_value(value));
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
        "GroupDemo" | "FoldoutDemo" | "InspectorSectionDemo" | "TreeRowDemo" => Some("expanded"),
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
    let value_property = value_property_for_action(action);
    match action
        .split_once('.')
        .map(|(kind, _)| kind)
        .unwrap_or(action)
    {
        "Commit" => match input {
            UiComponentShowcaseDemoEventInput::Value(value) => {
                let property = value_property.to_string();
                Ok((
                    UiComponentEvent::Commit {
                        property: property.clone(),
                        value,
                    },
                    Some(property),
                ))
            }
            UiComponentShowcaseDemoEventInput::None => {
                let property = value_property.to_string();
                Ok((
                    UiComponentEvent::Commit {
                        property: property.clone(),
                        value: UiValue::Null,
                    },
                    Some(property),
                ))
            }
            _ => Err(mismatch()),
        },
        "ValueChanged" | "Change" => match input {
            UiComponentShowcaseDemoEventInput::Value(value) => Ok((
                UiComponentEvent::ValueChanged {
                    property: value_property.to_string(),
                    value,
                },
                Some(value_property.to_string()),
            )),
            UiComponentShowcaseDemoEventInput::Toggle(value) => Ok((
                UiComponentEvent::ValueChanged {
                    property: value_property.to_string(),
                    value: UiValue::Bool(value),
                },
                Some(value_property.to_string()),
            )),
            _ => Err(mismatch()),
        },
        "BeginDrag" => Ok((
            UiComponentEvent::BeginDrag {
                property: "value".to_string(),
            },
            None,
        )),
        "Hover" => match input {
            UiComponentShowcaseDemoEventInput::Hover(hovered) => {
                Ok((UiComponentEvent::Hover { hovered }, None))
            }
            _ => Err(mismatch()),
        },
        "Press" => match input {
            UiComponentShowcaseDemoEventInput::Press(pressed) => {
                Ok((UiComponentEvent::Press { pressed }, None))
            }
            _ => Err(mismatch()),
        },
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
        "OpenPopupAt" => match input {
            UiComponentShowcaseDemoEventInput::OpenPopupAt { x, y } => {
                Ok((UiComponentEvent::OpenPopupAt { x, y }, None))
            }
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
            UiComponentShowcaseDemoEventInput::DropReference { payload } => Ok((
                UiComponentEvent::DropReference {
                    property: "value".to_string(),
                    payload,
                },
                Some("value".to_string()),
            )),
            _ => Err(mismatch()),
        },
        "DropHover" => match input {
            UiComponentShowcaseDemoEventInput::DropHover(hovered) => {
                Ok((UiComponentEvent::DropHover { hovered }, None))
            }
            _ => Err(mismatch()),
        },
        "ActiveDragTarget" => match input {
            UiComponentShowcaseDemoEventInput::ActiveDragTarget(active) => {
                Ok((UiComponentEvent::ActiveDragTarget { active }, None))
            }
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
            UiComponentShowcaseDemoEventInput::RenameMapEntry { from_key, to_key } => Ok((
                UiComponentEvent::RenameMapKey {
                    property: "entries".to_string(),
                    from_key,
                    to_key,
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

fn value_property_for_action(action: &str) -> &'static str {
    match action.rsplit_once('.').map(|(_, component)| component) {
        Some("SearchSelectQuery") => "query",
        Some("ArrayField") => "items",
        Some("MapField") => "entries",
        _ => "value",
    }
}
