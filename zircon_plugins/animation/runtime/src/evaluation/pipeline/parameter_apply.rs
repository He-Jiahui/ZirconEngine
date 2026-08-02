use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::asset::{AssetId, ProjectAssetManager};
use zircon_runtime::core::framework::animation::{AnimationPlaybackSettings, AnimationPoseSource};
use zircon_runtime::core::math::Real;
use zircon_runtime::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, ResourceHandle,
};
use zircon_runtime::scene::components::{
    AnimationGraphPlayerComponent, AnimationPlayerComponent, AnimationSequencePlayerComponent,
    AnimationSkeletonComponent, AnimationStateMachinePlayerComponent,
};
use zircon_runtime::scene::ecs::{ChangeTick, QueryState, Ref};
use zircon_runtime::scene::{EntityId, LevelSystem, World};

use super::animation_evaluation_pipeline::AnimationEvaluationProjectionStats;
use super::requests::{
    AnimationSceneScan, PendingClipEventSample, PendingGraphPoseSample, PendingPoseSample,
    PendingSequenceSample, PendingStateMachinePoseSample,
};

/// Persistent typed projection for the animation system. It caches only ECS
/// candidates that own animation components and records the last observed
/// component/resource revision for paused instances.
#[derive(Debug, Default)]
pub(super) struct AnimationEvaluationProjection {
    skeletons: Option<QueryState<(EntityId, Ref<'static, AnimationSkeletonComponent>)>>,
    clip_players: Option<QueryState<(EntityId, Ref<'static, AnimationPlayerComponent>)>>,
    sequence_players:
        Option<QueryState<(EntityId, Ref<'static, AnimationSequencePlayerComponent>)>>,
    graph_players: Option<QueryState<(EntityId, Ref<'static, AnimationGraphPlayerComponent>)>>,
    state_machine_players:
        Option<QueryState<(EntityId, Ref<'static, AnimationStateMachinePlayerComponent>)>>,
    clip_revisions: BTreeMap<EntityId, ClipProjectionRevision>,
    sequence_revisions: BTreeMap<EntityId, SequenceProjectionRevision>,
    graph_revisions: BTreeMap<EntityId, GraphProjectionRevision>,
    state_machine_revisions: BTreeMap<EntityId, StateMachineProjectionRevision>,
    stats: AnimationEvaluationProjectionStats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SkeletonProjectionRevision {
    component_change: ChangeTick,
    asset_revision: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClipProjectionRevision {
    player_change: ChangeTick,
    skeleton: Option<SkeletonProjectionRevision>,
    asset_revision: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SequenceProjectionRevision {
    player_change: ChangeTick,
    asset_revision: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GraphProjectionRevision {
    player_change: ChangeTick,
    skeleton: Option<SkeletonProjectionRevision>,
    asset_revision: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StateMachineProjectionRevision {
    player_change: ChangeTick,
    skeleton: Option<SkeletonProjectionRevision>,
    asset_revision: Option<u64>,
}

impl AnimationEvaluationProjection {
    pub(super) fn stats(&self) -> AnimationEvaluationProjectionStats {
        self.stats
    }

    fn scan(
        &mut self,
        world: &mut World,
        playback_settings: &AnimationPlaybackSettings,
        assets: Option<&ProjectAssetManager>,
        delta_seconds: Real,
        previous_graph_times: &BTreeMap<EntityId, Real>,
        previous_state_machine_times: &BTreeMap<EntityId, Real>,
        previous_state_machine_transitions: &BTreeMap<
            EntityId,
            zircon_runtime::scene::AnimationStateTransitionRuntime,
        >,
    ) -> AnimationSceneScan {
        let mut scan = AnimationSceneScan {
            next_graph_times: BTreeMap::new(),
            next_state_machine_times: BTreeMap::new(),
            ..AnimationSceneScan::default()
        };
        let mut skeleton_revisions = BTreeMap::new();

        if self.skeletons.is_none() {
            self.skeletons =
                Some(world.query::<(EntityId, Ref<'static, AnimationSkeletonComponent>)>());
        }
        for (entity, skeleton) in self
            .skeletons
            .as_mut()
            .expect("animation skeleton query is initialized")
            .iter_cached(world)
        {
            self.stats.skeleton_candidate_count =
                self.stats.skeleton_candidate_count.saturating_add(1);
            let skeleton_id = skeleton.skeleton.id();
            scan.skeletons_by_entity.insert(entity, skeleton_id);
            skeleton_revisions.insert(
                entity,
                SkeletonProjectionRevision {
                    component_change: skeleton.last_changed(),
                    asset_revision: skeleton_asset_revision(assets, skeleton_id),
                },
            );
        }

        self.scan_clip_players(
            world,
            playback_settings,
            assets,
            delta_seconds,
            &scan.skeletons_by_entity,
            &skeleton_revisions,
            &mut scan,
        );
        self.scan_sequence_players(world, playback_settings, assets, delta_seconds, &mut scan);
        self.scan_graph_players(
            world,
            playback_settings,
            assets,
            delta_seconds,
            previous_graph_times,
            &scan.skeletons_by_entity,
            &skeleton_revisions,
            &mut scan,
        );
        self.scan_state_machine_players(
            world,
            playback_settings,
            assets,
            delta_seconds,
            previous_state_machine_times,
            previous_state_machine_transitions,
            &scan.skeletons_by_entity,
            &skeleton_revisions,
            &mut scan,
        );
        scan
    }

    fn scan_clip_players(
        &mut self,
        world: &mut World,
        playback_settings: &AnimationPlaybackSettings,
        assets: Option<&ProjectAssetManager>,
        delta_seconds: Real,
        skeletons: &BTreeMap<EntityId, AssetId>,
        skeleton_revisions: &BTreeMap<EntityId, SkeletonProjectionRevision>,
        scan: &mut AnimationSceneScan,
    ) {
        if !playback_settings.skeletal_clips {
            self.clip_revisions.clear();
            return;
        }
        if self.clip_players.is_none() {
            self.clip_players =
                Some(world.query::<(EntityId, Ref<'static, AnimationPlayerComponent>)>());
        }

        let mut seen = BTreeSet::new();
        let mut updates = Vec::new();
        for (entity, player) in self
            .clip_players
            .as_mut()
            .expect("animation clip player query is initialized")
            .iter_cached(world)
        {
            self.stats.clip_player_candidate_count =
                self.stats.clip_player_candidate_count.saturating_add(1);
            seen.insert(entity);
            let clip_id = player.clip.id();
            let skeleton_id = skeletons.get(&entity).copied();
            if skeleton_id.is_some() {
                scan.pose_source_entities.insert(entity);
            }
            let revision = ClipProjectionRevision {
                player_change: player.last_changed(),
                skeleton: skeleton_revisions.get(&entity).copied(),
                asset_revision: clip_asset_revision(assets, clip_id),
            };
            let should_sample =
                player.playing || self.clip_revisions.get(&entity) != Some(&revision);
            self.clip_revisions.insert(entity, revision);
            if !should_sample {
                continue;
            }

            let previous_time_seconds = player.time_seconds;
            let time_seconds = if player.playing {
                (player.time_seconds + delta_seconds * player.playback_speed).max(0.0)
            } else {
                player.time_seconds
            };
            if player.playing && time_seconds != player.time_seconds {
                let mut updated = (*player).clone();
                updated.time_seconds = time_seconds;
                updates.push((entity, updated));
            }
            if player.playing {
                scan.clip_event_samples.push(PendingClipEventSample {
                    entity,
                    clip_id,
                    from_time_seconds: previous_time_seconds,
                    to_time_seconds: time_seconds,
                    looping: player.looping,
                });
            }
            if let Some(skeleton_id) = skeleton_id {
                scan.clip_samples.push(PendingPoseSample {
                    entity,
                    skeleton_id,
                    clip_id,
                    time_seconds,
                    looping: player.looping,
                    source: AnimationPoseSource::Clip,
                    active_state: None,
                });
                self.stats.clip_pose_request_count =
                    self.stats.clip_pose_request_count.saturating_add(1);
            }
        }
        self.clip_revisions
            .retain(|entity, _| seen.contains(entity));
        for (entity, player) in updates {
            let _ = world.set_animation_player(entity, Some(player));
        }
    }

    fn scan_sequence_players(
        &mut self,
        world: &mut World,
        playback_settings: &AnimationPlaybackSettings,
        assets: Option<&ProjectAssetManager>,
        delta_seconds: Real,
        scan: &mut AnimationSceneScan,
    ) {
        if !playback_settings.property_tracks {
            self.sequence_revisions.clear();
            return;
        }
        if self.sequence_players.is_none() {
            self.sequence_players =
                Some(world.query::<(EntityId, Ref<'static, AnimationSequencePlayerComponent>)>());
        }

        let mut seen = BTreeSet::new();
        let mut updates = Vec::new();
        for (entity, player) in self
            .sequence_players
            .as_mut()
            .expect("animation sequence player query is initialized")
            .iter_cached(world)
        {
            self.stats.sequence_player_candidate_count =
                self.stats.sequence_player_candidate_count.saturating_add(1);
            seen.insert(entity);
            let sequence_id = player.sequence.id();
            let revision = SequenceProjectionRevision {
                player_change: player.last_changed(),
                asset_revision: sequence_asset_revision(assets, sequence_id),
            };
            let should_sample =
                player.playing || self.sequence_revisions.get(&entity) != Some(&revision);
            self.sequence_revisions.insert(entity, revision);
            if !should_sample {
                continue;
            }

            let time_seconds = if player.playing {
                (player.time_seconds + delta_seconds * player.playback_speed).max(0.0)
            } else {
                player.time_seconds
            };
            if player.playing && time_seconds != player.time_seconds {
                let mut updated = (*player).clone();
                updated.time_seconds = time_seconds;
                updates.push((entity, updated));
            }
            scan.sequences.push(PendingSequenceSample {
                sequence_id,
                asset_revision: revision.asset_revision,
                time_seconds,
                looping: player.looping,
            });
            self.stats.sequence_request_count = self.stats.sequence_request_count.saturating_add(1);
        }
        self.sequence_revisions
            .retain(|entity, _| seen.contains(entity));
        for (entity, player) in updates {
            let _ = world.set_animation_sequence_player(entity, Some(player));
        }
    }

    fn scan_graph_players(
        &mut self,
        world: &mut World,
        playback_settings: &AnimationPlaybackSettings,
        assets: Option<&ProjectAssetManager>,
        delta_seconds: Real,
        previous_times: &BTreeMap<EntityId, Real>,
        skeletons: &BTreeMap<EntityId, AssetId>,
        skeleton_revisions: &BTreeMap<EntityId, SkeletonProjectionRevision>,
        scan: &mut AnimationSceneScan,
    ) {
        if !playback_settings.graphs {
            self.graph_revisions.clear();
            return;
        }
        if self.graph_players.is_none() {
            self.graph_players =
                Some(world.query::<(EntityId, Ref<'static, AnimationGraphPlayerComponent>)>());
        }

        let mut seen = BTreeSet::new();
        for (entity, player) in self
            .graph_players
            .as_mut()
            .expect("animation graph player query is initialized")
            .iter_cached(world)
        {
            self.stats.graph_player_candidate_count =
                self.stats.graph_player_candidate_count.saturating_add(1);
            seen.insert(entity);
            let previous_time_seconds = previous_times.get(&entity).copied().unwrap_or(0.0);
            let next_time_seconds =
                previous_time_seconds + if player.playing { delta_seconds } else { 0.0 };
            scan.next_graph_times.insert(entity, next_time_seconds);
            let skeleton_id = skeletons.get(&entity).copied();
            if skeleton_id.is_some() {
                scan.pose_source_entities.insert(entity);
            }
            let revision = GraphProjectionRevision {
                player_change: player.last_changed(),
                skeleton: skeleton_revisions.get(&entity).copied(),
                asset_revision: graph_asset_revision(assets, player.graph.id()),
            };
            let should_sample =
                player.playing || self.graph_revisions.get(&entity) != Some(&revision);
            self.graph_revisions.insert(entity, revision);
            if !should_sample {
                continue;
            }
            let Some(skeleton_id) = skeleton_id else {
                continue;
            };
            scan.graph_samples.push(PendingGraphPoseSample {
                entity,
                skeleton_id,
                graph_id: player.graph.id(),
                parameters: player.parameters.clone(),
                from_time_seconds: previous_time_seconds,
                to_time_seconds: next_time_seconds,
            });
            self.stats.graph_pose_request_count =
                self.stats.graph_pose_request_count.saturating_add(1);
        }
        self.graph_revisions
            .retain(|entity, _| seen.contains(entity));
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_state_machine_players(
        &mut self,
        world: &mut World,
        playback_settings: &AnimationPlaybackSettings,
        assets: Option<&ProjectAssetManager>,
        delta_seconds: Real,
        previous_times: &BTreeMap<EntityId, Real>,
        previous_transitions: &BTreeMap<
            EntityId,
            zircon_runtime::scene::AnimationStateTransitionRuntime,
        >,
        skeletons: &BTreeMap<EntityId, AssetId>,
        skeleton_revisions: &BTreeMap<EntityId, SkeletonProjectionRevision>,
        scan: &mut AnimationSceneScan,
    ) {
        if !playback_settings.state_machines {
            self.state_machine_revisions.clear();
            return;
        }
        if self.state_machine_players.is_none() {
            self.state_machine_players = Some(
                world.query::<(EntityId, Ref<'static, AnimationStateMachinePlayerComponent>)>(),
            );
        }

        let mut seen = BTreeSet::new();
        for (entity, player) in self
            .state_machine_players
            .as_mut()
            .expect("animation state machine player query is initialized")
            .iter_cached(world)
        {
            self.stats.state_machine_player_candidate_count = self
                .stats
                .state_machine_player_candidate_count
                .saturating_add(1);
            seen.insert(entity);
            let previous_time_seconds = previous_times.get(&entity).copied().unwrap_or(0.0);
            let next_time_seconds =
                previous_time_seconds + if player.playing { delta_seconds } else { 0.0 };
            scan.next_state_machine_times
                .insert(entity, next_time_seconds);
            let skeleton_id = skeletons.get(&entity).copied();
            if skeleton_id.is_some() {
                scan.pose_source_entities.insert(entity);
            }
            let revision = StateMachineProjectionRevision {
                player_change: player.last_changed(),
                skeleton: skeleton_revisions.get(&entity).copied(),
                asset_revision: state_machine_asset_revision(assets, player.state_machine.id()),
            };
            let should_sample =
                player.playing || self.state_machine_revisions.get(&entity) != Some(&revision);
            self.state_machine_revisions.insert(entity, revision);
            if !should_sample {
                continue;
            }
            let Some(skeleton_id) = skeleton_id else {
                continue;
            };
            scan.state_machine_samples
                .push(PendingStateMachinePoseSample {
                    entity,
                    skeleton_id,
                    state_machine_id: player.state_machine.id(),
                    parameters: player.parameters.clone(),
                    active_state: player.active_state.clone(),
                    from_time_seconds: previous_time_seconds,
                    to_time_seconds: next_time_seconds,
                    delta_seconds,
                    transition: previous_transitions.get(&entity).cloned(),
                });
            self.stats.state_machine_pose_request_count = self
                .stats
                .state_machine_pose_request_count
                .saturating_add(1);
        }
        self.state_machine_revisions
            .retain(|entity, _| seen.contains(entity));
    }
}

pub(super) fn scan_animation_scene(
    level: &LevelSystem,
    projection: &mut AnimationEvaluationProjection,
    playback_settings: &AnimationPlaybackSettings,
    assets: Option<&ProjectAssetManager>,
    delta_seconds: Real,
) -> AnimationSceneScan {
    let (previous_graph_times, previous_state_machine_times, previous_state_machine_transitions) =
        level.animation_playback_times();
    level.with_world_mut(|world| {
        projection.scan(
            world,
            playback_settings,
            assets,
            delta_seconds,
            &previous_graph_times,
            &previous_state_machine_times,
            &previous_state_machine_transitions,
        )
    })
}

fn clip_asset_revision(assets: Option<&ProjectAssetManager>, id: AssetId) -> Option<u64> {
    assets.and_then(|assets| {
        assets
            .resource_manager()
            .snapshot(ResourceHandle::<AnimationClipMarker>::new(id))
            .map(|snapshot| snapshot.revision())
    })
}

fn sequence_asset_revision(assets: Option<&ProjectAssetManager>, id: AssetId) -> Option<u64> {
    assets.and_then(|assets| {
        assets
            .resource_manager()
            .snapshot(ResourceHandle::<AnimationSequenceMarker>::new(id))
            .map(|snapshot| snapshot.revision())
    })
}

fn graph_asset_revision(assets: Option<&ProjectAssetManager>, id: AssetId) -> Option<u64> {
    assets.and_then(|assets| {
        assets
            .resource_manager()
            .snapshot(ResourceHandle::<AnimationGraphMarker>::new(id))
            .map(|snapshot| snapshot.revision())
    })
}

fn state_machine_asset_revision(assets: Option<&ProjectAssetManager>, id: AssetId) -> Option<u64> {
    assets.and_then(|assets| {
        assets
            .resource_manager()
            .snapshot(ResourceHandle::<AnimationStateMachineMarker>::new(id))
            .map(|snapshot| snapshot.revision())
    })
}

fn skeleton_asset_revision(assets: Option<&ProjectAssetManager>, id: AssetId) -> Option<u64> {
    assets.and_then(|assets| {
        assets
            .resource_manager()
            .snapshot(ResourceHandle::<AnimationSkeletonMarker>::new(id))
            .map(|snapshot| snapshot.revision())
    })
}
