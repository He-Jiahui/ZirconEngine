use super::*;
#[test]
fn submitted_text_snapshot_rejects_uncommitted_and_stale_present_counts() {
    let mut snapshot = UiSurfaceTextLayoutSnapshot::default();
    assert!(!snapshot.is_from_present(0));
    assert!(!snapshot.is_from_present(7));
    snapshot.presented_frame_count = 7;
    assert!(snapshot.is_from_present(7));
    assert!(!snapshot.is_from_present(8));
}
#[test]
fn text_line_contract_preserves_physical_fractional_frame_and_fallback_identity() {
    let line = UiSurfaceTextLine {
        original_line_text: "字体".into(),
        line_index: 0,
        byte_range: Some((0, 6)),
        frame: UiSurfaceRect::new(12.75, 21.5, 39.25, 24.0),
        baseline_y: 39.5,
        font_ids: vec!["actual-face-id".into()],
    };
    let json = serde_json::to_value(&line).unwrap();
    assert_eq!(json["frame"]["x"], 12.75);
    assert_eq!(json["frame"]["width"], 39.25);
    assert_eq!(json["baseline_y"], 39.5);
    assert_eq!(json["font_ids"][0], "actual-face-id");
    // Consumers normalize these physical values once with the real window DPI.
    assert_eq!(line.frame.width / 1.5, 39.25 / 1.5);
}
