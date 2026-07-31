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
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let dynamic_session_doc = read_repo("docs/zircon_runtime/dynamic_api/session.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
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
            "destroy_session_slot, insert_session_with_wake, with_session, with_session_activity,",
            "let handle = insert_session_with_wake(session, wake);",
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
            "use std::sync::{Arc, Mutex, MutexGuard, OnceLock};",
            "fn lock_registry() -> MutexGuard<'static, SessionRegistry>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "let registry = lock_registry();",
            "let mut session = slot.lock_session();",
        ],
    );
    assert_contains_all(
        "dynamic API session slot poison recovery helper",
        &dynamic_session_slot,
        &[
            "session: Mutex<Option<RuntimeDynamicSession>>",
            "pub(super) fn lock_session(&self) -> MutexGuard<'_, Option<RuntimeDynamicSession>>",
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
    for (label, source) in [
        ("module convention doc", module_doc.as_str()),
        ("dynamic API session doc", dynamic_session_doc.as_str()),
        (
            "status-output M3 lock-poison row data",
            status_rows.as_str(),
        ),
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
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
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
            "use super::task::{lock_spawn_result, lock_spawn_status, DynamicSceneSpawnTask};",
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
        (
            "status-output M3 lock-poison row data",
            status_rows.as_str(),
        ),
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
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
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
        (
            "status-output M3 lock-poison row data",
            status_rows.as_str(),
        ),
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
