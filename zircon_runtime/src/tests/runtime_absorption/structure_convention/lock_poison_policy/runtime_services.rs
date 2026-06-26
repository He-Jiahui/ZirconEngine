use super::*;

#[test]
fn runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot() {
    let table = read_runtime_src("plugin/bridge/table.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let plugin_bridge_doc = read_repo("docs/zircon_runtime/plugin/bridge.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert_contains_all(
        "Plugin bridge table provider poison recovery helper",
        &table,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "fn lock_provider(&self) -> MutexGuard<'_, Option<Arc<dyn Any + Send",
            "+ Sync>>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.lock_provider().is_some()",
            ".lock_provider()",
            "*self.lock_provider() = None;",
            "*self.lock_provider() = Some(provider);",
            "bridge_entry_provider_accessors_recover_poisoned_provider_lock",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("plugin bridge table", &table);
    assert!(
        !production_section(&table).contains("lock poisoned"),
        "plugin bridge table production code should recover poisoned provider locks"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("plugin bridge doc", plugin_bridge_doc.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 plugin bridge table lock poison recovery",
                "runtime_15_plugin_bridge_table_lock_poison_recovery_static_passed_cargo_deferred",
                "plugin/bridge/table.rs",
                "bridge_entry_provider_accessors_recover_poisoned_provider_lock",
                "runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot",
            ],
        );
    }
}

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
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
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
        ("status-output M3 foundation row data", status_rows.as_str()),
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
fn runtime_15_dynamic_api_session_lock_poison_recovery_guard_covers_session_registry() {
    let dynamic_session = read_runtime_src("dynamic_api/session.rs");
    let dynamic_session_tests = read_runtime_src("dynamic_api/session/tests/lock_poison.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let dynamic_session_doc = read_repo("docs/zircon_runtime/dynamic_api/session.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert_contains_all(
        "dynamic API session poison recovery helpers",
        &dynamic_session,
        &[
            "use std::sync::{Mutex, MutexGuard, OnceLock};",
            "fn lock_registry() -> MutexGuard<'static, SessionRegistry>",
            "fn lock_session(session: &Mutex<RuntimeDynamicSession>) -> MutexGuard<'_, RuntimeDynamicSession>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "let mut registry = lock_registry();",
            "let registry = lock_registry();",
            "let mut session = lock_session(session.as_ref());",
        ],
    );
    assert!(
        !dynamic_session.contains("registry().lock().unwrap()"),
        "dynamic API session registry should use lock_registry() instead of direct lock unwrap"
    );
    assert!(
        !dynamic_session.contains("session.lock().unwrap()"),
        "dynamic API session execution should use lock_session() instead of direct lock unwrap"
    );
    assert_contains_all(
        "dynamic API session poison recovery test",
        &dynamic_session_tests,
        &[
            "dynamic_api_session_registry_accessors_recover_poisoned_locks",
            "let _registry = lock_registry();",
            "let _session = lock_session(stored_session.as_ref());",
            "with_session(handle, |_| ZrStatus::ok())",
            "unsafe { destroy_session(handle) }",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("dynamic API session doc", dynamic_session_doc.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 dynamic API session lock poison recovery",
                "runtime_15_dynamic_api_session_lock_poison_recovery_static_passed_cargo_deferred",
                "dynamic_api/session.rs",
                "dynamic_api/session/tests/lock_poison.rs",
                "runtime_15_dynamic_api_session_lock_poison_recovery_guard_covers_session_registry",
            ],
        );
    }
}

#[test]
fn runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_guard_covers_spawn_task() {
    let task = read_runtime_src("scene/dynamic_scene/spawn_task/task.rs");
    let loader = read_runtime_src("scene/dynamic_scene/spawn_task/loader.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let dynamic_scene_doc = read_repo("docs/zircon_runtime/scene/dynamic_scene.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert_contains_all(
        "dynamic scene spawn task poison recovery helpers",
        &task,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "pub(super) fn lock_spawn_status(",
            "pub(super) fn lock_spawn_result(",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "dynamic_scene_spawn_task_accessors_recover_poisoned_locks",
        ],
    );
    assert_contains_all(
        "dynamic scene spawn loader uses shared lock helpers",
        &loader,
        &[
            "use super::task::{DynamicSceneSpawnTask, lock_spawn_result, lock_spawn_status};",
            "lock_spawn_status(&status_for_task).mark_running();",
            "let mut status = lock_spawn_status(&status_for_task);",
            "*lock_spawn_result(&result_for_task) = Some(prepared);",
        ],
    );

    for (label, source) in [
        ("dynamic scene spawn task", production_section(&task)),
        ("dynamic scene spawn loader", loader.as_str()),
    ] {
        assert!(
            !source.contains(LOCK_UNWRAP_CALL) && !source.contains("lock poisoned"),
            "{label} production code should recover poisoned locks instead of panicking"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("dynamic scene doc", dynamic_scene_doc.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 dynamic scene spawn task lock poison recovery",
                "runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_static_passed_cargo_deferred",
                "scene/dynamic_scene/spawn_task/task.rs",
                "scene/dynamic_scene/spawn_task/loader.rs",
                "dynamic_scene_spawn_task_accessors_recover_poisoned_locks",
                "runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_guard_covers_spawn_task",
            ],
        );
    }
}

#[test]
fn runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_guard_covers_batch_result_slots() {
    let executor = read_runtime_src("scene/ecs/schedule_parallel_executor.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ecs_doc = read_repo("docs/zircon_runtime/scene/ecs.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert_contains_all(
        "schedule parallel executor batch-result poison recovery helpers",
        &executor,
        &[
            "Arc, Mutex, MutexGuard,",
            "type ScheduleParallelBatchSlot<E> = Arc<Mutex<Option<ScheduleParallelBatchResult<E>>>>;",
            "fn lock_batch_result<E>(",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "*lock_batch_result(&batch_result_for_task) = Some(result);",
            "lock_batch_result(&batch_result)",
            "schedule_parallel_executor_batch_result_slot_recovers_poisoned_lock",
        ],
    );

    let production = production_section(&executor);
    assert!(
        !production.contains(LOCK_UNWRAP_CALL) && !production.contains("lock poisoned"),
        "schedule parallel executor production code should recover poisoned batch-result locks"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("ECS module doc", ecs_doc.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 scene ECS parallel executor lock poison recovery",
                "runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_static_passed_cargo_deferred",
                "scene/ecs/schedule_parallel_executor.rs",
                "schedule_parallel_executor_batch_result_slot_recovers_poisoned_lock",
                "runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_guard_covers_batch_result_slots",
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
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
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
        ("status-output M3 foundation row data", status_rows.as_str()),
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
