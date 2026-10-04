const SOURCE: &str = include_str!("../record_overlays.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("overlay recording should retain a test-module boundary")
}

#[test]
fn disabled_interaction_overlays_skip_render_pass_recording() {
    let source = production_source();
    let interaction_guard = source
        .find("let Some(interaction_overlays) = self.interaction_overlays.as_mut() else {")
        .expect("overlay recording should exit when interaction resources are absent");
    let selection_record = source
        .find("interaction_overlays.selection_outline.record(")
        .expect("interactive overlays should still record selection outlines");

    assert!(
        interaction_guard < selection_record,
        "the EnvironmentOnly path must exit before recording overlay passes"
    );
}
