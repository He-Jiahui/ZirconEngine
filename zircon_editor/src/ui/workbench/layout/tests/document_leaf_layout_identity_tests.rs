use super::*;

#[test]
fn direct_legacy_leaf_parse_has_a_stable_non_nil_id() {
    let legacy = serde_json::json!({"tabs": [], "active_tab": null});
    let first: DocumentLeafLayout = serde_json::from_value(legacy.clone()).unwrap();
    let second: DocumentLeafLayout = serde_json::from_value(legacy).unwrap();
    assert!(!first.node_id.is_nil());
    assert_eq!(first, second);
    assert_eq!(
        first,
        serde_json::from_str::<DocumentLeafLayout>(&serde_json::to_string(&first).unwrap())
            .unwrap()
    );
}
