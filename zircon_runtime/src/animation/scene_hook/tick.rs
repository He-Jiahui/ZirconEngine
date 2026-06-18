use std::collections::BTreeMap;

use crate::asset::ProjectAssetManager;
use crate::core::manager::resolve_animation_manager;
use crate::core::math::Real;
use crate::core::CoreHandle;
use crate::scene::LevelSystem;

use super::diagnostics::AnimationSceneFrameDiagnostics;
use super::events::{publish_clip_events, publish_events};
use super::graph::resolve_graph_pose_requests;
use super::node_pose::apply_pose_transforms_to_scene_nodes;
use super::pose::sample_pose_requests;
use super::scan::scan_animation_scene;
use super::sequences::apply_loaded_sequences;
use super::state_machine::resolve_state_machine_pose_requests;

pub(super) fn tick_animation_world(core: &CoreHandle, level: &LevelSystem, delta_seconds: Real) {
    let Ok(animation) = resolve_animation_manager(core) else {
        record_empty_animation_state(core, level);
        return;
    };

    let playback_settings = animation.playback_settings();
    if !playback_settings.enabled {
        record_empty_animation_state(core, level);
        return;
    }

    let asset_manager = core
        .resolve_manager::<ProjectAssetManager>(crate::asset::PROJECT_ASSET_MANAGER_NAME)
        .ok();
    let scan = scan_animation_scene(level, &playback_settings, delta_seconds);
    let mut frame_diagnostics = AnimationSceneFrameDiagnostics::from_scan(&scan);

    let Some(asset_manager) = &asset_manager else {
        level.record_animation_poses(BTreeMap::new());
        level.record_animation_playback_times(
            scan.next_graph_times,
            scan.next_state_machine_times,
            BTreeMap::new(),
        );
        frame_diagnostics.record(core);
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
    frame_diagnostics.published_events =
        publish_clip_events(asset_manager, level, scan.clip_event_samples);

    let mut animation_poses =
        sample_pose_requests(animation.as_ref(), asset_manager, scan.clip_samples);
    let (graph_poses, graph_events) =
        resolve_graph_pose_requests(animation.as_ref(), asset_manager, scan.graph_samples);
    animation_poses.extend(graph_poses);
    frame_diagnostics.published_events += publish_events(level, graph_events);
    let (state_machine_poses, state_machine_events, active_state_updates, transition_updates) =
        resolve_state_machine_pose_requests(
            animation.as_ref(),
            asset_manager,
            scan.state_machine_samples,
        );
    animation_poses.extend(state_machine_poses);
    frame_diagnostics.published_events += publish_events(level, state_machine_events);

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

    frame_diagnostics.output_poses = animation_poses.len();
    frame_diagnostics.applied_transforms =
        apply_pose_transforms_to_scene_nodes(level, &animation_poses);
    frame_diagnostics.state_transitions = transition_updates.len();
    level.record_animation_poses(animation_poses);
    level.record_animation_playback_times(
        scan.next_graph_times,
        scan.next_state_machine_times,
        transition_updates,
    );
    frame_diagnostics.record(core);
}

fn record_empty_animation_state(core: &CoreHandle, level: &LevelSystem) {
    level.record_animation_poses(BTreeMap::new());
    level.record_animation_playback_times(BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
    AnimationSceneFrameDiagnostics::default().record(core);
}
