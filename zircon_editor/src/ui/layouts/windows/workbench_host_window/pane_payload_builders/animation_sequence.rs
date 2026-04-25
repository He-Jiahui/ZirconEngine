use crate::ui::animation_editor::AnimationEditorPanePresentation;

use super::super::pane_payload::{AnimationSequencePanePayload, PanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    let animation = context
        .animation_pane
        .cloned()
        .unwrap_or_else(AnimationEditorPanePresentation::default);
    PanePayload::AnimationSequenceV1(AnimationSequencePanePayload {
        mode: animation.mode,
        asset_path: animation.asset_path,
        status: animation.status,
        selection: animation.selection_summary,
        current_frame: animation.current_frame,
        timeline_start_frame: animation.timeline_start_frame,
        timeline_end_frame: animation.timeline_end_frame,
        playback_label: animation.playback_label,
        track_items: animation.track_items,
    })
}
