use super::{UiComponentShowcaseDemoState, SHOWCASE_EVENT_LOG_LIMIT};

#[test]
fn showcase_event_log_retains_a_bounded_recent_window() {
    let mut state = UiComponentShowcaseDemoState::default();
    for index in 0..=SHOWCASE_EVENT_LOG_LIMIT {
        state.push_log("Change", &format!("control-{index}"), None);
    }

    assert_eq!(state.event_log.len(), SHOWCASE_EVENT_LOG_LIMIT);
    assert_eq!(
        state
            .event_log
            .front()
            .map(|entry| entry.control_id.as_str()),
        Some("control-1")
    );
    assert_eq!(
        state
            .event_log
            .back()
            .map(|entry| entry.control_id.as_str()),
        Some("control-128")
    );
}
