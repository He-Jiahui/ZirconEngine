use super::should_report_count;

#[test]
fn unhandled_ui_host_request_diagnostics_are_logarithmically_bounded() {
    assert!(should_report_count(1));
    assert!(should_report_count(2));
    assert!(!should_report_count(3));
    assert!(should_report_count(4));
}

#[test]
fn diagnostic_source_never_formats_dynamic_host_request_content() {
    let source = include_str!("../ui_host_request.rs");

    assert!(!source.contains(concat!("request.kind.", "href")));
    assert!(!source.contains(concat!("request.kind.", "popup_id")));
    assert!(!source.contains(concat!("request.kind.", "tooltip_id")));
    assert!(!source.contains(concat!("request.", "tree_id")));
}
