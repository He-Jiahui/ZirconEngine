use super::*;

#[test]
fn runtime_15_navigation_lock_poison_recovery_guard_covers_builtin_navigation_manager() {
    let navigation_runtime = read_runtime_src("navigation/runtime.rs");
    let navigation_tests = read_runtime_src("navigation/runtime/tests.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let navigation_doc = read_repo("docs/zircon_runtime/navigation/runtime.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );

    assert_contains_all(
        "BuiltinNavigationManager poison recovery helper",
        &navigation_runtime,
        &[
            "use std::sync::{Mutex, MutexGuard};",
            "fn lock_state(&self) -> MutexGuard<'_, BuiltinNavigationState>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "let mut state = self.lock_state();",
            "let state = self.lock_state();",
            "self.lock_state().stats.clone()",
        ],
    );
    assert!(
        !navigation_runtime.contains("expect(\"navigation state lock poisoned\")"),
        "BuiltinNavigationManager production paths should not panic on poisoned navigation state locks"
    );
    assert!(
        !navigation_runtime.contains(LOCK_UNWRAP_CALL),
        "BuiltinNavigationManager production paths should use lock_state() instead of {LOCK_UNWRAP_CALL}"
    );
    assert_contains_all(
        "BuiltinNavigationManager poison recovery test",
        &navigation_tests,
        &[
            "navigation_manager_accessors_recover_poisoned_state_lock",
            "manager.state.lock().unwrap()",
            "NavigationSettingsAsset::default_3d()",
            "manager.stats().loaded_nav_meshes",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("navigation runtime doc", navigation_doc.as_str()),
        (
            "status-output M3 lock-poison row data",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 navigation lock poison recovery",
                "runtime_15_navigation_lock_poison_recovery_static_passed_cargo_deferred",
                "navigation/runtime.rs",
                "runtime_15_navigation_lock_poison_recovery_guard_covers_builtin_navigation_manager",
            ],
        );
    }
}

#[test]
fn runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager() {
    let resource_manager = read_runtime_src("core/resource/manager/resource_manager.rs");
    let registry_ops = read_runtime_src("core/resource/manager/registry_ops.rs");
    let payload_ops = read_runtime_src("core/resource/manager/payload_ops.rs");
    let lease_ops = read_runtime_src("core/resource/manager/lease_ops.rs");
    let events = read_runtime_src("core/resource/manager/events.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let resource_doc = read_repo("docs/zircon_runtime/core/resource.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );

    assert_contains_all(
        "ResourceManager poison recovery helpers",
        &resource_manager,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};",
            "pub(super) type ResourcePayloadMap = HashMap<ResourceId, Arc<dyn ResourceData>>;",
            "pub(super) type ResourceRuntimeMap = HashMap<ResourceId, ResourceRuntimeSlot>;",
            "pub(super) type ResourceSubscriberList = Vec<Sender<ResourceEvent>>;",
            "pub(super) fn lock_registry_read(&self) -> RwLockReadGuard<'_, ResourceRegistry>",
            "pub(super) fn lock_registry_write(&self) -> RwLockWriteGuard<'_, ResourceRegistry>",
            "pub(super) fn lock_payloads_read(&self) -> RwLockReadGuard<'_, ResourcePayloadMap>",
            "pub(super) fn lock_payloads_write(&self) -> RwLockWriteGuard<'_, ResourcePayloadMap>",
            "pub(super) fn lock_runtime_read(&self) -> RwLockReadGuard<'_, ResourceRuntimeMap>",
            "pub(super) fn lock_runtime_write(&self) -> RwLockWriteGuard<'_, ResourceRuntimeMap>",
            "pub(super) fn lock_subscribers(&self) -> MutexGuard<'_, ResourceSubscriberList>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.lock_subscribers().push(sender)",
            "self.lock_registry_read()",
            "resource_manager_accessors_recover_poisoned_state_locks",
        ],
    );
    assert_contains_all(
        "ResourceManager registry ops use helpers",
        &registry_ops,
        &[
            "let mut registry = self.lock_registry_write();",
            "self.lock_payloads_write().remove(&removed.id);",
            "self.lock_runtime_write().remove(&removed.id);",
        ],
    );
    assert_contains_all(
        "ResourceManager payload ops use helpers",
        &payload_ops,
        &[
            "let mut registry = self.lock_registry_write();",
            "self.lock_payloads_write()",
            "self.lock_payloads_read().get(&id).cloned()",
        ],
    );
    assert_contains_all(
        "ResourceManager lease ops use helpers",
        &lease_ops,
        &[
            "let mut runtime = self.lock_runtime_write();",
            "self.lock_payloads_write().remove(&id);",
            "self.lock_runtime_read()",
        ],
    );
    assert_contains_all(
        "ResourceManager event ops use helpers",
        &events,
        &[
            "let mut subscribers = self.lock_subscribers();",
            "let mut runtime = self.lock_runtime_write();",
        ],
    );
    assert_contains_all(
        "ResourceManager poison recovery test",
        &resource_manager,
        &[
            "resource_manager_accessors_recover_poisoned_state_locks",
            "let _guard = manager.lock_subscribers();",
            "let _guard = manager.lock_registry_write();",
            "let _guard = manager.lock_payloads_write();",
            "let _guard = manager.lock_runtime_write();",
            "recv_timeout(Duration::from_secs(1))",
            "manager.runtime_state(id)",
            "manager.ref_count(id)",
        ],
    );

    for (label, source) in [
        ("resource manager root", resource_manager.as_str()),
        ("resource registry ops", registry_ops.as_str()),
        ("resource payload ops", payload_ops.as_str()),
        ("resource lease ops", lease_ops.as_str()),
        ("resource event ops", events.as_str()),
    ] {
        assert_no_direct_lock_unwrap_in_production(label, source);
        assert!(
            !production_section(source).contains("lock poisoned"),
            "{label} production code should recover poisoned locks instead of panicking"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("resource module doc", resource_doc.as_str()),
        (
            "status-output M3 lock-poison row data",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 core resource manager lock poison recovery",
                "runtime_15_core_resource_manager_lock_poison_recovery_static_passed_cargo_deferred",
                "core/resource/manager/resource_manager.rs",
                "core/resource/manager/registry_ops.rs",
                "runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager",
            ],
        );
    }
}
