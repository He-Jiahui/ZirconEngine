//! Pure scene edit commands executed by the shared transaction engine.

use std::any::Any;
use std::io;

use serde::{Deserialize, Serialize};
use zircon_runtime::scene::components::{CameraComponent, NodeKind, NodeRecord};
use zircon_runtime::scene::{DetachedEntityBatch, NodeId, Scene, SceneError};
use zircon_runtime_interface::math::Transform;
use zircon_runtime_interface::reflect::{
    ReflectError, ReflectFieldId, ReflectObjectAddress, ReflectReadRequest, ReflectWriteRequest,
    ReflectedValue,
};
use zircon_runtime_interface::resource::{MaterialMarker, ModelMarker, ResourceHandle};

use super::context::CoreEditContext;
use super::engine::{
    CommandEffect, CommandExecutionError, CommandJournalPayload, CommandJournalUnavailable,
    EditCommand, EditCommandError, EditContext, MergeOutcome,
};
use super::selection::SceneSelection;

#[path = "command/add_component.rs"]
mod add_component;
pub(crate) use add_component::{
    AddComponentCatalog, AddComponentCatalogEntry, AddComponentPayload, AddComponentRequest,
    AddComponentScope, AddDynamicComponentCommand, AddDynamicComponentFactory, GatewayStamp,
    ADD_COMPONENT_OPERATION,
};

#[path = "command/script_bindings.rs"]
mod script_bindings;
pub(crate) use script_bindings::{
    read_script_bindings, SetScriptBindingsCommand, SetScriptBindingsFactory,
    SetScriptBindingsPayload, SetScriptBindingsRequest, SET_SCRIPT_BINDINGS_JOURNAL,
    SET_SCRIPT_BINDINGS_OPERATION,
};

#[path = "command/batch_transform.rs"]
mod batch_transform;
#[path = "command/play_transform.rs"]
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
    SetScriptBindings(SetScriptBindingsCommand),
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

    fn plan_v1(
        &self,
        context: &dyn EditContext,
        phase: super::engine::RetentionPhaseV1,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
        peak_limit: u64,
    ) -> Result<super::engine::CommandRetentionPlanV1, EditCommandError> {
        use super::engine::{CommandRetentionPlanV1, RetentionPhaseV1};
        use zircon_runtime_interface::retained_memory::*;
        let mut plan = CommandRetentionPlanV1::for_owner(self, context, phase, model, limit)?;
        let core = context.as_any().downcast_ref::<CoreEditContext>().ok_or(
            EditCommandError::ContextTypeMismatch {
                expected: "CoreEditContext",
            },
        )?;
        if let Self::SetScriptBindings(command) = self {
            command.validate(
                core,
                matches!(phase, RetentionPhaseV1::Undo | RetentionPhaseV1::Rollback),
            )?;
            // Typed values, reflected reads and JSON materialization may overlap.
            plan.add_peak(
                plan.retained_bytes
                    .checked_mul(4)
                    .ok_or(RetentionContractError::Overflow)?,
            )?;
        }
        core.with_scene(|scene| -> Result<(), EditCommandError> {
            plan.owner_revision = Some(scene.world_generation());
            if let Self::SetScriptBindings(command) = self {
                // Staging is transient; quoted replay retains only the command
                // payload and reserves this scratch through materialization.
                let mut scratch =
                    RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(peak_limit));
                scene.dynamic_component_creation_peak_v1(
                    &[command.payload.node_id],
                    &serde_json::Value::Null,
                    zircon_runtime::asset::SceneScriptBindingAsset::COMPONENT_TYPE_PATH,
                    &mut scratch,
                )?;
                plan.add_peak(scratch.bytes().get())?;
            }
            if let Self::DeleteNode(command) = self {
                if !matches!(phase, RetentionPhaseV1::Undo | RetentionPhaseV1::Rollback) {
                    let mut m =
                        RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(limit));
                    scene.detached_subtrees_retention_bound_v1(&[command.root_id], &mut m)?;
                    plan.add_retained(m.bytes().get())?;
                    plan.add_peak(m.bytes().get())?;
                } else if let Some(batch) = &command.batch {
                    let mut m =
                        RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(limit));
                    batch.retained_bound_v1(&mut m)?;
                    batch.restore_retention_peak_v1(&mut m)?;
                    plan.add_peak(m.bytes().get())?;
                }
            } else if let Self::CreateNode(command) = self {
                if command.record.is_none() {
                    let kind = match command.intent {
                        CreateNodeIntent::Node { kind } => kind,
                        CreateNodeIntent::Mesh { .. } => NodeKind::Mesh,
                    };
                    let quote = scene.create_history_quote_v1(kind, model, limit)?;
                    plan.add_retained(quote.record_bytes)?;
                    // Default spawn, replacement mesh/name, callback materialization,
                    // and final snapshot clone may overlap. Four complete payload
                    // bounds cover these actual entry-point lifetimes.
                    plan.add_peak(
                        quote
                            .record_bytes
                            .checked_mul(3)
                            .ok_or(RetentionContractError::Overflow)?,
                    )?;
                    plan.create_quote_v1 = Some(quote);
                }
            }
            // Derived clone operations retain no more child allocation than the
            // admitted providers: reserve both the live owner and its clone.
            plan.add_peak(plan.retained_bytes)?;
            Ok(())
        })??;
        Ok(plan)
    }
}

