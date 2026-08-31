//! Pure scene edit commands executed by the shared transaction engine.

use std::any::Any;
use std::io;

use serde::{Deserialize, Serialize};
use zircon_runtime::scene::components::{CameraComponent, NodeKind, NodeRecord};
use zircon_runtime::scene::{DetachedEntityBatch, NodeId, Scene, SceneError};
use zircon_runtime_interface::math::Transform;
use zircon_runtime_interface::reflect::{
    ReflectFieldId, ReflectObjectAddress, ReflectReadRequest, ReflectWriteRequest, ReflectedValue,
};
use zircon_runtime_interface::resource::{MaterialMarker, ModelMarker, ResourceHandle};

use super::context::CoreEditContext;
use super::engine::{
    CommandEffect, CommandExecutionError, CommandJournalPayload, CommandJournalUnavailable,
    EditCommand, EditCommandError, EditContext, MergeOutcome,
};
use super::selection::SceneSelection;

mod batch_transform;
mod play_transform;
pub(crate) use batch_transform::{
    BatchTransformCommand, BatchTransformJournalPayload, BatchTransformTarget,
};
use play_transform::PlayTransformCommand;

pub(crate) enum EditorCommand {
    CreateNode(CreateNodeCommand),
    DeleteNode(DeleteNodeCommand),
    UpdateNode(UpdateNodeCommand),
    BatchTransform(BatchTransformCommand),
    PlayTransform(PlayTransformCommand),
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

    pub(crate) fn applied_transform_batch(command: BatchTransformCommand) -> Self {
        Self::BatchTransform(command)
    }

