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
mod tests {
    use super::*;

    #[test]
    fn optimization_batch_du_selective_animation_graph_projection_preserves_fields() {
        let presentation = AnimationEditorPanePresentation {
            mode: "graph".to_owned(),
            asset_path: "project://animations/locomotion.graph".to_owned(),
            status: "ready".to_owned(),
            selection_summary: "Locomotion".to_owned(),
            parameter_items: vec!["speed".to_owned()],
            node_items: vec!["idle".to_owned(), "run".to_owned()],
            state_items: vec!["grounded".to_owned()],
            transition_items: vec!["idle -> run".to_owned()],
            ..AnimationEditorPanePresentation::default()
        };

        let payload = animation_graph_payload(Some(&presentation));

        assert_eq!(payload.mode, presentation.mode);
        assert_eq!(payload.asset_path, presentation.asset_path);
        assert_eq!(payload.status, presentation.status);
        assert_eq!(payload.selection, presentation.selection_summary);
        assert_eq!(payload.parameter_items, presentation.parameter_items);
        assert_eq!(payload.node_items, presentation.node_items);
        assert_eq!(payload.state_items, presentation.state_items);
        assert_eq!(payload.transition_items, presentation.transition_items);
    }

    #[test]
    fn optimization_batch_du_selective_animation_graph_projection_defaults_when_absent() {
        let payload = animation_graph_payload(None);

        assert!(payload.mode.is_empty());
        assert!(payload.asset_path.is_empty());
        assert!(payload.status.is_empty());
        assert!(payload.selection.is_empty());
        assert!(payload.parameter_items.is_empty());
        assert!(payload.node_items.is_empty());
        assert!(payload.state_items.is_empty());
        assert!(payload.transition_items.is_empty());
    }
}
