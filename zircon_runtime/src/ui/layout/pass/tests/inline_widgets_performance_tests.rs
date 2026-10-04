#[test]
fn inline_widget_arrangement_reserves_known_collection_sizes() {
    let source = include_str!("../inline_widgets.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("inline widget production source");

    assert!(
        production.contains("HashSet::with_capacity(parent.children.len())"),
        "direct-child membership must reserve the known child count"
    );
    assert!(
        production.contains("HashSet::with_capacity(resolved.bindings().len())"),
        "managed-child membership must reserve the known binding count"
    );
    assert!(
        production.contains("Vec::with_capacity(roots.len())"),
        "preorder scratch must reserve the known root count"
    );
}
