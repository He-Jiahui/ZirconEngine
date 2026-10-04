use std::convert::Infallible;

use super::*;
use crate::graphics::scene::render_scene::resource_dependencies::{
    build_resource_reference_deltas, RenderSceneResourceReferenceDelta,
    RenderSceneResourceReferenceDeltaBuild, RenderSceneResourceReferenceUpdate,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderSceneDeltaTransactionError<StageError> {
    Preflight(RenderSceneApplyError),
    Staging(StageError),
}

struct PlannedUpdate {
    handle: RenderScenePrimitiveHandle,
    dirty: RenderScenePrimitiveDirtyFlags,
    primitive: RenderScenePrimitive,
}

struct RenderSceneDeltaApplyPlan {
    from_generation: RenderSceneGeneration,
    to_generation: RenderSceneGeneration,
    removals: Vec<(u64, RenderScenePrimitiveHandle)>,
    updates: Vec<PlannedUpdate>,
    additions: Vec<RenderScenePrimitive>,
    addition_handles: Vec<RenderScenePrimitiveHandle>,
    input_upsert_count: usize,
    input_removal_count: usize,
    primitive_comparison_count: usize,
    dirty_domain_counts: RenderSceneDirtyDomainCounts,
    reused_handle_slot_count: usize,
    appended_handle_slot_count: usize,
    resource_reference_build: RenderSceneResourceReferenceDeltaBuild,
}

pub(super) fn apply(
    scene: &mut RenderScene,
    delta: RenderSceneDelta,
) -> Result<RenderSceneChangeJournal, RenderSceneApplyError> {
    match apply_with_staging(scene, delta, |_| Ok::<(), Infallible>(())) {
        Ok((journal, ())) => Ok(journal),
        Err(RenderSceneDeltaTransactionError::Preflight(error)) => Err(error),
        Err(RenderSceneDeltaTransactionError::Staging(never)) => match never {},
    }
}

pub(super) fn apply_with_staging<StageOutput, StageError>(
    scene: &mut RenderScene,
    delta: RenderSceneDelta,
    stage: impl FnOnce(&[RenderSceneResourceReferenceDelta]) -> Result<StageOutput, StageError>,
) -> Result<(RenderSceneChangeJournal, StageOutput), RenderSceneDeltaTransactionError<StageError>> {
    let plan = RenderSceneDeltaApplyPlan::preflight(scene, delta)
        .map_err(RenderSceneDeltaTransactionError::Preflight)?;
    let staged = stage(&plan.resource_reference_build.deltas)
        .map_err(RenderSceneDeltaTransactionError::Staging)?;
    Ok((plan.commit(scene), staged))
}

impl RenderSceneDeltaApplyPlan {
    fn preflight(
        scene: &RenderScene,
        mut delta: RenderSceneDelta,
    ) -> Result<Self, RenderSceneApplyError> {
        delta
            .upserts
            .sort_by_key(RenderScenePrimitive::stable_instance_key);
        delta.removals.sort_unstable();
        validate_delta_keys(&delta)?;
        let input_upsert_count = delta.upserts.len();
        let input_removal_count = delta.removals.len();

        let removals = delta
            .removals
            .iter()
            .filter_map(|stable_instance_key| {
                scene
                    .stable_key_to_handle
                    .get(stable_instance_key)
                    .copied()
                    .map(|handle| (*stable_instance_key, handle))
            })
            .collect::<Vec<_>>();
        for (stable_instance_key, handle) in &removals {
            if scene.primitive_for_handle(*handle).is_none() {
                return Err(RenderSceneApplyError::InvalidLiveHandle {
                    stable_instance_key: *stable_instance_key,
                });
            }
        }

        let mut updates = Vec::new();
        let mut additions = Vec::new();
        let mut primitive_comparison_count = 0;
        let mut dirty_domain_counts = RenderSceneDirtyDomainCounts::default();
        for primitive in delta.upserts {
            let stable_instance_key = primitive.stable_instance_key();
            let Some(handle) = scene
                .stable_key_to_handle
                .get(&stable_instance_key)
                .copied()
            else {
                additions.push(primitive);
                continue;
            };
            let previous = scene.primitive_for_handle(handle).ok_or(
                RenderSceneApplyError::InvalidLiveHandle {
                    stable_instance_key,
                },
            )?;
            let previous_node_id = previous.descriptor().node_id;
            let incoming_node_id = primitive.descriptor().node_id;
            if previous_node_id != incoming_node_id {
                return Err(RenderSceneApplyError::StableKeyOwnerChanged {
                    stable_instance_key,
                    previous_node_id,
                    incoming_node_id,
                });
            }
            primitive_comparison_count += 1;
            let dirty = primitive.dirty_from(previous);
            if !dirty.is_empty() {
                dirty_domain_counts.record(dirty);
                updates.push(PlannedUpdate {
                    handle,
                    dirty,
                    primitive,
                });
            }
        }

        let has_mutations = !(removals.is_empty() && updates.is_empty() && additions.is_empty());
        let from_generation = scene.generation;
        let to_generation = if has_mutations {
            scene
                .generation
                .next()
                .ok_or(RenderSceneApplyError::GenerationExhausted)?
        } else {
            scene.generation
        };
        let addition_handles = scene.plan_addition_handles(&removals, additions.len())?;
        let reused_handle_slot_count = addition_handles
            .iter()
            .filter(|handle| (handle.slot as usize) < scene.handle_slots.len())
            .count();
        let appended_handle_slot_count = addition_handles
            .len()
            .saturating_sub(reused_handle_slot_count);
        let resource_reference_build = build_resource_reference_deltas(
            removals.iter().map(|(_, handle)| {
                scene
                    .primitive_for_handle(*handle)
                    .expect("render-scene removal was preflighted")
            }),
            updates.iter().map(|update| {
                RenderSceneResourceReferenceUpdate::new(
                    update.dirty,
                    scene
                        .primitive_for_handle(update.handle)
                        .expect("render-scene update was preflighted"),
                    &update.primitive,
                )
            }),
            additions.iter(),
        );

        Ok(Self {
            from_generation,
            to_generation,
            removals,
            updates,
            additions,
            addition_handles,
            input_upsert_count,
            input_removal_count,
            primitive_comparison_count,
            dirty_domain_counts,
            reused_handle_slot_count,
            appended_handle_slot_count,
            resource_reference_build,
        })
    }

    fn commit(self, scene: &mut RenderScene) -> RenderSceneChangeJournal {
        debug_assert_eq!(scene.generation, self.from_generation);
        let removed = self
            .removals
            .into_iter()
            .map(|(_, handle)| {
                scene
                    .remove_primitive(handle)
                    .expect("preflighted render-scene removal must commit")
            })
            .collect::<Vec<_>>();
        let updated = self
            .updates
            .into_iter()
            .map(|update| {
                scene
                    .install_update(update.handle, update.dirty, update.primitive)
                    .expect("preflighted render-scene update must commit")
            })
            .collect::<Vec<_>>();
        let added = self
            .additions
            .into_iter()
            .zip(self.addition_handles)
            .map(|(primitive, handle)| scene.install_addition(handle, primitive))
            .collect::<Vec<_>>();
        let dense_relocation_count = removed
            .iter()
            .filter(|removal| removal.relocation().is_some())
            .count();
        scene.generation = self.to_generation;

        RenderSceneChangeJournal::new_with_resource_reference_build(
            scene.world,
            self.from_generation,
            self.to_generation,
            removed,
            updated,
            added,
            self.resource_reference_build,
            RenderSceneApplyStats::new(
                self.input_upsert_count,
                self.input_removal_count,
                self.input_upsert_count
                    .saturating_add(self.input_removal_count),
                self.primitive_comparison_count,
                self.dirty_domain_counts,
                self.reused_handle_slot_count,
                self.appended_handle_slot_count,
                dense_relocation_count,
            )
            .with_storage_stats(scene.storage_stats()),
        )
    }
}
