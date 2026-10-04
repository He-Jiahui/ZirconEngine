use std::collections::BTreeMap;
use std::sync::Arc;

use zircon_runtime::scene::{NodeId, Scene, WorldInspectionField};
use zircon_runtime_interface::math::Vec3;
use zircon_runtime_interface::reflect::{ReflectObjectAddress, ReflectReadRequest, ReflectedValue};
use zircon_runtime_interface::resource::{MaterialMarker, ModelMarker, ResourceHandle};

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::MergeMode;
use crate::core::editing::intent::EditorIntent;
use crate::core::play::WorldDomain;

use super::no_project_open::no_project_open;
use super::parse_parent_field::parse_parent_field;
use super::{
    editor_state::EditorState, EditorStateOperationError, InspectorEditError,
    InspectorTransformField,
};

const NAME_COMPONENT_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";
const HIERARCHY_COMPONENT_TYPE_PATH: &str = "zircon_runtime::scene::components::Hierarchy";
const LOCAL_TRANSFORM_COMPONENT_TYPE_PATH: &str =
    "zircon_runtime::scene::components::LocalTransform";

impl EditorState {
    pub(crate) fn select_node_in_world(
        &mut self,
        world_domain: WorldDomain,
        node_id: NodeId,
    ) -> Result<bool, EditorStateOperationError> {
        let active = self.viewport_controller.selection().active_domain();
        if active != world_domain {
            return Err(EditorStateOperationError::SelectionWorldMismatch {
                requested: world_domain,
                active,
            });
        }

        if world_domain == WorldDomain::Edit {
            if self
                .world
                .with_world(|scene| scene.find_node(node_id).is_none())?
                .ok_or_else(no_project_open)?
            {
                return Err(EditorStateOperationError::SelectedNodeMissing { node_id });
            }
        } else if !self.is_playing() {
            return Err(EditorStateOperationError::PlayWorldNotActive);
        }

        let changed = self
            .viewport_controller
            .selection_mut()
            .select_only_active(node_id);
        if world_domain == WorldDomain::Edit {
            self.sync_selection_state();
        }
        self.set_status_line(match world_domain {
            WorldDomain::Edit => format!("Selected node {node_id}"),
            WorldDomain::Play(instance) => {
                format!("Selected runtime entity {node_id} in {instance:?}")
            }
        });
        Ok(changed)
    }

    pub fn delete_selected(&mut self) -> Result<bool, EditorStateOperationError> {
        if self
            .viewport_controller
            .selection()
            .active_items()
            .is_empty()
        {
            self.set_status_line("Nothing selected");
            return Ok(false);
        }
        let selected = self
            .viewport_controller
            .selection()
            .active_items()
            .iter()
            .copied()
            .collect::<Vec<_>>();
        self.apply_intent(EditorIntent::DeleteNodes(selected))
    }

