#[test]
#[ignore = "writes native Editor visual evidence; run through the managed Windows validator"]
fn export_editor_zui_visual_evidence() {
    let summary = capture_catalog().unwrap_or_else(|error| panic!("{error}"));
    assert!(
        summary.is_ready(),
        "Editor captures remain unready ({} failed, {} pending); inspect {}",
        summary.failed,
        summary.pending,
        summary.report_path.display()
    );
}
