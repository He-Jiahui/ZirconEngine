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

#[derive(Clone, Debug)]
pub struct DefaultAnimationManager {
    // The registry owns this service, so its runtime back-reference must not complete an Arc cycle.
    core: Option<CoreWeak>,
    playback_settings: Arc<Mutex<AnimationPlaybackSettings>>,
    ik_commands: Arc<Mutex<HashMap<WorldHandle, Vec<AnimationIkCommand>>>>,
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

    fn lock_ik_commands(&self) -> MutexGuard<'_, HashMap<WorldHandle, Vec<AnimationIkCommand>>> {
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

    fn queue_ik_command(&self, command: AnimationIkCommand) -> Result<(), AnimationIkCommandError> {
        command.validate()?;
        let world = command.world();
        let mut queues = self.lock_ik_commands();
        let queue = queues.entry(world).or_default();
        if queue.len() >= MAX_PENDING_IK_COMMANDS_PER_WORLD {
            return Err(AnimationIkCommandError::QueueFull {
                world,
                capacity: MAX_PENDING_IK_COMMANDS_PER_WORLD,
            });
        }
        queue.push(command);
        Ok(())
    }

    fn drain_ik_commands(&self, world: WorldHandle) -> Vec<AnimationIkCommand> {
        self.lock_ik_commands().remove(&world).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};

    use crate::core::framework::animation::{AnimationManager, AnimationPlaybackSettings};

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
}
