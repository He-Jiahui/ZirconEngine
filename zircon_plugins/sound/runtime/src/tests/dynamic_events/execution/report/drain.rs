use super::super::super::*;

use super::support::report_fixture;

#[test]
fn dynamic_event_execution_drains_pending_events_after_reporting() {
    let fixture = report_fixture();

    assert_eq!(
        fixture
            .sound
            .execute_dynamic_events()
            .unwrap()
            .executions
            .len(),
        3
    );
    assert!(fixture
        .sound
        .execute_dynamic_events()
        .unwrap()
        .executions
        .is_empty());
}
