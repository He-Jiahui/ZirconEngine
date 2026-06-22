#[cfg(test)]
pub(super) use zircon_runtime::asset::assets::AnimationConditionOperatorAsset;
use zircon_runtime::asset::assets::{
    AnimationGraphAsset, AnimationSequenceAsset, AnimationStateMachineAsset,
};
use zircon_runtime::core::framework::animation::AnimationTrackPath;

mod graph;
mod lifecycle;
mod parameters;
mod presentation;
mod sequence;
mod state_machine;
mod support;

const DEFAULT_SEQUENCE_FRAMES_PER_SECOND: f32 = 30.0;
const DEFAULT_STATE_MACHINE_TRANSITION_FPS: f32 = 30.0;

#[derive(Clone, Debug)]
pub struct AnimationEditorSessionError(pub String);

impl std::fmt::Display for AnimationEditorSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AnimationEditorSessionError {}

#[derive(Clone, Debug)]
struct AnimationSequenceDocument {
    asset: AnimationSequenceAsset,
    current_frame: u32,
    timeline_start_frame: u32,
    timeline_end_frame: u32,
    selected_span: Option<(AnimationTrackPath, u32, u32)>,
    playing: bool,
    looping: bool,
    speed: f32,
}

#[derive(Clone, Debug)]
enum AnimationEditorDocument {
    Sequence(AnimationSequenceDocument),
    Graph(AnimationGraphAsset),
    StateMachine(AnimationStateMachineAsset),
}

#[derive(Clone, Debug)]
pub struct AnimationEditorSession {
    asset_path: String,
    document: AnimationEditorDocument,
    dirty: bool,
}

#[cfg(test)]
mod tests;
