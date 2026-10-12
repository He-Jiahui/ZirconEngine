#[test]
fn reflected_inspector_updates_borrow_the_draft_map() {
    let source = include_str!("../editor_state_selection.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    assert!(!implementation.contains("self.inspector_dynamic_fields.clone()"));
    assert!(implementation.contains("let dynamic_fields = &self.inspector_dynamic_fields"));
}

#[test]
fn inspector_command_buffer_reserves_builtin_update_slots() {
    let source = include_str!("../editor_state_selection.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    assert!(implementation.contains("let mut commands = Vec::new();"));
    assert!(implementation.contains("commands.reserve(selected.len().saturating_mul(4));"));
    assert!(implementation.contains("if commands.is_empty()"));
}
