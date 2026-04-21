//! Animation framework contracts for sequence, graph, state-machine, parameter, and pose evaluation.

mod graph_clip_instance;
mod graph_evaluation;
mod manager;
mod parameter_map;
mod parameter_value;
mod playback_settings;
mod pose_bone;
mod pose_output;
mod pose_source;
mod state_machine_evaluation;
mod track_path;
mod track_path_error;

pub use graph_clip_instance::AnimationGraphClipInstance;
pub use graph_evaluation::AnimationGraphEvaluation;
pub use manager::AnimationManager;
pub use parameter_map::AnimationParameterMap;
pub use parameter_value::AnimationParameterValue;
pub use playback_settings::AnimationPlaybackSettings;
pub use pose_bone::AnimationPoseBone;
pub use pose_output::AnimationPoseOutput;
pub use pose_source::AnimationPoseSource;
pub use state_machine_evaluation::AnimationStateMachineEvaluation;
pub use track_path::AnimationTrackPath;
pub use track_path_error::AnimationTrackPathError;
