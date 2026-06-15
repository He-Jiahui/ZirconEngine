pub(super) const EXPECTED_RUNTIME_10_SOURCE_FILES: &[&str] = &[
    "zircon_runtime_interface/src/runtime_api/api_table.rs",
    "zircon_runtime_interface/src/plugin_api.rs",
    "zircon_runtime_interface/src/tests/abi_safety_contracts.rs",
    "zircon_runtime/src/dynamic_api/exports.rs",
    "zircon_runtime/src/dynamic_api/session.rs",
    "zircon_runtime/src/dynamic_api/session/events.rs",
    "zircon_runtime/src/dynamic_api/tests/api_table.rs",
    "zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs",
    "zircon_runtime/src/dynamic_api/tests/session_entry_points.rs",
    "zircon_runtime/src/dynamic_api/tests/session_profiles.rs",
    "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs",
    "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared.rs",
    "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/headless_profiles.rs",
    "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/event_split.rs",
    "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/test_owner_split.rs",
    "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ffi_panic_boundary.rs",
    "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/mirror_docs.rs",
    "zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late.rs",
    "zircon_app/src/entry/runtime_library/loaded_runtime.rs",
    "zircon_app/src/entry/runtime_library/runtime_session.rs",
    "zircon_app/src/entry/runtime_library/tests.rs",
];

pub(super) const EXPECTED_RUNTIME_10_FUNCTION_TABLES: &[(&str, &str, usize)] = &[
    (
        "zircon_runtime_interface/src/runtime_api/api_table.rs",
        "ZrHostApiV1",
        4,
    ),
    (
        "zircon_runtime_interface/src/runtime_api/api_table.rs",
        "ZrRuntimeApiV1",
        13,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrHostApiV3",
        7,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrHostEcsApiV1",
        3,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrHostAssetApiV1",
        1,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrHostEventApiV1",
        2,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrHostBridgeApiV1",
        1,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrHostDiagnosticsApiV1",
        2,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrPluginStateSnapshotApiV1",
        4,
    ),
    (
        "zircon_runtime_interface/src/plugin_api.rs",
        "ZrPluginApiV1",
        4,
    ),
];

pub(super) const EXPECTED_RUNTIME_10_SESSION_OPERATIONS: &[&str] = &[
    "create_session",
    "destroy_session",
    "handle_event",
    "capture_frame",
    "capture_accessibility_tree",
    "bind_viewport_surface",
    "unbind_viewport_surface",
    "present_viewport",
    "profile_control",
    "tick_frame",
    "drain_host_requests",
];

pub(super) const EXPECTED_RUNTIME_10_BEHAVIOR_TEST_ANCHORS: &[(&str, &[&str])] = &[
    (
        "zircon_runtime/src/dynamic_api/tests/api_table.rs",
        &["runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary"],
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs",
        &[
            "destroy_session_reports_explicit_not_found_for_missing_nonzero_handle",
            "destroy_session_removes_registry_entry_so_destroyed_handles_become_missing",
            "session_destroy_reports_explicit_not_found_after_headless_destroy",
        ],
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_entry_points.rs",
        &[
            "all_session_entry_points_reject_invalid_handle",
            "destroyed_headless_session_entry_points_reject_old_handle",
            "missing_session_entry_points_reject_nonzero_handle",
        ],
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_profiles.rs",
        &[
            "create_session_accepts_named_headless_profile_without_render_bridge",
            "minimal_and_headless_profiles_skip_render_bridge_bootstrap",
        ],
    ),
    (
        "zircon_app/src/entry/runtime_library/tests.rs",
        &[
            "runtime_api_pointer_rejects_null_from_entry_symbol",
            "runtime_api_pointer_rejects_version_mismatch_before_session_creation",
            "runtime_api_pointer_rejects_missing_required_functions_before_session_creation",
            "runtime_library_loader_reports_missing_entry_symbol_source_path",
            "runtime_library_loader_reports_missing_entry_symbol_from_dynamic_library",
            "runtime_session_create_reports_first_call_failure_context",
        ],
    ),
];

pub(super) const EXPECTED_RUNTIME_10_MIRROR_DOCS: &[&str] = &[
    "docs/zircon_runtime/dynamic_api/session.md",
    "docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md",
    "docs/plans/zircon_runtime/runtime/index.md",
    "docs/engine-architecture/runtime-architecture-review-m0.md",
    "docs/engine-architecture/runtime-interface-convergence.md",
    "docs/engine-architecture/runtime-interface-cdylib-loader.md",
];

pub(super) fn slice_between<'a>(source: &'a str, start_anchor: &str, end_anchor: &str) -> &'a str {
    let start = source
        .find(start_anchor)
        .unwrap_or_else(|| panic!("source should contain start anchor `{start_anchor}`"));
    let tail = &source[start..];
    let end = tail.find(end_anchor).unwrap_or_else(|| {
        panic!("source should contain end anchor `{end_anchor}` after `{start_anchor}`")
    });
    &tail[..end]
}
