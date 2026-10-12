use super::*;

#[test]
fn runtime_15_core_runtime_task_lock_poison_recovery_guard_covers_job_handles() {
    let job_handle = read_runtime_src("core/runtime/tasks/job_handle.rs");
    let task_node = read_runtime_src("core/runtime/tasks/job_handle/task_node.rs");
    let job_handle_tests = read_runtime_src("core/runtime/tasks/job_handle/tests/cases.rs");
    let job_scheduler = read_runtime_src("core/runtime/tasks/job_scheduler.rs");
    let pending_scheduler = read_runtime_src("core/runtime/tasks/job_scheduler/pending.rs");
    let pending_scheduler_tests =
        read_runtime_src("core/runtime/tasks/job_scheduler/tests/cases.rs");
    let _runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let _runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let _review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let _structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let _module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let _tasks_doc = read_repo("docs/zircon_runtime/core/tasks.md");

    assert_contains_all(
        "JobHandle poison recovery",
        &job_handle,
        &[
            "mod task_node;",
            "self.node.lock_inner().lifecycle.is_terminal()",
            "inner = self.node.wait_inner_timeout(inner, WORKER_WAIT_IDLE_PARK)",
            "inner = self.node.wait_inner(inner)",
        ],
    );
    assert_contains_all(
        "JobHandle poison regressions",
        &job_handle_tests,
        &[
            "job_handle_accessors_recover_poisoned_state_lock",
            "job_handle_wait_recovers_poisoned_state_lock",
        ],
    );
    assert_contains_all(
        "TaskNode poison recovery",
        &task_node,
        &[
            "fn lock_inner(&self) -> MutexGuard<'_, TaskNodeState>",
            "fn wait_inner<'a>(",
            "fn wait_inner_timeout<'a>(",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "poison_inner_for_test",
        ],
    );
    assert_contains_all(
        "PendingScheduledJob poison recovery",
        &pending_scheduler,
        &[
            "fn lock_work(&self) -> MutexGuard<'_, Option<PendingScheduledWork>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "let Some(work) = self.lock_work().take()",
        ],
    );
    assert_contains_all(
        "JobScheduler pending owner wiring",
        &job_scheduler,
        &[
            "mod pending;",
            "complete_scheduled_task",
            "PendingScheduledJob",
        ],
    );
    assert_contains_all(
        "PendingScheduledJob poison regression",
        &pending_scheduler_tests,
        &["pending_scheduled_job_recovers_poisoned_task_lock"],
    );

    for (label, source) in [
        ("job handle", job_handle.as_str()),
        ("task node", task_node.as_str()),
        ("job scheduler", job_scheduler.as_str()),
        ("pending scheduler", pending_scheduler.as_str()),
    ] {
        assert_no_direct_lock_unwrap_in_production(label, source);
        assert!(
            !production_section(source).contains("lock poisoned"),
            "{label} production code should recover poisoned locks instead of panicking"
        );
    }
}

#[test]
fn runtime_15_core_runtime_profiling_lock_poison_recovery_guard_covers_global_recorder() {
    let profiling = read_runtime_src("core/runtime/diagnostics/profiling/mod.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let diagnostics_doc = read_repo("docs/zircon_runtime/core/diagnostics.md");

    assert_contains_all(
        "runtime profiling recorder poison recovery",
        &profiling,
        &[
            "use std::sync::{Mutex, MutexGuard, OnceLock};",
            "fn lock_recorder() -> MutexGuard<'static, ProfileRecorder>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "lock_recorder().start_capture(config)",
            "lock_recorder().stop_capture()",
            "lock_recorder().reset()",
            "lock_recorder().snapshot()",
            "let mut recorder = lock_recorder();",
            "profile_recorder_accessors_recover_poisoned_global_lock",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("runtime profiling recorder", &profiling);
    assert!(
        !production_section(&profiling).contains("lock poisoned"),
        "runtime profiling recorder production code should recover poisoned locks instead of panicking"
    );
}
