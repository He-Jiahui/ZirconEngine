from __future__ import annotations


LEGACY_RUNTIME_API_FILES = (
    "zircon_runtime_interface/src/runtime_api/api_table.rs",
    "zircon_runtime/src/dynamic_api/exports.rs",
    "zircon_app/src/entry/runtime_library/loaded_runtime.rs",
)
LEGACY_RUNTIME_API_SYMBOLS = (
    "ZrRuntimeApiV1",
    "ZrRuntimeGetApiFnV1",
    "ZR_RUNTIME_GET_API_SYMBOL_V1",
    "zircon_runtime_get_api_v1",
    "RuntimeApi::V1",
)


FFI_PANIC_ANCHORS = (
    ("zircon_runtime/src/dynamic_api/exports.rs", "fn catch_ffi_panic("),
    ("zircon_runtime/src/dynamic_api/exports.rs", "catch_unwind(AssertUnwindSafe"),
    ("zircon_runtime/src/dynamic_api/exports.rs", "ZrStatusCode::Panic"),
    (
        "zircon_runtime/src/dynamic_api/exports.rs",
        "runtime dynamic API panic caught at FFI boundary",
    ),
    ("zircon_runtime/src/dynamic_api/exports.rs", "zircon_runtime_get_api_v2_inner"),
    ("zircon_runtime/src/dynamic_api/exports.rs", "Err(_) => core::ptr::null()"),
    (
        "zircon_runtime/src/dynamic_api/tests/api_table.rs",
        "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ffi_panic_boundary.rs",
        "runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge",
    ),
    ("docs/zircon_runtime/dynamic_api/session.md", "## FFI Panic Boundary"),
)

LOADER_FAILURE_ANCHORS = (
    (
        "zircon_app/src/entry/runtime_library/loaded_runtime.rs",
        ".get::<ZrRuntimeGetApiFnV2>(ZR_RUNTIME_GET_API_SYMBOL_V2)",
    ),
    (
        "zircon_app/src/entry/runtime_library/loaded_runtime.rs",
        "validate_runtime_api_pointer",
    ),
    (
        "zircon_app/src/entry/runtime_library/loaded_runtime.rs",
        "runtime_api_required_layout_available",
    ),
    (
        "zircon_app/src/entry/runtime_library/loaded_runtime.rs",
        "runtime_api_supports_viewport_surface_present",
    ),
    (
        "zircon_app/src/entry/runtime_library/runtime_session.rs",
        'ensure_status(status, "create runtime session")?;',
    ),
    (
        "zircon_app/src/entry/runtime_library/tests.rs",
        "runtime_api_pointer_rejects_null_from_entry_symbol",
    ),
    (
        "zircon_app/src/entry/runtime_library/tests.rs",
        "runtime_api_pointer_rejects_version_mismatch_before_session_creation",
    ),
    (
        "zircon_app/src/entry/runtime_library/tests.rs",
        "runtime_api_pointer_rejects_missing_required_functions_before_session_creation",
    ),
    (
        "zircon_app/src/entry/runtime_library/tests.rs",
        "runtime_api_pointer_rejects_missing_required_operation_functions",
    ),
    (
        "zircon_app/src/entry/runtime_library/tests.rs",
        "runtime_session_does_not_recheck_required_v2_mirror_or_operation_capabilities",
    ),
    (
        "zircon_app/src/entry/runtime_library/runtime_session/operation.rs",
        "operation_output_abi_rejects_foreign_versions",
    ),
    (
        "zircon_app/src/entry/runtime_library/tests.rs",
        "runtime_library_loader_reports_missing_entry_symbol_from_dynamic_library",
    ),
    (
        "zircon_app/src/entry/runtime_library/tests.rs",
        "runtime_session_create_reports_first_call_failure_context",
    ),
)
