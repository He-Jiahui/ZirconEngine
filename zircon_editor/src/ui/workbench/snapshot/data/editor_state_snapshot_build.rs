use crate::core::editing::engine::HistoryContextId;
use crate::core::extension::{
    FieldEditorContainer, FieldEditorInstance, InspectTarget, InspectTargetType,
    InspectorCustomizationChain, InspectorField,
};
use crate::ui::workbench::state::EditorState;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use zircon_runtime::scene::{NodeId, Scene};
use zircon_runtime_interface::reflect::{
    ReflectFieldValue, ReflectFieldsRequest, ReflectObjectAddress, ReflectTypeRegistration,
    ReflectedValue,
};

use super::super::{
    EditorDataSnapshot, InspectorPluginComponentPropertySnapshot, InspectorPluginComponentSnapshot,
    InspectorSnapshot, SceneEntries,
};

impl EditorState {
    pub fn snapshot(&self) -> EditorDataSnapshot {
        let field_editors = FieldEditorContainer::builtin();
        self.snapshot_with_inspector_customizations(
            &InspectorCustomizationChain::default(),
            &field_editors,
        )
    }

    pub(crate) fn snapshot_with_inspector_customizations(
        &self,
        inspector_customizations: &InspectorCustomizationChain,
        field_editors: &FieldEditorContainer,
    ) -> EditorDataSnapshot {
        let selection = self.viewport_controller.selection();
        let selected = selection.active_primary();
        let selected_items = selection.active_items().iter().copied().collect::<Vec<_>>();
        let (scene_entries, inspector) = self
            .world
            .try_with_world(|scene| {
                let hierarchy = scene.inspection_artifact();
                let selected = selected.filter(|entity| hierarchy.hierarchy_row(*entity).is_some());
                let selected_items = selected_items
                    .iter()
                    .copied()
                    .filter(|entity| hierarchy.hierarchy_row(*entity).is_some())
                    .collect::<BTreeSet<_>>();
                let inspector = selected.map(|id| InspectorSnapshot {
                    id,
                    name: self.name_field.clone(),
                    parent: self.parent_field.clone(),
                    translation: self.transform_fields.clone(),
                    plugin_components: inspector_plugin_components(
                        scene,
                        id,
                        &self.inspector_dynamic_fields,
                        inspector_customizations,
                        field_editors,
                    ),
                });
                let scene_entries = self
                    .scene_entry_projection_cache
                    .project(&hierarchy, &selected_items);

                (scene_entries, inspector)
            })
            .unwrap_or_else(|| (SceneEntries::default(), None));
        let (asset_activity, asset_browser) = self.asset_workspace.build_surface_snapshots();

        let history = (!self.is_playing())
            .then(|| self.transactions().history_status(HistoryContextId::Global))
            .and_then(Result::ok);
        EditorDataSnapshot {
            scene_entries,
            inspector,
            status_line: self.status_line.clone(),
            console_output: self.console_output(),
            status_task_progress: self.status_task_progress.clone(),
            hovered_axis: self.viewport_controller.hovered_axis(),
            viewport_size: self.viewport_controller.viewport().size,
            scene_viewport_settings: self.viewport_controller.chrome_settings(),
            mesh_import_path: self.mesh_import_path.clone(),
            project_overview: self.asset_workspace.project_overview(),
            asset_activity,
            asset_browser,
            project_path: self.project_path.clone(),
            session_mode: self.session_mode,
            welcome: self.welcome.clone(),
            project_open: self.project_open,
            can_undo: history.is_some_and(|history| history.can_undo),
            can_redo: history.is_some_and(|history| history.can_redo),
            bridge_diagnostics: self.bridge_diagnostics.clone(),
        }
    }
}

