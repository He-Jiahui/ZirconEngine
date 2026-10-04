use super::{mark_panel_selected, mark_text_state, mark_toggle_state};
use crate::ui::layouts::views::ViewTemplateNodeData;

#[test]
fn visual_selection_state_does_not_impersonate_keyboard_focus() {
    let mut nodes = vec![node("toggle"), node("panel"), node("label")];

    mark_toggle_state(&mut nodes, "toggle", true);
    mark_panel_selected(&mut nodes, "panel", true);
    mark_text_state(&mut nodes, &["label"], true);

    assert!(nodes.iter().all(|node| node.selected));
    assert!(nodes.iter().all(|node| !node.focused));
}

fn node(control_id: &str) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        control_id: control_id.into(),
        ..ViewTemplateNodeData::default()
    }
}
