use crate::core::editing::animation_document::{
    AnimationAuthoringDocumentKind, AnimationAuthoringDocumentReadHandle,
};
use zircon_runtime::core::framework::animation::AnimationTrackPath;

mod curve_foundation;
mod error;
mod graph;
mod lifecycle;
mod parameters;
mod presentation;
mod sequence;
mod state_machine;
mod support;
mod timeline_foundation;

pub use curve_foundation::AnimationCurveFoundationView;
pub use error::{AnimationEditorBinaryKindMismatch, AnimationEditorSessionError};
pub use timeline_foundation::AnimationTimelineFoundationView;

const DEFAULT_SEQUENCE_FRAMES_PER_SECOND: f32 = 30.0;

#[derive(Clone, Debug)]
pub(super) struct AnimationSequenceSessionState {
    current_frame: u32,
    timeline_start_frame: u32,
    timeline_end_frame: u32,
    selected_span: Option<(AnimationTrackPath, u32, u32)>,
    playing: bool,
    looping: bool,
    speed: f32,
}

#[derive(Clone, Debug)]
pub struct AnimationEditorSession {
    asset_path: String,
    document: AnimationAuthoringDocumentReadHandle,
    sequence: Option<AnimationSequenceSessionState>,
}

impl AnimationEditorSession {
    pub(crate) fn new(asset_path: String, document: AnimationAuthoringDocumentReadHandle) -> Self {
        let sequence =
            document
                .read()
                .asset()
                .as_sequence()
                .map(|asset| AnimationSequenceSessionState {
                    current_frame: 0,
                    timeline_start_frame: 0,
                    timeline_end_frame: support::duration_frames(
                        asset.duration_seconds,
                        asset.frames_per_second,
                    ),
                    selected_span: None,
                    playing: false,
                    looping: false,
                    speed: 1.0,
                });
        Self {
            asset_path,
            document,
            sequence,
        }
    }

    pub(crate) fn document_kind(&self) -> AnimationAuthoringDocumentKind {
        self.document.kind()
    }

    pub(crate) fn document(&self) -> &AnimationAuthoringDocumentReadHandle {
        &self.document
    }
}

#[cfg(test)]
mod route_loading_tests;
#[cfg(test)]
mod tests;
