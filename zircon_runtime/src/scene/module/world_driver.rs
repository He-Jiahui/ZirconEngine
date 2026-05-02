use std::collections::BTreeMap;

use crate::animation::apply_sequence_to_world;
use crate::asset::{AssetId, ProjectAssetManager};
use crate::core::framework::animation::{
    AnimationGraphClipInstance, AnimationManager, AnimationParameterMap, AnimationPoseOutput,
    AnimationPoseSource,
};
use crate::core::framework::physics::PhysicsWorldStepPlan;
use crate::core::manager::{resolve_animation_manager, resolve_physics_manager};
use crate::core::math::Real;
use crate::core::{CoreError, CoreHandle};
use crate::physics::{build_world_sync_state, integrate_builtin_physics_steps};
use crate::scene::level_system::AnimationStateTransitionRuntime;
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
    delta_seconds: Real,
    transition: Option<AnimationStateTransitionRuntime>,
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
                let (
                    previous_graph_times,
                    previous_state_machine_times,
                    previous_state_machine_transitions,
                ) = level.animation_playback_times();
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
                                                delta_seconds,
                                                transition: previous_state_machine_transitions
                                                    .get(&entity)
                                                    .cloned(),
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
                    let (state_machine_poses, active_state_updates, transition_updates) =
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
                    level.record_animation_playback_times(
                        next_graph_times,
                        next_state_machine_times,
                        transition_updates,
                    );
                } else {
                    level.record_animation_poses(BTreeMap::new());
                    level.record_animation_playback_times(
                        next_graph_times,
                        next_state_machine_times,
                        BTreeMap::new(),
                    );
                }
            } else {
                level.record_animation_poses(BTreeMap::new());
                level.record_animation_playback_times(
                    BTreeMap::new(),
                    BTreeMap::new(),
                    BTreeMap::new(),
                );
            }
        }

        Ok(())
    }
}

fn tick_physics_world(core: &CoreHandle, level: &LevelSystem, delta_seconds: Real) {
    let Ok(physics) = resolve_physics_manager(core) else {
        level.record_physics_step(PhysicsWorldStepPlan::default(), Vec::new());
        return;
    };

    let plan = physics.plan_world_step(level.world_handle(), delta_seconds);
    let sync = level.with_world_mut(|world| {
        integrate_builtin_physics_steps(world, plan);
        build_world_sync_state(level.world_handle(), world)
    });
    physics.sync_world(sync);
    level.record_physics_step(plan, physics.drain_contacts(level.world_handle()));
}

fn apply_loaded_sequences(
    level: &LevelSystem,
    loaded_sequences: &[(crate::asset::AnimationSequenceAsset, Real, bool)],
) {
    level.with_world_mut(|world| {
        for (sequence, time_seconds, looping) in loaded_sequences {
            let _ = apply_sequence_to_world(world, sequence, *time_seconds, *looping);
        }
    });
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
            sample_graph_evaluation_pose(
                animation,
                asset_manager,
                pending.entity,
                pending.skeleton_id,
                pending.time_seconds,
                AnimationPoseSource::Graph,
                None,
                &evaluation,
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
    BTreeMap<EntityId, AnimationStateTransitionRuntime>,
) {
    let mut poses = BTreeMap::new();
    let mut active_state_updates = Vec::new();
    let mut transition_updates = BTreeMap::new();

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
        let transition = resolve_state_machine_transition_runtime(
            pending.transition.clone(),
            evaluation.transition.as_ref(),
            pending.time_seconds,
            pending.delta_seconds,
        );
        let state_update = transition
            .as_ref()
            .map(|transition| {
                if transition.elapsed_seconds >= transition.duration_seconds {
                    transition.to_state.clone()
                } else {
                    transition.from_state.clone()
                }
            })
            .or_else(|| evaluation.active_state.clone());
        active_state_updates.push((pending.entity, state_update.clone()));

        if let Some(active_transition) = transition.as_ref() {
            let Some((entity, pose)) = sample_state_transition_pose(
                animation,
                asset_manager,
                &state_machine,
                &evaluation.parameters,
                &pending,
                active_transition,
            ) else {
                continue;
            };
            poses.insert(entity, pose);
            if active_transition.elapsed_seconds < active_transition.duration_seconds {
                transition_updates.insert(entity, active_transition.clone());
            }
            continue;
        }

        let Some((entity, pose)) = sample_state_graph_pose(
            animation,
            asset_manager,
            &state_machine,
            evaluation.graph.as_ref(),
            &evaluation.parameters,
            pending.entity,
            pending.skeleton_id,
            pending.time_seconds,
            state_update,
        ) else {
            continue;
        };
        poses.insert(entity, pose);
    }

    (poses, active_state_updates, transition_updates)
}