    pub(crate) fn applied_play_transform(
        node_id: NodeId,
        interaction_id: u64,
        world_replacement_epoch: u64,
        before: Transform,
        after: Transform,
    ) -> Option<Self> {
        PlayTransformCommand::new(
            node_id,
            interaction_id,
            world_replacement_epoch,
            before,
            after,
            true,
        )
        .map(Self::PlayTransform)
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

    pub(crate) fn from_journal_create(
        payload: CreateNodeJournalPayload,
    ) -> Result<Self, EditCommandError> {
        CreateNodeCommand::from_journal(payload).map(Self::CreateNode)
    }

    pub(crate) fn from_journal_delete(payload: DeleteNodeJournalPayload) -> Self {
        Self::DeleteNode(DeleteNodeCommand::from_journal(payload))
    }

    pub(crate) fn from_journal_update(
        payload: UpdateNodeJournalPayload,
    ) -> Result<Self, EditCommandError> {
        UpdateNodeCommand::from_journal(payload).map(Self::UpdateNode)
    }

    pub(crate) fn from_journal_batch_transform(
        payload: BatchTransformJournalPayload,
    ) -> Result<Self, EditCommandError> {
        BatchTransformCommand::from_journal(payload.targets).map(Self::BatchTransform)
    }

    pub(crate) fn from_journal_reflected_field(
        payload: SetReflectedSceneFieldJournalPayload,
    ) -> Result<Self, EditCommandError> {
        SetReflectedSceneFieldCommand::from_journal(payload).map(Self::SetReflectedSceneField)
    }
}

impl EditCommand for EditorCommand {
    fn label(&self) -> &str {
        match self {
            Self::CreateNode(_) => "Create scene node",
            Self::DeleteNode(_) => "Delete scene node",
            Self::UpdateNode(_) => "Update scene node",
            Self::BatchTransform(_) => "Transform scene selection",
            Self::PlayTransform(_) => "Transform Play scene node",
            Self::SetReflectedSceneField(_) => "Set reflected scene field",
        }
    }

    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let context = core_context(context)?;
        match self {
            Self::CreateNode(command) => command.apply(context),
            Self::DeleteNode(command) => command.apply(context),
            Self::UpdateNode(command) => command.apply(context),
            Self::BatchTransform(command) => command.apply(context),
            Self::PlayTransform(command) => command.apply(context),
            Self::SetReflectedSceneField(command) => command.apply(context),
        }
    }

    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let context = core_context(context)?;
        match self {
            Self::CreateNode(command) => command.revert(context),
            Self::DeleteNode(command) => command.revert(context),
            Self::UpdateNode(command) => command.revert(context),
            Self::BatchTransform(command) => command.revert(context),
            Self::PlayTransform(command) => command.revert(context),
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
            (Self::PlayTransform(current), Self::PlayTransform(next))
                if current.node_id == next.node_id
                    && current.world_replacement_epoch == next.world_replacement_epoch =>
            {
                current.after = next.after;
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
            Self::BatchTransform(command) => command.journal_payload(),
            Self::PlayTransform(command) => command.journal_payload(),
            Self::SetReflectedSceneField(command) => command.journal_payload(),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) enum CreateNodeIntent {
    Node {
        kind: NodeKind,
    },
    Mesh {
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
    },
}

enum SceneWriteCompletion<T> {
    Completed(T),
    AppliedThenGatewayFailed { value: T, error: EditCommandError },
}

fn execute_scene_write<T>(
    context: &CoreEditContext,
    write: impl FnOnce(&mut Scene) -> Result<T, EditCommandError>,
) -> Result<SceneWriteCompletion<T>, CommandExecutionError> {
    let (result, post_callback_error) = context
        .with_scene_mut(write)
        .map_err(unchanged)?
        .into_parts();
    let value = result.map_err(unchanged)?;
    Ok(match post_callback_error {
        Some(error) => SceneWriteCompletion::AppliedThenGatewayFailed { value, error },
        None => SceneWriteCompletion::Completed(value),
    })
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

    fn from_journal(payload: CreateNodeJournalPayload) -> Result<Self, EditCommandError> {
        if !payload.intent.matches_record(&payload.record) {
            return Err(EditCommandError::InvariantViolation {
                invariant: "create scene node journal intent must match its retained record kind",
            });
        }
        Ok(Self {
            intent: payload.intent,
            record: Some(payload.record),
        })
    }

    fn apply(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        if let Some(retained) = self.record.as_ref() {
            let node_id = retained.id;
            let record = retained.clone();
            match execute_scene_write(context, move |scene| {
                scene
                    .insert_node_record(record)
                    .map_err(|error| external_error(error.to_string()))
            })? {
                SceneWriteCompletion::Completed(()) => {}
                SceneWriteCompletion::AppliedThenGatewayFailed { error, .. } => {
                    return Err(applied(error));
                }
            }
            context
                .set_scene_selection(SceneSelection::new(vec![node_id], Some(node_id)))
                .map_err(applied)?;
            return Ok(());
        }

        let intent = self.intent.clone();
        let completion = execute_scene_write(context, |scene| {
            let node_id = match intent {
                CreateNodeIntent::Node { kind } => scene
                    .spawn_node(kind)
                    .map_err(|error| scene_error("create node", error))?,
                CreateNodeIntent::Mesh { model, material } => scene
                    .spawn_mesh_node(model, material)
                    .map_err(|error| scene_error("import mesh node", error))?,
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
        })?;
        let record = match completion {
            SceneWriteCompletion::Completed(record) => record,
            SceneWriteCompletion::AppliedThenGatewayFailed { value, error } => {
                self.record = Some(value);
                return Err(applied(error));
            }
        };
        let node_id = record.id;
        self.record = Some(record);
        context
            .set_scene_selection(SceneSelection::new(vec![node_id], Some(node_id)))
            .map_err(applied)?;
        Ok(())
    }

    fn revert(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        let Some(node_id) = self.record.as_ref().map(|record| record.id) else {
            return Err(unchanged(EditCommandError::InvariantViolation {
                invariant: "create command must be applied before it can be reverted",
            }));
        };
        match execute_scene_write(context, |scene| {
            scene
                .remove_entity(node_id)
                .map_err(|error| external_error(error.to_string()))
        })? {
            SceneWriteCompletion::Completed(()) => Ok(()),
            SceneWriteCompletion::AppliedThenGatewayFailed { error, .. } => Err(applied(error)),
        }
    }

    fn journal_payload(&self) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        let record = self.record.as_ref().ok_or_else(|| {
            CommandJournalUnavailable::new("create scene node command has not retained its record")
        })?;
        journal_payload(
            "zircon.editor.scene.create_node",
            &CreateNodeJournalPayload {
                intent: self.intent.clone(),
                record: record.clone(),
            },
        )
    }
}

impl CreateNodeIntent {
    fn matches_record(&self, record: &NodeRecord) -> bool {
        match self {
            Self::Node { kind } => record.kind == *kind,
            Self::Mesh { .. } => record.kind == NodeKind::Mesh,
        }
    }
}

pub(crate) struct DeleteNodeCommand {
    root_id: NodeId,
    batch: Option<DetachedEntityBatch>,
    fallback_selection: Option<NodeId>,
    journal: DeleteNodeJournalPayload,
}

impl DeleteNodeCommand {
    fn capture(scene: &Scene, node_id: NodeId) -> Result<Self, EditCommandError> {
        if !scene.contains_entity(node_id) {
            return Err(EditCommandError::TargetMissing {
                target: format!("scene node {node_id}"),
            });
        }
        let camera_count = scene.camera_count();
        let detached_camera_count = scene.subtree_component_count::<CameraComponent>(node_id);
        if detached_camera_count >= camera_count {
            return Err(EditCommandError::InvariantViolation {
                invariant: "cannot delete the last remaining camera",
            });
        }
        let fallback_selection = scene
            .parent_of(node_id)
            .filter(|parent| scene.contains_entity(*parent));
        Ok(Self {
            root_id: node_id,
            batch: None,
            fallback_selection,
            journal: DeleteNodeJournalPayload {
                root_id: node_id,
                fallback_selection,
            },
        })
    }

    fn from_journal(payload: DeleteNodeJournalPayload) -> Self {
        Self {
            root_id: payload.root_id,
            batch: None,
            fallback_selection: payload.fallback_selection,
            journal: payload,
        }
    }

    fn apply(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        if self.batch.is_some() {
            return Err(unchanged(EditCommandError::InvariantViolation {
                invariant: "delete command must be reverted before it can be applied again",
            }));
        }
        let before = context.scene_selection().map_err(unchanged)?;
        let root_id = self.root_id;
        let fallback = self.fallback_selection;
        let completion = execute_scene_write(context, |scene| {
            let batch = scene
                .remove_entity_recursive(root_id)
                .map_err(|error| scene_error("detach scene subtree", error))?;
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
            Ok((batch, SceneSelection::new(items, primary)))
        })?;
        let ((batch, surviving), post_callback_error) = match completion {
            SceneWriteCompletion::Completed(value) => (value, None),
            SceneWriteCompletion::AppliedThenGatewayFailed { value, error } => (value, Some(error)),
        };
        self.batch = Some(batch);
        if let Some(error) = post_callback_error {
            return Err(applied(error));
        }
        context.set_scene_selection(surviving).map_err(applied)?;
        Ok(())
    }

    fn revert(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        let batch = self.batch.take().ok_or_else(|| {
            unchanged(EditCommandError::InvariantViolation {
                invariant: "delete command must retain a detached batch before it can be reverted",
            })
        })?;
        let mut retained = Some(batch);
        let restore = context.with_scene_mut(|scene| {
            let batch = retained
                .take()
                .ok_or(EditCommandError::InvariantViolation {
                    invariant:
                        "delete command restore callback must consume its detached batch once",
                })?;
            scene.restore_detached_entity_batch(batch).map_err(|error| {
                let (source, batch) = error.into_parts();
                retained = Some(batch);
                scene_error("restore detached entity batch", source)
            })
        });
        match restore {
            Ok(outcome) => {
                let (result, post_callback_error) = outcome.into_parts();
                match result {
                    Ok(()) => {
                        self.batch = retained;
                        match post_callback_error {
                            Some(error) => Err(applied(error)),
                            None => Ok(()),
                        }
                    }
                    Err(error) => {
                        self.batch = retained;
                        Err(unchanged(error))
                    }
                }
            }
            Err(error) => {
                self.batch = retained;
                Err(unchanged(error))
            }
        }
    }

    fn journal_payload(&self) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        journal_payload("zircon.editor.scene.delete_node", &self.journal)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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

    fn from_journal(payload: UpdateNodeJournalPayload) -> Result<Self, EditCommandError> {
        validate_replayed_edit_state(&payload.after)?;
        Self::new(payload.node_id, payload.before, payload.after, false).ok_or(
            EditCommandError::InvariantViolation {
                invariant: "update scene node journal must change the retained node state",
            },
        )
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
            match execute_scene_write(context, |scene| {
                apply_node_state(scene, self.node_id, &self.before, &self.after)
            })? {
                SceneWriteCompletion::Completed(()) => {}
                SceneWriteCompletion::AppliedThenGatewayFailed { error, .. } => {
                    return Err(applied(error));
                }
            }
        }
        Ok(())
    }

    fn revert(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        match execute_scene_write(context, |scene| {
            apply_node_state(scene, self.node_id, &self.after, &self.before)
        })? {
            SceneWriteCompletion::Completed(()) => Ok(()),
            SceneWriteCompletion::AppliedThenGatewayFailed { error, .. } => Err(applied(error)),
        }
    }

    fn journal_payload(&self) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        journal_payload(
            "zircon.editor.scene.update_node",
            &UpdateNodeJournalPayload {
                node_id: self.node_id,
                before: self.before.clone(),
                after: self.after.clone(),
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

fn validate_replayed_edit_state(state: &NodeEditState) -> Result<(), EditCommandError> {
    if state.name.is_empty() || state.name.trim() != state.name {
        return Err(EditCommandError::InvariantViolation {
            invariant: "replayed node name must be nonempty and normalized",
        });
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub(crate) struct SetReflectedSceneFieldCommand {
    node_id: NodeId,
    component_type_path: String,
    field_id: ReflectFieldId,
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
        let field_id =
            resolve_editable_reflected_field_id(scene, &component_type_path, &field_name)?;
        let before =
            read_reflected_component_field(scene, node_id, &component_type_path, field_id)?;
        Ok((before != after).then_some(Self {
            node_id,
            component_type_path,
            field_id,
            before,
            after,
        }))
    }

    fn from_journal(
        payload: SetReflectedSceneFieldJournalPayload,
    ) -> Result<Self, EditCommandError> {
        if payload.component_type_path.trim().is_empty() {
            return Err(EditCommandError::InvariantViolation {
                invariant: "replayed reflected field journal requires a component type",
            });
        }
        if payload.before == payload.after {
            return Err(EditCommandError::InvariantViolation {
                invariant: "replayed reflected field journal must change the field value",
            });
        }
        Ok(Self {
            node_id: payload.node_id,
            component_type_path: payload.component_type_path,
            field_id: payload.field_id,
            before: payload.before,
            after: payload.after,
        })
    }

    fn apply(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        match execute_scene_write(context, |scene| {
            write_reflected_component_field(
                scene,
                self.node_id,
                &self.component_type_path,
                self.field_id,
                self.after.clone(),
            )
        })? {
            SceneWriteCompletion::Completed(_) => Ok(()),
            SceneWriteCompletion::AppliedThenGatewayFailed { error, .. } => Err(applied(error)),
        }
    }

    fn revert(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        match execute_scene_write(context, |scene| {
            write_reflected_component_field(
                scene,
                self.node_id,
                &self.component_type_path,
                self.field_id,
                self.before.clone(),
            )
        })? {
            SceneWriteCompletion::Completed(_) => Ok(()),
            SceneWriteCompletion::AppliedThenGatewayFailed { error, .. } => Err(applied(error)),
        }
    }

    fn journal_payload(&self) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        journal_payload(
            "zircon.editor.scene.set_reflected_field",
            &SetReflectedSceneFieldJournalPayload {
                node_id: self.node_id,
                component_type_path: self.component_type_path.clone(),
                field_id: self.field_id,
                before: self.before.clone(),
                after: self.after.clone(),
            },
        )
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct CreateNodeJournalPayload {
    intent: CreateNodeIntent,
    record: NodeRecord,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct DeleteNodeJournalPayload {
    root_id: NodeId,
    fallback_selection: Option<NodeId>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct UpdateNodeJournalPayload {
    node_id: NodeId,
    before: NodeEditState,
    after: NodeEditState,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SetReflectedSceneFieldJournalPayload {
    node_id: NodeId,
    component_type_path: String,
    field_id: ReflectFieldId,
    before: ReflectedValue,
    after: ReflectedValue,
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

fn resolve_editable_reflected_field_id(
    scene: &Scene,
    component_type_path: &str,
    field_name: &str,
) -> Result<ReflectFieldId, EditCommandError> {
    let schema = scene
        .reflect_schema(component_type_path)
        .map_err(|error| reflect_error(error.to_string()))?;
    let field = schema
        .type_info
        .fields
        .iter()
        .find(|field| field.name == field_name)
        .ok_or_else(|| {
            reflect_error(format!(
                "reflected field `{field_name}` is not registered on `{component_type_path}`"
            ))
        })?;
    if !field.editable {
        return Err(reflect_error(format!(
            "reflected field `{field_name}` on `{component_type_path}` is not editable"
        )));
    }
    Ok(field.id)
}

fn read_reflected_component_field(
    scene: &Scene,
    node_id: NodeId,
    component_type_path: &str,
    field_id: ReflectFieldId,
) -> Result<ReflectedValue, EditCommandError> {
    let address = ReflectObjectAddress::component(node_id, component_type_path)
        .map_err(|error| reflect_error(error.to_string()))?;
    scene
        .reflect_read(ReflectReadRequest::new(address, field_id))
        .map(|response| response.field.value)
        .map_err(|error| reflect_error(error.to_string()))
}

fn write_reflected_component_field(
    scene: &mut Scene,
    node_id: NodeId,
    component_type_path: &str,
    field_id: ReflectFieldId,
    value: ReflectedValue,
) -> Result<bool, EditCommandError> {
    let address = ReflectObjectAddress::component(node_id, component_type_path)
        .map_err(|error| reflect_error(error.to_string()))?;
    scene
        .reflect_write(ReflectWriteRequest::new(address, field_id, value))
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

fn applied(source: EditCommandError) -> CommandExecutionError {
    CommandExecutionError::applied(source)
}

fn scene_error(operation: &'static str, source: SceneError) -> EditCommandError {
    EditCommandError::SceneMutation { operation, source }
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

    #[test]
    fn delete_node_undo_keeps_only_the_move_only_runtime_inverse_delta() {
        let source = include_str!("command.rs");
        let start = source
            .find("pub(crate) struct DeleteNodeCommand")
            .expect("delete command declaration should remain available");
        let end = source[start..]
            .find("pub(crate) struct NodeEditState")
            .map(|offset| start + offset)
            .expect("delete command region should end before node edit state");
        let delete_command = &source[start..end];

        assert!(delete_command.contains("batch: Option<DetachedEntityBatch>"));
        assert!(!delete_command.contains("records: Vec<NodeRecord>"));
        assert!(!delete_command.contains("subtree_records("));
        assert!(!delete_command.contains("insert_node_records("));
        assert!(!delete_command.contains(".expect("));
    }
}
