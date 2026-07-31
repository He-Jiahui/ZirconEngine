//! Pure scene edit commands executed by the shared transaction engine.

use std::any::Any;
use std::io;

use serde::Serialize;
use zircon_runtime::scene::components::{NodeKind, NodeRecord};
use zircon_runtime::scene::{NodeId, Scene};
use zircon_runtime_interface::math::Transform;
use zircon_runtime_interface::reflect::{
    ReflectObjectAddress, ReflectReadRequest, ReflectWriteRequest, ReflectedValue,
};
use zircon_runtime_interface::resource::{MaterialMarker, ModelMarker, ResourceHandle};

use super::context::CoreEditContext;
use super::engine::{
    CommandEffect, CommandExecutionError, CommandJournalPayload, CommandJournalUnavailable,
    EditCommand, EditCommandError, EditContext, MergeOutcome,
};
use super::selection::SceneSelection;

#[derive(Clone, Debug)]
pub(crate) enum EditorCommand {
    CreateNode(CreateNodeCommand),
    DeleteNode(DeleteNodeCommand),
    UpdateNode(UpdateNodeCommand),
    SetReflectedSceneField(SetReflectedSceneFieldCommand),
}

impl EditorCommand {
    pub(crate) fn create_node(kind: NodeKind) -> Self {
        Self::CreateNode(CreateNodeCommand::new(CreateNodeIntent::Node { kind }))
    }

    pub(crate) fn import_mesh(
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    ) -> Self {
        Self::CreateNode(CreateNodeCommand::new(CreateNodeIntent::Mesh {
            model,
            material,
        }))
    }

    pub(crate) fn delete_node(scene: &Scene, node_id: NodeId) -> Result<Self, EditCommandError> {
        DeleteNodeCommand::capture(scene, node_id).map(Self::DeleteNode)
    }

    pub(crate) fn rename_node(
        scene: &Scene,
        node_id: NodeId,
        name: String,
    ) -> Result<Option<Self>, EditCommandError> {
        UpdateNodeCommand::capture_name(scene, node_id, name)
            .map(|command| command.map(Self::UpdateNode))
    }

    pub(crate) fn set_parent(
        scene: &Scene,
        node_id: NodeId,
        parent: Option<NodeId>,
    ) -> Result<Option<Self>, EditCommandError> {
        UpdateNodeCommand::capture_parent(scene, node_id, parent)
            .map(|command| command.map(Self::UpdateNode))
    }

    pub(crate) fn set_transform(
        scene: &Scene,
        node_id: NodeId,
        transform: Transform,
    ) -> Result<Option<Self>, EditCommandError> {
        UpdateNodeCommand::capture_transform(scene, node_id, transform)
            .map(|command| command.map(Self::UpdateNode))
    }

    pub(crate) fn applied_transform(
        node_id: NodeId,
        before: NodeEditState,
        after: NodeEditState,
    ) -> Option<Self> {
        UpdateNodeCommand::new(node_id, before, after, true).map(Self::UpdateNode)
    }

    pub(crate) fn set_reflected_scene_field(
        scene: &Scene,
        node_id: NodeId,
        component_type_path: impl Into<String>,
        field_name: impl Into<String>,
        after: ReflectedValue,
    ) -> Result<Option<Self>, EditCommandError> {
        SetReflectedSceneFieldCommand::capture(
            scene,
            node_id,
            component_type_path.into(),
            field_name.into(),
            after,
        )
        .map(|command| command.map(Self::SetReflectedSceneField))
    }
}