    pub fn apply_inspector_changes(&mut self) -> Result<bool, EditorStateOperationError> {
        self.prepare_non_gizmo_scene_action()?;
        if self
            .viewport_controller
            .selection()
            .active_items()
            .is_empty()
        {
            return Err(InspectorEditError::NoSelection.into());
        }
        if self.inspector_edited_fields.is_empty() && self.inspector_dynamic_fields.is_empty() {
            return Ok(false);
        }
        if self.inspector_draft_context.is_some() && !self.inspector_draft_context_is_current() {
            return Ok(false);
        }
        let selected = self
            .viewport_controller
            .selection()
            .active_items()
            .iter()
            .copied()
            .collect::<Vec<_>>();

        let edited = self.inspector_edited_fields;
        let name = if edited.name {
            let name = self.name_field.trim().to_owned();
            if name.is_empty() {
                return Err(InspectorEditError::EmptyNodeName.into());
            }
            Some(name)
        } else {
            None
        };
        let parent = edited
            .parent
            .then(|| parse_parent_field(&self.parent_field))
            .transpose()?;
        let translation = parse_edited_axes(
            &self.transform_fields,
            edited.translation,
            InspectorTransformField::Translation,
        )?;
        let scale = parse_edited_axes(
            &self.scale_fields,
            edited.scale,
            InspectorTransformField::Scale,
        )?;
        let mut commands = Vec::new();

        for node_id in &selected {
            let mut reflected_updates = self.prepare_reflected_node_updates(
                *node_id,
                name.as_deref(),
                parent,
                translation,
                scale,
            )?;
            reflected_updates.extend(self.prepare_reflected_component_updates(*node_id)?);
            for update in reflected_updates {
                let result = self.capture_scene_command(|scene| {
                    EditorCommand::set_reflected_scene_field(
                        scene,
                        *node_id,
                        update.component_type_path,
                        update.field_name,
                        update.value,
                    )
                })?;
                if let Some(command) = result {
                    if commands.is_empty() {
                        commands.reserve(selected.len().saturating_mul(4));
                    }
                    commands.push(command);
                }
            }
        }

        if commands.is_empty() {
            self.clear_inspector_draft_edits();
            return Ok(false);
        }
        self.execute_scene_commands("Apply inspector changes", commands, MergeMode::Disable)?;
        self.set_status_line(format!(
            "Applied inspector changes to {} selected nodes",
            selected.len()
        ));
        Ok(true)
    }

    pub(crate) fn apply_play_inspector_changes(
        &mut self,
        node_id: NodeId,
        changes: &[(String, String)],
    ) -> Result<bool, EditorStateOperationError> {
        self.prepare_non_gizmo_scene_action()?;
        let WorldDomain::Play(instance) = self.viewport_controller.selection().active_domain()
        else {
            return Err(EditorStateOperationError::PlayWorldNotActive);
        };
        if !self.is_playing()
            || self.viewport_controller.selection().active_primary() != Some(node_id)
        {
            return Err(EditorStateOperationError::PlayWorldNotActive);
        }

        let draft_changes = if changes.is_empty() {
            self.pending_play_inspector_changes()
        } else {
            Vec::new()
        };
        let changes = if changes.is_empty() {
            draft_changes.as_slice()
        } else {
            changes
        };
        if changes.is_empty() {
            self.set_status_line("Play Inspector values are unchanged");
            return Ok(false);
        }

        self.bind_transaction_context()?;
        let history_context = self.scene_history_context()?;
        debug_assert_eq!(
            history_context,
            crate::core::editing::engine::HistoryContextId::PlaySession(instance)
        );
        let editor_context = Arc::clone(&self.context);
        let mut scope = editor_context
            .transactions()
            .begin("Apply play inspector changes", history_context)?;
        scope.set_merge_mode(MergeMode::Disable);
        let capture = scope
            .with_context_mut::<CoreEditContext, _>(|context| {
                context.with_scene(|scene| capture_play_inspector_commands(scene, node_id, changes))
            })?
            .ok_or(EditorStateOperationError::TransactionContextMissing)?;
        let capture = capture?;
        let commands = capture?;
        if commands.is_empty() {
            scope.cancel()?;
            self.clear_inspector_draft_edits();
            self.set_status_line("Play Inspector values are unchanged");
            return Ok(false);
        }
        for command in commands {
            scope.push(command)?;
        }
        scope.commit_after_apply(|selection_after| {
            self.sync_selection_from_transaction_snapshot(selection_after)
        })?;
        self.clear_inspector_draft_edits();
        self.set_status_line("Applied Inspector changes to the play world");
        Ok(true)
    }

