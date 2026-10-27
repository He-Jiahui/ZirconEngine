//! Selected-entity copies use the same records and native extras as scene persistence.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use zircon_runtime::scene::components::{
    AnimationGraphPlayerComponent, AnimationStateMachinePlayerComponent, ColliderComponent,
    JointComponent, MeshRenderer, Name, NodeRecord, PostProcessSettingsComponent,
    PostProcessVolumeComponent,
};
use zircon_runtime::scene::{NodeId, Scene};
use zircon_runtime_interface::retained_memory::{
    verified_std_model_v1, RetainedAccountingModelV1, RetainedBoundBuilderV1, RetainedBytesV1,
    RetainedOwnerV1, RetentionContractError,
};

use super::{
    applied, execute_scene_write, journal_payload, scene_error, unchanged, CommandEffect,
    CommandExecutionError, CommandJournalPayload, CommandJournalUnavailable, CoreEditContext,
    EditCommandError, SceneSelection, SceneWriteCompletion,
};
use crate::core::editing::engine::{
    CommandRetentionPlanV1, HistoryRetentionBudgetV1, RetentionPhaseV1,
};

pub(crate) struct DuplicateNodesCommand {
    records: Vec<NodeRecord>,
    extras: Vec<NodeExtras>,
    primary: Option<NodeId>,
    // Prefix compensation can restore only the rows an interrupted undo removed.
    live: Vec<bool>,
    captured_generation_v1: Option<u64>,
    pub(super) prepared_generation_v1: Option<u64>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NodeExtras {
    dynamic: Vec<(String, Value)>,
    post_process_settings: Option<PostProcessSettingsComponent>,
    post_process_volume: Option<PostProcessVolumeComponent>,
}

impl DuplicateNodesCommand {
    pub(super) fn capture(
        scene: &Scene,
        mut node_ids: Vec<NodeId>,
        primary: Option<NodeId>,
    ) -> Result<Self, EditCommandError> {
        node_ids.sort_unstable();
        node_ids.dedup();
        if node_ids.is_empty() {
            return Err(invalid("duplicate requires selected scene nodes"));
        }
        for &id in &node_ids {
            if !scene.contains_scene_node(id) {
                return Err(EditCommandError::TargetMissing {
                    target: format!("scene node {id}"),
                });
            }
        }
        // Prove all recursive payload allocations from borrowed components before cloning.
        capture_bound_v1(scene, &node_ids)?;
        // Parent-first records let undo remove children first without recursive deletion.
        node_ids.sort_by_key(|&id| parent_depth(scene, id));
        let mut records = Vec::with_capacity(node_ids.len());
        let mut extras = Vec::with_capacity(node_ids.len());
        for &id in &node_ids {
            records.push(
                scene
                    .node_record(id)
                    .ok_or_else(|| EditCommandError::TargetMissing {
                        target: format!("scene node {id}"),
                    })?,
            );
            let mut dynamic =
                Vec::with_capacity(scene.dynamic_component_values_for_entity(id).count());
            dynamic.extend(
                scene
                    .dynamic_component_values_for_entity(id)
                    .map(|(path, value)| (path.to_owned(), value.clone())),
            );
            dynamic.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            extras.push(NodeExtras {
                dynamic,
                post_process_settings: scene.get::<PostProcessSettingsComponent>(id).cloned(),
                post_process_volume: scene.get::<PostProcessVolumeComponent>(id).cloned(),
            });
        }
        let remap = scene
            .prepare_node_record_copies(&mut records)
            .map_err(|error| scene_error("prepare duplicate scene nodes", error))?;
        let primary = primary
            .and_then(|id| remap.get(id))
            .or_else(|| records.first().map(|record| record.id));
        Ok(Self {
            live: vec![false; records.len()],
            records,
            extras,
            primary,
            captured_generation_v1: Some(scene.world_generation()),
            prepared_generation_v1: None,
        })
    }

    pub(super) fn from_journal(
        payload: DuplicateNodesJournalPayload,
    ) -> Result<Self, EditCommandError> {
        if payload.records.is_empty() || payload.records.len() != payload.extras.len() {
            return Err(invalid(
                "duplicate journal requires aligned node records and components",
            ));
        }
        for (index, record) in payload.records.iter().enumerate() {
            if record.parent == Some(record.id)
                || payload.records[..index]
                    .iter()
                    .any(|other| other.id == record.id)
                || payload.records[index + 1..]
                    .iter()
                    .any(|other| Some(other.id) == record.parent)
            {
                return Err(invalid(
                    "duplicate journal requires unique parent-first node records",
                ));
            }
        }
        if payload
            .primary
            .is_some_and(|id| !payload.records.iter().any(|record| record.id == id))
        {
            return Err(invalid(
                "duplicate primary must belong to the copied selection",
            ));
        }
        for extra in &payload.extras {
            for (index, (path, _)) in extra.dynamic.iter().enumerate() {
                if path.is_empty()
                    || extra.dynamic[..index]
                        .iter()
                        .any(|(other, _)| other == path)
                {
                    return Err(invalid(
                        "duplicate journal requires unique dynamic component paths",
                    ));
                }
            }
        }
        Ok(Self {
            live: vec![false; payload.records.len()],
            records: payload.records,
            extras: payload.extras,
            primary: payload.primary,
            captured_generation_v1: None,
            prepared_generation_v1: None,
        })
    }

