mod graph;
mod parameters;
mod pose;
mod sampling;
mod state_machine;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::framework::animation::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSkeletonAsset, AnimationStateMachineAsset,
};
use crate::core::framework::animation::{
    AnimationGraphEvaluation, AnimationIkCommand, AnimationIkCommandError, AnimationManager,
    AnimationParameterMap, AnimationParameterValue, AnimationPlaybackSettings, AnimationPoseOutput,
    AnimationResult, AnimationStateMachineEvaluation, AnimationTrackPath,
};
use crate::core::framework::scene::WorldHandle;
use crate::core::{CoreError, CoreHandle, CoreWeak};

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
    pub fn new(core: Option<&CoreHandle>) -> Self {
        let playback_settings = core
            .and_then(|core| {
                core.load_config(crate::animation::ANIMATION_PLAYBACK_CONFIG_KEY)
                    .ok()
            })
            .unwrap_or_default();
        Self {
            core: core.map(CoreHandle::downgrade),
            playback_settings: Arc::new(Mutex::new(playback_settings)),
            ik_commands: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn store_playback_settings(
        &self,
        playback_settings: AnimationPlaybackSettings,
    ) -> Result<(), CoreError> {
        *self.lock_playback_settings() = playback_settings.clone();
        if let Some(core) = self.core.as_ref().and_then(CoreWeak::upgrade) {
            core.store_config(
                crate::animation::ANIMATION_PLAYBACK_CONFIG_KEY,
                &playback_settings,
            )?;
        }
        Ok(())
    }

    fn lock_playback_settings(&self) -> MutexGuard<'_, AnimationPlaybackSettings> {
        self.playback_settings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_ik_commands(&self) -> MutexGuard<'_, HashMap<WorldHandle, WorldIkCommandQueue>> {
        self.ik_commands
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl AnimationManager for DefaultAnimationManager {
    fn playback_settings(&self) -> AnimationPlaybackSettings {
        self.lock_playback_settings().clone()
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
        time_seconds: crate::core::math::Real,
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
        let mut queues = self.lock_ik_commands();
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
        self.drain_ik_commands_excluding(world, replacement_epoch, &[])
    }

    fn drain_ik_commands_excluding(
        &self,
        world: WorldHandle,
        replacement_epoch: u64,
        deferred_entities: &[crate::scene::EntityId],
    ) -> Vec<AnimationIkCommand> {
        let mut queues = self.lock_ik_commands();
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
    use std::panic::{self, AssertUnwindSafe};

    use crate::core::framework::animation::{
        AnimationIkCommand, AnimationIkCommandError, AnimationLookAtCommand, AnimationManager,
        AnimationPlaybackSettings, AnimationTargetId,
    };
    use crate::core::framework::scene::WorldHandle;
    use crate::core::math::Vec3;

    use super::DefaultAnimationManager;

    #[test]
    fn animation_manager_playback_settings_recover_poisoned_lock() {
        let manager = DefaultAnimationManager::default();
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.lock_playback_settings();
            panic!("poison animation playback settings");
        }));

        let mut playback_settings = AnimationPlaybackSettings::default();
        playback_settings.enabled = false;
        manager
            .store_playback_settings(playback_settings.clone())
            .expect("store playback settings after poisoned lock");

        assert_eq!(manager.playback_settings(), playback_settings);
    }

    #[test]
    fn replacement_epoch_retires_deferred_ik_commands_and_rejects_late_old_epoch() {
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

        let manager = DefaultAnimationManager::default();
        let world = WorldHandle::new(7);
        manager
            .queue_ik_command(1, look_at(world, 17))
            .expect("old World command queues");
        assert!(manager
            .drain_ik_commands_excluding(world, 1, &[17])
            .is_empty());

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
}
