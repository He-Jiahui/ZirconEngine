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
