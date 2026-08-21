use std::collections::BTreeSet;

use crate::core::framework::scene::{EntityId, WorldHandle};
use crate::core::math::Real;

use super::{
    AnimationClipAsset, AnimationGpuSkinningReadiness, AnimationGraphAsset,
    AnimationGraphEvaluation, AnimationIkCommand, AnimationIkCommandError, AnimationParameterMap,
    AnimationParameterValue, AnimationPlaybackSettings, AnimationPoseOutput, AnimationResult,
    AnimationRuntimeStatus, AnimationSkeletonAsset, AnimationStateMachineAsset,
    AnimationStateMachineEvaluation, AnimationTickReport, AnimationTickRequest, AnimationTrackPath,
};

pub trait AnimationManager: Send + Sync {
    fn playback_settings(&self) -> AnimationPlaybackSettings;
    fn normalize_track_path(&self, path: &AnimationTrackPath) -> AnimationTrackPath;
    fn parameter_defaults(&self, graph: &AnimationGraphAsset) -> AnimationParameterMap;
    fn parameter_value(
        &self,
        parameters: &AnimationParameterMap,
        name: &str,
    ) -> Option<AnimationParameterValue>;
    fn set_parameter(
        &self,
        parameters: &mut AnimationParameterMap,
        name: &str,
        value: AnimationParameterValue,
    );
    fn evaluate_graph(
        &self,
        graph: &AnimationGraphAsset,
        parameters: &AnimationParameterMap,
    ) -> AnimationGraphEvaluation;
    fn evaluate_state_machine(
        &self,
        state_machine: &AnimationStateMachineAsset,
        current_state: Option<&str>,
        parameters: &AnimationParameterMap,
    ) -> AnimationStateMachineEvaluation;
    fn sample_clip_pose(
        &self,
        skeleton: &AnimationSkeletonAsset,
        clip: &AnimationClipAsset,
        time_seconds: Real,
        looping: bool,
    ) -> AnimationResult<AnimationPoseOutput>;
    fn tick_world_contract(&self, request: AnimationTickRequest) -> AnimationTickReport {
        AnimationTickReport::new(request.world)
    }
    fn runtime_status(&self, world: WorldHandle) -> AnimationRuntimeStatus {
        AnimationRuntimeStatus::new(world)
    }
    fn gpu_skinning_readiness(&self) -> AnimationGpuSkinningReadiness {
        AnimationGpuSkinningReadiness::default()
    }
    fn queue_ik_command(
        &self,
        _replacement_epoch: u64,
        _command: AnimationIkCommand,
    ) -> Result<(), AnimationIkCommandError> {
        Err(AnimationIkCommandError::Unsupported)
    }
    fn drain_ik_commands(
        &self,
        _world: WorldHandle,
        _replacement_epoch: u64,
    ) -> Vec<AnimationIkCommand> {
        Vec::new()
    }
    fn drain_ik_commands_excluding(
        &self,
        world: WorldHandle,
        replacement_epoch: u64,
        deferred_entities: &BTreeSet<EntityId>,
    ) -> Vec<AnimationIkCommand>;
}
