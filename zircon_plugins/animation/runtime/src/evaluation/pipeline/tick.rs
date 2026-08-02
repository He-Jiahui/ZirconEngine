use std::collections::BTreeMap;
use std::sync::Arc;

use zircon_runtime::asset::project_asset_manager_handle;
use zircon_runtime::core::CoreHandle;
use zircon_runtime::core::framework::animation::AnimationPoseOutput;
use zircon_runtime::core::framework::physics::{
    SimulatedPoseFeed, SkeletalPoseTarget, SkeletalPoseTargets,
};
use zircon_runtime::core::manager::{animation_manager_handle, resolve_manager_service};
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::{EntityId, LevelSystem};

use super::AnimationEvaluationPipeline;
use super::direct_clip_worker::sample_direct_clip_pose_requests;
use super::events::{enqueue_clip_event_samples, publish_clip_events, publish_events};
use super::graph_evaluate::resolve_graph_pose_requests;
use super::parameter_apply::scan_animation_scene;
use super::pose_apply::apply_pose_transforms_to_scene_nodes;
use super::sequences::{LoadedSequenceSample, apply_loaded_sequences};
use super::simulated_pose_blend::blend_simulated_pose_feed;
use super::state_machine_layers::apply_state_machine_layers;
use super::state_machine_step::resolve_state_machine_pose_requests;
use crate::ik::apply_ik_commands;

