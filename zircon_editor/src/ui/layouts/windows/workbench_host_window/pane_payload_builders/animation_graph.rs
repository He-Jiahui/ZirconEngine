use crate::ui::animation_editor::AnimationEditorPanePresentation;

use super::super::pane_payload::{AnimationGraphPanePayload, PanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    let animation = context
        .animation_pane
        .cloned()
        .unwrap_or_else(AnimationEditorPanePresentation::default);
    PanePayload::AnimationGraphV1(AnimationGraphPanePayload {
        mode: animation.mode,
        asset_path: animation.asset_path,
        status: animation.status,
        selection: animation.selection_summary,
        parameter_items: animation.parameter_items,
        node_items: animation.node_items,
        state_items: animation.state_items,
        transition_items: animation.transition_items,
    })
}