    fn prepare_reflected_node_updates(
        &self,
        node_id: NodeId,
        name: Option<&str>,
        parent: Option<Option<NodeId>>,
        translation: [Option<f32>; 3],
        scale: [Option<f32>; 3],
    ) -> Result<Vec<ReflectedInspectorUpdate>, EditorStateOperationError> {
        let mut updates = Vec::with_capacity(4);
        if let Some(name) = name {
            updates.push(ReflectedInspectorUpdate {
                component_type_path: NAME_COMPONENT_TYPE_PATH.to_string(),
                field_name: "value".to_string(),
                value: ReflectedValue::String(name.to_owned()),
            });
        }
        if let Some(parent) = parent {
            updates.push(ReflectedInspectorUpdate {
                component_type_path: HIERARCHY_COMPONENT_TYPE_PATH.to_string(),
                field_name: "parent".to_string(),
                value: ReflectedValue::Entity(parent),
            });
        }
        if translation.iter().any(Option::is_some) || scale.iter().any(Option::is_some) {
            let transform_updates = self
                .world
                .with_world(|scene| {
                    let mut transform_updates = BTreeMap::new();
                    insert_play_vec3_update(
                        scene,
                        node_id,
                        "translation",
                        translation,
                        &mut transform_updates,
                    )?;
                    insert_play_vec3_update(
                        scene,
                        node_id,
                        "scale",
                        scale,
                        &mut transform_updates,
                    )?;
                    Ok::<_, InspectorEditError>(transform_updates)
                })?
                .ok_or_else(no_project_open)??;
            updates.extend(transform_updates.into_iter().map(
                |((component_type_path, field_name), value)| ReflectedInspectorUpdate {
                    component_type_path,
                    field_name,
                    value,
                },
            ));
        }
        Ok(updates)
    }

    fn prepare_reflected_component_updates(
        &self,
        node_id: NodeId,
    ) -> Result<Vec<ReflectedInspectorUpdate>, EditorStateOperationError> {
        let dynamic_fields = &self.inspector_dynamic_fields;
        if dynamic_fields.is_empty() {
            return Ok(Vec::new());
        }
        let updates = self
            .world
            .with_world(|scene| {
                dynamic_fields
                    .iter()
                    .map(|(field_id, value)| {
                        let (component_type_path, field_name) = split_reflected_field_id(field_id)?;
                        let current = read_reflected_inspector_value(
                            scene,
                            node_id,
                            &component_type_path,
                            &field_name,
                            field_id,
                        )?;
                        let value = reflected_value_from_text(value, &current)?;
                        Ok(ReflectedInspectorUpdate {
                            component_type_path,
                            field_name,
                            value,
                        })
                    })
                    .collect::<Result<Vec<_>, InspectorEditError>>()
            })?
            .ok_or_else(no_project_open)??;
        Ok(updates)
    }

    pub fn import_mesh_asset(
        &mut self,
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
        display_path: impl Into<String>,
    ) -> Result<bool, EditorStateOperationError> {
        self.prepare_non_gizmo_scene_action()?;
        let command = EditorCommand::import_mesh(model, material);
        self.execute_scene_command("Import mesh scene node", command)?;
        let id = self
            .viewport_controller
            .selection()
            .active_primary()
            .ok_or(EditorStateOperationError::ImportedMeshNodeNotSelected)?;
        self.mesh_import_path = display_path.into();
        self.set_status_line(format!("Imported mesh node {id}"));
        Ok(true)
    }

    pub(crate) fn sync_selection_state(&mut self) {
        self.clear_inspector_draft_edits();
        let selected_state = self
            .viewport_controller
            .selection()
            .active_primary()
            .and_then(|selected| {
                match self
                    .world
                    .with_world(|scene| selected_inspector_state(scene, selected))
                {
                    Ok(Some(selected_state)) => selected_state,
                    Ok(None) => None,
                    Err(error) => {
                        self.report_authoring_world_access_failure(
                            "selection synchronization",
                            &error,
                        );
                        None
                    }
                }
            });
        if let Some(node) = selected_state {
            self.inspector_dynamic_fields.clear();
            self.name_field = node.name;
            self.parent_field = node
                .parent
                .map(|value| value.to_string())
                .unwrap_or_default();
            self.transform_fields = [
                format!("{:.2}", node.translation.x),
                format!("{:.2}", node.translation.y),
                format!("{:.2}", node.translation.z),
            ];
            self.scale_fields = [
                format!("{:.2}", node.scale.x),
                format!("{:.2}", node.scale.y),
                format!("{:.2}", node.scale.z),
            ];
            self.viewport_controller
                .set_orbit_target(node.world_translation);
            return;
        }

        self.name_field.clear();
        self.parent_field.clear();
        self.transform_fields = [String::new(), String::new(), String::new()];
        self.scale_fields = [String::new(), String::new(), String::new()];
        self.inspector_dynamic_fields.clear();
    }
}