    pub(super) fn plan_v1(
        &self,
        scene: &Scene,
        phase: RetentionPhaseV1,
        model: RetainedAccountingModelV1,
        peak_limit: u64,
        plan: &mut CommandRetentionPlanV1,
    ) -> Result<(), EditCommandError> {
        self.validate_generation(scene)?;
        let undo = matches!(phase, RetentionPhaseV1::Undo | RetentionPhaseV1::Rollback);
        if undo {
            for (record, &live) in self.records.iter().zip(&self.live) {
                if live && !scene.contains_scene_node(record.id) {
                    return Err(EditCommandError::TargetMissing {
                        target: format!("duplicated scene node {}", record.id),
                    });
                }
            }
        }
        // Record clones, component value clones and inverse publication can overlap.
        plan.add_peak(
            plan.retained_bytes
                .checked_mul(3)
                .ok_or(RetentionContractError::Overflow)?,
        )?;
        let mut scratch =
            RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(peak_limit));
        scratch.array::<NodeId>(self.records.len())?;
        scratch.array::<usize>(self.records.len())?;
        for extra in &self.extras {
            // Bound temporary archetype/signature growth for the authored native set
            // and every copied dynamic presence row. Persistent World tables are not history.
            let rows = extra
                .dynamic
                .len()
                .checked_add(32)
                .and_then(|n| n.checked_mul(16))
                .ok_or(RetentionContractError::Overflow)?;
            scratch.array::<zircon_runtime::scene::ecs::ComponentId>(rows)?;
        }
        scene.node_record_insertion_peak_v1(&self.records, &mut scratch)?;
        plan.add_peak(scratch.bytes().get())?;
        Ok(())
    }

    fn validate_generation(&self, scene: &Scene) -> Result<(), EditCommandError> {
        if self
            .captured_generation_v1
            .is_some_and(|generation| generation != scene.world_generation())
        {
            return Err(RetentionContractError::Stale.into());
        }
        Ok(())
    }

    fn validate_prepared_generation(&self, scene: &Scene) -> Result<(), EditCommandError> {
        self.validate_generation(scene)?;
        if self
            .prepared_generation_v1
            .is_some_and(|generation| generation != scene.world_generation())
        {
            return Err(RetentionContractError::Stale.into());
        }
        Ok(())
    }

    pub(super) fn apply(
        &mut self,
        context: &mut CoreEditContext,
    ) -> Result<(), CommandExecutionError> {
        let mut missing = Vec::with_capacity(self.records.len());
        missing.extend(
            self.live
                .iter()
                .enumerate()
                .filter_map(|(index, &live)| (!live).then_some(index)),
        );
        if missing.is_empty() {
            return Err(unchanged(invalid("duplicate command is already applied")));
        }
        let mut wrote = false;
        let completion = execute_scene_write(context, |scene| {
            self.validate_prepared_generation(scene)?;
            for &index in &missing {
                if let Some(parent) = self.records[index].parent {
                    if !scene.contains_scene_node(parent)
                        && !self.records.iter().any(|record| record.id == parent)
                    {
                        return Err(EditCommandError::TargetMissing {
                            target: format!("duplicate parent {parent}"),
                        });
                    }
                }
            }
            let records = missing
                .iter()
                .map(|&index| self.records[index].clone())
                .collect();
            scene
                .insert_owned_node_records(records)
                .map_err(|error| scene_error("insert duplicate scene nodes", error))?;
            wrote = true;
            self.captured_generation_v1 = None;
            for &index in &missing {
                self.live[index] = true;
            }
            for &index in &missing {
                self.extras[index].insert(scene, self.records[index].id)?;
            }
            Ok(())
        })
        .map_err(|mut error| {
            if wrote && error.effect == CommandEffect::Unchanged {
                error.effect = CommandEffect::Applied;
            }
            error
        })?;
        if let SceneWriteCompletion::AppliedThenGatewayFailed { error, .. } = completion {
            return Err(applied(error));
        }
        let ids = self.records.iter().map(|record| record.id).collect();
        context
            .set_scene_selection(SceneSelection::new(ids, self.primary))
            .map_err(applied)
    }

