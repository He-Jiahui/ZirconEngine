#[test]
fn runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge() {
    let exports_source = include_str!("../../../dynamic_api/exports.rs");
    let session_source = include_str!("../../../dynamic_api/session.rs");
    let api_table_tests = include_str!("../../../dynamic_api/tests/api_table.rs");
    let session_doc = include_str!("../../../../../docs/zircon_runtime/dynamic_api/session.md");
    let runtime_10_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");

    for required_exports_anchor in [
        "fn catch_ffi_panic(",
        "catch_unwind(AssertUnwindSafe",
        "ZrStatusCode::Panic",
        "runtime dynamic API panic caught at FFI boundary",
        "zircon_runtime_get_api_v1_inner",
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
    ] {
        assert!(
            exports_source.contains(&format!("Some({wrapper})")),
            "`ZrRuntimeApiV1` should advertise `{wrapper}` instead of the session owner `{inner}`"
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
            "`ZrRuntimeApiV1` must not bypass the panic wrapper by advertising `{inner}` directly"
        );
        assert!(
            session_source.contains(&format!("pub(super) unsafe fn {inner}(")),
            "private dynamic session owner `{inner}` should stay Rust ABI so the exports wrapper can catch unwinds"
        );
    }

    assert!(
        !session_source.contains("pub(super) unsafe extern \"C\" fn"),
        "private dynamic session owner functions must not drift back to extern C"
    );

    for required_test_anchor in [
        "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
        "pub(super) unsafe extern \\\"C\\\" fn",
        "pub(super) unsafe fn {inner}(",
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
            runtime_10_plan.contains(required_plan_anchor)
                || runtime_index.contains(required_plan_anchor),
            "Runtime 10 plan/index should record `{required_plan_anchor}`"
        );
    }
}