struct ReflectedInspectorUpdate {
    component_type_path: String,
    field_name: String,
    value: ReflectedValue,
}

fn capture_play_inspector_commands(
    scene: &Scene,
    node_id: NodeId,
    changes: &[(String, String)],
) -> Result<Vec<EditorCommand>, EditorStateOperationError> {
    let mut updates = BTreeMap::<(String, String), ReflectedValue>::new();
    let mut translation_axes = [None; 3];
    let mut scale_axes = [None; 3];

    for (field_id, value) in changes {
        match field_id.as_str() {
            "name" => {
                let value = value.trim();
                if value.is_empty() {
                    return Err(InspectorEditError::EmptyNodeName.into());
                }
                updates.insert(
                    (NAME_COMPONENT_TYPE_PATH.to_owned(), "value".to_owned()),
                    ReflectedValue::String(value.to_owned()),
                );
            }
            "parent" => {
                updates.insert(
                    (
                        HIERARCHY_COMPONENT_TYPE_PATH.to_owned(),
                        "parent".to_owned(),
                    ),
                    ReflectedValue::Entity(parse_entity_value(value)?),
                );
            }
            "transform.translation.x" => {
                translation_axes[0] = Some(parse_play_inspector_axis(
                    value,
                    InspectorTransformField::Translation,
                )?);
            }
            "transform.translation.y" => {
                translation_axes[1] = Some(parse_play_inspector_axis(
                    value,
                    InspectorTransformField::Translation,
                )?);
            }
            "transform.translation.z" => {
                translation_axes[2] = Some(parse_play_inspector_axis(
                    value,
                    InspectorTransformField::Translation,
                )?);
            }
            "transform.scale.x" => {
                scale_axes[0] = Some(parse_play_inspector_axis(
                    value,
                    InspectorTransformField::Scale,
                )?);
            }
            "transform.scale.y" => {
                scale_axes[1] = Some(parse_play_inspector_axis(
                    value,
                    InspectorTransformField::Scale,
                )?);
            }
            "transform.scale.z" => {
                scale_axes[2] = Some(parse_play_inspector_axis(
                    value,
                    InspectorTransformField::Scale,
                )?);
            }
            _ => {
                let (component_type_path, field_name) = split_reflected_field_id(field_id)?;
                let current = read_reflected_inspector_value(
                    scene,
                    node_id,
                    &component_type_path,
                    &field_name,
                    field_id,
                )?;
                updates.insert(
                    (component_type_path, field_name),
                    reflected_value_from_text(value, &current)?,
                );
            }
        }
    }

    insert_play_vec3_update(
        scene,
        node_id,
        "translation",
        translation_axes,
        &mut updates,
    )?;
    insert_play_vec3_update(scene, node_id, "scale", scale_axes, &mut updates)?;

    let mut commands = Vec::with_capacity(updates.len());
    for ((component_type_path, field_name), value) in updates {
        if let Some(command) = EditorCommand::set_reflected_scene_field(
            scene,
            node_id,
            component_type_path,
            field_name,
            value,
        )? {
            commands.push(command);
        }
    }
    Ok(commands)
}

fn insert_play_vec3_update(
    scene: &Scene,
    node_id: NodeId,
    field_name: &str,
    axes: [Option<f32>; 3],
    updates: &mut BTreeMap<(String, String), ReflectedValue>,
) -> Result<(), InspectorEditError> {
    if axes.iter().all(Option::is_none) {
        return Ok(());
    }
    let field_id = format!("transform.{field_name}");
    let current = read_reflected_inspector_value(
        scene,
        node_id,
        LOCAL_TRANSFORM_COMPONENT_TYPE_PATH,
        field_name,
        &field_id,
    )?;
    let ReflectedValue::Vec3(mut value) = current else {
        return Err(InspectorEditError::UnsupportedValueKind);
    };
    for (current, update) in value.iter_mut().zip(axes) {
        if let Some(update) = update {
            *current = update;
        }
    }
    updates.insert(
        (
            LOCAL_TRANSFORM_COMPONENT_TYPE_PATH.to_owned(),
            field_name.to_owned(),
        ),
        ReflectedValue::Vec3(value),
    );
    Ok(())
}

