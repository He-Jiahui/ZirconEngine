use crate::ui::animation_editor::AnimationEditorPanePresentation;

use super::super::pane_payload::{AnimationSequencePanePayload, PanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    PanePayload::AnimationSequenceV1(animation_sequence_payload(context.animation_pane))
}

pub(super) fn animation_sequence_payload(
    animation: Option<&AnimationEditorPanePresentation>,
) -> AnimationSequencePanePayload {
    let Some(animation) = animation else {
        return AnimationSequencePanePayload {
            mode: String::new(),
            asset_path: String::new(),
            status: String::new(),
            selection: String::new(),
            current_frame: 0,
            timeline_start_frame: 0,
            timeline_end_frame: 0,
            playback_label: String::new(),
            track_items: Vec::new(),
        };
    };
    AnimationSequencePanePayload {
        mode: animation.mode.clone(),
        asset_path: animation.asset_path.clone(),
        status: animation.status.clone(),
        selection: animation.selection_summary.clone(),
        current_frame: animation.current_frame,
        timeline_start_frame: animation.timeline_start_frame,
        timeline_end_frame: animation.timeline_end_frame,
        playback_label: animation.playback_label.clone(),
        track_items: animation.track_items.clone(),
    }
}

#[cfg(test)]
#[path = "tests/animation_sequence.rs"]
mod tests;
