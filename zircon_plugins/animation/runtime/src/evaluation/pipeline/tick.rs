use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use zircon_runtime::asset::project_asset_manager_handle;
use zircon_runtime::core::framework::animation::{
    AnimationParameterValue, AnimationPoseMap, AnimationPoseOutput, AnimationPoseSnapshot,
};
use zircon_runtime::core::framework::physics::{
    SimulatedPoseFeed, SkeletalPoseTarget, SkeletalPoseTargets,
};
use zircon_runtime::core::manager::{animation_manager_handle, resolve_manager_service};
use zircon_runtime::core::math::Real;
use zircon_runtime::core::CoreHandle;
use zircon_runtime::scene::components::AnimationStateMachinePlayerComponent;
use zircon_runtime::scene::{EntityId, LevelSystem};

use super::animation_evaluation_pipeline::PresentationPoseChange;
use super::diagnostics::AnimationSceneFrameDiagnostics;
use super::direct_clip_worker::sample_direct_clip_pose_requests;
use super::events::{enqueue_clip_event_samples, publish_clip_events, publish_events};
use super::graph_evaluate::resolve_graph_pose_requests;
use super::parameter_apply::{
    apply_clip_player_updates, apply_sequence_player_updates, scan_animation_scene,
};
use super::pose_apply::apply_pose_transforms_to_scene_nodes;
use super::sequences::{apply_loaded_sequences, LoadedSequenceSample};
use super::simulated_pose_blend::blend_simulated_pose_feed;
use super::state_machine_layers::apply_state_machine_layers;
use super::state_machine_step::resolve_state_machine_pose_requests;
use super::AnimationEvaluationPipeline;
pub(crate) fn tick_animation_world(core: &CoreHandle, level: &LevelSystem, delta_seconds: Real) {
    let Ok(animation) =
        animation_manager_handle(core).and_then(|handle| resolve_manager_service(core, handle))
    else {
        record_empty_animation_state(core, level);
        return;
    };

    let playback_settings = animation.playback_settings();
    if !playback_settings.enabled {
        record_empty_animation_state(core, level);
        return;
    }

    let asset_manager = project_asset_manager_handle(core)
        .and_then(|handle| resolve_manager_service(core, handle))
        .ok();
    let replacement_epoch = level.capture_world_replacement_epoch();
    let Some((
        previous_graph_times,
        previous_state_machine_times,
        previous_state_machine_transitions,
    )) = level.animation_playback_times(replacement_epoch)
    else {
        return;
    };
    let Some((mut projection, clip_event_admission_cursor)) = level
        .with_world_mut_if_replacement_epoch(replacement_epoch, |world| {
            if !world.contains_resource::<AnimationEvaluationPipeline>() {
                world.insert_resource(AnimationEvaluationPipeline::default());
            }
            let pipeline = world.resource_mut::<AnimationEvaluationPipeline>();
            let _ = pipeline.begin_evaluation_frame(replacement_epoch);
            (
                pipeline.take_projection(),
                pipeline.clip_event_admission_cursor(),
            )
        })
    else {
        return;
    };
    let Some(transaction) = scan_animation_scene(
        level,
        replacement_epoch,
        &mut projection,
        &playback_settings,
        asset_manager.as_deref(),
        delta_seconds,
    ) else {
        return;
    };
    let mut scan = transaction.scan;
    let mut frame_diagnostics = AnimationSceneFrameDiagnostics::from_scan(&scan);
    let mut clip_player_updates = transaction.clip_player_updates;
    let mut sequence_player_updates = transaction.sequence_player_updates;
    let revision_stage = transaction.revision_stage;
    let Some(asset_manager) = &asset_manager else {
        let Some(admission) = enqueue_clip_event_samples(
            level,
            replacement_epoch,
            clip_event_admission_cursor,
            scan.clip_event_samples,
        ) else {
            return;
        };
        retain_non_deferred_entity_updates(&mut clip_player_updates, &admission.deferred_entities);
        restore_deferred_entity_map(
            &mut scan.next_graph_times,
            previous_graph_times.as_ref(),
            &admission.deferred_entities,
        );
        restore_deferred_entity_map(
            &mut scan.next_state_machine_times,
            previous_state_machine_times.as_ref(),
            &admission.deferred_entities,
        );
        let Some((pose_snapshot, pose_change)) =
            level.with_world_mut_if_replacement_epoch(replacement_epoch, |world| {
                let pipeline = world.resource_mut::<AnimationEvaluationPipeline>();
                projection.commit_revision_stage(revision_stage, &admission.deferred_entities);
                pipeline.restore_projection(projection);
                pipeline.set_clip_event_admission_cursor(admission.next_cursor);
                let update =
                    pipeline.update_presentation_poses(&scan.pose_source_entities, BTreeMap::new());
                match update {
                    Some(update) => (update.snapshot, Some(update.change)),
                    None => (pipeline.presentation_poses(), None),
                }
            })
        else {
            return;
        };
        if !apply_clip_player_updates(level, replacement_epoch, clip_player_updates) {
            return;
        }
        retain_non_deferred_entity_updates(
            &mut sequence_player_updates,
            &admission.deferred_entities,
        );
        if !apply_sequence_player_updates(level, replacement_epoch, sequence_player_updates) {
            return;
        }
        let Some(published_events) =
            publish_events(level, replacement_epoch, admission.diagnostics)
        else {
            return;
        };
        if let Some(change) = pose_change {
            if !publish_skeletal_pose_targets(level, replacement_epoch, &pose_snapshot, &change) {
                return;
            }
        }
        if !level.record_animation_pose_snapshot(replacement_epoch, pose_snapshot) {
            return;
        }
        if !level.record_animation_playback_times(
            replacement_epoch,
            scan.next_graph_times,
            scan.next_state_machine_times,
            previous_state_machine_transitions.as_ref().clone(),
        ) {
            return;
        }
        frame_diagnostics.published_events = published_events;
        frame_diagnostics.record(core);
        return;
    };

    let mut pending_clip_event_samples = scan.clip_event_samples;
    let loaded_sequences = scan
        .sequences
        .into_iter()
        .filter_map(|pending| {
            asset_manager
                .load_animation_sequence_asset(pending.sequence_id)
                .ok()
                .map(|sequence| LoadedSequenceSample {
                    entity: pending.entity,
                    asset_id: pending.sequence_id,
                    asset_revision: pending.asset_revision,
                    sequence,
                    time_seconds: pending.time_seconds,
                    looping: pending.looping,
                })
        })
        .collect::<Vec<_>>();
    let mut loaded_sequences = loaded_sequences;
    let Some(had_queued_clip_events) = level
        .animation_clip_event_backlog_len(replacement_epoch)
        .map(|pending| pending > 0)
    else {
        return;
    };
    let mut published_events: usize = 0;
    if had_queued_clip_events {
        let Some(event_count) = publish_clip_events(asset_manager, level, replacement_epoch) else {
            return;
        };
        published_events = published_events.saturating_add(event_count);
    }
    let state_machine_sample_entities = scan
        .state_machine_samples
        .iter()
        .map(|pending| pending.entity)
        .collect::<BTreeSet<_>>();
    let Some((
        mut animation_poses,
        graph_poses,
        graph_event_samples,
        state_machine_poses,
        state_machine_event_samples,
        mut layer_diagnostics,
        mut active_state_updates,
        mut transition_updates,
        state_machine_journal,
    )) = level.with_world_mut_if_replacement_epoch(replacement_epoch, |world| {
        let pipeline = world.resource_mut::<AnimationEvaluationPipeline>();
        pipeline.begin_state_machine_runtime_transaction(&state_machine_sample_entities);
        pipeline.restore_projection(projection);
        pipeline
            .clip_evaluator_mut()
            .bind_resources(&asset_manager.resource_manager());
        let animation_poses = sample_direct_clip_pose_requests(
            core,
            pipeline,
            Arc::clone(asset_manager),
            scan.clip_samples,
        );
        let (graph_poses, graph_event_samples) =
            resolve_graph_pose_requests(pipeline, asset_manager, scan.graph_samples);
        let layer_samples = scan.state_machine_samples.clone();
        let (
            mut state_machine_poses,
            mut state_machine_event_samples,
            active_state_updates,
            transition_updates,
        ) = resolve_state_machine_pose_requests(
            pipeline,
            asset_manager,
            scan.state_machine_samples,
        );
        let layer_result = apply_state_machine_layers(
            pipeline,
            asset_manager,
            &layer_samples,
            &mut state_machine_poses,
        );
        state_machine_event_samples.extend(layer_result.events);
        let state_machine_journal = pipeline.finish_state_machine_runtime_transaction();
        (
            animation_poses,
            graph_poses,
            graph_event_samples,
            state_machine_poses,
            state_machine_event_samples,
            layer_result.diagnostics,
            active_state_updates,
            transition_updates,
            state_machine_journal,
        )
    })
    else {
        return;
    };
    animation_poses.extend(graph_poses);
    animation_poses.extend(state_machine_poses);
    pending_clip_event_samples.extend(graph_event_samples);
    pending_clip_event_samples.extend(state_machine_event_samples);
    let Some(admission) = enqueue_clip_event_samples(
        level,
        replacement_epoch,
        clip_event_admission_cursor,
        pending_clip_event_samples,
    ) else {
        return;
    };
    if level
        .with_world_mut_if_replacement_epoch(replacement_epoch, |world| {
            let pipeline = world.resource_mut::<AnimationEvaluationPipeline>();
            pipeline.commit_projection_revision_stage(revision_stage, &admission.deferred_entities);
            pipeline.finish_clip_event_admission(
                state_machine_journal,
                &admission.deferred_entities,
                admission.next_cursor,
            );
        })
        .is_none()
    {
        return;
    }
    retain_non_deferred_entity_map(&mut animation_poses, &admission.deferred_entities);
    restore_deferred_entity_map(
        &mut scan.next_graph_times,
        previous_graph_times.as_ref(),
        &admission.deferred_entities,
    );
    restore_deferred_entity_map(
        &mut scan.next_state_machine_times,
        previous_state_machine_times.as_ref(),
        &admission.deferred_entities,
    );
    restore_deferred_entity_map(
        &mut transition_updates,
        previous_state_machine_transitions.as_ref(),
        &admission.deferred_entities,
    );
    retain_non_deferred_entity_updates(&mut clip_player_updates, &admission.deferred_entities);
    retain_non_deferred_entity_updates(&mut sequence_player_updates, &admission.deferred_entities);
    loaded_sequences.retain(|sample| !admission.deferred_entities.contains(&sample.entity));
    active_state_updates.retain(|update| !admission.deferred_entities.contains(&update.entity));
    layer_diagnostics
        .retain(|diagnostic| !admission.deferred_entities.contains(&diagnostic.entity));
    let evaluation_diagnostics = level
        .with_world_mut_if_replacement_epoch(replacement_epoch, |world| {
            world
                .resource_mut::<AnimationEvaluationPipeline>()
                .drain_clip_evaluation_diagnostics_excluding(&admission.deferred_entities)
        })
        .unwrap_or_default();
    if !loaded_sequences.is_empty()
        && !apply_loaded_sequences(level, replacement_epoch, &loaded_sequences)
    {
        return;
    }
    if !apply_sequence_player_updates(level, replacement_epoch, sequence_player_updates) {
        return;
    }
    let Some(event_count) = publish_events(level, replacement_epoch, admission.diagnostics) else {
        return;
    };
    published_events = published_events.saturating_add(event_count);
    if !had_queued_clip_events {
        let Some(event_count) = publish_clip_events(asset_manager, level, replacement_epoch) else {
            return;
        };
        published_events = published_events.saturating_add(event_count);
    }
    if !apply_clip_player_updates(level, replacement_epoch, clip_player_updates) {
        return;
    }
    let Some(event_count) = publish_events(level, replacement_epoch, layer_diagnostics) else {
        return;
    };
    published_events = published_events.saturating_add(event_count);
    let Some(event_count) = publish_events(level, replacement_epoch, evaluation_diagnostics) else {
        return;
    };
    published_events = published_events.saturating_add(event_count);
    if level
        .with_world_mut_if_replacement_epoch(replacement_epoch, |world| {
            let pipeline = world.resource::<AnimationEvaluationPipeline>();
            if world.contains_resource::<SimulatedPoseFeed>() {
                blend_simulated_pose_feed(
                    pipeline,
                    world.resource::<SimulatedPoseFeed>(),
                    &scan.skeletons_by_entity,
                    &mut animation_poses,
                );
            }
        })
        .is_none()
    {
        return;
    }

    if !active_state_updates.is_empty() {
        if level
            .with_world_mut_if_replacement_epoch(replacement_epoch, |world| {
                for update in active_state_updates {
                    let Some(mut player) =
                        world.animation_state_machine_player(update.entity).cloned()
                    else {
                        continue;
                    };
                    apply_active_state_update(
                        &mut player,
                        update.active_state,
                        update.consumed_triggers.as_deref().unwrap_or_default(),
                    );
                    let _ = world.set_animation_state_machine_player(update.entity, Some(player));
                }
            })
            .is_none()
        {
            return;
        }
    }

    frame_diagnostics.output_poses = animation_poses.len();
    let Some(applied_transforms) =
        apply_pose_transforms_to_scene_nodes(level, replacement_epoch, &animation_poses)
    else {
        return;
    };
    frame_diagnostics.applied_transforms = applied_transforms;
    let Some((pose_snapshot, pose_change)) =
        level.with_world_mut_if_replacement_epoch(replacement_epoch, |world| {
            let pipeline = world.resource_mut::<AnimationEvaluationPipeline>();
            let update =
                pipeline.update_presentation_poses(&scan.pose_source_entities, animation_poses);
            match update {
                Some(update) => (update.snapshot, Some(update.change)),
                None => (pipeline.presentation_poses(), None),
            }
        })
    else {
        return;
    };
    if let Some(change) = pose_change {
        if !publish_skeletal_pose_targets(level, replacement_epoch, &pose_snapshot, &change) {
            return;
        }
    }
    if !level.record_animation_pose_snapshot(replacement_epoch, pose_snapshot) {
        return;
    }
    frame_diagnostics.published_events = published_events;
    frame_diagnostics.state_transitions = transition_updates.len();
    if !level.record_animation_playback_times(
        replacement_epoch,
        scan.next_graph_times,
        scan.next_state_machine_times,
        transition_updates,
    ) {
        return;
    }
    frame_diagnostics.record(core);
}

