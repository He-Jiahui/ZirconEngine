use std::collections::{BTreeMap, VecDeque};

use toml::Value as TomlValue;
use zircon_runtime::ui::component::{UiComponentDescriptorRegistry, apply_component_event};
use zircon_runtime_interface::ui::component::{
    UiComponentEventEnvelope, UiComponentState, UiDragSourceMetadata, UiValue,
};

use super::host_nodes::RetainedUiHostModel;

mod categories;
mod defaults;
mod events;
mod state_panel;

use categories::{project_selected_category_state, should_keep_for_selected_category};
use defaults::{component_id_for_control, default_state_for_control};
pub(crate) use events::{
    UiComponentShowcaseDemoError, UiComponentShowcaseDemoEventInput,
    resolve_showcase_component_event,
};

pub(crate) const SHOWCASE_DOCUMENT_ID: &str = "res://ui/editor/component_showcase.zui";
const SHOWCASE_EVENT_LOG_LIMIT: usize = 128;

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
    event_log: VecDeque<UiComponentShowcaseDemoLogEntry>,
}

impl Default for UiComponentShowcaseDemoState {
    fn default() -> Self {
        Self {
            selected_category: "All".to_string(),
            states: BTreeMap::new(),
            event_log: VecDeque::new(),
        }
    }
}

impl UiComponentShowcaseDemoState {
    #[cfg(test)]
    pub(crate) fn selected_category(&self) -> &str {
        &self.selected_category
    }

