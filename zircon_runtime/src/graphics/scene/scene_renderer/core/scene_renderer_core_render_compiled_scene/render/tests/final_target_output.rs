#[test]
fn final_target_selection_consumes_the_prepared_frame_plan_without_replanning() {
    let source = include_str!("../final_target_output.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("final target selection test boundary");

    assert!(source.contains("streamer.output_target_frame_plan()"));
    assert!(!source.contains(".graph_import_plan("));
    assert!(!source.contains("frame.texture_writeback_plan("));
}