impl EditCommand for EditorCommand {
    fn retention_plan_v1(
        &self,
        context: &dyn EditContext,
        phase: super::engine::RetentionPhaseV1,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<super::engine::CommandRetentionPlanV1, EditCommandError> {
        self.plan_v1(context, phase, model, limit, limit)
    }

    fn retention_plan_with_limits_v1(
        &self,
        context: &dyn EditContext,
        phase: super::engine::RetentionPhaseV1,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limits: super::engine::CommandRetentionLimitsV1,
    ) -> Result<super::engine::CommandRetentionPlanV1, EditCommandError> {
        self.plan_v1(
            context,
            phase,
            model,
            limits.retained_bytes,
            limits.materialization_bytes,
        )
    }

    fn prepare_retention_v1(
        &mut self,
        context: &mut dyn EditContext,
        phase: super::engine::RetentionPhaseV1,
        permit: &mut super::engine::RetentionMaterializationV1,
    ) -> Result<(), CommandExecutionError> {
        permit.consume(self, phase).map_err(unchanged)?;
        let core = core_context(context)?;
        let generation = core
            .with_scene(|scene| scene.world_generation())
            .map_err(unchanged)?;
        if permit.plan().owner_revision != Some(generation) {
            return Err(unchanged(
                zircon_runtime_interface::retained_memory::RetentionContractError::Stale.into(),
            ));
        }
        if let Self::DeleteNode(command) = self {
            command.prepared_generation_v1 = Some(generation);
        }
        if let Self::CreateNode(command) = self {
            if let Some(quote) = &permit.plan().create_quote_v1 {
                core.with_scene(|scene| scene.validate_create_history_quote_v1(quote))
                    .map_err(unchanged)?
                    .map_err(|error| unchanged(error.into()))?;
            }
            command.prepared_history_v1 = permit.plan().create_quote_v1.clone();
        }
        Ok(())
    }

    fn label(&self) -> &str {
        match self {
            Self::CreateNode(_) => "Create scene node",
            Self::DeleteNode(_) => "Delete scene node",
            Self::UpdateNode(_) => "Update scene node",
            Self::BatchTransform(_) => "Transform scene selection",
            Self::PlayTransform(_) => "Transform Play scene node",
            Self::SetReflectedSceneField(_) => "Set reflected scene field",
            Self::SetScriptBindings(_) => "Set script bindings",
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
            Self::SetScriptBindings(command) => command.apply(context),
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
            Self::SetScriptBindings(command) => command.revert(context),
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
            Self::SetScriptBindings(command) => command.journal_payload(),
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
    let (result, post_callback_errors) = context
        .with_scene_mut(write)
        .map_err(unchanged)?
        .into_parts();
    let value = match result {
        Ok(value) => value,
        Err(error) => return Err(scene_write_failure(error, post_callback_errors)),
    };
    Ok(match post_callback_errors.into_iter().next() {
        Some(error) => SceneWriteCompletion::AppliedThenGatewayFailed { value, error },
        None => SceneWriteCompletion::Completed(value),
    })
}

#[derive(Clone, Debug)]
pub(crate) struct CreateNodeCommand {
    intent: CreateNodeIntent,
    record: Option<NodeRecord>,
    prepared_history_v1:
        Option<zircon_runtime::core::framework::retained_memory::RetainedCreateQuoteV1>,
}

impl CreateNodeCommand {
    fn new(intent: CreateNodeIntent) -> Self {
        Self {
            intent,
            record: None,
            prepared_history_v1: None,
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
            prepared_history_v1: None,
        })
    }

    fn apply(&mut self, context: &mut CoreEditContext) -> Result<(), CommandExecutionError> {
        if let Some(retained) = self.record.as_ref() {
            let node_id = retained.id;
            let record = retained.clone();
            match execute_scene_write(context, move |scene| {
                scene
                    .insert_node_record(record)
                    .map_err(|error| external_scene_error("restore created scene node", error))
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
        let prepared = self.prepared_history_v1.clone();
        let mut provider_breach = false;
        let completion = execute_scene_write(context, |scene| {
            use zircon_runtime::core::framework::retained_memory::verified_std_model_v1;
            let quote = match &prepared {
                Some(quote) => {
                    scene.validate_create_history_quote_v1(quote)?;
                    quote.clone()
                }
                None => {
                    // Standalone EditCommand callers have no history ledger. They
                    // still use the same pure producer and pre-clone proof; engine
                    // callers always transfer their already reserved preparation.
                    let kind = match intent {
                        CreateNodeIntent::Node { kind } => kind,
                        CreateNodeIntent::Mesh { .. } => NodeKind::Mesh,
                    };
                    scene.create_history_quote_v1(
                        kind,
                        verified_std_model_v1()?,
                        super::engine::HistoryRetentionBudgetV1::default().record_bytes,
                    )?
                }
            };
            let node_id = match intent {
                CreateNodeIntent::Node { kind } => scene
                    .spawn_node(kind)
                    .map_err(|error| scene_error("create node", error))?,
                CreateNodeIntent::Mesh { model, material } => scene
                    .spawn_mesh_node(model, material)
                    .map_err(|error| scene_error("import mesh node", error))?,
            };
            // Includes spawn_mesh_node's final display Name and renderer inserts.
            // Reject a violated provider before allocating the snapshot clone.
            scene
                .check_created_node_retention_v1(node_id, &quote)
                .map_err(|error| {
                    provider_breach = true;
                    EditCommandError::from(error)
                })?;
            match scene.node_record(node_id) {
                Some(record) => Ok(record),
                None => {
                    if let Err(error) = scene.remove_entity(node_id) {
                        if error.lifecycle_target_change().is_some() {
                            return Err(scene_error("clean up missing created scene node", error));
                        }
                    }
                    Err(EditCommandError::TargetMissing {
                        target: format!("created node {node_id}"),
                    })
                }
            }
        });
        let completion = match completion {
            Ok(value) => value,
            Err(error) if provider_breach => return Err(applied(error.source)),
            Err(error) => return Err(error),
        };
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
                .map_err(|error| external_scene_error("remove created scene node", error))
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
    prepared_generation_v1: Option<u64>,
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
            prepared_generation_v1: None,
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
            prepared_generation_v1: None,
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
        let mut retained = None;
        let completion = execute_scene_write(context, |scene| {
            if self
                .prepared_generation_v1
                .is_some_and(|generation| generation != scene.world_generation())
            {
                return Err(
                    zircon_runtime_interface::retained_memory::RetentionContractError::Stale.into(),
                );
            }
            let prepared = scene
                .prepare_entity_subtrees([root_id])
                .map_err(|error| scene_error("prepare scene subtree detach", error))?;
            if self
                .prepared_generation_v1
                .is_some_and(|generation| generation != prepared.world_generation())
            {
                return Err(
                    zircon_runtime_interface::retained_memory::RetentionContractError::Stale.into(),
                );
            }
            if prepared.affected_camera_count() >= prepared.world_camera_count() {
                return Err(EditCommandError::InvariantViolation {
                    invariant: "cannot delete the last remaining camera",
                });
            }
            let batch = match scene.remove_prepared_entity_subtrees(prepared) {
                Ok(batch) => batch,
                Err(error) => {
                    let (source, batch) = error.into_parts();
                    retained = batch;
                    return Err(scene_error("detach scene subtree", source));
                }
            };
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
        });
        self.batch = retained;
        let ((batch, surviving), post_callback_error) = match completion? {
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
                retained = batch;
                scene_error("restore detached entity batch", source)
            })
        });
        match restore {
            Ok(outcome) => {
                let (result, post_callback_errors) = outcome.into_parts();
                match result {
                    Ok(()) => {
                        self.batch = retained;
                        match post_callback_errors.into_iter().next() {
                            Some(error) => Err(applied(error)),
                            None => Ok(()),
                        }
                    }
                    Err(error) => {
                        self.batch = retained;
                        Err(scene_write_failure(error, post_callback_errors))
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
        let name = scene
            .get::<zircon_runtime::scene::components::Name>(node_id)
            .filter(|_| scene.contains_scene_node(node_id))
            .ok_or_else(|| EditCommandError::TargetMissing {
                target: format!("scene node {node_id}"),
            })?;
        // Capture only the three retained fields; find_node also clones unrelated component payloads.
        Ok(Self {
            name: name.0.clone(),
            parent: scene.parent_of(node_id),
            transform: scene.local_transform(node_id).unwrap_or_default(),
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
            .map_err(|error| external_scene_error("update scene node parent", error))?;
    }
    if before.name != after.name {
        scene
            .rename_node(node_id, after.name.clone())
            .map_err(|error| external_scene_error("rename scene node", error))?;
    }
    if before.transform != after.transform {
        scene
            .update_transform(node_id, after.transform)
            .map_err(|error| external_scene_error("update scene node transform", error))?;
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
        .map_err(reflect_source_error)?;
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
        .map_err(reflect_source_error)?;
    scene
        .reflect_read(ReflectReadRequest::new(address, field_id))
        .map(|response| response.field.value)
        .map_err(reflect_source_error)
}

fn write_reflected_component_field(
    scene: &mut Scene,
    node_id: NodeId,
    component_type_path: &str,
    field_id: ReflectFieldId,
    value: ReflectedValue,
) -> Result<bool, EditCommandError> {
    let address = ReflectObjectAddress::component(node_id, component_type_path)
        .map_err(reflect_source_error)?;
    scene
        .reflect_write(ReflectWriteRequest::new(address, field_id, value))
        .map(|response| response.changed)
        .map_err(reflect_source_error)
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

fn lifecycle_interrupted(source: &EditCommandError) -> bool {
    if source.is_interrupted() {
        return true;
    }
    match source {
        EditCommandError::SceneMutation { source, .. } => {
            source.lifecycle_target_change().is_some()
        }
        EditCommandError::ReflectError { source } => source
            .downcast_ref::<ReflectError>()
            .is_some_and(|error| error.lifecycle_target_change().is_some()),
        EditCommandError::RollbackFailed {
            command_error,
            rollback_error,
        } => lifecycle_interrupted(command_error) || lifecycle_interrupted(rollback_error),
        _ => false,
    }
}

fn scene_write_failure(
    source: EditCommandError,
    post_callback_errors: Vec<EditCommandError>,
) -> CommandExecutionError {
    if lifecycle_interrupted(&source) {
        let mut interrupted = CommandExecutionError::interrupted(source);
        for post_callback_error in post_callback_errors {
            interrupted = CommandExecutionError::interrupted_with_post_callback_error(
                interrupted.into_source(),
                Some(post_callback_error),
            );
        }
        interrupted
    } else {
        unchanged(source)
    }
}

fn applied(source: EditCommandError) -> CommandExecutionError {
    CommandExecutionError::applied(source)
}

fn external_scene_error(operation: &'static str, source: SceneError) -> EditCommandError {
    if source.lifecycle_target_change().is_some() {
        scene_error(operation, source)
    } else {
        external_error(source.to_string())
    }
}

fn scene_error(operation: &'static str, source: SceneError) -> EditCommandError {
    EditCommandError::SceneMutation { operation, source }
}

fn external_error(message: String) -> EditCommandError {
    EditCommandError::ExternalEffect {
        source: Box::new(io::Error::other(message)),
    }
}

fn reflect_source_error(source: ReflectError) -> EditCommandError {
    EditCommandError::ReflectError {
        source: Box::new(source),
    }
}

fn reflect_error(message: String) -> EditCommandError {
    EditCommandError::ReflectError {
        source: Box::new(io::Error::other(message)),
    }
}

#[cfg(test)]
#[path = "tests/command_performance_source_guards.rs"]
mod performance_source_guards;

zircon_runtime_interface::impl_retained_owner_v1!(NodeEditState { name });
zircon_runtime_interface::impl_retained_owner_v1!(CreateNodeCommand {
    record,
    prepared_history_v1
});
zircon_runtime_interface::impl_retained_owner_v1!(UpdateNodeCommand { before, after });
zircon_runtime_interface::impl_retained_owner_v1!(DeleteNodeCommand { batch });
zircon_runtime_interface::impl_retained_owner_v1!(SetReflectedSceneFieldCommand {
    component_type_path,
    before,
    after
});
impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for EditorCommand {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        use zircon_runtime_interface::retained_memory::RetainedOwnerV1;
        match self {
            Self::CreateNode(v) => v.retained_bound_v1(m),
            Self::DeleteNode(v) => v.retained_bound_v1(m),
            Self::UpdateNode(v) => v.retained_bound_v1(m),
            Self::BatchTransform(v) => v.retained_bound_v1(m),
            Self::PlayTransform(v) => v.retained_bound_v1(m),
            Self::SetReflectedSceneField(v) => v.retained_bound_v1(m),
            Self::SetScriptBindings(v) => v.retained_bound_v1(m),
        }
    }
}
