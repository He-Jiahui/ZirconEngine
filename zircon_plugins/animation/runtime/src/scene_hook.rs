use std::collections::BTreeMap;

use zircon_runtime::asset::{AssetId, ProjectAssetManager};
use zircon_runtime::core::framework::animation::{
    AnimationGraphBlendMode, AnimationGraphClipInstance, AnimationManager, AnimationParameterMap,
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::manager::resolve_animation_manager;
use zircon_runtime::core::math::{Quat, Real, Vec3};
use zircon_runtime::core::{CoreError, CoreHandle};
use zircon_runtime::plugin::{
    SceneRuntimeHook, SceneRuntimeHookContext, SceneRuntimeHookDescriptor,
    SceneRuntimeHookRegistration,
};
use zircon_runtime::scene::{AnimationStateTransitionRuntime, EntityId, LevelSystem, SystemStage};

use crate::clip_event::sample_clip_events;

#[derive(Clone, Debug, Default)]
pub struct AnimationSceneRuntimeHook;

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
struct PendingClipEventSample {
    entity: EntityId,
    clip_id: AssetId,
    from_time_seconds: Real,
    to_time_seconds: Real,
    looping: bool,
}

#[derive(Clone, Debug)]
struct PendingGraphPoseSample {
    entity: EntityId,
    skeleton_id: AssetId,
    graph_id: AssetId,
    parameters: AnimationParameterMap,
    from_time_seconds: Real,
    to_time_seconds: Real,
}

#[derive(Clone, Debug)]
struct PendingStateMachinePoseSample {
    entity: EntityId,
    skeleton_id: AssetId,
    state_machine_id: AssetId,
    parameters: AnimationParameterMap,
    active_state: Option<String>,
    from_time_seconds: Real,
    to_time_seconds: Real,
    delta_seconds: Real,
    transition: Option<AnimationStateTransitionRuntime>,
}

pub fn scene_hook_registration() -> SceneRuntimeHookRegistration {
    SceneRuntimeHookRegistration::new(
        SceneRuntimeHookDescriptor::new(
            "animation.scene.post_update",
            crate::PLUGIN_ID,
            SystemStage::PostUpdate,
        ),
        AnimationSceneRuntimeHook,
    )
}

impl SceneRuntimeHook for AnimationSceneRuntimeHook {
    fn run(&self, context: SceneRuntimeHookContext<'_>) -> Result<(), CoreError> {
        tick_animation_world(context.core, context.level, context.delta_seconds);
        Ok(())
    }
}

fn tick_animation_world(core: &CoreHandle, level: &LevelSystem, delta_seconds: Real) {
    let Ok(animation) = resolve_animation_manager(core) else {
        level.record_animation_poses(BTreeMap::new());
        level.record_animation_playback_times(BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
        return;
    };

    let playback_settings = animation.playback_settings();
    if !playback_settings.enabled {
        level.record_animation_poses(BTreeMap::new());
        level.record_animation_playback_times(BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
        return;
    }

    let asset_manager = core
        .resolve_manager::<ProjectAssetManager>(zircon_runtime::asset::PROJECT_ASSET_MANAGER_NAME)
        .ok();
    let (previous_graph_times, previous_state_machine_times, previous_state_machine_transitions) =
        level.animation_playback_times();
    let (
        pending_sequences,
        pending_clip_samples,
        pending_clip_event_samples,
        pending_graph_samples,
        pending_state_machine_samples,
        next_graph_times,
        next_state_machine_times,
    ) = level.with_world_mut(|world| {
        let entity_ids = world.nodes().iter().map(|node| node.id).collect::<Vec<_>>();
        let mut pending_sequences = Vec::<PendingSequenceSample>::new();
        let mut pending_clip_samples = Vec::<PendingPoseSample>::new();
        let mut pending_clip_event_samples = Vec::<PendingClipEventSample>::new();
        let mut pending_graph_samples = Vec::<PendingGraphPoseSample>::new();
        let mut pending_state_machine_samples = Vec::<PendingStateMachinePoseSample>::new();
        let mut next_graph_times = BTreeMap::<EntityId, Real>::new();
        let mut next_state_machine_times = BTreeMap::<EntityId, Real>::new();

        for entity in entity_ids {
            if playback_settings.skeletal_clips {
                if let Some(mut player) = world.animation_player(entity).cloned() {
                    let previous_time_seconds = player.time_seconds;
                    if player.playing {
                        player.time_seconds =
                            (player.time_seconds + delta_seconds * player.playback_speed).max(0.0);
                    }
                    let clip_id = player.clip.id();
                    let time_seconds = player.time_seconds;
                    let looping = player.looping;
                    if player.playing {
                        pending_clip_event_samples.push(PendingClipEventSample {
                            entity,
                            clip_id,
                            from_time_seconds: previous_time_seconds,
                            to_time_seconds: time_seconds,
                            looping,
                        });
                    }
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
                if let Some(mut player) = world.animation_sequence_player(entity).cloned() {
                    if player.playing {
                        player.time_seconds =
                            (player.time_seconds + delta_seconds * player.playback_speed).max(0.0);
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
                    let previous_time_seconds =
                        previous_graph_times.get(&entity).copied().unwrap_or(0.0);
                    let next_time_seconds =
                        previous_time_seconds + if player.playing { delta_seconds } else { 0.0 };
                    next_graph_times.insert(entity, next_time_seconds);
                    if player.playing {
                        if let Some(skeleton) = world.animation_skeleton(entity).cloned() {
                            pending_graph_samples.push(PendingGraphPoseSample {
                                entity,
                                skeleton_id: skeleton.skeleton.id(),
                                graph_id: player.graph.id(),
                                parameters: player.parameters,
                                from_time_seconds: previous_time_seconds,
                                to_time_seconds: next_time_seconds,
                            });
                        }
                    }
                }
            }

            if playback_settings.state_machines {
                if let Some(player) = world.animation_state_machine_player(entity).cloned() {
                    let previous_time_seconds = previous_state_machine_times
                        .get(&entity)
                        .copied()
                        .unwrap_or(0.0);
                    let next_time_seconds =
                        previous_time_seconds + if player.playing { delta_seconds } else { 0.0 };
                    next_state_machine_times.insert(entity, next_time_seconds);
                    if player.playing {
                        if let Some(skeleton) = world.animation_skeleton(entity).cloned() {
                            pending_state_machine_samples.push(PendingStateMachinePoseSample {
                                entity,
                                skeleton_id: skeleton.skeleton.id(),
                                state_machine_id: player.state_machine.id(),
                                parameters: player.parameters,
                                active_state: player.active_state,
                                from_time_seconds: previous_time_seconds,
                                to_time_seconds: next_time_seconds,
                                delta_seconds,
                                transition: previous_state_machine_transitions
                                    .get(&entity)
                                    .cloned(),
                            });
                        }
                    }
                }
            }
        }

        (
            pending_sequences,
            pending_clip_samples,
            pending_clip_event_samples,
            pending_graph_samples,
            pending_state_machine_samples,
            next_graph_times,
            next_state_machine_times,
        )
    });

    let Some(asset_manager) = &asset_manager else {
        level.record_animation_poses(BTreeMap::new());
        level.record_animation_playback_times(
            next_graph_times,
            next_state_machine_times,
            BTreeMap::new(),
        );
        return;
    };

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
        apply_loaded_sequences(animation.as_ref(), level, &loaded_sequences);
    }
    publish_clip_events(asset_manager, level, pending_clip_event_samples);

    let mut animation_poses =
        sample_pose_requests(animation.as_ref(), asset_manager, pending_clip_samples);
    let (graph_poses, graph_events) =
        resolve_graph_pose_requests(animation.as_ref(), asset_manager, pending_graph_samples);
    animation_poses.extend(graph_poses);
    publish_events(level, graph_events);
    let (state_machine_poses, state_machine_events, active_state_updates, transition_updates) =
        resolve_state_machine_pose_requests(
            animation.as_ref(),
            asset_manager,
            pending_state_machine_samples,
        );
    animation_poses.extend(state_machine_poses);
    publish_events(level, state_machine_events);

    if !active_state_updates.is_empty() {
        level.with_world_mut(|world| {
            for (entity, active_state) in active_state_updates {
                let Some(mut player) = world.animation_state_machine_player(entity).cloned() else {
                    continue;
                };
                player.active_state = active_state;
                let _ = world.set_animation_state_machine_player(entity, Some(player));
            }
        });
    }

    level.record_animation_poses(animation_poses);
    level.record_animation_playback_times(
        next_graph_times,
        next_state_machine_times,
        transition_updates,
    );
}

fn publish_clip_events(
    asset_manager: &ProjectAssetManager,
    level: &LevelSystem,
    pending_samples: Vec<PendingClipEventSample>,
) {
    let events = pending_samples
        .into_iter()
        .filter_map(|pending| {
            let clip = asset_manager
                .load_animation_clip_asset(pending.clip_id)
                .ok()?;
            Some(sample_clip_events(
                &clip,
                pending.entity,
                pending.from_time_seconds,
                pending.to_time_seconds,
                pending.looping,
            ))
        })
        .flatten()
        .collect::<Vec<_>>();
    if events.is_empty() {
        return;
    }
    level.with_world_mut(|world| publish_events_to_world(world, events));
}

fn publish_events(level: &LevelSystem, events: Vec<crate::AnimationClipEvent>) {
    if events.is_empty() {
        return;
    }
    level.with_world_mut(|world| publish_events_to_world(world, events));
}

fn publish_events_to_world(
    world: &mut zircon_runtime::scene::World,
    events: Vec<crate::AnimationClipEvent>,
) {
    for event in events {
        world.send_event(event);
    }
}

fn apply_loaded_sequences(
    animation: &dyn AnimationManager,
    level: &LevelSystem,
    loaded_sequences: &[(zircon_runtime::asset::AnimationSequenceAsset, Real, bool)],
) {
    level.with_world_mut(|world| {
        for (sequence, time_seconds, looping) in loaded_sequences {
            let _ = animation.apply_sequence_to_world(world, sequence, *time_seconds, *looping);
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
) -> (
    BTreeMap<EntityId, AnimationPoseOutput>,
    Vec<crate::AnimationClipEvent>,
) {
    let mut poses = BTreeMap::new();
    let mut events = Vec::new();
    for pending in pending_samples {
        let Some(graph) = asset_manager
            .load_animation_graph_asset(pending.graph_id)
            .ok()
        else {
            continue;
        };
        let evaluation = animation.evaluate_graph(&graph, &pending.parameters);
        events.extend(sample_graph_evaluation_clip_events(
            asset_manager,
            pending.entity,
            pending.from_time_seconds,
            pending.to_time_seconds,
            &evaluation,
        ));
        if let Some((entity, pose)) = sample_graph_evaluation_pose(
            animation,
            asset_manager,
            pending.entity,
            pending.skeleton_id,
            pending.to_time_seconds,
            AnimationPoseSource::Graph,
            None,
            &evaluation,
        ) {
            poses.insert(entity, pose);
        }
    }
    (poses, events)
}

fn sample_graph_evaluation_clip_events(
    asset_manager: &ProjectAssetManager,
    entity: EntityId,
    from_time_seconds: Real,
    to_time_seconds: Real,
    evaluation: &zircon_runtime::core::framework::animation::AnimationGraphEvaluation,
) -> Vec<crate::AnimationClipEvent> {
    evaluation
        .clips
        .iter()
        .filter_map(|clip| {
            let clip_id = asset_manager.resolve_asset_id(&clip.clip.locator)?;
            let clip_asset = asset_manager.load_animation_clip_asset(clip_id).ok()?;
            Some(sample_clip_events(
                &clip_asset,
                entity,
                resolve_graph_clip_time_seconds(from_time_seconds, clip.playback_speed),
                resolve_graph_clip_time_seconds(to_time_seconds, clip.playback_speed),
                clip.looping,
            ))
        })
        .flatten()
        .collect()
}

fn resolve_state_machine_pose_requests(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingStateMachinePoseSample>,
) -> (
    BTreeMap<EntityId, AnimationPoseOutput>,
    Vec<crate::AnimationClipEvent>,
    Vec<(EntityId, Option<String>)>,
    BTreeMap<EntityId, AnimationStateTransitionRuntime>,
) {
    let mut poses = BTreeMap::new();
    let mut events = Vec::new();
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
            pending.to_time_seconds,
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
            events.extend(sample_state_transition_clip_events(
                animation,
                asset_manager,
                &state_machine,
                &evaluation.parameters,
                &pending,
                active_transition,
            ));
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

        events.extend(sample_state_graph_clip_events(
            animation,
            asset_manager,
            evaluation.graph.as_ref(),
            &evaluation.parameters,
            pending.entity,
            pending.from_time_seconds,
            pending.to_time_seconds,
        ));
        let Some((entity, pose)) = sample_state_graph_pose(
            animation,
            asset_manager,
            &state_machine,
            evaluation.graph.as_ref(),
            &evaluation.parameters,
            pending.entity,
            pending.skeleton_id,
            pending.to_time_seconds,
            state_update,
        ) else {
            continue;
        };
        poses.insert(entity, pose);
    }

    (poses, events, active_state_updates, transition_updates)
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
    evaluation: &zircon_runtime::core::framework::animation::AnimationGraphEvaluation,
) -> Option<(EntityId, AnimationPoseOutput)> {
    let total_weight = evaluation
        .clips
        .iter()
        .filter(|clip| clip.blend_mode == AnimationGraphBlendMode::Base)
        .filter_map(finite_positive_graph_clip_weight)
        .sum::<Real>();
    if total_weight <= Real::EPSILON {
        return None;
    }

    let mut base_poses = Vec::new();
    let mut additive_poses = Vec::new();
    for clip in &evaluation.clips {
        let Some(weight) = finite_positive_graph_clip_weight(clip) else {
            continue;
        };
        let normalized_weight = match clip.blend_mode {
            AnimationGraphBlendMode::Base => weight / total_weight,
            AnimationGraphBlendMode::Additive => weight,
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
        match clip.blend_mode {
            AnimationGraphBlendMode::Base => base_poses.push(GraphWeightedPose {
                pose,
                weight: normalized_weight,
                target_ids: clip.target_ids.clone(),
            }),
            AnimationGraphBlendMode::Additive => additive_poses.push(GraphWeightedPose {
                pose,
                weight: normalized_weight,
                target_ids: clip.target_ids.clone(),
            }),
        }
    }

    let mut pose = blend_graph_base_poses(base_poses, source, active_state)?;
    apply_graph_additive_poses(&mut pose, additive_poses);
    Some((entity, pose))
}

#[derive(Clone, Debug)]
struct GraphWeightedPose {
    pose: AnimationPoseOutput,
    weight: Real,
    target_ids: Vec<String>,
}

fn finite_positive_graph_clip_weight(clip: &AnimationGraphClipInstance) -> Option<Real> {
    (clip.weight.is_finite() && clip.weight > 0.0).then_some(clip.weight)
}

fn blend_graph_base_poses(
    weighted_poses: Vec<GraphWeightedPose>,
    source: AnimationPoseSource,
    active_state: Option<String>,
) -> Option<AnimationPoseOutput> {
    let first = weighted_poses.first()?.clone();
    let first_weight = first.weight;
    let first_target_ids = first.target_ids;
    let first_pose = first.pose;
    let mut bones = first_pose.bones;
    for bone in &mut bones {
        if graph_pose_targets_bone(&first_target_ids, bone) {
            bone.local_transform.translation *= first_weight;
            bone.local_transform.scale *= first_weight;
            bone.local_transform.rotation *= first_weight;
        }
    }

    for weighted in weighted_poses.into_iter().skip(1) {
        for bone in &mut bones {
            if !graph_pose_targets_bone(&weighted.target_ids, bone) {
                continue;
            }
            let Some(other) = weighted
                .pose
                .bones
                .iter()
                .find(|other| other.name == bone.name)
            else {
                continue;
            };
            bone.local_transform.translation += other.local_transform.translation * weighted.weight;
            bone.local_transform.scale += other.local_transform.scale * weighted.weight;
            let mut rotation = other.local_transform.rotation;
            if bone.local_transform.rotation.dot(rotation) < 0.0 {
                rotation = -rotation;
            }
            bone.local_transform.rotation += rotation * weighted.weight;
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

fn blend_weighted_poses(
    weighted_poses: Vec<(AnimationPoseOutput, Real)>,
    source: AnimationPoseSource,
    active_state: Option<String>,
) -> Option<AnimationPoseOutput> {
    blend_graph_base_poses(
        weighted_poses
            .into_iter()
            .map(|(pose, weight)| GraphWeightedPose {
                pose,
                weight,
                target_ids: Vec::new(),
            })
            .collect(),
        source,
        active_state,
    )
}

fn apply_graph_additive_poses(
    base_pose: &mut AnimationPoseOutput,
    additive_poses: Vec<GraphWeightedPose>,
) {
    for additive in additive_poses {
        for bone in &mut base_pose.bones {
            if !graph_pose_targets_bone(&additive.target_ids, bone) {
                continue;
            }
            let Some(additive_bone) = additive
                .pose
                .bones
                .iter()
                .find(|additive_bone| additive_bone.name == bone.name)
            else {
                continue;
            };
            bone.local_transform.translation +=
                additive_bone.local_transform.translation * additive.weight;
            bone.local_transform.scale +=
                (additive_bone.local_transform.scale - Vec3::ONE) * additive.weight;
            let rotation_delta =
                Quat::IDENTITY.slerp(additive_bone.local_transform.rotation, additive.weight);
            bone.local_transform.rotation =
                (rotation_delta * bone.local_transform.rotation).normalize();
        }
    }
}

fn graph_pose_targets_bone(target_ids: &[String], bone: &AnimationPoseBone) -> bool {
    target_ids.is_empty()
        || target_ids.iter().any(|target_id| {
            let target_id = target_id.trim();
            target_id == bone.name
                || target_id
                    .rsplit('/')
                    .next()
                    .is_some_and(|leaf| leaf == bone.name)
        })
}

fn resolve_state_machine_transition_runtime(
    previous: Option<AnimationStateTransitionRuntime>,
    requested: Option<
        &zircon_runtime::core::framework::animation::AnimationStateTransitionEvaluation,
    >,
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
    state_machine: &zircon_runtime::asset::AnimationStateMachineAsset,
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

fn sample_state_transition_clip_events(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    state_machine: &zircon_runtime::asset::AnimationStateMachineAsset,
    parameters: &AnimationParameterMap,
    pending: &PendingStateMachinePoseSample,
    transition: &AnimationStateTransitionRuntime,
) -> Vec<crate::AnimationClipEvent> {
    let mut events = Vec::new();
    let from_graph = state_machine_graph_reference(state_machine, &transition.from_state);
    let to_graph = state_machine_graph_reference(state_machine, &transition.to_state);
    let (from_start_seconds, to_start_seconds) = pending
        .transition
        .as_ref()
        .map(|previous| (previous.from_time_seconds, previous.to_time_seconds))
        .unwrap_or((pending.from_time_seconds, 0.0));

    events.extend(sample_state_graph_clip_events(
        animation,
        asset_manager,
        from_graph,
        parameters,
        pending.entity,
        from_start_seconds,
        transition.from_time_seconds,
    ));
    events.extend(sample_state_graph_clip_events(
        animation,
        asset_manager,
        to_graph,
        parameters,
        pending.entity,
        to_start_seconds,
        transition.to_time_seconds,
    ));
    events
}

fn sample_state_graph_clip_events(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    graph_reference: Option<&zircon_runtime::core::resource::AssetReference>,
    parameters: &AnimationParameterMap,
    entity: EntityId,
    from_time_seconds: Real,
    to_time_seconds: Real,
) -> Vec<crate::AnimationClipEvent> {
    let Some(graph_reference) = graph_reference else {
        return Vec::new();
    };
    let Some(graph_id) = asset_manager.resolve_asset_id(&graph_reference.locator) else {
        return Vec::new();
    };
    let Ok(graph) = asset_manager.load_animation_graph_asset(graph_id) else {
        return Vec::new();
    };
    let graph_evaluation = animation.evaluate_graph(&graph, parameters);
    sample_graph_evaluation_clip_events(
        asset_manager,
        entity,
        from_time_seconds,
        to_time_seconds,
        &graph_evaluation,
    )
}

fn sample_state_graph_pose(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    _state_machine: &zircon_runtime::asset::AnimationStateMachineAsset,
    graph_reference: Option<&zircon_runtime::core::resource::AssetReference>,
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
    state_machine: &'a zircon_runtime::asset::AnimationStateMachineAsset,
    state_name: &str,
) -> Option<&'a zircon_runtime::core::resource::AssetReference> {
    state_machine
        .states
        .iter()
        .find(|state| state.name == state_name)
        .map(|state| &state.graph)
}
