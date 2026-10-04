#[test]
fn frame_history_shares_the_wide_validation_key() {
    let history = include_str!("../viewport_frame_history.rs");
    let record = include_str!(
        "../../render_framework/submit_frame_extract/record_submission/record_history.rs"
    );
    let arc_contract = concat!("Arc<", "FrameHistoryValidationKey>");
    let deep_clone = concat!("history_validation_key()", ".clone()");

    assert!(history.contains(arc_contract));
    assert!(record.contains("history_validation_key_shared()"));
    assert!(!record.contains(deep_clone));
}
