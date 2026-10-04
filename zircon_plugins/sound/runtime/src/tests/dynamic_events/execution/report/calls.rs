// 执行待处理事件后只核对已注册的两个执行器写入调用日志，确保缺失执行器不会被调用。
use super::super::super::*;

use super::support::report_fixture;

#[test]
fn dynamic_event_execution_calls_registered_executors_only() {
    let fixture = report_fixture();

    fixture.sound.execute_dynamic_events().unwrap();

    assert_eq!(
        *fixture.calls.lock().unwrap(),
        vec!["analytics".to_string(), "gameplay_audio".to_string()]
    );
}