fn retain_non_deferred_entity_map<T>(
    values: &mut BTreeMap<EntityId, T>,
    deferred_entities: &BTreeSet<EntityId>,
) {
    values.retain(|entity, _| !deferred_entities.contains(entity));
}

fn apply_active_state_update(
    player: &mut AnimationStateMachinePlayerComponent,
    active_state: Option<String>,
    consumed_triggers: &[String],
) {
    player.active_state = active_state;
    for trigger in consumed_triggers {
        let is_trigger = player
            .parameters
            .get(trigger)
            .is_some_and(|value| matches!(value, AnimationParameterValue::Trigger));
        if is_trigger {
            player.parameters.remove(trigger);
        }
    }
}

fn restore_deferred_entity_map<T: Clone>(
    next: &mut BTreeMap<EntityId, T>,
    previous: &BTreeMap<EntityId, T>,
    deferred_entities: &BTreeSet<EntityId>,
) {
    for entity in deferred_entities {
        if let Some(value) = previous.get(entity) {
            next.insert(*entity, value.clone());
        } else {
            next.remove(entity);
        }
    }
}

fn retain_non_deferred_entity_updates<T>(
    values: &mut Vec<(EntityId, T)>,
    deferred_entities: &BTreeSet<EntityId>,
) {
    values.retain(|(entity, _)| !deferred_entities.contains(entity));
}

