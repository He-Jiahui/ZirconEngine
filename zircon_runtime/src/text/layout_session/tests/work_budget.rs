use super::SharedTextLayoutSession;
use crate::core::framework::text::TextDirection;
use crate::text::{TextRange, TextStyle};

#[test]
fn session_work_receipt_counts_only_backend_cache_misses() {
    let mut session = SharedTextLayoutSession::new();
    session.begin_frame(1);
    let style = TextStyle::default();
    let range = TextRange { start: 0, end: 9 };

    session
        .shape_horizontal_range("work miss", &style, TextDirection::LeftToRight, range)
        .into_result()
        .expect("shape cache miss");
    session
        .shape_horizontal_range("work miss", &style, TextDirection::LeftToRight, range)
        .into_result()
        .expect("shape cache hit");

    let report = session.shaping_work_report();
    assert_eq!(report.inline_request_count, 1);
    assert_eq!(report.oversized_synchronous_request_count, 0);
    assert_eq!(report.synchronous_input_bytes, "work miss".len());
    assert_eq!(report.max_synchronous_input_bytes, "work miss".len());
}
