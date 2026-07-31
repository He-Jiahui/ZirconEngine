use crate::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    focus::{
        focus_chain, UiFocusCause, UiFocusContract, UiFocusMode, UiFocusVisible,
        UiFocusVisibleReason,
    },
    navigation::{UiNavigationBoundary, UiTabIndex},
    tree::{UiTree, UiTreeNode},
};

fn focusable_node(node_id: u64, mode: UiFocusMode, tab_index: Option<UiTabIndex>) -> UiTreeNode {
    let mut node = UiTreeNode::new(
        UiNodeId::new(node_id),
        UiNodePath::new(format!("root/{node_id}")),
    );
    node.focus = UiFocusContract {
        focusable: true,
        mode,
        ..UiFocusContract::default()
    };
    node.navigation.tab_index = tab_index;
    node
}

#[test]
fn focus_chain_orders_explicit_tab_indices_before_preorder_defaults() {
    let mut tree = UiTree::new(UiTreeId::new("ui.focus-chain"));
    let root = focusable_node(1, UiFocusMode::All, None);
    let click_only = focusable_node(2, UiFocusMode::Click, None);
    let later_tab = focusable_node(3, UiFocusMode::All, Some(UiTabIndex::new(20)));
    let earlier_tab = focusable_node(4, UiFocusMode::All, Some(UiTabIndex::new(10)));
    let excluded = focusable_node(5, UiFocusMode::None, Some(UiTabIndex::new(1)));

    tree.insert_root(root);
    tree.insert_child(UiNodeId::new(1), click_only)
        .expect("click-only node attaches");
    tree.insert_child(UiNodeId::new(1), later_tab)
        .expect("later tab node attaches");
    tree.insert_child(UiNodeId::new(1), earlier_tab)
        .expect("earlier tab node attaches");
    tree.insert_child(UiNodeId::new(1), excluded)
        .expect("excluded node attaches");

    assert_eq!(
        focus_chain(&tree),
        vec![UiNodeId::new(4), UiNodeId::new(3), UiNodeId::new(1)]
    );
}

#[test]
fn focus_causes_preserve_focus_visible_intent() {
    assert_eq!(
        UiFocusCause::Navigation.focus_visible(),
        UiFocusVisible::visible(UiFocusVisibleReason::KeyboardNavigation)
    );
    assert_eq!(
        UiFocusCause::Pointer.focus_visible(),
        UiFocusVisible::hidden(UiFocusVisibleReason::PointerInteraction)
    );
}

#[test]
fn navigation_contract_defaults_to_escape_and_round_trips_trap_boundary() {
    let navigation: crate::ui::navigation::UiNavigationContract =
        serde_json::from_str("{}").expect("legacy navigation contract deserializes");
    assert_eq!(navigation.boundary, UiNavigationBoundary::Escape);

    let legacy_tab: UiTabIndex =
        serde_json::from_str(r#"{ "order": 7 }"#).expect("legacy tab index deserializes");
    assert_eq!(legacy_tab.order, 7);
    assert!(!legacy_tab.tabbable);

    let serialized =
        serde_json::to_value(UiNavigationBoundary::Trap).expect("navigation boundary serializes");
    assert_eq!(serialized, serde_json::json!({ "kind": "trap" }));
    let deserialized: UiNavigationBoundary =
        serde_json::from_value(serialized).expect("navigation boundary deserializes");
    assert_eq!(deserialized, UiNavigationBoundary::Trap);
}
