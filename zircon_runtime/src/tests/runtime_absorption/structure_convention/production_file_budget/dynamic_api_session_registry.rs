use super::{assert_contains_all, assert_contains_all_exact, read_repo, read_runtime_src};

#[test]
fn runtime_15_dynamic_api_session_registry_is_child_owner() {
    let parent = read_runtime_src("dynamic_api/session.rs");
    let registry_facade = read_runtime_src("dynamic_api/session/registry/mod.rs");
    let session_store = read_runtime_src("dynamic_api/session/registry/session_store.rs");
    let session_slot = read_runtime_src("dynamic_api/session/registry/session_slot.rs");
    let ffi = read_runtime_src("dynamic_api/session/ffi.rs");
    let state = read_runtime_src("dynamic_api/session/state.rs");
    let lock_poison_tests = read_runtime_src("dynamic_api/session/tests/lock_poison.rs");
    let current_anchor_owner = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-19-dynamic-api-filter-plan-anchor-current-owner.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let dynamic_session_doc = read_repo("docs/zircon_runtime/dynamic_api/session.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let status_row_owner = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/core_rhi_dynamic.rs",
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
        &["mod registry;"],
    );
    assert_contains_all(
        "dynamic API session FFI child routes registry lifecycle operations",
        &ffi,
        &[
            "destroy_session_slot, insert_session_with_wake, with_session, with_session_activity,",
            "pub(in crate::dynamic_api) unsafe fn create_session(",
            "let handle = insert_session_with_wake(session, wake);",
            "pub(in crate::dynamic_api) unsafe fn destroy_session(",
            "destroy_session_slot(handle)",
        ],
    );
    assert_contains_all(
        "dynamic API session state child owns the registry payload",
        &state,
        &["struct RuntimeDynamicSession", "impl RuntimeDynamicSession"],
    );
    assert_contains_all(
        "dynamic API session lock-poison tests consume the registry lock helper",
        &lock_poison_tests,
        &[
            "insert_session, poison_registry_lock_for_test, with_session",
            "with_session(handle, |_| panic!(\"poison dynamic API session lock\"))",
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
        "HashMap<u64, Arc<SessionSlot>>",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "dynamic_api/session.rs should delegate {moved_owner} to dynamic_api/session/registry/session_store.rs"
        );
    }
    assert_contains_all(
        "dynamic API session registry facade stays route-only",
        &registry_facade,
        &[
            "mod session_store;",
            "pub(super) use session_store::{",
            "destroy_session_slot, insert_session, insert_session_with_wake, with_session,",
        ],
    );
    for forbidden_behavior in ["static ", "struct ", "impl ", "fn "] {
        assert!(
            !registry_facade.contains(forbidden_behavior),
            "dynamic_api/session/registry/mod.rs must stay a zero-behavior facade; found `{forbidden_behavior}`"
        );
    }
    assert_contains_all(
        "dynamic API session store child owns handle storage and lock recovery",
        &session_store,
        &[
            "static SESSION_REGISTRY: OnceLock<Mutex<SessionRegistry>>",
            "struct SessionRegistry",
            "sessions: HashMap<u64, Arc<SessionSlot>>",
            "fn registry() -> &'static Mutex<SessionRegistry>",
            "fn lock_registry() -> MutexGuard<'static, SessionRegistry>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "fn insert_session(",
            "fn insert_session_with_wake(",
            "fetch_add(1, Ordering::SeqCst)",
            "fn with_session(",
            "fn with_session_activity(",
            "fn destroy_session_slot(",
            "invalid_argument(b\"invalid runtime session handle\")",
            "not_found(b\"runtime session not found\")",
        ],
    );
    let insert_with_wake = section_between(
        &session_store,
        "fn insert_session_with_wake(",
        "fn with_session(",
    );
    assert_contains_all(
        "dynamic API session wake-aware insertion body",
        insert_with_wake,
        &[
            "wake: RuntimeWakeRegistration",
            "SessionSlot::new(session, wake)",
        ],
    );
    let with_session_activity = section_between(
        &session_store,
        "fn with_session_activity(",
        "fn destroy_session_slot(",
    );
    assert_contains_all(
        "dynamic API session activity dispatch body",
        with_session_activity,
        &[
            "slot.begin_action()",
            "let mut session = slot.lock_session();",
            "action(session, slot.frame_activity())",
            "drop(action_guard)",
        ],
    );
    let destroy_session = section_between(
        &session_store,
        "fn destroy_session_slot(",
        "fn find_session_slot(",
    );
    assert_contains_all(
        "dynamic API session destroy body",
        destroy_session,
        &[
            "slot.begin_close()",
            "slot.frame_activity().disable_wake_entries();",
            "slot.wait_for_actions();",
            "slot.frame_activity().wait_for_wake_callbacks();",
            "drop(slot.take_session());",
            "registry.sessions.remove(&handle.raw());",
        ],
    );
    assert_contains_all(
        "dynamic API session slot child owns poison-safe execution and close lifecycle",
        &session_slot,
        &[
            "pub(in crate::dynamic_api::session) struct SessionSlot",
            "session: Mutex<Option<RuntimeDynamicSession>>",
            "pub(super) fn begin_action(self: &Arc<Self>)",
            "pub(super) fn begin_close(&self) -> bool",
            "pub(super) fn lock_session(&self) -> MutexGuard<'_, Option<RuntimeDynamicSession>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
        ],
    );

    for (path, source) in [
        ("dynamic_api/session.rs", parent.as_str()),
        ("dynamic_api/session/ffi.rs", ffi.as_str()),
        ("dynamic_api/session/state.rs", state.as_str()),
        (
            "dynamic_api/session/registry/mod.rs",
            registry_facade.as_str(),
        ),
        (
            "dynamic_api/session/registry/session_store.rs",
            session_store.as_str(),
        ),
        (
            "dynamic_api/session/registry/session_slot.rs",
            session_slot.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    let current_registry_row = section_between(
        &current_anchor_owner,
        "Runtime 15 M4 dynamic API session registry owner split",
        "Runtime 15 M4 dynamic API shader prewarm tests owner split",
    );
    assert_contains_all_exact(
        "Runtime 15 dynamic-API filter current child owner",
        current_registry_row,
        &[
            "Runtime 15 M4 dynamic API session registry owner split",
            "runtime_15_dynamic_api_session_registry_owner_split_static_passed_cargo_deferred",
            "dynamic_api/session.rs",
            "dynamic_api/session/registry/session_store.rs",
            "runtime_15_dynamic_api_session_registry_is_child_owner",
        ],
    );
    for (label, source) in [
        ("current child owner", current_anchor_owner.as_str()),
        ("status-output route mirror", status_rows.as_str()),
        ("status-output row owner", status_row_owner.as_str()),
    ] {
        assert!(
            !source.contains("dynamic_api/session/registry.rs"),
            "{label} should not retain the deleted flat registry path"
        );
    }
    for (label, source) in [
        ("status-output route mirror", status_rows.as_str()),
        ("status-output row owner", status_row_owner.as_str()),
    ] {
        let registry_row = section_between(
            source,
            "Runtime 15 M4 dynamic API session registry owner split",
            "Runtime 15 M4 dynamic API shader prewarm tests owner split",
        );
        assert_contains_all_exact(
            label,
            registry_row,
            &[
                "runtime_15_dynamic_api_session_registry_owner_split_static_passed_cargo_deferred",
                "dynamic_api/session.rs",
                "dynamic_api/session/registry/session_store.rs",
                "runtime_15_dynamic_api_session_registry_is_child_owner",
            ],
        );
    }
    for (label, source) in [
        ("module convention doc", module_doc.as_str()),
        ("dynamic API session doc", dynamic_session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 dynamic API session registry owner split",
                "runtime_15_dynamic_api_session_registry_owner_split_static_passed_cargo_deferred",
                "dynamic_api/session.rs",
                "dynamic_api/session/registry/mod.rs",
                "dynamic_api/session/registry/session_store.rs",
                "zero-behavior",
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

fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_offset = source
        .find(start)
        .unwrap_or_else(|| panic!("missing section start `{start}`"));
    let after_start = &source[start_offset..];
    let end_offset = after_start
        .find(end)
        .unwrap_or_else(|| panic!("missing section end `{end}`"));
    &after_start[..end_offset]
}
