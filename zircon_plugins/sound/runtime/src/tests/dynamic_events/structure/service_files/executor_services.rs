use super::super::support::{assert_source_contains, src_root};

#[test]
fn dynamic_event_executor_behavior_stays_in_focused_service_files() {
    let src = src_root();

    assert_source_contains(
        &src,
        "service_types/mod.rs",
        &["mod dynamic_event_executors;"],
    );
    assert_source_contains(
        &src,
        "service_types/dynamic_event_executors/registration.rs",
        &["register_dynamic_event_executor"],
    );
    assert_source_contains(
        &src,
        "service_types/dynamic_event_executors/unregistration.rs",
        &["unregister_dynamic_event_executor"],
    );
    assert_source_contains(
        &src,
        "service_types/dynamic_event_executors/execution.rs",
        &[
            "execute_dynamic_events_impl",
            "SoundDynamicEventExecutionReport",
        ],
    );
}
