use crate::asset::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset,
};
use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;
use crate::scene::World;

use super::{
    AnimationGpuSkinningReadiness, AnimationGraphEvaluation, AnimationParameterMap,
    AnimationParameterValue, AnimationPlaybackSettings, AnimationPoseOutput, AnimationResult,
    AnimationRuntimeStatus, AnimationSequenceApplyReport, AnimationStateMachineEvaluation,
    AnimationTickReport, AnimationTickRequest, AnimationTimelineDescriptor, AnimationTrackPath,
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
    fn sequence_timeline_descriptor(
        &self,
        sequence: &AnimationSequenceAsset,
    ) -> AnimationTimelineDescriptor {
        AnimationTimelineDescriptor::from_sequence(sequence)
    }
    fn clip_timeline_descriptor(&self, clip: &AnimationClipAsset) -> AnimationTimelineDescriptor {
        AnimationTimelineDescriptor::from_clip(clip)
    }
    fn sequence_track_paths(&self, sequence: &AnimationSequenceAsset) -> Vec<AnimationTrackPath> {
        sequence.track_paths()
    }
    fn apply_sequence_to_world(
        &self,
        _world: &mut World,
        sequence: &AnimationSequenceAsset,
        _time_seconds: Real,
        _looping: bool,
    ) -> AnimationResult<AnimationSequenceApplyReport> {
        Ok(AnimationSequenceApplyReport {
            applied_tracks: Vec::new(),
            missing_tracks: self.sequence_track_paths(sequence),
        })
    }
}