fn parse_play_inspector_axis(
    value: &str,
    field: InspectorTransformField,
) -> Result<f32, InspectorEditError> {
    let value = value
        .trim()
        .parse::<f32>()
        .map_err(|_| InspectorEditError::InvalidTransformFields { field })?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(InspectorEditError::InvalidTransformFields { field })
    }
}

fn read_reflected_inspector_value(
    scene: &Scene,
    node_id: NodeId,
    component_type_path: &str,
    field_name: &str,
    field_id: &str,
) -> Result<ReflectedValue, InspectorEditError> {
    let field_stable_id = scene
        .reflect_schema(component_type_path)
        .ok()
        .and_then(|schema| {
            schema
                .type_info
                .fields
                .into_iter()
                .find(|field| field.name == field_name)
                .map(|field| field.id)
        })
        .ok_or_else(|| InspectorEditError::UnsupportedFieldId {
            field_id: field_id.to_owned(),
        })?;
    let address =
        ReflectObjectAddress::component(node_id, component_type_path).map_err(|source| {
            InspectorEditError::ReflectionRead {
                field_id: field_id.to_owned(),
                source,
            }
        })?;
    scene
        .reflect_read(ReflectReadRequest::new(address, field_stable_id))
        .map(|response| response.field.value)
        .map_err(|source| InspectorEditError::ReflectionRead {
            field_id: field_id.to_owned(),
            source,
        })
}

struct SelectedInspectorState {
    name: String,
    parent: Option<NodeId>,
    translation: Vec3,
    world_translation: Vec3,
    scale: Vec3,
}

fn selected_inspector_state(scene: &Scene, selected: NodeId) -> Option<SelectedInspectorState> {
    let hierarchy = scene.inspection_artifact();
    let row = hierarchy.hierarchy_row(selected)?;
    let fields = scene.inspection_fields_artifact(selected)?;
    let translation = inspection_vec3_field(fields.fields(), "translation")?;
    let world_translation = scene.world_transform(selected)?.translation;
    let scale = inspection_vec3_field(fields.fields(), "scale")?;
    Some(SelectedInspectorState {
        name: row.display_name.clone(),
        parent: row.parent,
        translation,
        world_translation,
        scale,
    })
}

fn parse_edited_axes(
    fields: &[String; 3],
    edited: [bool; 3],
    field: InspectorTransformField,
) -> Result<[Option<f32>; 3], InspectorEditError> {
    let mut values = [None; 3];
    for axis in 0..3 {
        if edited[axis] {
            values[axis] = Some(parse_play_inspector_axis(&fields[axis], field)?);
        }
    }
    Ok(values)
}

fn inspection_vec3_field(fields: &[WorldInspectionField], field_name: &str) -> Option<Vec3> {
    fields
        .iter()
        .find(|field| {
            field.component_type_path == LOCAL_TRANSFORM_COMPONENT_TYPE_PATH
                && field.field_name == field_name
        })
        .and_then(|field| match &field.value {
            ReflectedValue::Vec3([x, y, z]) => Some(Vec3::new(*x, *y, *z)),
            _ => None,
        })
}

fn split_reflected_field_id(field_id: &str) -> Result<(String, String), InspectorEditError> {
    let (component_type_path, field_name) =
        field_id
            .rsplit_once('.')
            .ok_or_else(|| InspectorEditError::UnsupportedFieldId {
                field_id: field_id.to_string(),
            })?;
    if component_type_path.trim().is_empty() || field_name.trim().is_empty() {
        return Err(InspectorEditError::UnsupportedFieldId {
            field_id: field_id.to_string(),
        });
    }
    Ok((component_type_path.to_string(), field_name.to_string()))
}

