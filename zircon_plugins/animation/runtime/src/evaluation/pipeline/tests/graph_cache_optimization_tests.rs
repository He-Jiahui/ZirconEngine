#[test]
fn optimization_batch_20260830cg_graph_cache_checks_resident_snapshots_first() {
    let source = include_str!("../graph_cache.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    for (start, end, loader) in [
        (
            "fn load_graph_snapshot(",
            "fn load_skeleton_snapshot(",
            "load_animation_graph_asset",
        ),
        (
            "fn load_skeleton_snapshot(",
            "#[cfg(test)]",
            "load_animation_skeleton_asset",
        ),
    ] {
        let start = source.find(start).expect("snapshot helper");
        let helper = production.get(start..).unwrap_or(&source[start..]);
        let helper = helper.split(end).next().expect("snapshot helper boundary");
        assert!(
            helper.find("resources.snapshot").expect("resident lookup")
                < helper.find(loader).expect("loader fallback")
        );
        assert!(helper.contains(".or_else(||"));
    }
}
