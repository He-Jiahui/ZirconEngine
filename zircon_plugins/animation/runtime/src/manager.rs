mod graph;
mod parameters;
mod poison_recovery;
mod pose;
mod sampling;
mod state_machine;

use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::animation::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSkeletonAsset, AnimationStateMachineAsset,
};
use zircon_runtime::core::framework::animation::{
    AnimationGraphEvaluation, AnimationManager, AnimationParameterMap, AnimationParameterValue,
    AnimationPlaybackSettings, AnimationPoseOutput, AnimationResult,
    AnimationStateMachineEvaluation, AnimationTrackPath,
};
use zircon_runtime::core::framework::foundation::ConfigManager;
use zircon_runtime::core::manager::{
    config_manager_handle, resolve_manager_service, ManagerServiceHandle, CONFIG_MANAGER_NAME,
};
use zircon_runtime::core::{CoreError, CoreWeak};

#[derive(Clone, Debug)]
pub struct DefaultAnimationManager {
    // The registry owns this service, so its runtime back-reference must not complete an Arc cycle.
    core: Option<CoreWeak>,
    config_manager: Option<ManagerServiceHandle<dyn ConfigManager>>,
    playback_settings: Arc<Mutex<AnimationPlaybackSettings>>,
}

impl Default for DefaultAnimationManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DefaultAnimationManager {
    pub fn new(core: Option<&CoreWeak>) -> Self {
        let config_manager = core
            .and_then(CoreWeak::upgrade)
            .and_then(|core| config_manager_handle(&core).ok());
        let playback_settings = core
            .zip(config_manager.as_ref())
            .and_then(|(weak_core, handle)| {
                weak_core
                    .upgrade()
                    .and_then(|core| resolve_manager_service(&core, handle.clone()).ok())
            })
            .and_then(|config| {
                config
                    .get_value(crate::ANIMATION_PLAYBACK_CONFIG_KEY)
                    .and_then(|value| serde_json::from_value(value).ok())
            })
            .unwrap_or_default();
        Self {
            core: core.cloned(),
            config_manager,
            playback_settings: Arc::new(Mutex::new(playback_settings)),
        }
    }

    pub fn store_playback_settings(
        &self,
        playback_settings: AnimationPlaybackSettings,
    ) -> Result<(), CoreError> {
        match (
            self.core.as_ref().and_then(CoreWeak::upgrade),
            self.config_manager.clone(),
        ) {
            (Some(core), Some(handle)) => {
                let value = serde_json::to_value(&playback_settings).map_err(|error| {
                    CoreError::ConfigParse(
                        crate::ANIMATION_PLAYBACK_CONFIG_KEY.to_owned(),
                        error.to_string(),
                    )
                })?;
                let config = resolve_manager_service(&core, handle)?;
                config
                    .set_value(crate::ANIMATION_PLAYBACK_CONFIG_KEY, value)
                    .map_err(|error| {
                        CoreError::Initialization(CONFIG_MANAGER_NAME.to_owned(), error.to_string())
                    })?;
            }
            (Some(_), None) => {
                return Err(CoreError::MissingService(CONFIG_MANAGER_NAME.to_owned()));
            }
            (None, _) => {}
        }
        *poison_recovery::lock_recover(&self.playback_settings) = playback_settings;
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
}
