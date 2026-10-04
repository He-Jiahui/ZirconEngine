#[test]
fn abort_parent_lookup_uses_the_compiled_parent_index() {
    let source = include_str!("../abort.rs");
    let parent_of = source
        .split("fn parent_of(")
        .nth(1)
        .and_then(|body| body.split("\n}").next())
        .expect("parent_of body");

    assert!(parent_of.contains("tree.parent_index(node_index)"));
    assert!(!parent_of.contains("tree.nodes().iter()"));
    assert!(!parent_of.contains(".contains(&node_index)"));
}