    pub(super) fn revert(
        &mut self,
        context: &CoreEditContext,
    ) -> Result<(), CommandExecutionError> {
        let mut wrote = false;
        let completion = execute_scene_write(context, |scene| {
            self.validate_prepared_generation(scene)?;
            // Preflight all owned IDs before the first removal. Unselected descendants
            // are never included in the inverse; World handles any remaining children.
            for (record, &live) in self.records.iter().zip(&self.live) {
                if live && !scene.contains_scene_node(record.id) {
                    return Err(EditCommandError::TargetMissing {
                        target: format!("duplicated scene node {}", record.id),
                    });
                }
            }
            for index in (0..self.records.len()).rev() {
                if self.live[index] {
                    scene
                        .remove_entity(self.records[index].id)
                        .map_err(|error| scene_error("remove duplicate scene node", error))?;
                    self.live[index] = false;
                    wrote = true;
                }
            }
            Ok(())
        })
        .map_err(|mut error| {
            if wrote && error.effect == CommandEffect::Unchanged {
                error.effect = CommandEffect::Applied;
            }
            error
        })?;
        match completion {
            SceneWriteCompletion::Completed(()) => Ok(()),
            SceneWriteCompletion::AppliedThenGatewayFailed { error, .. } => Err(applied(error)),
        }
    }

    pub(super) fn journal_payload(
        &self,
    ) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        journal_payload(
            "zircon.editor.scene.duplicate_nodes",
            &DuplicateNodesJournalPayload {
                records: self.records.clone(),
                extras: self.extras.clone(),
                primary: self.primary,
            },
        )
    }
}

impl NodeExtras {
    fn insert(&self, scene: &mut Scene, id: NodeId) -> Result<(), EditCommandError> {
        for (path, value) in &self.dynamic {
            scene
                .set_dynamic_component(id, path, value.clone())
                .map_err(|error| scene_error("copy dynamic scene component", error))?;
        }
        if let Some(settings) = &self.post_process_settings {
            scene
                .insert(id, settings.clone())
                .map_err(|error| scene_error("copy native post-process settings", error))?;
        }
        if let Some(volume) = &self.post_process_volume {
            scene
                .insert(id, volume.clone())
                .map_err(|error| scene_error("copy native post-process volume", error))?;
        }
        Ok(())
    }
}

fn capture_bound_v1(scene: &Scene, ids: &[NodeId]) -> Result<(), EditCommandError> {
    let mut meter = RetainedBoundBuilderV1::with_limit(
        verified_std_model_v1()?,
        RetainedBytesV1::new(HistoryRetentionBudgetV1::default().record_bytes),
    );
    meter.layout(
        std::mem::size_of::<DuplicateNodesCommand>(),
        std::mem::align_of::<DuplicateNodesCommand>(),
    )?;
    meter.array::<NodeRecord>(ids.len())?;
    meter.array::<NodeExtras>(ids.len())?;
    meter.array::<bool>(ids.len())?;
    meter.array::<NodeId>(ids.len())?;
    meter.btree_construction::<NodeId, NodeId>(ids.len())?;
    meter.add(
        zircon_runtime_interface::retained_memory::FreshHashStorageV1::for_request::<NodeId>(
            ids.len(),
        )?
        .allocation::<NodeId>()?,
    )?;
    for &id in ids {
        // These are precisely NodeRecord's allocation-bearing native fields.
        macro_rules! component {
            ($($ty:ty),* $(,)?) => {$(if let Some(value) = scene.get::<$ty>(id) {
                value.retained_bound_v1(&mut meter)?;
            })*};
        }
        component!(
            Name,
            MeshRenderer,
            ColliderComponent,
            JointComponent,
            AnimationGraphPlayerComponent,
            AnimationStateMachinePlayerComponent
        );
        meter.array::<(String, Value)>(scene.dynamic_component_values_for_entity(id).count())?;
        for (path, value) in scene.dynamic_component_values_for_entity(id) {
            meter.array::<u8>(path.len())?;
            value.retained_bound_v1(&mut meter)?;
        }
    }
    Ok(())
}

fn parent_depth(scene: &Scene, id: NodeId) -> usize {
    let mut cursor = scene.parent_of(id);
    let mut depth = 0;
    while let Some(parent) = cursor {
        depth += 1;
        cursor = scene.parent_of(parent);
    }
    depth
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DuplicateNodesJournalPayload {
    records: Vec<NodeRecord>,
    extras: Vec<NodeExtras>,
    primary: Option<NodeId>,
}

fn invalid(invariant: &'static str) -> EditCommandError {
    EditCommandError::InvariantViolation { invariant }
}

zircon_runtime_interface::impl_retained_owner_v1!(NodeExtras { dynamic });
zircon_runtime_interface::impl_retained_owner_v1!(DuplicateNodesCommand {
    records,
    extras,
    live
});

#[cfg(test)]
#[path = "duplicate/tests.rs"]
mod tests;
