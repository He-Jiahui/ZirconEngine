#[test]
fn modal_restore_splice_streams_remaining_entries() {
    let source = include_str!("../modal_scope.rs");
    let (_, restore_path) = source
        .split_once("    fn take_modal_restore_state(")
        .expect("modal restore helper must remain present");
    let (restore_path, _) = restore_path
        .split_once("    fn resolve_restore_target(")
        .expect("modal restore helper must end before target resolution");

    assert!(restore_path.contains("let state = self.focus.modal_restore_stack.remove(index);"));
    assert!(restore_path.contains("for dependent_index in 0..self.focus.modal_restore_stack.len()"));
    assert!(!restore_path.contains("dependent_indices"));
    assert!(!restore_path.contains(".collect::<Vec<_>>()"));
}
