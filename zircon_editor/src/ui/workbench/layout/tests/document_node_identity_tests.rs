use super::*;

#[test]
fn legacy_tree_migrates_ids_once_and_reopens_exactly() {
    let legacy = serde_json::json!({
        "SplitNode": {
            "axis": "Horizontal",
            "ratio": 0.63,
            "first": {"Tabs": {"tabs": [], "active_tab": null}},
            "second": {"Tabs": {"tabs": [], "active_tab": null}}
        }
    });
    let migrated: DocumentNode =
        serde_json::from_value(legacy.clone()).expect("legacy tree migrates");
    assert_eq!(migrated, serde_json::from_value(legacy).unwrap());
    let DocumentNode::SplitNode { first, second, .. } = &migrated else {
        unreachable!()
    };
    assert!(!migrated.node_id().is_nil());
    assert!(!first.node_id().is_nil());
    assert!(!second.node_id().is_nil());
    assert_ne!(migrated.node_id(), first.node_id());
    assert_ne!(migrated.node_id(), second.node_id());
    assert_ne!(first.node_id(), second.node_id());
    let saved = serde_json::to_string(&migrated).unwrap();
    let reopened: DocumentNode = serde_json::from_str(&saved).unwrap();
    assert_eq!(migrated, reopened);
    assert!(!reopened.node_id().is_nil());
}
