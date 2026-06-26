use super::*;

#[test]
fn runtime_15_core_runtime_task_lock_poison_recovery_guard_covers_job_handles() {
    let job_handle = read_runtime_src("core/runtime/tasks/job_handle.rs");
    let job_scheduler = read_runtime_src("core/runtime/tasks/job_scheduler.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let tasks_doc = read_repo("docs/zircon_runtime/core/tasks.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert_contains_all(
        "JobHandle poison recovery",
        &job_handle,
        &[
            "use std::sync::{Arc, Condvar, Mutex, MutexGuard};",
            "fn lock_inner(&self) -> MutexGuard<'_, JobStateInner>",
            "fn wait_inner<'a>(",
            "fn wait_inner_timeout<'a>(",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.state.lock_inner().is_complete",
            "inner = self.state.wait_inner_timeout(inner, WORKER_WAIT_IDLE_PARK)",
            "inner = self.state.wait_inner(inner)",
            "job_handle_accessors_recover_poisoned_state_lock",
            "job_handle_wait_recovers_poisoned_state_lock",
        ],
    );
    assert_contains_all(
        "PendingScheduledJob poison recovery",
        &job_scheduler,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "fn lock_task(&self) -> MutexGuard<'_, Option<ScheduledJob>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "let Some(task) = self.lock_task().take()",
            "let mut task = self.lock_task();",
            "pending_scheduled_job_recovers_poisoned_task_lock",
        ],
    );

    for (label, source) in [
        ("job handle", job_handle.as_str()),
        ("job scheduler", job_scheduler.as_str()),
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
        ("core tasks doc", tasks_doc.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 core runtime task lock poison recovery",
                "runtime_15_core_runtime_task_lock_poison_recovery_static_passed_cargo_deferred",
                "core/runtime/tasks/job_handle.rs",
                "core/runtime/tasks/job_scheduler.rs",
                "job_handle_accessors_recover_poisoned_state_lock",
                "pending_scheduled_job_recovers_poisoned_task_lock",
                "runtime_15_core_runtime_task_lock_poison_recovery_guard_covers_job_handles",
            ],
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
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

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

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("core diagnostics doc", diagnostics_doc.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 core runtime profiling lock poison recovery",
                "runtime_15_core_runtime_profiling_lock_poison_recovery_static_passed_cargo_deferred",
                "core/runtime/diagnostics/profiling/mod.rs",
                "profile_recorder_accessors_recover_poisoned_global_lock",
                "runtime_15_core_runtime_profiling_lock_poison_recovery_guard_covers_global_recorder",
            ],
        );
    }
}
