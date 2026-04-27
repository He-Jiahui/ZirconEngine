use std::collections::BTreeMap;

use crate::asset::{AssetId, ProjectAssetManager};
use crate::core::framework::animation::{
    AnimationGraphClipInstance, AnimationManager, AnimationParameterMap, AnimationPoseOutput,
    AnimationPoseSource,
};
use crate::core::manager::resolve_animation_manager;
use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle};
use crate::scene::{EntityId, LevelSystem};

#[derive(Clone, Debug)]
struct PendingSequenceSample {
    sequence_id: AssetId,
    time_seconds: Real,
    looping: bool,
}

#[derive(Clone, Debug)]
struct PendingPoseSample {
    entity: EntityId,
    skeleton_id: AssetId,
    clip_id: AssetId,
    time_seconds: Real,
    looping: bool,
    source: AnimationPoseSource,
    active_state: Option<String>,
}

#[derive(Clone, Debug)]
struct PendingGraphPoseSample {
    entity: EntityId,
    skeleton_id: AssetId,
    graph_id: AssetId,
    parameters: AnimationParameterMap,
    time_seconds: Real,
}

#[derive(Clone, Debug)]
struct PendingStateMachinePoseSample {
    entity: EntityId,
    skeleton_id: AssetId,
    state_machine_id: AssetId,
    parameters: AnimationParameterMap,
    active_state: Option<String>,
    time_seconds: Real,
}

#[derive(Debug, Default)]
pub struct WorldDriver;

