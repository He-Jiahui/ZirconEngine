const SOURCE: &str = include_str!("../construct.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("overlay construction should retain a test-module boundary")
}

#[test]
fn interaction_only_pipelines_are_constructed_lazily() {
    let source = production_source();
    let interaction_gate = source
        .find("interaction_overlays_enabled.then(|| {")
        .expect("interaction overlay resources should be conditionally constructed");

    for constructor in [
        "create_line_pipeline(device, final_color_format, scene_layout)",
        "create_grid_buffer(device)",
        "SceneGizmoPass::new(",
    ] {
        let position = source
            .find(constructor)
            .unwrap_or_else(|| panic!("missing interaction constructor `{constructor}`"));
        assert!(
            interaction_gate < position,
            "{constructor} must remain behind the interaction-overlay gate"
        );
    }
}
