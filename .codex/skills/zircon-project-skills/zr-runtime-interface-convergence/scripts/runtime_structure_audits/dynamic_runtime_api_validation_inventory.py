from __future__ import annotations

from runtime_structure_audits.dynamic_runtime_api_archive_inventory import (
    RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE,
)


BEHAVIOR_TEST_ANCHORS = (
    (
        "zircon_runtime/src/dynamic_api/tests/api_table.rs",
        "runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs",
        "destroy_session_reports_explicit_not_found_for_missing_nonzero_handle",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs",
        "destroy_session_removes_registry_entry_only_after_event_mirror_quiescent_teardown",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs",
        "session_destroy_reports_explicit_not_found_after_headless_destroy",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_entry_points.rs",
        "all_session_entry_points_reject_invalid_handle",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_entry_points.rs",
        "destroyed_headless_session_entry_points_reject_old_handle",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_entry_points.rs",
        "missing_session_entry_points_reject_nonzero_handle",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_profiles.rs",
        "create_session_accepts_named_headless_profile_without_render_bridge",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/session_profiles.rs",
        "minimal_and_headless_profiles_skip_render_bridge_bootstrap",
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
        "runtime_api_pointer_names_every_missing_required_function",
    ),
    (
        "zircon_app/src/entry/runtime_library/tests.rs",
        "runtime_library_loader_reports_missing_entry_symbol_source_path",
    ),
    (
        "zircon_app/src/entry/runtime_library/tests.rs",
        "runtime_library_loader_reports_missing_entry_symbol_from_dynamic_library",
    ),
    (
        "zircon_app/src/entry/runtime_library/tests.rs",
        "runtime_session_create_reports_first_call_failure_context",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/profile_control.rs",
        "profile_control_runtime_diagnostics_snapshot_returns_store_and_scene_reload_report",
    ),
    (
        "zircon_runtime/tests/runtime_owned_result_v7.rs",
        "runtime_v7_owned_results_require_opaque_exactly_once_release",
    ),
    (
        "zircon_runtime/tests/runtime_owned_result_v7.rs",
        "runtime_v7_release_is_concurrent_and_exactly_once",
    ),
    (
        "zircon_runtime/tests/runtime_owned_result_v7.rs",
        "runtime_v7_destroy_is_retryable_after_outstanding_result_release",
    ),
    (
        "zircon_runtime/tests/runtime_owned_result_v7.rs",
        "runtime_v7_release_rejects_a_different_session_without_changing_owner_census",
    ),
    (
        "zircon_runtime/tests/runtime_owned_result_v7.rs",
        "runtime_v7_release_performance_acceptance",
    ),
)

CARGO_GATE_ANCHORS = (
    (
        "docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md",
        "cargo test -p zircon_runtime --lib dynamic_api --locked -- --nocapture",
    ),
    (
        "docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md",
        "cargo test -p zircon_app --locked",
    ),
    (
        "docs/zircon_runtime/dynamic_api/session.md",
        "cargo test -p zircon_runtime --lib dynamic_api --locked --jobs 1 --message-format short",
    ),
    (
        "docs/engine-architecture/runtime-interface-cdylib-loader.md",
        "Full `cargo test -p zircon_app --locked` remains pending",
    ),
)

MIRROR_DOCS_GUARD = "runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts"

DOC_ANCHORS = (
    (
        "docs/engine-architecture/runtime-interface-convergence.md",
        "dynamic_runtime_api_boundary",
    ),
    (
        "docs/engine-architecture/runtime-interface-cdylib-loader.md",
        "dynamic_runtime_api_boundary",
    ),
    (
        "docs/engine-architecture/runtime-interface-cdylib-loader.md",
        "host_request_payload_anchors = 38/38",
    ),
    (
        "docs/engine-architecture/runtime-architecture-review-m0.md",
        "dynamic_runtime_api_boundary",
    ),
    (
        "docs/engine-architecture/runtime-architecture-review-m0.md",
        "host_request_payload_anchors = 38/38",
    ),
    (
        "docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md",
        "dynamic_runtime_api_boundary",
    ),
    (
        "docs/plans/zircon_runtime/runtime/10/2026-07-09-dynamic-api-and-interface-convergence-output-records.md",
        "host_request_payload_anchors = 38/38",
    ),
    (
        RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE,
        "dynamic_runtime_api_boundary",
    ),
    (
        RUNTIME_15_RUNTIME_INDEX_OUTPUT_ARCHIVE,
        "host_request_payload_anchors = 38/38",
    ),
    ("docs/zircon_runtime/dynamic_api/session.md", "dynamic_runtime_api_boundary"),
    (
        "docs/zircon_runtime/dynamic_api/session.md",
        "host_request_payload_anchors = 38/38",
    ),
    (
        "docs/engine-architecture/runtime-interface-convergence.md",
        "host_request_payload_anchors = 38/38",
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/mirror_docs.rs",
        MIRROR_DOCS_GUARD,
    ),
)
