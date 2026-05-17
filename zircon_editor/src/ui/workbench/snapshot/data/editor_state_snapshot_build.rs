use crate::core::editor_extension::ComponentDrawerDescriptor;
use crate::ui::workbench::state::EditorState;
use serde_json::Value;
use std::collections::BTreeMap;
use zircon_runtime::scene::{NodeId, Scene};
use zircon_runtime_interface::reflect::{
    ReflectFieldValue, ReflectFieldsRequest, ReflectObjectAddress, ReflectTypeRegistration,
    ReflectedValue,
};

use super::super::{
    AssetSurfaceMode, EditorDataSnapshot, InspectorPluginComponentPropertySnapshot,
    InspectorPluginComponentSnapshot, InspectorSnapshot, SceneEntry,
};

impl EditorState {
    pub fn snapshot(&self) -> EditorDataSnapshot {
        self.snapshot_with_component_drawers(&BTreeMap::new())
    }

    pub(crate) fn snapshot_with_component_drawers(
        &self,
        component_drawers: &BTreeMap<String, ComponentDrawerDescriptor>,
    ) -> EditorDataSnapshot {
        let selected = self.viewport_controller.selected_node();
        let (scene_entries, inspector) = self
            .world
            .try_with_world(|scene| {
                let selected = scene.editor_projection(selected).selected_entity;
                let inspector = selected.map(|id| InspectorSnapshot {
                    id,
                    name: self.name_field.clone(),
                    parent: self.parent_field.clone(),
                    translation: self.transform_fields.clone(),
                    plugin_components: inspector_plugin_components(
                        scene,
                        id,
                        &self.inspector_dynamic_fields,
                        component_drawers,
                    ),
                });
                let scene_entries = scene
                    .node_records()
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
    component_drawers: &BTreeMap<String, ComponentDrawerDescriptor>,
) -> Vec<InspectorPluginComponentSnapshot> {
    scene
        .dynamic_components_for_entity(node_id)
        .into_iter()
        .map(|component| {
            let component_id = component.component_id;
            let schema = scene.reflect_schema(&component_id).ok();
            let plugin_id = schema
                .as_ref()
                .and_then(|schema| schema.plugin_id.clone())
                .or_else(|| {
                    component
                        .descriptor
                        .as_ref()
                        .map(|descriptor| descriptor.plugin_id.clone())
                })
                .unwrap_or_else(|| plugin_id_from_component_id(&component_id));
            let display_name = schema
                .as_ref()
                .map(|schema| schema.display_name.clone())
                .or_else(|| {
                    component
                        .descriptor
                        .as_ref()
                        .map(|descriptor| descriptor.display_name.clone())
                })
                .unwrap_or_else(|| component_display_name(&component_id));
            let drawer = component_drawers.get(&component_id);
            let drawer_available = schema.is_some() && drawer.is_some();
            let diagnostic = inspector_plugin_component_diagnostic(
                &component_id,
                schema.is_some(),
                drawer.is_some(),
            );
            let properties = if let Some(schema) = schema.as_ref() {
                inspector_plugin_component_reflected_properties(
                    scene,
                    node_id,
                    &component_id,
                    schema,
                    draft_fields,
                )
                .unwrap_or_else(|| {
                    inspector_plugin_component_json_properties(
                        &component_id,
                        &component.value,
                        false,
                        draft_fields,
                    )
                })
            } else {
                inspector_plugin_component_json_properties(
                    &component_id,
                    &component.value,
                    false,
                    draft_fields,
                )
            };
            InspectorPluginComponentSnapshot {
                component_id,
                display_name,
                plugin_id,
                drawer_available,
                drawer_ui_document: drawer.map(|drawer| drawer.ui_document().to_string()),
                drawer_controller: drawer.map(|drawer| drawer.controller().to_string()),
                drawer_template_id: drawer
                    .and_then(ComponentDrawerDescriptor::template_id)
                    .map(str::to_string),
                drawer_data_root: drawer
                    .and_then(ComponentDrawerDescriptor::data_root)
                    .map(str::to_string),
                drawer_bindings: drawer
                    .map(|drawer| drawer.bindings().to_vec())
                    .unwrap_or_default(),
                diagnostic,
                properties,
            }
        })
        .collect()
}

fn inspector_plugin_component_diagnostic(
    component_id: &str,
    has_runtime_schema: bool,
    has_editor_drawer: bool,
) -> Option<String> {
    if !has_runtime_schema {
        return Some(format!(
            "Plugin component drawer unavailable for `{component_id}`; serialized data stays protected until the plugin reloads."
        ));
    }
    if !has_editor_drawer {
        return Some(format!(
            "Plugin component drawer unavailable for `{component_id}`; editing is protected until an enabled editor extension registers a drawer."
        ));
    }
    None
}

fn inspector_plugin_component_reflected_properties(
    scene: &Scene,
    node_id: NodeId,
    component_id: &str,
    schema: &ReflectTypeRegistration,
    draft_fields: &BTreeMap<String, String>,
) -> Option<Vec<InspectorPluginComponentPropertySnapshot>> {
    let address = ReflectObjectAddress::component(node_id, component_id).ok()?;
    let fields = scene
        .reflect_fields(ReflectFieldsRequest::new(address))
        .ok()?
        .fields;
    let mut properties = schema
        .type_info
        .fields
        .iter()
        .filter(|field| field.editor_visible)
        .filter_map(|field| {
            let value = fields
                .iter()
                .find(|candidate| candidate.field_name == field.name)?;
            Some(inspector_plugin_component_property_from_reflected_field(
                component_id,
                value,
                &field.display_name,
                &field.value_type_path,
                field.editable,
                draft_fields,
            ))
        })
        .collect::<Vec<_>>();
    properties.sort_by(|left, right| left.name.cmp(&right.name));
    Some(properties)
}

fn inspector_plugin_component_json_properties(
    component_id: &str,
    value: &Value,
    editable: bool,
    draft_fields: &BTreeMap<String, String>,
) -> Vec<InspectorPluginComponentPropertySnapshot> {
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

fn inspector_plugin_component_property_from_reflected_field(
    component_id: &str,
    field: &ReflectFieldValue,
    display_name: &str,
    value_type_path: &str,
    editable: bool,
    draft_fields: &BTreeMap<String, String>,
) -> InspectorPluginComponentPropertySnapshot {
    let field_id = format!("{component_id}.{}", field.field_name);
    let value = reflected_value_label(&field.value);
    InspectorPluginComponentPropertySnapshot {
        field_id: field_id.clone(),
        name: field.field_name.clone(),
        label: property_label(display_name),
        value: draft_fields.get(&field_id).cloned().unwrap_or(value),
        value_kind: value_type_path.to_string(),
        editable: editable && reflected_value_primitive_editable(&field.value),
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

fn reflected_value_label(value: &ReflectedValue) -> String {
    match value {
        ReflectedValue::Null => String::new(),
        ReflectedValue::Bool(value) => value.to_string(),
        ReflectedValue::Integer(value) => value.to_string(),
        ReflectedValue::Unsigned(value) => value.to_string(),
        ReflectedValue::Scalar(value) => value.to_string(),
        ReflectedValue::String(value)
        | ReflectedValue::Enum(value)
        | ReflectedValue::Resource(value) => value.clone(),
        ReflectedValue::Vec2(value) => format!("{}, {}", value[0], value[1]),
        ReflectedValue::Vec3(value) => format!("{}, {}, {}", value[0], value[1], value[2]),
        ReflectedValue::Vec4(value) | ReflectedValue::Quaternion(value) => {
            format!("{}, {}, {}, {}", value[0], value[1], value[2], value[3])
        }
        ReflectedValue::Entity(Some(value)) => value.to_string(),
        ReflectedValue::Entity(None) => String::new(),
        ReflectedValue::List(values) => format!("{} items", values.len()),
        ReflectedValue::Map(values) => format!("{} fields", values.len()),
        ReflectedValue::Json(value) => json_value_label(value),
    }
}

fn reflected_value_primitive_editable(value: &ReflectedValue) -> bool {
    matches!(
        value,
        ReflectedValue::Bool(_)
            | ReflectedValue::Integer(_)
            | ReflectedValue::Unsigned(_)
            | ReflectedValue::Scalar(_)
            | ReflectedValue::String(_)
            | ReflectedValue::Enum(_)
            | ReflectedValue::Resource(_)
            | ReflectedValue::Vec2(_)
            | ReflectedValue::Vec3(_)
            | ReflectedValue::Vec4(_)
            | ReflectedValue::Quaternion(_)
            | ReflectedValue::Entity(_)
    )
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
