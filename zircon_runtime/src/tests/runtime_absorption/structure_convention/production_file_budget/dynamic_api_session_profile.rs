use super::{assert_contains_all, assert_contains_all_exact, read_repo, read_runtime_src};

#[test]
fn runtime_15_dynamic_api_session_profile_is_child_owner() {
    let parent = read_runtime_src("dynamic_api/session.rs");
    let ffi = read_runtime_src("dynamic_api/session/ffi.rs");
    let state = read_runtime_src("dynamic_api/session/state.rs");
    let profile = read_runtime_src("dynamic_api/session/profile.rs");
    let current_anchor_owner = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-19-dynamic-api-filter-plan-anchor-current-owner.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let dynamic_session_doc = read_repo("docs/zircon_runtime/dynamic_api/session.md");

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

    assert_contains_all_exact(
        "Runtime 15 dynamic-API filter current child owner",
        &current_anchor_owner,
        &[
            "Runtime 15 M4 dynamic API session profile owner split",
            "runtime_15_dynamic_api_session_profile_owner_split_static_passed_cargo_deferred",
            "dynamic_api/session.rs",
            "dynamic_api/session/profile.rs",
            "runtime_15_dynamic_api_session_profile_is_child_owner",
        ],
    );
}
