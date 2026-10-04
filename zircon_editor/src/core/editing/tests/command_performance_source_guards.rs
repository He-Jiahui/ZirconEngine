#[test]
fn create_node_redo_clones_the_retained_record_only_once() {
    let source = include_str!("../command.rs");
    let second_clone = ["insert_node_record", "(record.clone())"].concat();

    assert!(!source.contains(&second_clone));
}

#[test]
fn delete_node_undo_keeps_only_the_move_only_runtime_inverse_delta() {
    let source = include_str!("../command.rs");
    let start = source
        .find("pub(crate) struct DeleteNodeCommand")
        .expect("delete command declaration should remain available");
    let end = source[start..]
        .find("pub(crate) struct NodeEditState")
        .map(|offset| start + offset)
        .expect("delete command region should end before node edit state");
    let delete_command = &source[start..end];

    assert!(delete_command.contains("batch: Option<DetachedEntityBatch>"));
    assert!(!delete_command.contains("records: Vec<NodeRecord>"));
    assert!(!delete_command.contains("subtree_records("));
    assert!(!delete_command.contains("insert_node_records("));
    assert!(!delete_command.contains(".expect("));
}