impl EditCommand for EditorCommand {
    fn label(&self) -> &str {
        match self {
            Self::CreateNode(_) => "Create scene node",
            Self::DeleteNode(_) => "Delete scene node",
            Self::UpdateNode(_) => "Update scene node",
            Self::SetReflectedSceneField(_) => "Set reflected scene field",
        }
    }

    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let context = core_context(context)?;
        match self {
            Self::CreateNode(command) => command.apply(context),
            Self::DeleteNode(command) => command.apply(context),
            Self::UpdateNode(command) => command.apply(context),
            Self::SetReflectedSceneField(command) => command.apply(context),
        }
    }

    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let context = core_context(context)?;
        match self {
            Self::CreateNode(command) => command.revert(context),
            Self::DeleteNode(command) => command.revert(context),
            Self::UpdateNode(command) => command.revert(context),
            Self::SetReflectedSceneField(command) => command.revert(context),
        }
    }

    fn try_merge(&mut self, next: &dyn EditCommand) -> MergeOutcome {
        let Some(next) = next.as_any().downcast_ref::<Self>() else {
            return MergeOutcome::Reject;
        };
        match (self, next) {
            (Self::UpdateNode(current), Self::UpdateNode(next))
                if current.node_id == next.node_id =>
            {
                current.after = next.after.clone();
                current.already_applied = false;
                MergeOutcome::Merged
            }
            _ => MergeOutcome::Reject,
        }
    }

    fn journal_payload(&self) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        match self {
            Self::CreateNode(command) => command.journal_payload(),
            Self::DeleteNode(command) => command.journal_payload(),
            Self::UpdateNode(command) => command.journal_payload(),
            Self::SetReflectedSceneField(command) => command.journal_payload(),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug, Serialize)]
