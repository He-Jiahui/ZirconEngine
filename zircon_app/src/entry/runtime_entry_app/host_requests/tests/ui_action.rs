use super::should_report_count;

#[test]
fn unhandled_action_diagnostics_are_logarithmically_bounded() {
    assert!(should_report_count(1));
    assert!(should_report_count(2));
    assert!(!should_report_count(3));
    assert!(should_report_count(4));
    assert!(!should_report_count(usize::MAX as u64));
}

#[test]
fn diagnostic_source_never_formats_action_payload_or_secure_reference() {
    let source = include_str!("../ui_action.rs");

    assert!(!source.contains(concat!("request.invocation.", "payload")));
    assert!(!source.contains(concat!("request.", "secure_value")));
}
