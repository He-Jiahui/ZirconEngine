use super::*;

#[test]
fn runtime_15_dynamic_api_session_lock_poison_recovery_guard_covers_session_registry() {
    let dynamic_session = read_runtime_src("dynamic_api/session.rs");
    let dynamic_session_ffi = read_runtime_src("dynamic_api/session/ffi.rs");
    let dynamic_session_registry_facade = read_runtime_src("dynamic_api/session/registry/mod.rs");
    let dynamic_session_store = read_runtime_src("dynamic_api/session/registry/session_store.rs");
    let dynamic_session_slot = read_runtime_src("dynamic_api/session/registry/session_slot.rs");
    let dynamic_session_tests = read_runtime_src("dynamic_api/session/tests/lock_poison.rs");
    let current_anchor_owner = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-19-dynamic-api-filter-plan-anchor-current-owner.md",
    );

    assert_contains_all(
        "dynamic API session registry module mount",
        &dynamic_session,
        &["mod registry;"],
    );
    assert_contains_all(
        "dynamic API session FFI registry consumer",
        &dynamic_session_ffi,
        &[
            "try_create_session_with_wake, with_session, with_session_activity,",
            "match try_create_session_with_wake(profile, project_config, wake)",
            "ptr::write(out_session, handle)",
            "ZrStatus::ok()",
            "destroy_session_slot(handle)",
            "with_session(handle, |session|",
        ],
    );
    assert_contains_all(
        "dynamic API session registry facade",
        &dynamic_session_registry_facade,
        &["mod session_store;", "pub(super) use session_store::{"],
    );
    assert!(
        !dynamic_session_registry_facade.contains("fn lock_session(")
            && !dynamic_session_registry_facade.contains("static SESSION_REGISTRY"),
        "dynamic API session registry facade must not retain state or forwarding helpers"
    );
    assert_contains_all(
        "dynamic API session store poison recovery helpers",
        &dynamic_session_store,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard, OnceLock, TryLockError};",
            "fn lock_registry() -> MutexGuard<'static, SessionRegistry>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "let registry = lock_registry();",
            "match slot.dispatch_scoped(action)",
            "let slot = find_session_slot(handle)?;",
        ],
    );
    assert_contains_all(
        "dynamic API session slot poison recovery helper",
        &dynamic_session_slot,
        &[
            "owner: RuntimeSessionOwner",
            "lifecycle: Mutex<SessionSlotLifecycle>",
            "fn lock_lifecycle(&self) -> MutexGuard<'_, SessionSlotLifecycle>",
            "pub(super) fn dispatch_scoped<R: Send>(",
            "self.owner.dispatch_scoped(action)",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
        ],
    );
    for (label, source) in [
        ("dynamic API session owner", dynamic_session.as_str()),
        ("dynamic API session FFI", dynamic_session_ffi.as_str()),
        ("dynamic API session store", dynamic_session_store.as_str()),
        ("dynamic API session slot", dynamic_session_slot.as_str()),
    ] {
        let compact: String = source
            .chars()
            .filter(|char| !char.is_whitespace())
            .collect();
        assert!(
            !compact.contains(LOCK_UNWRAP_CALL),
            "{label} should recover poisoned locks instead of directly unwrapping them"
        );
    }
    assert_contains_all(
        "dynamic API session poison recovery test",
        &dynamic_session_tests,
        &[
            "dynamic_api_session_registry_accessors_recover_poisoned_locks",
            "poison_registry_lock_for_test();",
            "with_session(handle, |_| panic!(\"poison dynamic API session lock\"))",
            "with_session(handle, |_| ZrStatus::ok())",
            "unsafe { destroy_session(handle) }",
        ],
    );

    assert_contains_all_exact(
        "Runtime 15 dynamic-API filter current child owner",
        &current_anchor_owner,
        &[
            "Runtime 15 M3 dynamic API session lock poison recovery",
            "runtime_15_dynamic_api_session_lock_poison_recovery_static_passed_cargo_deferred",
            "dynamic_api/session.rs",
            "dynamic_api/session/tests/lock_poison.rs",
            "runtime_15_dynamic_api_session_lock_poison_recovery_guard_covers_session_registry",
        ],
    );
}

#[test]
fn runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_guard_covers_spawn_task() {
    let task = read_runtime_src("scene/dynamic/spawn_task/task.rs");
    let loader = read_runtime_src("scene/dynamic/spawn_task/loader.rs");

    assert_contains_all(
        "dynamic scene spawn task poison recovery helpers",
        &task,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "pub(super) fn lock_spawn_result(",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
        ],
    );
    assert_contains_all(
        "dynamic scene spawn loader uses canonical task state and shared result helper",
        &loader,
        &[
            "use super::task::{lock_spawn_result, DynamicSceneSpawnTask};",
            "let task = TaskHandle::try_schedule_detached",
            "let task = scope.submit",
            "let mut result = lock_spawn_result(&result);",
        ],
    );
    assert!(
        !task.contains("Mutex<TaskStatus>")
            && !task.contains("AtomicBool")
            && !loader.contains("lock_spawn_status"),
        "dynamic scene must not restore a second lifecycle or cancellation authority"
    );

    let task_production = production_section(&task);
    for (label, source) in [
        ("dynamic scene spawn task", task_production.as_str()),
        ("dynamic scene spawn loader", loader.as_str()),
    ] {
        assert!(
            !source.contains(LOCK_UNWRAP_CALL) && !source.contains("lock poisoned"),
            "{label} production code should recover poisoned locks instead of panicking"
        );
    }
    let recovery_tests = read_runtime_src("scene/dynamic/spawn_task/tests/task.rs");
    assert_contains_all(
        "mounted poison recovery regression",
        &recovery_tests,
        &["fn dynamic_scene_spawn_task_accessors_recover_poisoned_locks("],
    );
}

#[test]
fn runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_guard_covers_batch_result_slots() {
    let executor = read_runtime_src("scene/ecs/schedule_parallel_executor.rs");
    let executor_tests = read_runtime_src("scene/ecs/tests/schedule_parallel_executor.rs");

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
        ],
    );
    assert_contains_all(
        "schedule parallel executor batch-result poison recovery test",
        &executor_tests,
        &["schedule_parallel_executor_batch_result_slot_recovers_poisoned_lock"],
    );

    let production = production_section(&executor);
    assert!(
        !production.contains(LOCK_UNWRAP_CALL) && !production.contains("lock poisoned"),
        "schedule parallel executor production code should recover poisoned batch-result locks"
    );
}
