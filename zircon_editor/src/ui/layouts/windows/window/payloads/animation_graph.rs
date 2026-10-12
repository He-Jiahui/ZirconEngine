use crate::ui::animation_editor::AnimationEditorPanePresentation;

use super::super::pane_payload::{AnimationGraphPanePayload, PanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    PanePayload::AnimationGraphV1(animation_graph_payload(context.animation_pane))
}

pub(super) fn animation_graph_payload(
    animation: Option<&AnimationEditorPanePresentation>,
) -> AnimationGraphPanePayload {
    let Some(animation) = animation else {
        return AnimationGraphPanePayload {
            mode: String::new(),
            asset_path: String::new(),
            status: String::new(),
            selection: String::new(),
            parameter_items: Vec::new(),
            node_items: Vec::new(),
            state_items: Vec::new(),
            transition_items: Vec::new(),
        };
    };
    AnimationGraphPanePayload {
        mode: animation.mode.clone(),
        asset_path: animation.asset_path.clone(),
        status: animation.status.clone(),
        selection: animation.selection_summary.clone(),
        parameter_items: animation.parameter_items.clone(),
        node_items: animation.node_items.clone(),
        state_items: animation.state_items.clone(),
        transition_items: animation.transition_items.clone(),
    }
}

#[cfg(test)]
#[path = "tests/animation_graph.rs"]
mod tests;
