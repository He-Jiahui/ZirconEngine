#[test]
fn frame_summary_builds_diagnostic_names_without_temporary_vec() {
    let source = include_str!("../frame_phase_queue_summary.rs");

    assert!(!source.contains(concat!(".collect::<Vec<_>>()", ".join(\"+\")")));
    assert!(source.contains(concat!("String::with_", "capacity(capacity)")));
}