fn resolve_graph_clip_time_seconds(base_time_seconds: Real, playback_speed: Real) -> Real {
    (base_time_seconds * playback_speed).max(0.0)
}

fn sample_graph_evaluation_pose(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    entity: EntityId,
    skeleton_id: AssetId,
    base_time_seconds: Real,
    source: AnimationPoseSource,
    active_state: Option<String>,
    evaluation: &crate::core::framework::animation::AnimationGraphEvaluation,
) -> Option<(EntityId, AnimationPoseOutput)> {
    let total_weight = evaluation
        .clips
        .iter()
        .filter_map(finite_positive_graph_clip_weight)
        .sum::<Real>();
    if total_weight <= Real::EPSILON {
        return None;
    }

    let mut weighted_poses = Vec::new();
    for clip in &evaluation.clips {
        let Some(weight) = finite_positive_graph_clip_weight(clip) else {
            continue;
        };
        let clip_id = asset_manager.resolve_asset_id(&clip.clip.locator)?;
        let (_, pose) = sample_pose_request(
            animation,
            asset_manager,
            PendingPoseSample {
                entity,
                skeleton_id,
                clip_id,
                time_seconds: resolve_graph_clip_time_seconds(
                    base_time_seconds,
                    clip.playback_speed,
                ),
                looping: clip.looping,
                source,
                active_state: active_state.clone(),
            },
        )?;
        weighted_poses.push((pose, weight / total_weight));
    }

    blend_weighted_poses(weighted_poses, source, active_state).map(|pose| (entity, pose))
}

fn finite_positive_graph_clip_weight(clip: &AnimationGraphClipInstance) -> Option<Real> {
    (clip.weight.is_finite() && clip.weight > 0.0).then_some(clip.weight)
}

fn blend_weighted_poses(
    weighted_poses: Vec<(AnimationPoseOutput, Real)>,
    source: AnimationPoseSource,
    active_state: Option<String>,
) -> Option<AnimationPoseOutput> {
    let (first_pose, first_weight) = weighted_poses.first()?.clone();
    let mut bones = first_pose.bones;
    for bone in &mut bones {
        bone.local_transform.translation *= first_weight;
        bone.local_transform.scale *= first_weight;
        bone.local_transform.rotation *= first_weight;
    }

    for (pose, weight) in weighted_poses.into_iter().skip(1) {
        for bone in &mut bones {
            let Some(other) = pose.bones.iter().find(|other| other.name == bone.name) else {
                continue;
            };
            bone.local_transform.translation += other.local_transform.translation * weight;
            bone.local_transform.scale += other.local_transform.scale * weight;
            let mut rotation = other.local_transform.rotation;
            if bone.local_transform.rotation.dot(rotation) < 0.0 {
                rotation = -rotation;
            }
            bone.local_transform.rotation += rotation * weight;
        }
    }

    for bone in &mut bones {
        bone.local_transform.rotation = bone.local_transform.rotation.normalize();
    }

    Some(AnimationPoseOutput {
        source,
        active_state,
        bones,
    })
}

