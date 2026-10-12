#[test]
fn capture_reuses_node_record_order_without_resorting_entities() {
    let source = include_str!("../capture.rs");
    let capture = source
        .split("pub(super) fn dynamic_scene_from_world")
        .nth(1)
        .and_then(|source| source.split("fn dynamic_entity_from_node").next())
        .expect("read dynamic scene capture body");

    assert!(capture.contains(".node_records()"));
    assert!(
        !capture.contains("entities.sort_by_key"),
        "World::node_records already publishes entity-id order; capture must not sort it again"
    );
}
