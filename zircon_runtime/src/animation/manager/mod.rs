mod graph;
mod parameters;
mod pose;
mod sampling;
mod state_machine;

use std::sync::{Arc, Mutex, MutexGuard};

use crate::asset::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSkeletonAsset, AnimationStateMachineAsset,
};
use crate::core::framework::animation::{
    AnimationGraphEvaluation, AnimationManager, AnimationParameterMap, AnimationParameterValue,
    AnimationPlaybackSettings, AnimationPoseOutput, AnimationResult, AnimationSequenceApplyReport,
    AnimationStateMachineEvaluation, AnimationTrackPath,
};
use crate::core::{CoreError, CoreHandle};

#[derive(Clone, Debug)]
pub struct DefaultAnimationManager {
    core: Option<CoreHandle>,
    playback_settings: Arc<Mutex<AnimationPlaybackSettings>>,
}

impl Default for DefaultAnimationManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DefaultAnimationManager {
    pub fn new(core: Option<CoreHandle>) -> Self {
        let playback_settings = core
            .as_ref()
            .and_then(|core| {
                core.load_config(crate::animation::ANIMATION_PLAYBACK_CONFIG_KEY)
                    .ok()
            })
            .unwrap_or_default();
        Self {
            core,
            playback_settings: Arc::new(Mutex::new(playback_settings)),
        }
    }

    pub fn store_playback_settings(
        &self,
        playback_settings: AnimationPlaybackSettings,
    ) -> Result<(), CoreError> {
        *self.lock_playback_settings() = playback_settings.clone();
        if let Some(core) = &self.core {
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

    fn apply_sequence_to_world(
        &self,
        world: &mut crate::scene::World,
        sequence: &crate::asset::AnimationSequenceAsset,
        time_seconds: crate::core::math::Real,
        looping: bool,
    ) -> AnimationResult<AnimationSequenceApplyReport> {
        crate::animation::sequence::apply_sequence_to_world(world, sequence, time_seconds, looping)
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
