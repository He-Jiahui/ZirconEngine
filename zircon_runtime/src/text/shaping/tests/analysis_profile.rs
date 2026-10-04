use super::{
    begin_enabled, record_bidi_metrics, record_line_break_metrics, record_script_emoji_metrics,
    take, TEXT_ANALYSIS_PROFILE_COUNTER_NAMES,
};
use std::collections::HashSet;

#[test]
fn request_profile_distinguishes_duplicate_line_break_builds() {
    begin_enabled(12, true);
    record_bidi_metrics(12, 3);
    record_script_emoji_metrics(12, 5);
    record_line_break_metrics(12, 7);
    record_line_break_metrics(12, 11);

    let metrics = take().expect("enabled request profiling must retain one request aggregate");
    assert_eq!(metrics.request_count, 1);
    assert_eq!(metrics.request_input_bytes, 12);
    assert_eq!(metrics.bidi_build_count, 1);
    assert_eq!(metrics.bidi_input_bytes, 12);
    assert_eq!(metrics.bidi_build_nanos, 3);
    assert_eq!(metrics.script_emoji_build_count, 1);
    assert_eq!(metrics.script_emoji_input_bytes, 12);
    assert_eq!(metrics.script_emoji_build_nanos, 5);
    assert_eq!(metrics.line_break_build_count, 2);
    assert_eq!(metrics.line_break_input_bytes, 24);
    assert_eq!(metrics.line_break_build_nanos, 18);
    assert!(take().is_none(), "completion must detach the TLS aggregate");
}

#[test]
fn analysis_profile_uses_only_fixed_request_names() {
    let unique = TEXT_ANALYSIS_PROFILE_COUNTER_NAMES
        .into_iter()
        .collect::<HashSet<_>>();
    assert_eq!(unique.len(), 11);
    assert!(
        unique.iter().all(|name| name.starts_with("text_analysis_")),
        "analysis profiling must use one fixed low-cardinality namespace"
    );
}