enum CreateNodeIntent {
    Node {
        kind: NodeKind,
    },
    Mesh {
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct CreateNodeCommand {
    intent: CreateNodeIntent,
    record: Option<NodeRecord>,
}

impl CreateNodeCommand {
    fn new(intent: CreateNodeIntent) -> Self {
        Self {
            intent,
            record: None,
        }
    }

    fn apply(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        if let Some(retained) = self.record.as_ref() {
            let node_id = retained.id;
            let record = retained.clone();
            context
                .with_scene_mut(move |scene| scene.insert_node_record(record))
                .map_err(unchanged)?
                .map_err(|error| unchanged(external_error(error.to_string())))?;
            context
                .set_scene_selection(SceneSelection::new(vec![node_id], Some(node_id)))
                .map_err(unchanged)?;
            return Ok(());
        }

        let intent = self.intent.clone();
        let record = context
            .with_scene_mut(|scene| {
                let node_id = match intent {
                    CreateNodeIntent::Node { kind } => scene.spawn_node(kind),
                    CreateNodeIntent::Mesh { model, material } => {
                        scene.spawn_mesh_node(model, material)
                    }
                };
                match scene.node_record(node_id) {
                    Some(record) => Ok(record),
                    None => {
                        let _ = scene.remove_entity(node_id);
                        Err(EditCommandError::TargetMissing {
                            target: format!("created node {node_id}"),
                        })
                    }
                }
            })
            .map_err(unchanged)?
            .map_err(unchanged)?;
        let node_id = record.id;
        self.record = Some(record);
        context
            .set_scene_selection(SceneSelection::new(vec![node_id], Some(node_id)))
            .map_err(unchanged)?;
        Ok(())
    }

    fn revert(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        let Some(node_id) = self.record.as_ref().map(|record| record.id) else {
            return Err(unchanged(EditCommandError::InvariantViolation {
                invariant: "create command must be applied before it can be reverted",
            }));
        };
        let removed = context
            .with_scene_mut(|scene| scene.remove_entity(node_id))
            .map_err(unchanged)?;
        if !removed {
            return Err(unchanged(EditCommandError::TargetMissing {
                target: format!("scene node {node_id}"),
            }));
        }
        Ok(())
    }

    fn journal_payload(&self) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        let record = self.record.as_ref().ok_or_else(|| {
            CommandJournalUnavailable::new("create scene node command has not retained its record")
        })?;
        journal_payload(
            "zircon.editor.scene.create_node",
            &CreateNodeJournalPayload {
                intent: &self.intent,
                record,
            },
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DeleteNodeCommand {
    root_id: NodeId,
    records: Vec<NodeRecord>,
    previous_active_camera: NodeId,
    fallback_selection: Option<NodeId>,
    active_camera_after: Option<NodeId>,
}

impl DeleteNodeCommand {
    fn capture(scene: &Scene, node_id: NodeId) -> Result<Self, EditCommandError> {
        let records = scene.subtree_records(node_id);
        if records.is_empty() {
            return Err(EditCommandError::TargetMissing {
                target: format!("scene node {node_id}"),
            });
        }
        let removed_camera_count = records
            .iter()
            .filter(|record| record.camera.is_some())
            .count();
        if removed_camera_count >= scene.camera_count() {
            return Err(EditCommandError::InvariantViolation {
                invariant: "cannot delete the last remaining camera",
            });
        }
        Ok(Self {
            root_id: node_id,
            records,
            previous_active_camera: scene.active_camera(),
            fallback_selection: scene
                .parent_of(node_id)
                .filter(|parent| scene.contains_entity(*parent)),
            active_camera_after: None,
        })
    }

    fn apply(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        let before = context.scene_selection().map_err(unchanged)?;
        let root_id = self.root_id;
        let fallback = self.fallback_selection;
        let preferred_camera = self.active_camera_after;
        let (active_camera, surviving) = context
            .with_scene_mut(|scene| {
                let removed = scene.remove_entity_recursive(root_id);
                if removed.is_empty() {
                    return Err(EditCommandError::TargetMissing {
                        target: format!("scene node {root_id}"),
                    });
                }
                if let Some(camera) =
                    preferred_camera.filter(|camera| scene.contains_entity(*camera))
                {
                    scene.set_active_camera(camera);
                }
                let active_camera = scene.active_camera();
                let mut items = before
                    .items()
                    .iter()
                    .copied()
                    .filter(|node| scene.contains_entity(*node))
                    .collect::<Vec<_>>();
                if items.is_empty() {
                    if let Some(node) = fallback.or(Some(active_camera)) {
                        items.push(node);
                    }
                }
                let primary = before
                    .primary()
                    .filter(|node| items.contains(node))
                    .or_else(|| items.first().copied());
                Ok((active_camera, SceneSelection::new(items, primary)))
            })
            .map_err(unchanged)?
            .map_err(unchanged)?;
        self.active_camera_after = Some(active_camera);
        context.set_scene_selection(surviving).map_err(unchanged)?;
        Ok(())
    }

    fn revert(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        context
            .with_scene_mut(|scene| {
                scene
                    .insert_node_records(&self.records)
                    .map_err(|error| external_error(error.to_string()))?;
                scene.set_active_camera(self.previous_active_camera);
                Ok::<(), EditCommandError>(())
            })
            .map_err(unchanged)?
            .map_err(unchanged)
    }

    fn journal_payload(&self) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        journal_payload(
            "zircon.editor.scene.delete_node",
            &DeleteNodeJournalPayload {
                root_id: self.root_id,
                records: &self.records,
                previous_active_camera: self.previous_active_camera,
                fallback_selection: self.fallback_selection,
                active_camera_after: self.active_camera_after,
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct NodeEditState {
    pub(crate) name: String,
    pub(crate) parent: Option<NodeId>,
    pub(crate) transform: Transform,
}

impl NodeEditState {
    pub(crate) fn capture(scene: &Scene, node_id: NodeId) -> Result<Self, EditCommandError> {
        let node = scene
            .find_node(node_id)
            .ok_or_else(|| EditCommandError::TargetMissing {
                target: format!("scene node {node_id}"),
            })?;
        Ok(Self {
            name: node.name.clone(),
            parent: node.parent,
            transform: node.transform,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct UpdateNodeCommand {
    node_id: NodeId,
    before: NodeEditState,
    after: NodeEditState,
    already_applied: bool,
}

impl UpdateNodeCommand {
    pub(crate) fn new(
        node_id: NodeId,
        before: NodeEditState,
        after: NodeEditState,
        already_applied: bool,
    ) -> Option<Self> {
        (before != after).then_some(Self {
            node_id,
            before,
            after,
            already_applied,
        })
    }

    fn capture_name(
        scene: &Scene,
        node_id: NodeId,
        name: String,
    ) -> Result<Option<Self>, EditCommandError> {
        let before = NodeEditState::capture(scene, node_id)?;
        let mut after = before.clone();
        after.name = name;
        Self::capture_with_before(node_id, before, after)
    }

    fn capture_parent(
        scene: &Scene,
        node_id: NodeId,
        parent: Option<NodeId>,
    ) -> Result<Option<Self>, EditCommandError> {
        let before = NodeEditState::capture(scene, node_id)?;
        let mut after = before.clone();
        after.parent = parent;
        Self::capture_with_before(node_id, before, after)
    }

    fn capture_transform(
        scene: &Scene,
        node_id: NodeId,
        transform: Transform,
    ) -> Result<Option<Self>, EditCommandError> {
        let before = NodeEditState::capture(scene, node_id)?;
        let mut after = before.clone();
        after.transform = transform;
        Self::capture_with_before(node_id, before, after)
    }

    fn capture_with_before(
        node_id: NodeId,
        before: NodeEditState,
        after: NodeEditState,
    ) -> Result<Option<Self>, EditCommandError> {
        let after = normalize_edit_state(after)?;
        Ok(Self::new(node_id, before, after, false))
    }

    fn apply(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        if self.already_applied {
            self.already_applied = false;
        } else {
            context
                .with_scene_mut(|scene| {
                    apply_node_state(scene, self.node_id, &self.before, &self.after)
                })
                .map_err(unchanged)?
                .map_err(unchanged)?;
        }
        Ok(())
    }

    fn revert(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        context
            .with_scene_mut(|scene| {
                apply_node_state(scene, self.node_id, &self.after, &self.before)
            })
            .map_err(unchanged)?
            .map_err(unchanged)
    }

    fn journal_payload(&self) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        journal_payload(
            "zircon.editor.scene.update_node",
            &UpdateNodeJournalPayload {
                node_id: self.node_id,
                before: &self.before,
                after: &self.after,
            },
        )
    }
}

fn apply_node_state(
    scene: &mut Scene,
    node_id: NodeId,
    before: &NodeEditState,
    after: &NodeEditState,
) -> Result<(), EditCommandError> {
    if scene.find_node(node_id).is_none() {
        return Err(EditCommandError::TargetMissing {
            target: format!("scene node {node_id}"),
        });
    }
    if before.parent != after.parent {
        scene
            .set_parent_checked(node_id, after.parent)
            .map_err(|error| external_error(error.to_string()))?;
    }
    if before.name != after.name {
        scene
            .rename_node(node_id, after.name.clone())
            .map_err(|error| external_error(error.to_string()))?;
    }
    if before.transform != after.transform {
        scene
            .update_transform(node_id, after.transform)
            .map_err(|error| external_error(error.to_string()))?;
    }
    Ok(())
}

fn normalize_edit_state(mut state: NodeEditState) -> Result<NodeEditState, EditCommandError> {
    state.name = state.name.trim().to_string();
    if state.name.is_empty() {
        return Err(EditCommandError::InvariantViolation {
            invariant: "node name cannot be empty",
        });
    }
    Ok(state)
}

#[derive(Clone, Debug)]
pub(crate) struct SetReflectedSceneFieldCommand {
    node_id: NodeId,
    component_type_path: String,
    field_name: String,
    before: ReflectedValue,
    after: ReflectedValue,
}

impl SetReflectedSceneFieldCommand {
    fn capture(
        scene: &Scene,
        node_id: NodeId,
        component_type_path: String,
        field_name: String,
        after: ReflectedValue,
    ) -> Result<Option<Self>, EditCommandError> {
        ensure_reflected_field_editable(scene, &component_type_path, &field_name)?;
        let before =
            read_reflected_component_field(scene, node_id, &component_type_path, &field_name)?;
        Ok((before != after).then_some(Self {
            node_id,
            component_type_path,
            field_name,
            before,
            after,
        }))
    }

    fn apply(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        context
            .with_scene_mut(|scene| {
                write_reflected_component_field(
                    scene,
                    self.node_id,
                    &self.component_type_path,
                    &self.field_name,
                    self.after.clone(),
                )
            })
            .map_err(unchanged)?
            .map_err(unchanged)?;
        Ok(())
    }

    fn revert(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        context
            .with_scene_mut(|scene| {
                write_reflected_component_field(
                    scene,
                    self.node_id,
                    &self.component_type_path,
                    &self.field_name,
                    self.before.clone(),
                )
            })
            .map_err(unchanged)?
            .map(|_| ())
            .map_err(unchanged)
    }

    fn journal_payload(&self) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        journal_payload(
            "zircon.editor.scene.set_reflected_field",
            &SetReflectedSceneFieldJournalPayload {
                node_id: self.node_id,
                component_type_path: &self.component_type_path,
                field_name: &self.field_name,
                before: &self.before,
                after: &self.after,
            },
        )
    }
}

#[derive(Serialize)]
struct CreateNodeJournalPayload<'a> {
    intent: &'a CreateNodeIntent,
    record: &'a NodeRecord,
}

#[derive(Serialize)]
struct DeleteNodeJournalPayload<'a> {
    root_id: NodeId,
    records: &'a [NodeRecord],
    previous_active_camera: NodeId,
    fallback_selection: Option<NodeId>,
    active_camera_after: Option<NodeId>,
}

#[derive(Serialize)]
struct UpdateNodeJournalPayload<'a> {
    node_id: NodeId,
    before: &'a NodeEditState,
    after: &'a NodeEditState,
}

#[derive(Serialize)]
struct SetReflectedSceneFieldJournalPayload<'a> {
    node_id: NodeId,
    component_type_path: &'a str,
    field_name: &'a str,
    before: &'a ReflectedValue,
    after: &'a ReflectedValue,
}

fn journal_payload(
    command_type: &'static str,
    payload: &impl Serialize,
) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
    serde_json::to_value(payload)
        .map(|payload| CommandJournalPayload::new(command_type, 1, payload))
        .map_err(|error| {
            CommandJournalUnavailable::new(format!(
                "{command_type} journal payload serialization failed: {error}"
            ))
        })
}

fn ensure_reflected_field_editable(
    scene: &Scene,
    component_type_path: &str,
    field_name: &str,
) -> Result<(), EditCommandError> {
    let schema = scene
        .reflect_schema(component_type_path)
        .map_err(|error| reflect_error(error.to_string()))?;
    let editable = schema
        .type_info
        .fields
        .iter()
        .find(|field| field.name == field_name)
        .map(|field| field.editable)
        .ok_or_else(|| {
            reflect_error(format!(
                "reflected field `{field_name}` is not registered on `{component_type_path}`"
            ))
        })?;
    if !editable {
        return Err(reflect_error(format!(
            "reflected field `{field_name}` on `{component_type_path}` is not editable"
        )));
    }
    Ok(())
}

fn read_reflected_component_field(
    scene: &Scene,
    node_id: NodeId,
    component_type_path: &str,
    field_name: &str,
) -> Result<ReflectedValue, EditCommandError> {
    let address = ReflectObjectAddress::component(node_id, component_type_path)
        .map_err(|error| reflect_error(error.to_string()))?;
    scene
        .reflect_read(ReflectReadRequest::new(address, field_name))
        .map(|response| response.field.value)
        .map_err(|error| reflect_error(error.to_string()))
}

fn write_reflected_component_field(
    scene: &mut Scene,
    node_id: NodeId,
    component_type_path: &str,
    field_name: &str,
    value: ReflectedValue,
) -> Result<bool, EditCommandError> {
    let address = ReflectObjectAddress::component(node_id, component_type_path)
        .map_err(|error| reflect_error(error.to_string()))?;
    scene
        .reflect_write(ReflectWriteRequest::new(address, field_name, value))
        .map(|response| response.changed)
        .map_err(|error| reflect_error(error.to_string()))
}

fn core_context(
    context: &mut dyn EditContext,
) -> Result<&mut CoreEditContext, CommandExecutionError> {
    context
        .as_any_mut()
        .downcast_mut::<CoreEditContext>()
        .ok_or_else(|| {
            unchanged(EditCommandError::ContextTypeMismatch {
                expected: "CoreEditContext",
            })
        })
}

fn unchanged(source: EditCommandError) -> CommandExecutionError {
    CommandExecutionError {
        effect: CommandEffect::Unchanged,
        source,
    }
}

fn external_error(message: String) -> EditCommandError {
    EditCommandError::ExternalEffect {
        source: Box::new(io::Error::other(message)),
    }
}

fn reflect_error(message: String) -> EditCommandError {
    EditCommandError::ReflectError {
        source: Box::new(io::Error::other(message)),
    }
}

#[cfg(test)]
mod performance_source_guards {
    #[test]
    fn create_node_redo_clones_the_retained_record_only_once() {
        let source = include_str!("command.rs");
        let second_clone = ["insert_node_record", "(record.clone())"].concat();

        assert!(!source.contains(&second_clone));
    }
}