fn inspector_plugin_components(
    scene: &Scene,
    node_id: NodeId,
    draft_fields: &BTreeMap<String, String>,
    inspector_customizations: &InspectorCustomizationChain,
    field_editors: &FieldEditorContainer,
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
            let customization = InspectTargetType::new(component_id.clone())
                .ok()
                .and_then(|target_type| {
                    InspectTarget::new(target_type, format!("component:{component_id}")).ok()
                })
                .and_then(|target| inspector_customizations.matching(&target));
            let customization_available = schema.is_some() && customization.is_some();
            let diagnostic = inspector_plugin_component_diagnostic(
                &component_id,
                schema.is_some(),
                customization.is_some(),
            );
            let properties = if let Some(schema) = schema.as_ref() {
                inspector_plugin_component_reflected_properties(
                    scene,
                    node_id,
                    &component_id,
                    schema,
                    draft_fields,
                    field_editors,
                )
                .unwrap_or_else(|| {
                    inspector_plugin_component_json_properties(
                        &component_id,
                        &component.value,
                        false,
                        draft_fields,
                        field_editors,
                    )
                })
            } else {
                inspector_plugin_component_json_properties(
                    &component_id,
                    &component.value,
                    false,
                    draft_fields,
                    field_editors,
                )
            };
            InspectorPluginComponentSnapshot {
                component_id,
                display_name,
                plugin_id,
                customization_available,
                customization_ui_document: customization
                    .and_then(|customization| customization.surface())
                    .map(|surface| surface.ui_document().to_string()),
                customization_controller: customization
                    .and_then(|customization| customization.surface())
                    .map(|surface| surface.controller().to_string()),
                customization_template_id: customization
                    .and_then(|customization| customization.surface())
                    .and_then(|surface| surface.template_id())
                    .map(str::to_string),
                customization_data_root: customization
                    .and_then(|customization| customization.surface())
                    .and_then(|surface| surface.data_root())
                    .map(str::to_string),
                customization_bindings: customization
                    .and_then(|customization| customization.surface())
                    .map(|surface| surface.bindings().to_vec())
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
    has_inspector_customization: bool,
) -> Option<String> {
    if !has_runtime_schema {
        return Some(format!(
            "Plugin inspector customization unavailable for `{component_id}`; serialized data stays protected until the plugin reloads."
        ));
    }
    if !has_inspector_customization {
        return Some(format!(
            "Plugin inspector customization unavailable for `{component_id}`; editing is protected until an enabled editor extension registers a customization."
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
    field_editors: &FieldEditorContainer,
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
                field_editors,
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
    field_editors: &FieldEditorContainer,
) -> Vec<InspectorPluginComponentPropertySnapshot> {
    let Some(object) = value.as_object() else {
        let field_id = format!("{component_id}.value");
        let value_kind = json_value_kind(value).to_string();
        let value = json_value_label(value);
        return vec![InspectorPluginComponentPropertySnapshot {
            field_editor: field_editor_for(
                &field_id,
                "Value",
                &value_kind,
                &value,
                false,
                field_editors,
            ),
            field_id,
            name: "value".to_string(),
            label: "Value".to_string(),
            value,
            value_kind,
            editable: false,
        }];
    };

    let mut properties = object
        .iter()
        .map(|(name, value)| {
            let field_id = format!("{component_id}.{name}");
            let (value, primitive_editable) = json_edit_value(value);
            let label = property_label(name);
            let value_kind = json_value_kind(object.get(name).unwrap_or(&Value::Null)).to_string();
            let editable = editable && primitive_editable;
            let value = draft_fields.get(&field_id).cloned().unwrap_or(value);
            InspectorPluginComponentPropertySnapshot {
                field_editor: field_editor_for(
                    &field_id,
                    &label,
                    &value_kind,
                    &value,
                    editable,
                    field_editors,
                ),
                field_id: field_id.clone(),
                name: name.clone(),
                label,
                value,
                value_kind,
                editable,
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
    field_editors: &FieldEditorContainer,
) -> InspectorPluginComponentPropertySnapshot {
    let field_id = format!("{component_id}.{}", field.field_name);
    let value = reflected_value_label(&field.value);
    let label = property_label(display_name);
    let value = draft_fields.get(&field_id).cloned().unwrap_or(value);
    let editable = editable && reflected_value_primitive_editable(&field.value);
    InspectorPluginComponentPropertySnapshot {
        field_editor: field_editor_for(
            &field_id,
            &label,
            value_type_path,
            &value,
            editable,
            field_editors,
        ),
        field_id: field_id.clone(),
        name: field.field_name.clone(),
        label,
        value,
        value_kind: value_type_path.to_string(),
        editable,
    }
}

fn field_editor_for(
    field_id: &str,
    label: &str,
    value_kind: &str,
    value: &str,
    editable: bool,
    field_editors: &FieldEditorContainer,
) -> FieldEditorInstance {
    InspectorField::new(field_id, label, value_kind, value, editable)
        .map(|field| field_editors.resolve(field))
        .unwrap_or_else(|_| FieldEditorInstance::automatic())
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
