use crate::ui::workbench::state::EditorState;
use serde_json::Value;
use std::collections::BTreeMap;
use zircon_runtime::plugin::{ComponentPropertyDescriptor, ComponentTypeDescriptor};
use zircon_runtime::scene::{NodeId, Scene};

use super::super::{
    AssetSurfaceMode, EditorDataSnapshot, InspectorPluginComponentPropertySnapshot,
    InspectorPluginComponentSnapshot, InspectorSnapshot, SceneEntry,
};

impl EditorState {
    pub fn snapshot(&self) -> EditorDataSnapshot {
        let selected = self.viewport_controller.selected_node();
        let (scene_entries, inspector) = self
            .world
            .try_with_world(|scene| {
                let inspector = selected
                    .and_then(|id| scene.find_node(id).map(|node| (id, node)))
                    .map(|(id, _node)| InspectorSnapshot {
                        id,
                        name: self.name_field.clone(),
                        parent: self.parent_field.clone(),
                        translation: self.transform_fields.clone(),
                        plugin_components: inspector_plugin_components(
                            scene,
                            id,
                            &self.inspector_dynamic_fields,
                        ),
                    });
                let scene_entries = scene
                    .nodes()
                    .iter()
                    .map(|node| SceneEntry {
                        id: node.id,
                        name: node.name.clone(),
                        depth: hierarchy_depth(scene, node.id),
                        selected: selected == Some(node.id),
                    })
                    .collect();

                (scene_entries, inspector)
            })
            .unwrap_or_else(|| (Vec::new(), None));

        EditorDataSnapshot {
            scene_entries,
            inspector,
            status_line: self.status_line.clone(),
            hovered_axis: self.viewport_controller.hovered_axis(),
            viewport_size: self.viewport_controller.viewport().size,
            scene_viewport_settings: self.viewport_controller.settings().clone(),
            mesh_import_path: self.mesh_import_path.clone(),
            project_overview: self.asset_workspace.project_overview(),
            asset_activity: self
                .asset_workspace
                .build_snapshot(AssetSurfaceMode::Activity),
            asset_browser: self
                .asset_workspace
                .build_snapshot(AssetSurfaceMode::Explorer),
            project_path: self.project_path.clone(),
            session_mode: self.session_mode,
            welcome: self.welcome.clone(),
            project_open: self.project_open,
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
        }
    }
}

fn inspector_plugin_components(
    scene: &Scene,
    node_id: NodeId,
    draft_fields: &BTreeMap<String, String>,
) -> Vec<InspectorPluginComponentSnapshot> {
    scene
        .dynamic_components_for_entity(node_id)
        .into_iter()
        .map(|component| {
            let descriptor = component.descriptor.as_ref();
            let component_id = component.component_id;
            let plugin_id = descriptor
                .map(|descriptor| descriptor.plugin_id.clone())
                .unwrap_or_else(|| plugin_id_from_component_id(&component_id));
            let display_name = descriptor
                .map(|descriptor| descriptor.display_name.clone())
                .unwrap_or_else(|| component_display_name(&component_id));
            let drawer_available = descriptor.is_some();
            let diagnostic = (!drawer_available).then(|| {
                format!(
                    "Plugin component drawer unavailable for `{component_id}`; serialized data stays protected until the plugin reloads."
                )
            });
            let properties =
                inspector_plugin_component_properties(&component_id, &component.value, descriptor, draft_fields);
            InspectorPluginComponentSnapshot {
                component_id,
                display_name,
                plugin_id,
                drawer_available,
                diagnostic,
                properties,
            }
        })
        .collect()
}

fn inspector_plugin_component_properties(
    component_id: &str,
    value: &Value,
    descriptor: Option<&ComponentTypeDescriptor>,
    draft_fields: &BTreeMap<String, String>,
) -> Vec<InspectorPluginComponentPropertySnapshot> {
    if let Some(descriptor) = descriptor.filter(|descriptor| !descriptor.properties.is_empty()) {
        return descriptor
            .properties
            .iter()
            .map(|property| {
                inspector_plugin_component_property_from_descriptor(
                    component_id,
                    value,
                    property,
                    draft_fields,
                )
            })
            .collect();
    }

    let editable = descriptor.is_some();
    let Some(object) = value.as_object() else {
        return vec![InspectorPluginComponentPropertySnapshot {
            field_id: format!("{component_id}.value"),
            name: "value".to_string(),
            label: "Value".to_string(),
            value: json_value_label(value),
            value_kind: json_value_kind(value).to_string(),
            editable: false,
        }];
    };

    let mut properties = object
        .iter()
        .map(|(name, value)| {
            let field_id = format!("{component_id}.{name}");
            let (value, primitive_editable) = json_edit_value(value);
            InspectorPluginComponentPropertySnapshot {
                field_id: field_id.clone(),
                name: name.clone(),
                label: property_label(name),
                value: draft_fields.get(&field_id).cloned().unwrap_or(value),
                value_kind: json_value_kind(object.get(name).unwrap_or(&Value::Null)).to_string(),
                editable: editable && primitive_editable,
            }
        })
        .collect::<Vec<_>>();
    properties.sort_by(|left, right| left.name.cmp(&right.name));
    properties
}

fn inspector_plugin_component_property_from_descriptor(
    component_id: &str,
    component_value: &Value,
    property: &ComponentPropertyDescriptor,
    draft_fields: &BTreeMap<String, String>,
) -> InspectorPluginComponentPropertySnapshot {
    let field_id = format!("{component_id}.{}", property.name);
    let value = component_value
        .as_object()
        .and_then(|object| object.get(&property.name))
        .unwrap_or(&Value::Null);
    let (value, primitive_editable) = json_edit_value(value);
    InspectorPluginComponentPropertySnapshot {
        field_id: field_id.clone(),
        name: property.name.clone(),
        label: property_label(&property.name),
        value: draft_fields.get(&field_id).cloned().unwrap_or(value),
        value_kind: property.value_type.clone(),
        editable: property.editable && primitive_editable,
    }
}

fn plugin_id_from_component_id(component_id: &str) -> String {
    component_id
        .split_once('.')
        .map(|(plugin_id, _)| plugin_id.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn component_display_name(component_id: &str) -> String {
    component_id
        .rsplit('.')
        .next()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(component_id)
        .to_string()
}

fn property_label(name: &str) -> String {
    let mut label = String::new();
    for (index, segment) in name
        .split('_')
        .filter(|segment| !segment.is_empty())
        .enumerate()
    {
        if index > 0 {
            label.push(' ');
        }
        let mut chars = segment.chars();
        if let Some(first) = chars.next() {
            label.extend(first.to_uppercase());
            label.push_str(chars.as_str());
        }
    }
    if label.is_empty() {
        name.to_string()
    } else {
        label
    }
}

fn json_edit_value(value: &Value) -> (String, bool) {
    match value {
        Value::Bool(value) => (value.to_string(), true),
        Value::Number(value) => (value.to_string(), true),
        Value::String(value) => (value.clone(), true),
        Value::Null => (String::new(), false),
        Value::Array(_) | Value::Object(_) => (json_value_label(value), false),
    }
}

fn json_value_label(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn json_value_kind(value: &Value) -> &'static str {
    match value {
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::Null => "null",
    }
}

fn hierarchy_depth(scene: &Scene, node_id: zircon_runtime::scene::NodeId) -> usize {
    let mut depth = 0;
    let mut cursor = scene.find_node(node_id).and_then(|node| node.parent);
    while let Some(parent) = cursor {
        depth += 1;
        cursor = scene.find_node(parent).and_then(|node| node.parent);
    }
    depth
}
