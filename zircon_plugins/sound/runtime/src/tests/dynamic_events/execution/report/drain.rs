// 首次执行消耗已排队事件并返回三项报告，重复执行应为空，避免同一调用再次触发处理器。
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