fn resolve_state_machine_transition_runtime(
    previous: Option<AnimationStateTransitionRuntime>,
    requested: Option<&crate::core::framework::animation::AnimationStateTransitionEvaluation>,
    time_seconds: Real,
    delta_seconds: Real,
) -> Option<AnimationStateTransitionRuntime> {
    let delta_seconds = if delta_seconds.is_finite() {
        delta_seconds.max(0.0)
    } else {
        0.0
    };
    if let Some(mut previous) = previous {
        previous.elapsed_seconds = (previous.elapsed_seconds + delta_seconds)
            .min(previous.duration_seconds)
            .max(0.0);
        previous.from_time_seconds = (previous.from_time_seconds + delta_seconds).max(0.0);
        previous.to_time_seconds = (previous.to_time_seconds + delta_seconds).max(0.0);
        return Some(previous);
    }

    requested.map(|requested| AnimationStateTransitionRuntime {
        from_state: requested.from_state.clone(),
        to_state: requested.to_state.clone(),
        duration_seconds: requested.duration_seconds,
        elapsed_seconds: delta_seconds.min(requested.duration_seconds).max(0.0),
        from_time_seconds: time_seconds.max(0.0),
        to_time_seconds: delta_seconds,
    })
}

fn sample_state_transition_pose(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    state_machine: &crate::asset::AnimationStateMachineAsset,
    parameters: &AnimationParameterMap,
    pending: &PendingStateMachinePoseSample,
    transition: &AnimationStateTransitionRuntime,
) -> Option<(EntityId, AnimationPoseOutput)> {
    let from_graph = state_machine_graph_reference(state_machine, &transition.from_state)?;
    let to_graph = state_machine_graph_reference(state_machine, &transition.to_state)?;
    let (_, from_pose) = sample_state_graph_pose(
        animation,
        asset_manager,
        state_machine,
        Some(from_graph),
        parameters,
        pending.entity,
        pending.skeleton_id,
        transition.from_time_seconds,
        Some(transition.from_state.clone()),
    )?;
    let (_, to_pose) = sample_state_graph_pose(
        animation,
        asset_manager,
        state_machine,
        Some(to_graph),
        parameters,
        pending.entity,
        pending.skeleton_id,
        transition.to_time_seconds,
        Some(transition.to_state.clone()),
    )?;
    let progress = (transition.elapsed_seconds / transition.duration_seconds).clamp(0.0, 1.0);
    blend_weighted_poses(
        vec![(from_pose, 1.0 - progress), (to_pose, progress)],
        AnimationPoseSource::StateMachine,
        Some(if progress >= 1.0 {
            transition.to_state.clone()
        } else {
            transition.from_state.clone()
        }),
    )
    .map(|pose| (pending.entity, pose))
}

fn sample_state_graph_pose(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    _state_machine: &crate::asset::AnimationStateMachineAsset,
    graph_reference: Option<&crate::core::resource::AssetReference>,
    parameters: &AnimationParameterMap,
    entity: EntityId,
    skeleton_id: AssetId,
    time_seconds: Real,
    active_state: Option<String>,
) -> Option<(EntityId, AnimationPoseOutput)> {
    let graph_reference = graph_reference?;
    let graph_id = asset_manager.resolve_asset_id(&graph_reference.locator)?;
    let graph = asset_manager.load_animation_graph_asset(graph_id).ok()?;
    let graph_evaluation = animation.evaluate_graph(&graph, parameters);
    sample_graph_evaluation_pose(
        animation,
        asset_manager,
        entity,
        skeleton_id,
        time_seconds,
        AnimationPoseSource::StateMachine,
        active_state,
        &graph_evaluation,
    )
}

fn state_machine_graph_reference<'a>(
    state_machine: &'a crate::asset::AnimationStateMachineAsset,
    state_name: &str,
) -> Option<&'a crate::core::resource::AssetReference> {
    state_machine
        .states
        .iter()
        .find(|state| state.name == state_name)
        .map(|state| &state.graph)
}
