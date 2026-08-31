use std::collections::BTreeSet;
use std::path::Path;

use super::inventory::{
    BEHAVIOR_TEST_ANCHORS, BOUNDED_STREAM_IO_CAPTURE_ANCHORS, BOUNDED_STREAM_IO_LANE_ANCHORS,
    BOUNDED_STREAM_IO_STATE_ANCHORS, BOUNDED_STREAM_IO_WORKER_ANCHORS, CALLBACK_DISPATCHER_ANCHORS,
    DIAGNOSTIC_ANCHORS, ENGINE_TASK_GRAPH_ANCHORS, EXPECTED_DIRECT_RAYON_PATHS,
    EXPECTED_JOB_SYSTEM_MODULES, FORBIDDEN_LEGACY_DIAGNOSTIC_ANCHORS,
    FORBIDDEN_SCHEDULE_EXECUTOR_RAYON_ANCHORS, JOB_HANDLE_ANCHORS, JOB_SCHEDULER_ANCHORS,
    JOB_SCHEDULER_PENDING_ANCHORS, JOB_SCHEDULER_TEST_ANCHORS, JOB_SYSTEM_MODULE_MAX_LINES,
    MIRROR_DOC_ANCHORS, PARALLEL_FOR_ANCHORS, REPORT_ANCHORS, RETAINED_BYTE_BUDGET_ANCHORS,
    SCHEDULE_EXECUTOR_ANCHORS, TASKS_MOD_DECLARATIONS, TASKS_MOD_PUBLIC_ANCHORS,
    TASK_CANCELLATION_POLICY_ANCHORS, TASK_DESCRIPTOR_ANCHORS, TASK_DIAGNOSTIC_JOURNAL_ANCHORS,
    TASK_DIAGNOSTIC_OBSERVATION_ANCHORS, TASK_GRAPH_CANCELLATION_ANCHORS,
    TASK_GRAPH_HANDLE_ANCHORS, TASK_GRAPH_LEASE_ANCHORS, TASK_GRAPH_SCHEDULER_ADMISSION_ANCHORS,
    TASK_GRAPH_SCOPE_ANCHORS, TASK_ID_ANCHORS, TASK_POOL_DESCRIPTOR_ANCHORS,
    TASK_POOL_KIND_ANCHORS, TASK_POOL_SUBMISSION_ANCHORS, TASK_STATE_ANCHORS, TASK_STATUS_ANCHORS,
    TIMER_ANCHORS,
};
use super::source_helpers::{collect_direct_rayon_paths, line_count};