fn publish_skeletal_pose_targets(
    level: &LevelSystem,
    replacement_epoch: u64,
    animation_poses: &AnimationPoseMap,
    change: &PresentationPoseChange,
) -> bool {
    level
        .with_world_mut_if_replacement_epoch(replacement_epoch, |world| {
            if !world.contains_resource::<SkeletalPoseTargets>() {
                return;
            }
            let targets = world.resource_mut::<SkeletalPoseTargets>();
            match change {
                PresentationPoseChange::Full => {
                    targets.clear();
                    for (entity, pose) in animation_poses {
                        targets.replace(*entity, skeletal_pose_targets(pose));
                    }
                }
                PresentationPoseChange::Partial { changed_entities } => {
                    for entity in changed_entities {
                        if let Some(pose) = animation_poses.get(entity) {
                            targets.replace(*entity, skeletal_pose_targets(pose));
                        } else {
                            targets.remove(*entity);
                        }
                    }
                }
            }
        })
        .is_some()
}

fn skeletal_pose_targets(pose: &AnimationPoseOutput) -> Arc<[SkeletalPoseTarget]> {
    Arc::from(
        pose.bones
            .iter()
            .map(|bone| SkeletalPoseTarget {
                bone_name: bone.name.clone(),
                local_transform: bone.local_transform,
                normalized_weight: 1.0,
            })
            .collect::<Vec<_>>(),
    )
}

fn record_empty_animation_state(core: &CoreHandle, level: &LevelSystem) {
    let replacement_epoch = level.capture_world_replacement_epoch();
    let empty_pose_snapshot = AnimationPoseSnapshot::default();
    if level
        .with_world_mut_if_replacement_epoch(replacement_epoch, |world| {
            if let Some(pipeline) = world.get_resource_mut::<AnimationEvaluationPipeline>() {
                pipeline.ensure_empty_evaluation_state(replacement_epoch);
            }
        })
        .is_none()
        || !publish_skeletal_pose_targets(
            level,
            replacement_epoch,
            empty_pose_snapshot.as_ref(),
            &PresentationPoseChange::Full,
        )
        || !level.record_animation_pose_snapshot(replacement_epoch, empty_pose_snapshot)
    {
        return;
    }
    if !level.record_animation_playback_times(
        replacement_epoch,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    ) {
        return;
    }
    AnimationSceneFrameDiagnostics::default().record(core);
}

#[cfg(test)]
#[path = "tests/tick.rs"]
mod tests;
