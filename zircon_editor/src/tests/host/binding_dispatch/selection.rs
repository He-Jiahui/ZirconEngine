use super::support;
use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::SelectionHostEvent;
use crate::ui::binding::{
    EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind, SelectionCommand,
};
use crate::ui::binding_dispatch::{apply_selection_binding, dispatch_selection_binding};

#[test]
fn selection_binding_dispatches_and_applies_scene_node_selection() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    let binding = EditorUiBinding::new(
        "HierarchyView",
        "SceneNodeSelect",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode {
            node_id: cube,
        }),
    );

    assert_eq!(
        dispatch_selection_binding(&binding).unwrap(),
        SelectionHostEvent::SelectSceneNode { node_id: cube }
    );
    assert!(apply_selection_binding(&mut state, &binding).unwrap());
    assert_eq!(
        state.viewport_controller.selection().active_primary(),
        Some(cube)
    );
}
