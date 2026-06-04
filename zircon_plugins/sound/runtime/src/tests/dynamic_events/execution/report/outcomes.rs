use super::super::super::*;

use super::support::report_fixture;

#[test]
fn dynamic_event_execution_report_orders_statuses_by_priority_and_handler_id() {
    let fixture = report_fixture();

    let report = fixture.sound.execute_dynamic_events().unwrap();

    assert_eq!(report.executions.len(), 3);
    assert_eq!(report.executions[0].delivery.handler.plugin_id, "analytics");
    assert_eq!(
        report.executions[0].status,
        SoundDynamicEventExecutionStatus::Succeeded
    );
    assert_eq!(
        report.executions[1].delivery.handler.plugin_id,
        "gameplay_audio"
    );
    assert_eq!(
        report.executions[1].status,
        SoundDynamicEventExecutionStatus::Failed
    );
    assert_eq!(
        report.executions[1].detail.as_deref(),
        Some("foley unavailable")
    );
    assert_eq!(
        report.executions[2].delivery.handler.plugin_id,
        "timeline_sequence"
    );
    assert_eq!(
        report.executions[2].status,
        SoundDynamicEventExecutionStatus::SkippedMissingExecutor
    );
}
