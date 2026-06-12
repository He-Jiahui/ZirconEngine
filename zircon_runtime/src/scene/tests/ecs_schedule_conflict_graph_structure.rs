fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|text| text.split(end).next())
        .unwrap_or_else(|| panic!("read section from {start} to {end}"))
}

#[test]
fn schedule_conflict_graph_from_nodes_uses_size_hint_staging() {
    let source = include_str!("../ecs/schedule_conflict_graph.rs");
    let from_nodes = section_between(
        source,
        "pub fn from_nodes(nodes: impl IntoIterator<Item = ScheduleConflictNode>) -> Self",
        "pub(crate) fn from_node_vec",
    );

    assert!(
        from_nodes.contains("let node_iter = nodes.into_iter();")
            && from_nodes.contains("let (lower_bound, _) = node_iter.size_hint();")
            && from_nodes.contains("let mut nodes = Vec::with_capacity(lower_bound);")
            && from_nodes.contains("for node in node_iter")
            && from_nodes.contains("nodes.push(node);")
            && from_nodes.contains("Self::from_node_vec(nodes)")
            && !from_nodes.contains("collect::<Vec<_>>()"),
        "schedule conflict graph construction must stage iterator input through a pre-sized direct-push vector"
    );
}
