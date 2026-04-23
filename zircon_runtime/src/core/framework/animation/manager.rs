use crate::asset::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset,
};
use crate::core::math::Real;

use super::{
    AnimationGraphEvaluation, AnimationParameterMap, AnimationParameterValue,
    AnimationPlaybackSettings, AnimationPoseOutput, AnimationStateMachineEvaluation,
    AnimationTrackPath,
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
    ) -> Result<AnimationPoseOutput, String>;
    fn sequence_track_paths(&self, sequence: &AnimationSequenceAsset) -> Vec<AnimationTrackPath> {
        sequence.track_paths()
    }
}
