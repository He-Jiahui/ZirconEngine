#[test]
fn window_metrics_keeps_its_own_legacy_dirty_domain() {
    let source = include_str!("../dirty_flags.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("dirty flag production source");
    let layout_assignments = production
        .match_indices("self.layout_dirty = true")
        .map(|(index, _)| &production[index.saturating_sub(180)..index])
        .collect::<Vec<_>>();

    assert_eq!(layout_assignments.len(), 2);
    assert!(layout_assignments.iter().all(|context| {
        context.contains("HostInvalidationMask::LAYOUT")
            && context.contains("HostInvalidationMask::TREE_STRUCTURE")
            && !context.contains("requires_layout()")
    }));
}
