use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_dynamic_api_session_registry_is_child_owner() {
    let parent = read_runtime_src("dynamic_api/session.rs");
    let registry = read_runtime_src("dynamic_api/session/registry.rs");
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
        "dynamic API session parent delegates registry ownership",
        &parent,
        &[
            "mod registry;",
            "use registry::{insert_session, lock_registry, with_session};",
            "use registry::lock_session;",
            "pub(super) unsafe fn create_session(",
            "pub(super) unsafe fn destroy_session(",
            "struct RuntimeDynamicSession",
            "impl RuntimeDynamicSession",
        ],
    );
    for moved_owner in [
        "static SESSION_REGISTRY",
        "struct SessionRegistry",
        "fn registry()",
        "fn lock_registry()",
        "fn lock_session(",
        "fn insert_session(",
        "fn with_session(",
        "AtomicU64",
        "Ordering::SeqCst",
        "HashMap<u64, Arc<Mutex<RuntimeDynamicSession>>>",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "dynamic_api/session.rs should delegate {moved_owner} to dynamic_api/session/registry.rs"
        );
    }
    assert_contains_all(
        "dynamic API session registry child owns handle storage and lock recovery",
        &registry,
        &[
            "static SESSION_REGISTRY: OnceLock<Mutex<SessionRegistry>>",
            "pub(super) struct SessionRegistry",
            "pub(super) sessions: HashMap<u64, Arc<Mutex<RuntimeDynamicSession>>>",
            "fn registry() -> &'static Mutex<SessionRegistry>",
            "pub(super) fn lock_registry() -> MutexGuard<'static, SessionRegistry>",
            "pub(super) fn lock_session(",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "pub(super) fn insert_session(session: RuntimeDynamicSession) -> ZrRuntimeSessionHandle",
            "fetch_add(1, Ordering::SeqCst)",
            "pub(super) fn with_session(",
            "invalid_argument(b\"invalid runtime session handle\")",
            "not_found(b\"runtime session not found\")",
        ],
    );

    for (path, source) in [
        ("dynamic_api/session.rs", parent.as_str()),
        ("dynamic_api/session/registry.rs", registry.as_str()),
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
                "Runtime 15 M4 dynamic API session registry owner split",
                "runtime_15_dynamic_api_session_registry_owner_split_static_passed_cargo_deferred",
                "dynamic_api/session.rs",
                "dynamic_api/session/registry.rs",
                "runtime_15_dynamic_api_session_registry_is_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M4 dynamic API session registry owner split",
            "runtime_15_dynamic_api_session_registry_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M4 dynamic API session registry owner split",
            "2026-06-24",
        ],
    );
}
