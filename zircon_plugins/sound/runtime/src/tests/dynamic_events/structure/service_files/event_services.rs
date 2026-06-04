use super::super::support::{assert_source_contains, src_root};

#[test]
fn dynamic_event_service_behavior_stays_in_focused_event_files() {
    let src = src_root();

    assert_source_contains(&src, "service_types/mod.rs", &["mod dynamic_events;"]);
    assert_source_contains(
        &src,
        "service_types/dynamic_events/catalog.rs",
        &["dynamic_event_catalog_impl", "register_dynamic_event_impl"],
    );
    assert_source_contains(
        &src,
        "service_types/dynamic_events/handlers.rs",
        &[
            "dynamic_event_handlers_impl",
            "register_dynamic_event_handler_impl",
        ],
    );
    assert_source_contains(
        &src,
        "service_types/dynamic_events/invocation.rs",
        &["submit_dynamic_event_impl", "drain_dynamic_events_impl"],
    );
    assert_source_contains(
        &src,
        "service_types/dynamic_events/dispatch.rs",
        &["dispatch_dynamic_events_impl"],
    );
}