fn reflected_value_from_text(
    value: &str,
    current: &ReflectedValue,
) -> Result<ReflectedValue, InspectorEditError> {
    match current {
        ReflectedValue::Bool(_) => parse_bool(value).map(ReflectedValue::Bool),
        ReflectedValue::Integer(_) => value
            .trim()
            .parse::<i64>()
            .map(ReflectedValue::Integer)
            .map_err(|_| InspectorEditError::InvalidSignedInteger {
                value: value.to_string(),
            }),
        ReflectedValue::Unsigned(_) => value
            .trim()
            .parse::<u64>()
            .map(ReflectedValue::Unsigned)
            .map_err(|_| InspectorEditError::InvalidUnsignedInteger {
                value: value.to_string(),
            }),
        ReflectedValue::Scalar(_) => value
            .trim()
            .parse::<f32>()
            .map(ReflectedValue::Scalar)
            .map_err(|_| InspectorEditError::InvalidNumber {
                value: value.to_string(),
            }),
        ReflectedValue::String(_) => Ok(ReflectedValue::String(value.to_string())),
        ReflectedValue::Enum(_) => Ok(ReflectedValue::Enum(value.to_string())),
        ReflectedValue::Resource(_) => Ok(ReflectedValue::Resource(value.to_string())),
        ReflectedValue::Vec2(_) => parse_f32_array::<2>(value, "Vec2").map(ReflectedValue::Vec2),
        ReflectedValue::Vec3(_) => parse_f32_array::<3>(value, "Vec3").map(ReflectedValue::Vec3),
        ReflectedValue::Vec4(_) => parse_f32_array::<4>(value, "Vec4").map(ReflectedValue::Vec4),
        ReflectedValue::Quaternion(_) => {
            parse_f32_array::<4>(value, "Quaternion").map(ReflectedValue::Quaternion)
        }
        ReflectedValue::Entity(_) => parse_entity_value(value).map(ReflectedValue::Entity),
        ReflectedValue::Null
        | ReflectedValue::List(_)
        | ReflectedValue::Map(_)
        | ReflectedValue::Json(_) => Err(InspectorEditError::UnsupportedValueKind),
    }
}

fn parse_bool(value: &str) -> Result<bool, InspectorEditError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(InspectorEditError::InvalidBool {
            value: value.to_string(),
        }),
    }
}

fn parse_f32_array<const N: usize>(
    value: &str,
    type_name: &'static str,
) -> Result<[f32; N], InspectorEditError> {
    let trimmed = value.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .or_else(|| {
            trimmed
                .strip_prefix('(')
                .and_then(|value| value.strip_suffix(')'))
        })
        .unwrap_or(trimmed);
    let components = inner
        .split(|character: char| character == ',' || character.is_ascii_whitespace())
        .filter(|component| !component.trim().is_empty())
        .collect::<Vec<_>>();

    if components.len() != N {
        return Err(InspectorEditError::InvalidVector {
            value: value.to_string(),
            type_name,
            component_count: N,
        });
    }

    let mut parsed = [0.0_f32; N];
    for (slot, component) in parsed.iter_mut().zip(components) {
        let component =
            component
                .parse::<f32>()
                .map_err(|_| InspectorEditError::InvalidVector {
                    value: value.to_string(),
                    type_name,
                    component_count: N,
                })?;
        if !component.is_finite() {
            return Err(InspectorEditError::InvalidVector {
                value: value.to_string(),
                type_name,
                component_count: N,
            });
        }
        *slot = component;
    }

    Ok(parsed)
}

fn parse_entity_value(value: &str) -> Result<Option<NodeId>, InspectorEditError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("none")
        || trimmed.eq_ignore_ascii_case("null")
    {
        return Ok(None);
    }
    trimmed
        .parse::<NodeId>()
        .map(Some)
        .map_err(|_| InspectorEditError::InvalidEntity {
            value: value.to_string(),
        })
}

#[cfg(test)]
#[path = "tests/editor_state_selection_performance_tests.rs"]
mod performance_tests;