pub(crate) fn tick_animation_world(core: &CoreHandle, level: &LevelSystem, delta_seconds: Real) {
    let Ok(animation) =
        animation_manager_handle(core).and_then(|handle| resolve_manager_service(core, handle))
    else {
        record_empty_animation_state(level);
        return;
    };

    let playback_settings = animation.playback_settings();
    if !playback_settings.enabled {
        record_empty_animation_state(level);
        return;
    }

    let asset_manager = project_asset_manager_handle(core)
        .and_then(|handle| resolve_manager_service(core, handle))
        .ok();
    let mut projection = level.with_world_mut(|world| {
        if !world.contains_resource::<AnimationEvaluationPipeline>() {
            world.insert_resource(AnimationEvaluationPipeline::default());
        }
        let pipeline = world.resource_mut::<AnimationEvaluationPipeline>();
        pipeline.begin_evaluation_frame();
        std::mem::take(&mut pipeline.projection)
    });
    let scan = scan_animation_scene(
        level,
        &mut projection,
        &playback_settings,
        asset_manager.as_deref(),
        delta_seconds,
    );
    enqueue_clip_event_samples(level, scan.clip_event_samples);
    level.record_animation_event_backlog_continuity(level.animation_clip_event_backlog_len() > 0);

    let Some(asset_manager) = &asset_manager else {
        let (pose_snapshot, targets_changed) = level.with_world_mut(|world| {
            let pipeline = world.resource_mut::<AnimationEvaluationPipeline>();
            pipeline.projection = projection;
            let changed =
                pipeline.update_presentation_poses(&scan.pose_source_entities, BTreeMap::new());
            let targets_changed = changed.is_some();
            let pose_snapshot = changed.unwrap_or_else(|| pipeline.presentation_poses());
            (pose_snapshot, targets_changed)
        });
        if targets_changed {
            publish_skeletal_pose_targets(level, &pose_snapshot);
        }
        level.record_animation_pose_snapshot(pose_snapshot);
        level.record_animation_playback_times(
            scan.next_graph_times,
            scan.next_state_machine_times,
            BTreeMap::new(),
        );
        return;
    };

    let loaded_sequences = scan
        .sequences
        .into_iter()
        .filter_map(|pending| {
            asset_manager
                .load_animation_sequence_asset(pending.sequence_id)
                .ok()
                .map(|sequence| LoadedSequenceSample {
                    asset_id: pending.sequence_id,
                    asset_revision: pending.asset_revision,
                    sequence,
                    time_seconds: pending.time_seconds,
                    looping: pending.looping,
                })
        })
        .collect::<Vec<_>>();
    if !loaded_sequences.is_empty() {
        apply_loaded_sequences(level, &loaded_sequences);
    }
    let (
        mut animation_poses,
        graph_poses,
        graph_event_samples,
        state_machine_poses,
        state_machine_event_samples,
        layer_diagnostics,
        evaluation_diagnostics,
        active_state_updates,
        transition_updates,
    ) = level.with_world_mut(|world| {
        let pipeline = world.resource_mut::<AnimationEvaluationPipeline>();
        pipeline.projection = projection;
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
        let evaluation_diagnostics = pipeline.drain_clip_evaluation_diagnostics();
        (
            animation_poses,
            graph_poses,
            graph_event_samples,
            state_machine_poses,
            state_machine_event_samples,
            layer_result.diagnostics,
            evaluation_diagnostics,
            active_state_updates,
            transition_updates,
        )
    });
    animation_poses.extend(graph_poses);
    animation_poses.extend(state_machine_poses);
    enqueue_clip_event_samples(level, graph_event_samples);
    enqueue_clip_event_samples(level, state_machine_event_samples);
    publish_clip_events(asset_manager, level);
    level.record_animation_event_backlog_continuity(level.animation_clip_event_backlog_len() > 0);
    publish_events(level, layer_diagnostics);
    publish_events(level, evaluation_diagnostics);
    let ik_commands = animation.drain_ik_commands(level.world_handle());
    let ik_diagnostics = level.with_world_mut(|world| {
        let pipeline = world.resource::<AnimationEvaluationPipeline>();
        if world.contains_resource::<SimulatedPoseFeed>() {
            blend_simulated_pose_feed(
                pipeline,
                world.resource::<SimulatedPoseFeed>(),
                &scan.skeletons_by_entity,
                &mut animation_poses,
            );
        }
        apply_ik_commands(
            pipeline,
            asset_manager,
            &scan.skeletons_by_entity,
            ik_commands,
            &mut animation_poses,
        )
    });
    publish_events(level, ik_diagnostics);

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

    apply_pose_transforms_to_scene_nodes(level, &animation_poses);
    let (pose_snapshot, targets_changed) = level.with_world_mut(|world| {
        let pipeline = world.resource_mut::<AnimationEvaluationPipeline>();
        let changed =
            pipeline.update_presentation_poses(&scan.pose_source_entities, animation_poses);
        let targets_changed = changed.is_some();
        let pose_snapshot = changed.unwrap_or_else(|| pipeline.presentation_poses());
        (pose_snapshot, targets_changed)
    });
    if targets_changed {
        publish_skeletal_pose_targets(level, &pose_snapshot);
    }
    level.record_animation_pose_snapshot(pose_snapshot);
    level.record_animation_playback_times(
        scan.next_graph_times,
        scan.next_state_machine_times,
        transition_updates,
    );
}

fn publish_skeletal_pose_targets(
    level: &LevelSystem,
    animation_poses: &BTreeMap<EntityId, AnimationPoseOutput>,
) {
    level.with_world_mut(|world| {
        if !world.contains_resource::<SkeletalPoseTargets>() {
            return;
        }
        let targets = world.resource_mut::<SkeletalPoseTargets>();
        targets.clear();
        for (entity, pose) in animation_poses {
            targets.replace(
                *entity,
                Arc::from(
                    pose.bones
                        .iter()
                        .map(|bone| SkeletalPoseTarget {
                            bone_name: bone.name.clone(),
                            local_transform: bone.local_transform,
                            normalized_weight: 1.0,
                        })
                        .collect::<Vec<_>>(),
                ),
            );
        }
    });
}

fn record_empty_animation_state(level: &LevelSystem) {
    level.with_world_mut(|world| {
        if let Some(pipeline) = world.get_resource_mut::<AnimationEvaluationPipeline>() {
            pipeline.reset_evaluation_state();
        }
    });
    publish_skeletal_pose_targets(level, &BTreeMap::new());
    level.record_animation_event_backlog_continuity(level.animation_clip_event_backlog_len() > 0);
    level.record_animation_poses(BTreeMap::new());
    level.record_animation_playback_times(BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
}
