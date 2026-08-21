mod graph;
mod parameters;
mod poison_recovery;
mod pose;
mod sampling;
mod state_machine;

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::animation::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSkeletonAsset, AnimationStateMachineAsset,
};
use zircon_runtime::core::framework::animation::{
    AnimationGraphEvaluation, AnimationIkCommand, AnimationIkCommandError, AnimationManager,
    AnimationParameterMap, AnimationParameterValue, AnimationPlaybackSettings, AnimationPoseOutput,
    AnimationResult, AnimationStateMachineEvaluation, AnimationTrackPath,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::{CoreError, CoreWeak};

const MAX_PENDING_IK_COMMANDS_PER_WORLD: usize = 4_096;

#[derive(Clone, Debug, Default)]
struct WorldIkCommandQueue {
    replacement_epoch: u64,
    commands: Vec<AnimationIkCommand>,
}

#[derive(Clone, Debug)]
pub struct DefaultAnimationManager {
    // The registry owns this service, so its runtime back-reference must not complete an Arc cycle.
    core: Option<CoreWeak>,
    playback_settings: Arc<Mutex<AnimationPlaybackSettings>>,
    ik_commands: Arc<Mutex<HashMap<WorldHandle, WorldIkCommandQueue>>>,
}

impl Default for DefaultAnimationManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DefaultAnimationManager {
    pub fn new(core: Option<&CoreWeak>) -> Self {
        let playback_settings = core
            .and_then(CoreWeak::upgrade)
            .and_then(|core| core.load_config(crate::ANIMATION_PLAYBACK_CONFIG_KEY).ok())
            .unwrap_or_default();
        Self {
            core: core.cloned(),
            playback_settings: Arc::new(Mutex::new(playback_settings)),
            ik_commands: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn store_playback_settings(
        &self,
        playback_settings: AnimationPlaybackSettings,
    ) -> Result<(), CoreError> {
        *poison_recovery::lock_recover(&self.playback_settings) = playback_settings.clone();
        if let Some(core) = self.core.as_ref().and_then(CoreWeak::upgrade) {
            core.store_config(crate::ANIMATION_PLAYBACK_CONFIG_KEY, &playback_settings)?;
        }
        Ok(())
    }
}

impl AnimationManager for DefaultAnimationManager {
    fn playback_settings(&self) -> AnimationPlaybackSettings {
        poison_recovery::lock_recover(&self.playback_settings).clone()
    }

    fn normalize_track_path(&self, path: &AnimationTrackPath) -> AnimationTrackPath {
        path.clone()
    }

    fn parameter_defaults(&self, graph: &AnimationGraphAsset) -> AnimationParameterMap {
        parameters::parameter_defaults(graph)
    }

    fn parameter_value(
        &self,
        parameters: &AnimationParameterMap,
        name: &str,
    ) -> Option<AnimationParameterValue> {
        parameters::parameter_value(parameters, name)
    }

    fn set_parameter(
        &self,
        parameters: &mut AnimationParameterMap,
        name: &str,
        value: AnimationParameterValue,
    ) {
        parameters::set_parameter(parameters, name, value)
    }

    fn evaluate_graph(
        &self,
        graph: &AnimationGraphAsset,
        overrides: &AnimationParameterMap,
    ) -> AnimationGraphEvaluation {
        graph::evaluate_graph(graph, overrides)
    }

    fn evaluate_state_machine(
        &self,
        state_machine: &AnimationStateMachineAsset,
        current_state: Option<&str>,
        parameters: &AnimationParameterMap,
    ) -> AnimationStateMachineEvaluation {
        state_machine::evaluate_state_machine(state_machine, current_state, parameters)
    }

    fn sample_clip_pose(
        &self,
        skeleton: &AnimationSkeletonAsset,
        clip: &AnimationClipAsset,
        time_seconds: zircon_runtime::core::math::Real,
        looping: bool,
    ) -> AnimationResult<AnimationPoseOutput> {
        pose::sample_clip_pose(skeleton, clip, time_seconds, looping)
    }

    fn queue_ik_command(
        &self,
        replacement_epoch: u64,
        command: AnimationIkCommand,
    ) -> Result<(), AnimationIkCommandError> {
        command.validate()?;
        let world = command.world();
        let mut queues = poison_recovery::lock_recover(&self.ik_commands);
        let queue = queues.entry(world).or_default();
        if replacement_epoch < queue.replacement_epoch {
            return Err(AnimationIkCommandError::StaleReplacementEpoch {
                world,
                submitted_epoch: replacement_epoch,
                current_epoch: queue.replacement_epoch,
            });
        }
        if replacement_epoch > queue.replacement_epoch {
            queue.replacement_epoch = replacement_epoch;
            queue.commands.clear();
        }
        if queue.commands.len() >= MAX_PENDING_IK_COMMANDS_PER_WORLD {
            return Err(AnimationIkCommandError::QueueFull {
                world,
                capacity: MAX_PENDING_IK_COMMANDS_PER_WORLD,
            });
        }
        queue.commands.push(command);
        Ok(())
    }

    fn drain_ik_commands(
        &self,
        world: WorldHandle,
        replacement_epoch: u64,
    ) -> Vec<AnimationIkCommand> {
        self.drain_ik_commands_excluding(world, replacement_epoch, &BTreeSet::new())
    }

    fn drain_ik_commands_excluding(
        &self,
        world: WorldHandle,
        replacement_epoch: u64,
        deferred_entities: &BTreeSet<zircon_runtime::scene::EntityId>,
    ) -> Vec<AnimationIkCommand> {
        let mut queues = poison_recovery::lock_recover(&self.ik_commands);
        let queue = queues.entry(world).or_default();
        if replacement_epoch < queue.replacement_epoch {
            return Vec::new();
        }
        if replacement_epoch > queue.replacement_epoch {
            queue.replacement_epoch = replacement_epoch;
            queue.commands.clear();
            return Vec::new();
        }
        let (retained, admitted) = std::mem::take(&mut queue.commands)
            .into_iter()
            .partition(|command| deferred_entities.contains(&command.entity()));
        queue.commands = retained;
        admitted
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use zircon_runtime::core::framework::animation::{
        AnimationIkCommand, AnimationIkCommandError, AnimationLookAtCommand, AnimationManager,
        AnimationTargetId,
    };
    use zircon_runtime::core::framework::scene::WorldHandle;
    use zircon_runtime::core::math::Vec3;
    use zircon_runtime::scene::EntityId;

    use super::DefaultAnimationManager;

    fn look_at(world: WorldHandle, entity: u64) -> AnimationIkCommand {
        AnimationIkCommand::LookAt(AnimationLookAtCommand {
            world,
            entity,
            bone: AnimationTargetId::from_segments(["head"]),
            target: Vec3::new(0.0, 1.0, 1.0),
            axis: Vec3::new(0.0, 0.0, 1.0),
            clamp_degrees: 45.0,
            weight: 1.0,
        })
    }

    #[test]
    fn selective_ik_drain_retains_deferred_entity_commands() {
        let manager = DefaultAnimationManager::default();
        let world = WorldHandle::new(7);
        let replacement_epoch = 3;
        manager
            .queue_ik_command(replacement_epoch, look_at(world, 17))
            .expect("admitted entity command queues");
        manager
            .queue_ik_command(replacement_epoch, look_at(world, 18))
            .expect("deferred entity command queues");

        let admitted =
            manager.drain_ik_commands_excluding(world, replacement_epoch, &BTreeSet::from([18]));
        assert_eq!(admitted.len(), 1);
        assert_eq!(admitted[0].entity(), 17);

        let retained = manager.drain_ik_commands(world, replacement_epoch);
        assert_eq!(retained.len(), 1);
        assert_eq!(retained[0].entity(), 18);
    }

    #[test]
    fn replacement_epoch_retires_deferred_ik_commands_and_rejects_late_old_epoch() {
        let manager = DefaultAnimationManager::default();
        let world = WorldHandle::new(7);
        manager
            .queue_ik_command(1, look_at(world, 17))
            .expect("old World command queues");
        assert!(
            manager
                .drain_ik_commands_excluding(world, 1, &BTreeSet::from([17]))
                .is_empty()
        );

        assert!(manager.drain_ik_commands(world, 2).is_empty());
        assert_eq!(
            manager.queue_ik_command(1, look_at(world, 18)),
            Err(AnimationIkCommandError::StaleReplacementEpoch {
                world,
                submitted_epoch: 1,
                current_epoch: 2,
            })
        );

        let current = look_at(world, 19);
        manager
            .queue_ik_command(2, current.clone())
            .expect("replacement World command queues");
        assert_eq!(manager.drain_ik_commands(world, 2), vec![current]);
    }

    #[test]
    fn selective_ik_drain_preserves_order_at_queue_scale() {
        const COMMANDS: EntityId = 1_024;

        let manager = DefaultAnimationManager::default();
        let world = WorldHandle::new(11);
        let replacement_epoch = 5;
        let deferred = (1..=COMMANDS)
            .filter(|entity| entity % 2 == 0)
            .collect::<BTreeSet<_>>();
        for entity in 1..=COMMANDS {
            manager
                .queue_ik_command(replacement_epoch, look_at(world, entity))
                .expect("scale command queues");
        }

        let admitted = manager.drain_ik_commands_excluding(world, replacement_epoch, &deferred);
        assert_eq!(
            admitted
                .iter()
                .map(AnimationIkCommand::entity)
                .collect::<Vec<_>>(),
            (1..=COMMANDS)
                .filter(|entity| entity % 2 == 1)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            manager
                .drain_ik_commands(world, replacement_epoch)
                .iter()
                .map(AnimationIkCommand::entity)
                .collect::<Vec<_>>(),
            (1..=COMMANDS)
                .filter(|entity| entity % 2 == 0)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    #[ignore = "managed release performance evidence"]
    fn borrowed_deferred_ik_set_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 21;
        const COMMANDS: EntityId = 4_096;
        const DEFERRED: usize = 2_048;

        let command_entities = (1..=COMMANDS).collect::<Vec<_>>();
        let deferred = command_entities
            .iter()
            .copied()
            .filter(|entity| entity % 2 == 0)
            .collect::<BTreeSet<_>>();
        assert_eq!(deferred.len(), DEFERRED);

        black_box(measure_legacy_membership(&command_entities, &deferred));
        black_box(measure_borrowed_set_membership(
            &command_entities,
            &deferred,
        ));

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut borrowed_set_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            let (legacy, borrowed_set) = if pair % 2 == 0 {
                (
                    measure_legacy_membership(&command_entities, &deferred),
                    measure_borrowed_set_membership(&command_entities, &deferred),
                )
            } else {
                let borrowed_set = measure_borrowed_set_membership(&command_entities, &deferred);
                let legacy = measure_legacy_membership(&command_entities, &deferred);
                (legacy, borrowed_set)
            };
            legacy_samples.push(legacy);
            borrowed_set_samples.push(borrowed_set);
        }

        let legacy_p95 = nearest_rank_percentile(&legacy_samples, 95);
        let borrowed_set_p95 = nearest_rank_percentile(&borrowed_set_samples, 95);
        let ratio_bps =
            borrowed_set_p95.as_nanos().saturating_mul(10_000) / legacy_p95.as_nanos().max(1);
        let legacy_ns = join_duration_samples(&legacy_samples);
        let borrowed_set_ns = join_duration_samples(&borrowed_set_samples);
        println!(
            "PERF-MVP-RUNTIME08C-IK-DEFERRED sample_pairs={SAMPLE_PAIRS} sample_order=alternating percentile_method=nearest_rank commands={COMMANDS} deferred={DEFERRED} legacy_materialized_entities={DEFERRED} optimized_materialized_entities=0 legacy_p95_ns={} optimized_p95_ns={} ratio_bps={ratio_bps} threshold_bps=2500 legacy_ns={legacy_ns} optimized_ns={borrowed_set_ns}",
            legacy_p95.as_nanos(),
            borrowed_set_p95.as_nanos(),
        );
        assert!(
            borrowed_set_p95.as_nanos().saturating_mul(4) <= legacy_p95.as_nanos(),
            "borrowed deferred-set P95 {borrowed_set_p95:?} must be at most 25% of legacy Vec membership P95 {legacy_p95:?}"
        );
    }

    fn measure_legacy_membership(
        command_entities: &[EntityId],
        deferred: &BTreeSet<EntityId>,
    ) -> Duration {
        let started = Instant::now();
        let materialized = deferred.iter().copied().collect::<Vec<_>>();
        let retained = command_entities
            .iter()
            .filter(|entity| materialized.contains(*entity))
            .count();
        assert_eq!(retained, deferred.len());
        black_box(materialized);
        started.elapsed()
    }

    fn measure_borrowed_set_membership(
        command_entities: &[EntityId],
        deferred: &BTreeSet<EntityId>,
    ) -> Duration {
        let started = Instant::now();
        let retained = command_entities
            .iter()
            .filter(|entity| deferred.contains(*entity))
            .count();
        assert_eq!(retained, deferred.len());
        black_box(retained);
        started.elapsed()
    }

    fn nearest_rank_percentile(samples: &[Duration], percentile: usize) -> Duration {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn join_duration_samples(samples: &[Duration]) -> String {
        samples
            .iter()
            .map(|sample| sample.as_nanos().to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}