#[test]
fn runtime_11_job_system_mirror_docs_match_structure_audit_counts() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tasks_dir = runtime_root
        .join("src")
        .join("core")
        .join("runtime")
        .join("tasks");

    for module in EXPECTED_JOB_SYSTEM_MODULES {
        let path = tasks_dir.join(module);
        assert!(
            path.exists(),
            "Runtime 11 JobSystem owner module `{module}` is missing; update job_system_boundary before changing the task owner folder"
        );
        let line_count = line_count(&path);
        assert!(
            line_count <= JOB_SYSTEM_MODULE_MAX_LINES,
            "Runtime 11 JobSystem owner module `{module}` has {line_count} lines, exceeding the {JOB_SYSTEM_MODULE_MAX_LINES}-line owner budget"
        );
    }

    let tasks_mod = include_str!("../../../core/runtime/tasks/mod.rs");
    assert_contains_all(
        "core/runtime/tasks/mod.rs declarations",
        tasks_mod,
        TASKS_MOD_DECLARATIONS,
    );
    assert_contains_all(
        "core/runtime/tasks/mod.rs public surface",
        tasks_mod,
        TASKS_MOD_PUBLIC_ANCHORS,
    );

    let callback_dispatcher = include_str!("../../../core/runtime/tasks/callback_dispatcher.rs");
    assert_contains_all(
        "TaskCallbackDispatcher",
        callback_dispatcher,
        CALLBACK_DISPATCHER_ANCHORS,
    );

    let diagnostic_observation =
        include_str!("../../../core/runtime/tasks/diagnostic_observation/mod.rs");
    assert_contains_all(
        "TaskDiagnostic observation contract",
        diagnostic_observation,
        TASK_DIAGNOSTIC_OBSERVATION_ANCHORS,
    );
    let diagnostic_journal =
        include_str!("../../../core/runtime/tasks/diagnostic_observation/journal.rs");
    assert_contains_all(
        "TaskDiagnostic bounded journal",
        diagnostic_journal,
        TASK_DIAGNOSTIC_JOURNAL_ANCHORS,
    );

    let bounded_stream_io_lane =
        include_str!("../../../core/runtime/tasks/bounded_stream_io/lane.rs");
    assert_contains_all(
        "bounded stream I/O lane",
        bounded_stream_io_lane,
        BOUNDED_STREAM_IO_LANE_ANCHORS,
    );
    let bounded_stream_io_capture =
        include_str!("../../../core/runtime/tasks/bounded_stream_io/capture.rs");
    assert_contains_all(
        "bounded stream I/O capture",
        bounded_stream_io_capture,
        BOUNDED_STREAM_IO_CAPTURE_ANCHORS,
    );
    let bounded_stream_io_state =
        include_str!("../../../core/runtime/tasks/bounded_stream_io/state.rs");
    assert_contains_all(
        "bounded stream I/O state",
        bounded_stream_io_state,
        BOUNDED_STREAM_IO_STATE_ANCHORS,
    );
    let bounded_stream_io_worker =
        include_str!("../../../core/runtime/tasks/bounded_stream_io/worker.rs");
    assert_contains_all(
        "bounded stream I/O worker",
        bounded_stream_io_worker,
        BOUNDED_STREAM_IO_WORKER_ANCHORS,
    );

    let job_scheduler = include_str!("../../../core/runtime/tasks/job_scheduler.rs");
    assert_contains_all("JobScheduler", job_scheduler, JOB_SCHEDULER_ANCHORS);
    let job_scheduler_pending =
        include_str!("../../../core/runtime/tasks/job_scheduler/pending.rs");
    assert_contains_all(
        "JobScheduler pending dependency owner",
        job_scheduler_pending,
        JOB_SCHEDULER_PENDING_ANCHORS,
    );
    let job_scheduler_tests = include_str!("../../../core/runtime/tasks/job_scheduler/tests.rs");
    assert_contains_all(
        "JobScheduler tests",
        job_scheduler_tests,
        JOB_SCHEDULER_TEST_ANCHORS,
    );

    let job_handle = include_str!("../../../core/runtime/tasks/job_handle.rs");
    assert_contains_all("JobHandle", job_handle, JOB_HANDLE_ANCHORS);

    let parallel_for = include_str!("../../../core/runtime/tasks/parallel_for.rs");
    assert_contains_all("parallel_for primitive", parallel_for, PARALLEL_FOR_ANCHORS);

    let task_pool = include_str!("../../../core/runtime/tasks/pool.rs");
    assert_contains_all(
        "TaskPool submission authority",
        task_pool,
        TASK_POOL_SUBMISSION_ANCHORS,
    );

    let diagnostics = include_str!("../../../core/runtime/tasks/diagnostics.rs");
    assert_contains_all("JobSystem diagnostics", diagnostics, DIAGNOSTIC_ANCHORS);

    let report = include_str!("../../../core/runtime/tasks/report.rs");
    assert_contains_all("JobSystem report", report, REPORT_ANCHORS);

    let retained_byte_budget = include_str!("../../../core/runtime/tasks/retained_byte_budget.rs");
    assert_contains_all(
        "retained byte budget",
        retained_byte_budget,
        RETAINED_BYTE_BUDGET_ANCHORS,
    );

    let timer = include_str!("../../../core/runtime/tasks/timer.rs");
    assert_contains_all("TaskTimer", timer, TIMER_ANCHORS);

    let task_graph = include_str!("../../../core/runtime/tasks/task_graph/engine_task_graph.rs");
    assert_contains_all("EngineTaskGraph", task_graph, ENGINE_TASK_GRAPH_ANCHORS);
    let task_graph_lease = include_str!("../../../core/runtime/tasks/task_graph/lease.rs");
    assert_contains_all(
        "TaskGraphClientLease",
        task_graph_lease,
        TASK_GRAPH_LEASE_ANCHORS,
    );
    let task_graph_cancellation =
        include_str!("../../../core/runtime/tasks/task_graph/scope/cancellation.rs");
    assert_contains_all(
        "TaskCancellationToken",
        task_graph_cancellation,
        TASK_GRAPH_CANCELLATION_ANCHORS,
    );
    let task_graph_scheduler_admission =
        include_str!("../../../core/runtime/tasks/task_graph/scope/scheduler_admission.rs");
    assert_contains_all(
        "TaskGraph scheduler admission",
        task_graph_scheduler_admission,
        TASK_GRAPH_SCHEDULER_ADMISSION_ANCHORS,
    );
    let task_graph_scope = include_str!("../../../core/runtime/tasks/task_graph/scope.rs");
    assert_contains_all("TaskGraphScope", task_graph_scope, TASK_GRAPH_SCOPE_ANCHORS);
    let task_graph_handle = include_str!("../../../core/runtime/tasks/task_graph/task_handle.rs");
    assert_contains_all("TaskHandle", task_graph_handle, TASK_GRAPH_HANDLE_ANCHORS);

    assert_contains_all(
        "TaskCancellationPolicy",
        include_str!("../../../core/runtime/tasks/task_cancellation_policy.rs"),
        TASK_CANCELLATION_POLICY_ANCHORS,
    );
    assert_contains_all(
        "TaskDescriptor",
        include_str!("../../../core/runtime/tasks/task_descriptor.rs"),
        TASK_DESCRIPTOR_ANCHORS,
    );
    assert_contains_all(
        "TaskId",
        include_str!("../../../core/runtime/tasks/task_id.rs"),
        TASK_ID_ANCHORS,
    );
    assert_contains_all(
        "TaskPoolDescriptor",
        include_str!("../../../core/runtime/tasks/task_pool_descriptor.rs"),
        TASK_POOL_DESCRIPTOR_ANCHORS,
    );
    assert_contains_all(
        "TaskPoolKind",
        include_str!("../../../core/runtime/tasks/task_pool_kind.rs"),
        TASK_POOL_KIND_ANCHORS,
    );
    assert_contains_all(
        "TaskState",
        include_str!("../../../core/runtime/tasks/task_state.rs"),
        TASK_STATE_ANCHORS,
    );
    assert_contains_all(
        "TaskStatus",
        include_str!("../../../core/runtime/tasks/task_status.rs"),
        TASK_STATUS_ANCHORS,
    );

    for legacy_anchor in FORBIDDEN_LEGACY_DIAGNOSTIC_ANCHORS {
        for (owner_name, owner_source) in [
            ("tasks mod", tasks_mod),
            ("JobScheduler", job_scheduler),
            ("JobHandle", job_handle),
            ("JobSystem diagnostics", diagnostics),
            ("JobSystem report", report),
        ] {
            assert!(
                !owner_source.contains(legacy_anchor),
                "{owner_name} must not retain retired task diagnostic anchor `{legacy_anchor}`"
            );
        }
    }

    let tasks_tests = include_str!("../../tasks.rs");
    let job_handle_tests = include_str!("../../../core/runtime/tasks/job_handle/tests.rs");
    let diagnostics_tests = include_str!("../../../core/runtime/tasks/diagnostics/tests.rs");
    let diagnostic_observation_tests =
        include_str!("../../../core/runtime/tasks/diagnostic_observation/tests.rs");
    let task_graph_scope_tests =
        include_str!("../../../core/runtime/tasks/task_graph/scope/tests.rs");
    let task_pool_tests = include_str!("../../../core/runtime/tasks/pool/tests.rs");
    let bounded_stream_io_tests =
        include_str!("../../../core/runtime/tasks/bounded_stream_io/tests.rs");
    let retained_byte_budget_tests =
        include_str!("../../../core/runtime/tasks/retained_byte_budget/tests.rs");
    let dynamic_scene_spawn_tests =
        include_str!("../../../scene/dynamic_scene/spawn_task/loader.rs");
    assert_sources_contain_all(
        "Runtime 11 task behavior tests",
        &[
            bounded_stream_io_tests,
            retained_byte_budget_tests,
            tasks_tests,
            job_handle_tests,
            job_scheduler_tests,
            task_pool_tests,
            diagnostics_tests,
            diagnostic_observation_tests,
            task_graph,
            task_graph_scope_tests,
            dynamic_scene_spawn_tests,
        ],
        BEHAVIOR_TEST_ANCHORS,
    );

    let schedule_executor = include_str!("../../../scene/ecs/schedule_parallel_executor.rs");
    assert_contains_all(
        "ScheduleParallelExecutor",
        schedule_executor,
        SCHEDULE_EXECUTOR_ANCHORS,
    );
    for forbidden_rayon_anchor in FORBIDDEN_SCHEDULE_EXECUTOR_RAYON_ANCHORS {
        assert!(
            !schedule_executor.contains(forbidden_rayon_anchor),
            "ScheduleParallelExecutor should not reintroduce direct Rayon anchor `{forbidden_rayon_anchor}`"
        );
    }

    let direct_rayon_paths = collect_direct_rayon_paths(&runtime_root.join("src"));
    let expected_direct_rayon_paths = EXPECTED_DIRECT_RAYON_PATHS
        .iter()
        .copied()
        .map(String::from)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        direct_rayon_paths, expected_direct_rayon_paths,
        "Runtime 11 direct-Rayon whitelist should match job_system_boundary; only task primitives are allowed"
    );

    let mirror_docs = [
        (
            "JobSystem module doc",
            include_str!("../../../../../docs/zircon_runtime/core/job_system.md"),
        ),
        (
            "Runtime 11 plan",
            include_str!(
                "../../../../../docs/plans/zircon_runtime/runtime/11-job-system-task-model.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "M0 review",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
            ),
        ),
        (
            "interface convergence",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-interface-convergence.md"
            ),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        assert_contains_all(doc_name, doc_source, MIRROR_DOC_ANCHORS);
    }
}

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(
            source.contains(anchor),
            "{label} should contain Runtime 11 JobSystem audit anchor `{anchor}`"
        );
    }
}

fn assert_sources_contain_all(label: &str, sources: &[&str], required: &[&str]) {
    for anchor in required {
        assert!(
            sources.iter().any(|source| source.contains(anchor)),
            "{label} should contain Runtime 11 JobSystem audit anchor `{anchor}`"
        );
    }
}
