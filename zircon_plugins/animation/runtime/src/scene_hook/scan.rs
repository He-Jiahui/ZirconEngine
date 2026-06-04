use std::collections::BTreeMap;

use zircon_runtime::core::framework::animation::{AnimationPlaybackSettings, AnimationPoseSource};
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::{EntityId, LevelSystem};

use super::pending::{
    AnimationSceneScan, PendingClipEventSample, PendingGraphPoseSample, PendingPoseSample,
    PendingSequenceSample, PendingStateMachinePoseSample,
};

pub(super) fn scan_animation_scene(
    level: &LevelSystem,
    playback_settings: &AnimationPlaybackSettings,
    delta_seconds: Real,
) -> AnimationSceneScan {
    let (previous_graph_times, previous_state_machine_times, previous_state_machine_transitions) =
        level.animation_playback_times();

    level.with_world_mut(|world| {
        let entity_ids = world.nodes().iter().map(|node| node.id).collect::<Vec<_>>();
        let mut scan = AnimationSceneScan {
            next_graph_times: BTreeMap::<EntityId, Real>::new(),
            next_state_machine_times: BTreeMap::<EntityId, Real>::new(),
            ..AnimationSceneScan::default()
        };

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
                        scan.clip_event_samples.push(PendingClipEventSample {
                            entity,
                            clip_id,
                            from_time_seconds: previous_time_seconds,
                            to_time_seconds: time_seconds,
                            looping,
                        });
                    }
                    let _ = world.set_animation_player(entity, Some(player));
                    if let Some(skeleton) = world.animation_skeleton(entity).cloned() {
                        scan.clip_samples.push(PendingPoseSample {
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
                    scan.sequences.push(PendingSequenceSample {
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
                    scan.next_graph_times.insert(entity, next_time_seconds);
                    if player.playing {
                        if let Some(skeleton) = world.animation_skeleton(entity).cloned() {
                            scan.graph_samples.push(PendingGraphPoseSample {
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
                    scan.next_state_machine_times
                        .insert(entity, next_time_seconds);
                    if player.playing {
                        if let Some(skeleton) = world.animation_skeleton(entity).cloned() {
                            scan.state_machine_samples
                                .push(PendingStateMachinePoseSample {
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

        scan
    })
}