    #[cfg(test)]
    pub(crate) fn event_log(&self) -> &VecDeque<UiComponentShowcaseDemoLogEntry> {
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

    pub(crate) fn value_i64(&self, control_id: &str, property: &str) -> Option<i64> {
        self.state_for_control(control_id)
            .and_then(|state| state.value(property).and_then(ui_value_as_i64).copied())
    }

    pub(crate) fn apply_component_event_envelope(
        &mut self,
        action: &str,
        envelope: &UiComponentEventEnvelope,
        changed_property: Option<&str>,
    ) -> Result<Option<UiValue>, UiComponentShowcaseDemoError> {
        let control_id = envelope.control_id.as_str();
        if let Some(category) = action.strip_prefix("SelectCategory.") {
            self.selected_category = category.to_string();
            self.push_log(action, control_id, Some(category.to_string()));
            return Ok(Some(UiValue::String(category.to_string())));
        }

        let component_id = envelope
            .component_id
            .as_deref()
            .or_else(|| component_id_for_control(control_id))
            .ok_or_else(|| UiComponentShowcaseDemoError::UnknownControl {
                control_id: control_id.to_string(),
            })?;
        let registry = UiComponentDescriptorRegistry::editor_showcase_shared();
        let descriptor = registry.descriptor(component_id).ok_or_else(|| {
            UiComponentShowcaseDemoError::MissingDescriptor {
                component_id: component_id.to_string(),
            }
        })?;
        let (result, changed_value, value_text) = {
            let state = self
                .states
                .entry(control_id.to_string())
                .or_insert_with(|| default_state_for_control(control_id));
            let result = apply_component_event(state, descriptor, envelope.event.clone());
            let changed_value = if result.is_ok() {
                changed_property
                    .and_then(|property| state.value(property))
                    .cloned()
            } else {
                None
            };
            let value_text = changed_value.as_ref().map(UiValue::display_text);
            (result, changed_value, value_text)
        };
        self.push_log(action, control_id, value_text);
        result?;
        Ok(changed_value)
    }

    pub(crate) fn apply_to_host_model(&self, host_model: &mut RetainedUiHostModel) {
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
            if control_id == "UiComponentShowcaseHeader" {
                node.attributes.insert(
                    "text".to_string(),
                    TomlValue::String(
                        "Runtime UI Component Showcase · material_dark / fyrox_panel / jetbrains_shell / unreal_window_model"
                            .to_string(),
                    ),
                );
                node.attributes.insert(
                    "text_tone".to_string(),
                    TomlValue::String("default".to_string()),
                );
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
                    TomlValue::Boolean(explicit_state.flags.popup_open),
                );
            }
            project_state_value_attribute(&mut node.attributes, &state, "popup_anchor_x");
            project_state_value_attribute(&mut node.attributes, &state, "popup_anchor_y");
            project_state_value_attribute(&mut node.attributes, &state, "query");
            project_state_value_attribute(&mut node.attributes, &state, "viewport_start");
            project_state_value_attribute(&mut node.attributes, &state, "viewport_count");
            project_state_value_attribute(&mut node.attributes, &state, "visible_end");
            project_state_value_attribute(&mut node.attributes, &state, "requested_start");
            project_state_value_attribute(&mut node.attributes, &state, "requested_count");
            project_state_value_attribute(&mut node.attributes, &state, "scroll_offset");
            project_state_value_attribute(&mut node.attributes, &state, "page_index");
            project_state_value_attribute(&mut node.attributes, &state, "page_size");
            project_state_value_attribute(&mut node.attributes, &state, "page_count");
            project_state_value_attribute(&mut node.attributes, &state, "page_start");
            project_state_value_attribute(&mut node.attributes, &state, "page_end");
            project_state_value_attribute(&mut node.attributes, &state, "world_position");
            project_state_value_attribute(&mut node.attributes, &state, "world_rotation");
            project_state_value_attribute(&mut node.attributes, &state, "world_scale");
            project_state_value_attribute(&mut node.attributes, &state, "world_size");
            project_state_value_attribute(&mut node.attributes, &state, "pixels_per_meter");
            project_state_value_attribute(&mut node.attributes, &state, "billboard");
            project_state_value_attribute(&mut node.attributes, &state, "depth_test");
            project_state_value_attribute(&mut node.attributes, &state, "render_order");
            project_state_value_attribute(&mut node.attributes, &state, "camera_target");
            for key in [
                "commands",
                "filtered_commands",
                "selected_command_id",
                "focused_index",
                "recent_commands",
                "notifications",
                "selected_notification_id",
                "visible_limit",
                "unread_count",
                "keyboard_navigation",
                "empty_text",
            ] {
                project_state_value_attribute(&mut node.attributes, &state, key);
            }
            let flags = &state.flags;
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
                TomlValue::String(state.validation.level_name().to_string()),
            );
            if let Some(message) = &state.validation.message {
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
        if self.event_log.len() == SHOWCASE_EVENT_LOG_LIMIT {
            self.event_log.pop_front();
        }
        self.event_log.push_back(UiComponentShowcaseDemoLogEntry {
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
        "VirtualListDemo" => Some("viewport_start"),
        "PagedListDemo" => Some("page_index"),
        "WorldSpaceSurfaceDemo" => Some("world_position"),
        control_id if component_id_for_control(control_id).is_some() => Some("value"),
        _ => None,
    }
}

fn ui_value_as_i64(value: &UiValue) -> Option<&i64> {
    match value {
        UiValue::Int(value) => Some(value),
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

#[cfg(test)]
mod performance_tests {
    use super::{SHOWCASE_EVENT_LOG_LIMIT, UiComponentShowcaseDemoState};

    #[test]
    fn showcase_event_log_retains_a_bounded_recent_window() {
        let mut state = UiComponentShowcaseDemoState::default();
        for index in 0..=SHOWCASE_EVENT_LOG_LIMIT {
            state.push_log("Change", &format!("control-{index}"), None);
        }

        assert_eq!(state.event_log.len(), SHOWCASE_EVENT_LOG_LIMIT);
        assert_eq!(
            state
                .event_log
                .front()
                .map(|entry| entry.control_id.as_str()),
            Some("control-1")
        );
        assert_eq!(
            state
                .event_log
                .back()
                .map(|entry| entry.control_id.as_str()),
            Some("control-128")
        );
    }
}