impl WorldDriver {
    pub fn tick_level(
        &self,
        core: &CoreHandle,
        level: &LevelSystem,
        delta_seconds: Real,
    ) -> Result<(), CoreError> {
        tick_physics_world(core, level, delta_seconds);

        if let Ok(animation) = resolve_animation_manager(core) {
            let playback_settings = animation.playback_settings();
            if playback_settings.enabled {
                let asset_manager = core
                    .resolve_manager::<ProjectAssetManager>(
                        crate::asset::PROJECT_ASSET_MANAGER_NAME,
                    )
                    .ok();
                let (previous_graph_times, previous_state_machine_times) =
                    level.animation_playback_times();
                let (
                    pending_sequences,
                    pending_clip_samples,
                    pending_graph_samples,
                    pending_state_machine_samples,
                    next_graph_times,
                    next_state_machine_times,
                ) = level.with_world_mut(|world| {
                    let entity_ids = world.nodes().iter().map(|node| node.id).collect::<Vec<_>>();
                    let mut pending_sequences = Vec::<PendingSequenceSample>::new();
                    let mut pending_clip_samples = Vec::<PendingPoseSample>::new();
                    let mut pending_graph_samples = Vec::<PendingGraphPoseSample>::new();
                    let mut pending_state_machine_samples =
                        Vec::<PendingStateMachinePoseSample>::new();
                    let mut next_graph_times = BTreeMap::<EntityId, Real>::new();
                    let mut next_state_machine_times = BTreeMap::<EntityId, Real>::new();
                    for entity in entity_ids {
                        if playback_settings.skeletal_clips {
                            if let Some(mut player) = world.animation_player(entity).cloned() {
                                if player.playing {
                                    player.time_seconds = (player.time_seconds
                                        + delta_seconds * player.playback_speed)
                                        .max(0.0);
                                }
                                let clip_id = player.clip.id();
                                let time_seconds = player.time_seconds;
                                let looping = player.looping;
                                let _ = world.set_animation_player(entity, Some(player));
                                if let Some(skeleton) = world.animation_skeleton(entity).cloned() {
                                    pending_clip_samples.push(PendingPoseSample {
                                        entity,
                                        skeleton_id: skeleton.skeleton.id(),
                                        clip_id,
                                        time_seconds,
                                        looping,
                                        source: AnimationPoseSource::Clip,
                                        active_state: None,
                                    });
                                }
                            }
                        }

                        if playback_settings.property_tracks {
                            if let Some(mut player) =
                                world.animation_sequence_player(entity).cloned()
                            {
                                if player.playing {
                                    player.time_seconds = (player.time_seconds
                                        + delta_seconds * player.playback_speed)
                                        .max(0.0);
                                }
                                let sequence_id = player.sequence.id();
                                let time_seconds = player.time_seconds;
                                let looping = player.looping;
                                let _ = world.set_animation_sequence_player(entity, Some(player));
                                pending_sequences.push(PendingSequenceSample {
                                    sequence_id,
                                    time_seconds,
                                    looping,
                                });
                            }
                        }

                        if playback_settings.graphs {
                            if let Some(player) = world.animation_graph_player(entity).cloned() {
                                let next_time_seconds =
                                    previous_graph_times.get(&entity).copied().unwrap_or(0.0)
                                        + if player.playing { delta_seconds } else { 0.0 };
                                next_graph_times.insert(entity, next_time_seconds);
                                if player.playing {
                                    if let Some(skeleton) =
                                        world.animation_skeleton(entity).cloned()
                                    {
                                        pending_graph_samples.push(PendingGraphPoseSample {
                                            entity,
                                            skeleton_id: skeleton.skeleton.id(),
                                            graph_id: player.graph.id(),
                                            parameters: player.parameters,
                                            time_seconds: next_time_seconds,
                                        });
                                    }
                                }
                            }
                        }

                        if playback_settings.state_machines {
                            if let Some(player) =
                                world.animation_state_machine_player(entity).cloned()
                            {
                                let next_time_seconds = previous_state_machine_times
                                    .get(&entity)
                                    .copied()
                                    .unwrap_or(0.0)
                                    + if player.playing { delta_seconds } else { 0.0 };
                                next_state_machine_times.insert(entity, next_time_seconds);
                                if player.playing {
                                    if let Some(skeleton) =
                                        world.animation_skeleton(entity).cloned()
                                    {
                                        pending_state_machine_samples.push(
                                            PendingStateMachinePoseSample {
                                                entity,
                                                skeleton_id: skeleton.skeleton.id(),
                                                state_machine_id: player.state_machine.id(),
                                                parameters: player.parameters,
                                                active_state: player.active_state,
                                                time_seconds: next_time_seconds,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }

                    (
                        pending_sequences,
                        pending_clip_samples,
                        pending_graph_samples,
                        pending_state_machine_samples,
                        next_graph_times,
                        next_state_machine_times,
                    )
                });
                level.record_animation_playback_times(next_graph_times, next_state_machine_times);

                if let Some(asset_manager) = &asset_manager {
                    let loaded_sequences = pending_sequences
                        .into_iter()
                        .filter_map(|pending| {
                            asset_manager
                                .load_animation_sequence_asset(pending.sequence_id)
                                .ok()
                                .map(|sequence| (sequence, pending.time_seconds, pending.looping))
                        })
                        .collect::<Vec<_>>();
                    if !loaded_sequences.is_empty() {
                        apply_loaded_sequences(level, &loaded_sequences);
                    }

                    let mut animation_poses = sample_pose_requests(
                        animation.as_ref(),
                        asset_manager,
                        pending_clip_samples,
                    );
                    animation_poses.extend(resolve_graph_pose_requests(
                        animation.as_ref(),
                        asset_manager,
                        pending_graph_samples,
                    ));
                    let (state_machine_poses, active_state_updates) =
                        resolve_state_machine_pose_requests(
                            animation.as_ref(),
                            asset_manager,
                            pending_state_machine_samples,
                        );
                    animation_poses.extend(state_machine_poses);
                    if !active_state_updates.is_empty() {
                        level.with_world_mut(|world| {
                            for (entity, active_state) in active_state_updates {
                                let Some(mut player) =
                                    world.animation_state_machine_player(entity).cloned()
                                else {
                                    continue;
                                };
                                player.active_state = active_state;
                                let _ =
                                    world.set_animation_state_machine_player(entity, Some(player));
                            }
                        });
                    }
                    level.record_animation_poses(animation_poses);
                } else {
                    level.record_animation_poses(BTreeMap::new());
                }
            } else {
                level.record_animation_poses(BTreeMap::new());
            }
        }

        Ok(())
    }
}

fn tick_physics_world(_core: &CoreHandle, _level: &LevelSystem, _delta_seconds: Real) {}

fn apply_loaded_sequences(
    _level: &LevelSystem,
    _loaded_sequences: &[(crate::asset::AnimationSequenceAsset, Real, bool)],
) {
}

fn sample_pose_requests(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingPoseSample>,
) -> BTreeMap<EntityId, AnimationPoseOutput> {
    pending_samples
        .into_iter()
        .filter_map(|pending| sample_pose_request(animation, asset_manager, pending))
        .collect()
}

fn sample_pose_request(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    pending: PendingPoseSample,
) -> Option<(EntityId, AnimationPoseOutput)> {
    let skeleton = asset_manager
        .load_animation_skeleton_asset(pending.skeleton_id)
        .ok()?;
    let clip = asset_manager
        .load_animation_clip_asset(pending.clip_id)
        .ok()?;
    let mut pose = animation
        .sample_clip_pose(&skeleton, &clip, pending.time_seconds, pending.looping)
        .ok()?;
    pose.source = pending.source;
    pose.active_state = pending.active_state;
    Some((pending.entity, pose))
}

fn resolve_graph_pose_requests(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingGraphPoseSample>,
) -> BTreeMap<EntityId, AnimationPoseOutput> {
    pending_samples
        .into_iter()
        .filter_map(|pending| {
            let graph = asset_manager
                .load_animation_graph_asset(pending.graph_id)
                .ok()?;
            let evaluation = animation.evaluate_graph(&graph, &pending.parameters);
            let clip = dominant_graph_clip(&evaluation)?;
            let clip_id = asset_manager.resolve_asset_id(&clip.clip.locator)?;
            sample_pose_request(
                animation,
                asset_manager,
                PendingPoseSample {
                    entity: pending.entity,
                    skeleton_id: pending.skeleton_id,
                    clip_id,
                    time_seconds: resolve_graph_clip_time_seconds(
                        pending.time_seconds,
                        clip.playback_speed,
                    ),
                    looping: clip.looping,
                    source: AnimationPoseSource::Graph,
                    active_state: None,
                },
            )
        })
        .collect()
}

fn resolve_state_machine_pose_requests(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingStateMachinePoseSample>,
) -> (
    BTreeMap<EntityId, AnimationPoseOutput>,
    Vec<(EntityId, Option<String>)>,
) {
    let mut poses = BTreeMap::new();
    let mut active_state_updates = Vec::new();

    for pending in pending_samples {
        let Some(state_machine) = asset_manager
            .load_animation_state_machine_asset(pending.state_machine_id)
            .ok()
        else {
            continue;
        };
        let evaluation = animation.evaluate_state_machine(
            &state_machine,
            pending.active_state.as_deref(),
            &pending.parameters,
        );
        active_state_updates.push((pending.entity, evaluation.active_state.clone()));

        let Some(graph_reference) = evaluation.graph else {
            continue;
        };
        let Some(graph_id) = asset_manager.resolve_asset_id(&graph_reference.locator) else {
            continue;
        };
        let Some(graph) = asset_manager.load_animation_graph_asset(graph_id).ok() else {
            continue;
        };
        let graph_evaluation = animation.evaluate_graph(&graph, &evaluation.parameters);
        let Some(clip) = dominant_graph_clip(&graph_evaluation) else {
            continue;
        };
        let Some(clip_id) = asset_manager.resolve_asset_id(&clip.clip.locator) else {
            continue;
        };
        let Some((entity, pose)) = sample_pose_request(
            animation,
            asset_manager,
            PendingPoseSample {
                entity: pending.entity,
                skeleton_id: pending.skeleton_id,
                clip_id,
                time_seconds: resolve_graph_clip_time_seconds(
                    pending.time_seconds,
                    clip.playback_speed,
                ),
                looping: clip.looping,
                source: AnimationPoseSource::StateMachine,
                active_state: evaluation.active_state.clone(),
            },
        ) else {
            continue;
        };
        poses.insert(entity, pose);
    }

    (poses, active_state_updates)
}

fn dominant_graph_clip<'a>(
    evaluation: &'a crate::core::framework::animation::AnimationGraphEvaluation,
) -> Option<&'a AnimationGraphClipInstance> {
    evaluation.clips.iter().max_by(|left, right| {
        left.weight
            .partial_cmp(&right.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn resolve_graph_clip_time_seconds(base_time_seconds: Real, playback_speed: Real) -> Real {
    (base_time_seconds * playback_speed).max(0.0)
}
