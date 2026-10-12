#[test]
fn hierarchy_pointer_reuses_the_committed_scene_projection() {
    let source = include_str!("../target.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);

    assert!(!production.contains("self.runtime.editor_snapshot()"));
    assert!(production.contains("if self.hierarchy_pointer_size != target_size"));
}
