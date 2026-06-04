use super::support::{assert_structural_module, src_root};

#[test]
fn dynamic_event_module_wiring_remains_folder_backed() {
    let src = src_root();

    assert_structural_module(
        &src,
        "service_types/dynamic_events/mod.rs",
        &[
            "mod catalog;",
            "mod dispatch;",
            "mod handlers;",
            "mod invocation;",
        ],
    );
    assert_structural_module(
        &src,
        "service_types/dynamic_event_executors/mod.rs",
        &["mod execution;", "mod registration;", "mod unregistration;"],
    );
    assert_structural_module(
        &src,
        "dynamic_events/mod.rs",
        &[
            "pub(crate) mod catalog;",
            "pub(crate) mod dispatch;",
            "pub(crate) mod handlers;",
            "pub(crate) mod invocation;",
        ],
    );
    assert_structural_module(
        &src,
        "dynamic_event_abi/mod.rs",
        &[
            "pub(crate) mod callback;",
            "pub(crate) mod executor;",
            "pub(crate) mod request;",
            "pub(crate) mod slice;",
            "pub(crate) mod status;",
        ],
    );
}
