use super::ComponentEntityIds;

#[test]
fn component_entity_ids_serialize_in_row_order_without_values() {
    let first = serde_json::json!({"ignored": 1});
    let second = serde_json::json!({"ignored": 2});
    let rows = vec![(7, &first), (11, &second)];

    assert_eq!(
        serde_json::to_string(&ComponentEntityIds(&rows)).unwrap(),
        r#"[7,11]"#
    );
}

#[test]
fn entity_exists_uses_the_world_entity_index() {
    let source = include_str!("../components.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;
    let function = source.split("pub(super) fn entity_exists").nth(1).unwrap();
    let function = function.split("pub(super) fn").next().unwrap();

    assert!(function.contains("world.contains_entity(entity)"));
    assert!(!function.contains("node_records()"));
}
