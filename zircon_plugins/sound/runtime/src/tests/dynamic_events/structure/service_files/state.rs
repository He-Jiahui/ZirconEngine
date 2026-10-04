// 通过生产源码片段确认执行器状态位于引擎状态文件；只核对字段标识存在，不执行事件运行时。
use super::super::support::{assert_source_contains, src_root};

#[test]
fn dynamic_event_executor_state_stays_in_engine_state_file() {
    let src = src_root();

    assert_source_contains(
        &src,
        "engine/state/dynamic_events.rs",
        &["SoundDynamicEventExecutor", "SoundDynamicEventExecutorKey"],
    );
}
