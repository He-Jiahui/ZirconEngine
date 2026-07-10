use std::collections::BTreeMap;

use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::manager::resolve_animation_manager;
use zircon_runtime::core::math::Real;
use zircon_runtime::core::CoreHandle;
use zircon_runtime::scene::LevelSystem;

use super::clip_sample::sample_pose_requests;
use super::events::{publish_clip_events, publish_events};
use super::graph_evaluate::resolve_graph_pose_requests;
use super::parameter_apply::scan_animation_scene;
use super::pose_apply::apply_pose_transforms_to_scene_nodes;
use super::sequences::apply_loaded_sequences;
use super::state_machine_step::resolve_state_machine_pose_requests;
use super::AnimationEvaluationPipeline;

pub(crate) fn tick_animation_world(core: &CoreHandle, level: &LevelSystem, delta_seconds: Real) {
    let Ok(animation) = resolve_animation_manager(core) else {
        record_empty_animation_state(level);
        return;
    };

    let playback_settings = animation.playback_settings();
    if !playback_settings.enabled {
        record_empty_animation_state(level);
        return;
    }

    let asset_manager = core
        .resolve_manager::<ProjectAssetManager>(zircon_runtime::asset::PROJECT_ASSET_MANAGER_NAME)
        .ok();
    let scan = scan_animation_scene(level, &playback_settings, delta_seconds);

    let Some(asset_manager) = &asset_manager else {
        level.record_animation_poses(BTreeMap::new());
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
                .map(|sequence| (sequence, pending.time_seconds, pending.looping))
        })
        .collect::<Vec<_>>();
    if !loaded_sequences.is_empty() {
        apply_loaded_sequences(animation.as_ref(), level, &loaded_sequences);
    }
    publish_clip_events(asset_manager, level, scan.clip_event_samples);

    let (
        mut animation_poses,
        graph_poses,
        graph_events,
        state_machine_poses,
        state_machine_events,
        evaluation_diagnostics,
        active_state_updates,
        transition_updates,
    ) = level.with_world_mut(|world| {
        let pipeline = world.resource_mut::<AnimationEvaluationPipeline>();
        pipeline
            .clip_evaluator_mut()
            .bind_resources(&asset_manager.resource_manager());
        let animation_poses = sample_pose_requests(
            pipeline.clip_evaluator_mut(),
            asset_manager,
            scan.clip_samples,
        );
        let (graph_poses, graph_events) =
            resolve_graph_pose_requests(pipeline, asset_manager, scan.graph_samples);
        let evaluator = pipeline.clip_evaluator_mut();
        let (state_machine_poses, state_machine_events, active_state_updates, transition_updates) =
            resolve_state_machine_pose_requests(
                animation.as_ref(),
                evaluator,
                asset_manager,
                scan.state_machine_samples,
            );
        let evaluation_diagnostics = evaluator.drain_diagnostics();
        (
            animation_poses,
            graph_poses,
            graph_events,
            state_machine_poses,
            state_machine_events,
            evaluation_diagnostics,
            active_state_updates,
            transition_updates,
        )
    });
    animation_poses.extend(graph_poses);
    publish_events(level, graph_events);
    animation_poses.extend(state_machine_poses);
    publish_events(level, state_machine_events);
    publish_events(level, evaluation_diagnostics);

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
    level.record_animation_poses(animation_poses);
    level.record_animation_playback_times(
        scan.next_graph_times,
        scan.next_state_machine_times,
        transition_updates,
    );
}

fn record_empty_animation_state(level: &LevelSystem) {
    level.record_animation_poses(BTreeMap::new());
    level.record_animation_playback_times(BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
}
