#[test]
fn runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge() {
    let exports_source = include_str!("../../../dynamic_api/exports.rs");
    let session_source = include_str!("../../../dynamic_api/session/ffi.rs");
    let operation_source = include_str!("../../../dynamic_api/session/operation.rs");
    let api_table_tests = include_str!("../../../dynamic_api/tests/api_table.rs");
    let session_doc = include_str!("../../../../../docs/zircon_runtime/dynamic_api/session.md");
    let runtime_10_output = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/10/2026-07-09-dynamic-api-and-interface-convergence-output-records.md"
    );
    let runtime_index_output = include_str!(
        "../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
    );

    for required_exports_anchor in [
        "fn catch_ffi_panic(",
        "catch_unwind(AssertUnwindSafe",
        "ZrStatusCode::Panic",
        "runtime dynamic API panic caught at FFI boundary",
        "zircon_runtime_get_api_v6_inner",
        "Err(_) => core::ptr::null()",
    ] {
        assert!(
            exports_source.contains(required_exports_anchor),
            "dynamic API exports should keep FFI panic boundary anchor `{required_exports_anchor}`"
        );
    }

    for (inner, wrapper) in [
        ("create_session", "create_session_ffi"),
        ("destroy_session", "destroy_session_ffi"),
        ("handle_event", "handle_event_ffi"),
        ("capture_frame", "capture_frame_ffi"),
        (
            "capture_accessibility_tree",
            "capture_accessibility_tree_ffi",
        ),
        ("bind_viewport_surface", "bind_viewport_surface_ffi"),
        ("unbind_viewport_surface", "unbind_viewport_surface_ffi"),
        ("present_viewport", "present_viewport_ffi"),
        ("profile_control", "profile_control_ffi"),
        ("tick_frame", "tick_frame_ffi"),
        ("drain_host_requests", "drain_host_requests_ffi"),
        ("subscribe_plugin_event", "subscribe_plugin_event_ffi"),
        ("unsubscribe_plugin_event", "unsubscribe_plugin_event_ffi"),
        ("drain_plugin_events", "drain_plugin_events_ffi"),
        ("submit_operation", "submit_operation_ffi"),
        ("poll_operation", "poll_operation_ffi"),
        ("harvest_operation", "harvest_operation_ffi"),
        ("query_world", "query_world_ffi"),
        ("watch_world", "watch_world_ffi"),
        ("unwatch_world", "unwatch_world_ffi"),
        ("drain_world_invalidations", "drain_world_invalidations_ffi"),
    ] {
        assert!(
            exports_source.contains(&format!("Some({wrapper})")),
            "`ZrRuntimeApiV6` should advertise `{wrapper}` instead of the session owner `{inner}`"
        );
        assert!(
            exports_source.contains(&format!("fn {wrapper}(")),
            "dynamic API exports should keep wrapper function `{wrapper}`"
        );
        assert!(
            exports_source.contains(&format!("catch_ffi_panic(|| unsafe {{ {inner}(")),
            "`{wrapper}` should delegate to `{inner}` inside catch_ffi_panic"
        );
        assert!(
            !exports_source.contains(&format!("Some({inner}),")),
            "`ZrRuntimeApiV6` must not bypass the panic wrapper by advertising `{inner}` directly"
        );
        let owner_source = if inner.ends_with("_operation") {
            operation_source
        } else {
            session_source
        };
        let owner_visibility = if inner.ends_with("_operation") {
            "pub(crate)"
        } else {
            "pub(in crate::dynamic_api)"
        };
        assert!(
            owner_source.contains(&format!("{owner_visibility} unsafe fn {inner}(")),
            "private dynamic session owner `{inner}` should stay Rust ABI so the exports wrapper can catch unwinds"
        );
    }

    assert!(
        !session_source.contains("pub(in crate::dynamic_api) unsafe extern \"C\" fn"),
        "private dynamic session owner functions must not drift back to extern C"
    );
    assert!(
        !operation_source.contains("pub(crate) unsafe extern \"C\" fn"),
        "private operation owner functions must not drift back to extern C"
    );

    for required_test_anchor in [
        "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
        "pub(super) unsafe extern \\\"C\\\" fn",
        "expected_visibility",
    ] {
        assert!(
            api_table_tests.contains(required_test_anchor),
            "dynamic API table tests should keep FFI panic guard evidence `{required_test_anchor}`"
        );
    }

    for required_doc_anchor in [
        "## FFI Panic Boundary",
        "Rust-ABI session owner functions",
        "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
        "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
    ] {
        assert!(
            session_doc.contains(required_doc_anchor),
            "dynamic API session docs should record `{required_doc_anchor}`"
        );
    }

    for required_plan_anchor in [
        "M1.3 FFI panic 边界",
        "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
        "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
        "session.rs` owner 函数保持 Rust ABI",
    ] {
        assert!(
            runtime_10_output.contains(required_plan_anchor)
                || runtime_index_output.contains(required_plan_anchor),
            "Runtime 10 output records should record `{required_plan_anchor}`"
        );
    }
}
