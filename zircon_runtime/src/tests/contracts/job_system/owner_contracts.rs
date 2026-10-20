//! Production task API ownership and retired diagnostic exclusion; called by the JobSystem contract.
use super::super::inventory::{
    BOUNDED_STREAM_IO_CAPTURE_ANCHORS, BOUNDED_STREAM_IO_LANE_ANCHORS,
    BOUNDED_STREAM_IO_STATE_ANCHORS, BOUNDED_STREAM_IO_WORKER_ANCHORS, CALLBACK_DISPATCHER_ANCHORS,
    DIAGNOSTIC_ANCHORS, ENGINE_TASK_GRAPH_ANCHORS, FORBIDDEN_LEGACY_DIAGNOSTIC_ANCHORS,
    JOB_HANDLE_ANCHORS, JOB_SCHEDULER_ANCHORS, JOB_SCHEDULER_PENDING_ANCHORS,
    JOB_SCHEDULER_TEST_ANCHORS, PARALLEL_FOR_ANCHORS, REPORT_ANCHORS, RETAINED_BYTE_BUDGET_ANCHORS,
    TASKS_MOD_DECLARATIONS, TASKS_MOD_PUBLIC_ANCHORS, TASK_CANCELLATION_POLICY_ANCHORS,
    TASK_DESCRIPTOR_ANCHORS, TASK_DIAGNOSTIC_JOURNAL_ANCHORS, TASK_DIAGNOSTIC_OBSERVATION_ANCHORS,
    TASK_GRAPH_ADMISSION_ANCHORS, TASK_GRAPH_CANCELLATION_ANCHORS, TASK_GRAPH_HANDLE_ANCHORS,
    TASK_GRAPH_LEASE_ANCHORS, TASK_GRAPH_SCOPE_ANCHORS, TASK_ID_ANCHORS, TASK_NODE_ANCHORS,
    TASK_POOL_DESCRIPTOR_ANCHORS, TASK_POOL_KIND_ANCHORS, TASK_POOL_SUBMISSION_ANCHORS,
    TASK_STATE_ANCHORS, TASK_STATUS_ANCHORS, TIMER_ANCHORS,
};

pub(super) fn assert_task_owner_contracts() -> (&'static str, &'static str) {
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
    let job_scheduler_tests =
        include_str!("../../../core/runtime/tasks/job_scheduler/tests/cases.rs");
    assert_contains_all(
        "JobScheduler tests",
        job_scheduler_tests,
        JOB_SCHEDULER_TEST_ANCHORS,
    );

    let job_handle = include_str!("../../../core/runtime/tasks/job_handle.rs");
    assert_contains_all("JobHandle", job_handle, JOB_HANDLE_ANCHORS);
    let task_node = include_str!("../../../core/runtime/tasks/job_handle/task_node.rs");
    assert_contains_all("TaskNode", task_node, TASK_NODE_ANCHORS);

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
    let task_graph_admission =
        include_str!("../../../core/runtime/tasks/task_graph/scope/task_admission.rs");
    assert_contains_all(
        "TaskGraph admission",
        task_graph_admission,
        TASK_GRAPH_ADMISSION_ANCHORS,
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
    let task_status = include_str!("../../../core/runtime/tasks/task_status.rs");
    let (task_status_test, task_status_owner_anchors) = TASK_STATUS_ANCHORS
        .split_last()
        .expect("TaskStatus owner and regression anchors");
    assert_eq!(
        *task_status_test,
        "runtime_task_status_has_one_terminal_state_and_no_poll_clock"
    );
    assert_contains_all("TaskStatus", task_status, task_status_owner_anchors);
    assert_contains_all(
        "TaskStatus test mount",
        task_status,
        &[
            "#[cfg(test)]",
            "#[path = \"tests/task_status.rs\"]",
            "mod tests;",
        ],
    );
    assert_contains_all(
        "TaskStatus lifecycle regression",
        include_str!("../../../core/runtime/tasks/tests/task_status.rs"),
        &[
            task_status_test,
            "status.mark_running();",
            "status.mark_failed(\"worker panicked\");",
            "assert!(status.is_terminal());",
        ],
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

    (job_scheduler_tests, task_graph)
}

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(
            source.contains(anchor),
            "{label} should contain Runtime 11 JobSystem audit anchor `{anchor}`"
        );
    }
}
