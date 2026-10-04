use super::*;
use crate::ui::workbench::layout::{DocumentNodeId, SplitAxis, TabStackLayout};

fn leaf(tabs: &[&str], active: Option<&str>) -> DocumentNode {
    DocumentNode::tabs(TabStackLayout {
        tabs: tabs.iter().map(|id| ViewInstanceId::new(*id)).collect(),
        active_tab: active.map(ViewInstanceId::new),
    })
}

#[test]
fn scene_document_is_captured_independently_from_absent_or_drawer_keyboard_focus() {
    let node = leaf(&["scene", "game"], Some("scene"));
    assert_eq!(
        previous_in_game_leaf(&node, &ViewInstanceId::new("game"), &[]),
        Some(ViewInstanceId::new("scene"))
    );
}

#[test]
fn already_active_game_restores_only_an_existing_scene_in_its_own_leaf() {
    let node = leaf(&["scene", "game"], Some("game"));
    assert_eq!(
        previous_in_game_leaf(
            &node,
            &ViewInstanceId::new("game"),
            &[ViewInstanceId::new("scene")]
        ),
        Some(ViewInstanceId::new("scene"))
    );
    assert_eq!(
        previous_in_game_leaf(
            &node,
            &ViewInstanceId::new("game"),
            &[ViewInstanceId::new("foreign-scene")]
        ),
        None
    );
}

#[test]
fn split_document_selection_does_not_pick_another_leaf_scene() {
    let node = DocumentNode::SplitNode {
        node_id: DocumentNodeId::default(),
        axis: SplitAxis::Horizontal,
        ratio: 0.5,
        first: Box::new(leaf(&["foreign-scene"], Some("foreign-scene"))),
        second: Box::new(leaf(&["scene", "game"], Some("scene"))),
    };
    assert_eq!(
        previous_in_game_leaf(
            &node,
            &ViewInstanceId::new("game"),
            &[
                ViewInstanceId::new("foreign-scene"),
                ViewInstanceId::new("scene")
            ]
        ),
        Some(ViewInstanceId::new("scene"))
    );
}
