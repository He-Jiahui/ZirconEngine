#[test]
fn direct_import_terminal_never_encodes_a_physical_writeback() {
    let source = include_str!("../output_target.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("output target executor test boundary");
    let direct = source
        .split_once(
            "pub(in crate::graphics::scene::scene_renderer) fn record_output_target_writeback",
        )
        .map(|(direct, _)| direct)
        .expect("direct import must precede writeback implementation");

    assert!(direct.contains("record_output_target_direct_import("));
    assert!(direct.contains("self.output_target_writeback_report = Some(plan);"));
    assert!(!direct.contains("encode_planned_output_target_writeback("));
    assert!(!direct.contains("copy_texture_to_texture("));
}
