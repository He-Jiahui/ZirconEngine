use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_dynamic_api_session_profile_is_child_owner() {
    let parent = read_runtime_src("dynamic_api/session.rs");
    let ffi = read_runtime_src("dynamic_api/session/ffi.rs");
    let state = read_runtime_src("dynamic_api/session/state.rs");
    let profile = read_runtime_src("dynamic_api/session/profile.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let dynamic_session_doc = read_repo("docs/zircon_runtime/dynamic_api/session.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m4_surface_cleanup.rs",
    );

    assert_contains_all(
        "dynamic API session parent delegates profile policy and keeps session lifecycle",
        &parent,
        &["mod profile;", "use profile::RuntimeDynamicSessionProfile;"],
    );
    assert_contains_all(
        "dynamic API session FFI child owns session lifecycle entry points",
        &ffi,
        &["pub(in crate::dynamic_api) unsafe fn create_session("],
    );
    assert_contains_all(
        "dynamic API session state child owns runtime session behavior",
        &state,
        &[
            "struct RuntimeDynamicSession",
            "impl RuntimeDynamicSession",
            "fn new(",
            "fn tick_frame(&mut self) -> RuntimeDynamicSessionResult<()>",
        ],
    );
    for moved_owner in [
        "enum RuntimeDynamicSessionProfile",
        "const DEFAULT_DYNAMIC_RUNTIME_MAX_FIXED_STEPS_PER_FRAME",
        "const RUNTIME_SESSION_PROFILE_RUNTIME",
        "fn from_bytes(bytes: &[u8]) -> Option<Self>",
        "fn max_fixed_steps_per_frame(self) -> u32",
        "fn diagnostic_log_schedule(self) -> DiagnosticStoreLogSchedule",
        "fn uses_render_bridge(self) -> bool",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "dynamic_api/session.rs should delegate {moved_owner} to dynamic_api/session/profile.rs"
        );
    }
    assert_contains_all(
        "dynamic API session profile child owns profile parsing and policy",
        &profile,
        &[
            "pub(super) enum RuntimeDynamicSessionProfile",
            "const DEFAULT_DYNAMIC_RUNTIME_MAX_FIXED_STEPS_PER_FRAME",
            "const RUNTIME_SESSION_PROFILE_RUNTIME",
            "pub(super) fn from_bytes(bytes: &[u8]) -> Option<Self>",
            "pub(super) fn max_fixed_steps_per_frame(self) -> u32",
            "pub(super) fn diagnostic_log_schedule(self) -> DiagnosticStoreLogSchedule",
            "pub(super) fn uses_render_bridge(self) -> bool",
            "DiagnosticStoreLogSchedule::repeating(DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT)",
        ],
    );

    for (path, source) in [
        ("dynamic_api/session.rs", parent.as_str()),
        ("dynamic_api/session/ffi.rs", ffi.as_str()),
        ("dynamic_api/session/state.rs", state.as_str()),
        ("dynamic_api/session/profile.rs", profile.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("dynamic API session doc", dynamic_session_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 dynamic API session profile owner split",
                "runtime_15_dynamic_api_session_profile_owner_split_static_passed_cargo_deferred",
                "dynamic_api/session.rs",
                "dynamic_api/session/profile.rs",
                "runtime_15_dynamic_api_session_profile_is_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M4 dynamic API session profile owner split",
            "runtime_15_dynamic_api_session_profile_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M4 dynamic API session profile owner split",
            "2026-06-24",
        ],
    );
}
